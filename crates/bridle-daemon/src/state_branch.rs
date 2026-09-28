//! The task record's other durable copy: the `bridle` state branch
//! (docs/design/storage.md, "The state branch"). One markdown file per task
//! under `tasks/<id>.md` (TOML frontmatter + body + thread) and an
//! append-only `events/<YYYY-MM>.jsonl`.
//!
//! Writes are batched, not committed one at a time: callers enqueue a
//! task's latest rendering and any event lines, and a periodic tick (or a
//! future immediate-flush case) calls [`StateBranch::flush_now`], which
//! writes whatever is pending and makes one commit. A crash between an
//! enqueue and the next flush loses that pending write (storage.md notes
//! this on `Task::body`); the database's own copy of `title`/`kind`/`state`
//! is unaffected, since that write already went to SQLite synchronously.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bridle_api::types::{Edge, Task, TaskKind, TaskState, ThreadEntry, ThreadEntryKind};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::worktree::{self, WorktreeError};

/// The project's state branch (storage.md). Fixed, not configurable: it's
/// bridle's own branch, not something a project should need to rename.
///
/// Named `bridle/state`, not the bare `bridle` storage.md's prose names --
/// a real git ref namespace collision, not a style choice: agent worktrees
/// already live on `bridle/<name>` branches (worktree.rs), and git refuses
/// to have both `refs/heads/bridle` and `refs/heads/bridle/<anything>` at
/// once (a ref can't be both a leaf and a directory). `bridle/state` sits
/// in the same `bridle/` namespace as every agent branch instead.
const BRANCH: &str = "bridle/state";

#[derive(Debug, thiserror::Error)]
pub enum StateBranchError {
    #[error(transparent)]
    Worktree(#[from] WorktreeError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("rendering task frontmatter: {0}")]
    Ser(#[from] toml::ser::Error),
    #[error("parsing task frontmatter: {0}")]
    De(#[from] toml::de::Error),
    #[error("parsing task file: {0}")]
    Parse(String),
}

#[derive(Debug, Default)]
struct Pending {
    /// task id -> fully rendered `tasks/<id>.md` contents. Keyed by id so a
    /// second edit before the next flush overwrites the first: the batch
    /// collapses to each task's latest state, not a queue of diffs.
    files: HashMap<String, String>,
    /// (the `events/<YYYY-MM>.jsonl` file it belongs to, one pre-rendered
    /// JSON line), in write order.
    events: Vec<(String, String)>,
    /// The full rendered contents of `edges.toml`, replaced wholesale on
    /// every [`StateBranch::enqueue_edges`] call: unlike tasks there's no
    /// per-edge id to key a per-file write on, and the whole set is small,
    /// so each enqueue re-renders it entirely rather than diffing.
    edges: Option<String>,
}

/// A handle onto the state branch's worktree. Cheap to clone (an `Arc`
/// around the pending-write buffer); every clone shares the same buffer and
/// the same on-disk worktree.
#[derive(Clone)]
pub struct StateBranch {
    /// The state branch's own worktree, e.g. `<workspace>/.bridle/state`.
    dir: PathBuf,
    pending: Arc<Mutex<Pending>>,
}

impl StateBranch {
    /// Ensures the `bridle` branch exists (creating it as a parentless
    /// orphan on the project's first run) and is checked out at `dir`, then
    /// returns a handle for enqueuing writes. `repo` is the project's main
    /// checkout, touched only to create the branch object/ref
    /// (`ensure_orphan_branch`) and the worktree registration (`git
    /// worktree add` writes into `repo/.git/worktrees/`, never into `repo`'s
    /// own working tree or index).
    pub async fn open(repo: &Path, dir: &Path) -> Result<Self, StateBranchError> {
        if !dir.join(".git").exists() {
            worktree::ensure_orphan_branch(repo, BRANCH, "initial bridle state").await?;
            if let Some(parent) = dir.parent() {
                std::fs::create_dir_all(parent)?;
            }
            worktree::add_existing(repo, dir, BRANCH).await?;
        }
        Ok(StateBranch {
            dir: dir.to_path_buf(),
            pending: Arc::new(Mutex::new(Pending::default())),
        })
    }

    /// Renders `task` and queues it to overwrite `tasks/<id>.md` at the
    /// next flush.
    pub fn enqueue_task(&self, task: &Task) -> Result<(), StateBranchError> {
        let rendered = render_task(task)?;
        self.pending
            .lock()
            .expect("state branch pending lock")
            .files
            .insert(task.id.clone(), rendered);
        Ok(())
    }

    /// Queues one line for `events/<YYYY-MM>.jsonl`, keyed by `at`'s month.
    pub fn enqueue_event(
        &self,
        task_id: &str,
        from: &str,
        to: &str,
        actor: &str,
        at: DateTime<Utc>,
    ) {
        let month = at.format("%Y-%m").to_string();
        let line = serde_json::json!({
            "task": task_id,
            "from": from,
            "to": to,
            "at": at,
            "actor": actor,
        })
        .to_string();
        self.pending
            .lock()
            .expect("state branch pending lock")
            .events
            .push((month, line));
    }

    /// Queues the full current edge set to overwrite `edges.toml` at the
    /// next flush. The caller passes every edge, not a delta: SQLite is the
    /// source of truth for what currently exists (docs/design/storage.md),
    /// this is just its durable copy on the state branch.
    pub fn enqueue_edges(&self, edges: &[Edge]) -> Result<(), StateBranchError> {
        let rendered = render_edges(edges)?;
        self.pending
            .lock()
            .expect("state branch pending lock")
            .edges = Some(rendered);
        Ok(())
    }

    /// Writes every pending file, event line and the edges snapshot, then
    /// makes one commit, if anything is pending; a pure no-op (no git calls
    /// at all) otherwise. Takes what's pending under the lock and does the
    /// slower file/git work outside it, so an `enqueue_*` racing this call
    /// can't be lost: it either lands in this flush or the next one.
    pub async fn flush_now(&self) -> Result<(), StateBranchError> {
        let Pending {
            files,
            events,
            edges,
        } = {
            let mut guard = self.pending.lock().expect("state branch pending lock");
            std::mem::take(&mut *guard)
        };
        if files.is_empty() && events.is_empty() && edges.is_none() {
            return Ok(());
        }

        let tasks_dir = self.dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir)?;
        for (id, contents) in &files {
            std::fs::write(tasks_dir.join(format!("{id}.md")), contents)?;
        }
        if let Some(contents) = &edges {
            std::fs::write(self.dir.join("edges.toml"), contents)?;
        }

        if !events.is_empty() {
            let events_dir = self.dir.join("events");
            std::fs::create_dir_all(&events_dir)?;
            let mut by_month: HashMap<String, String> = HashMap::new();
            for (month, line) in &events {
                let buf = by_month.entry(month.clone()).or_default();
                buf.push_str(line);
                buf.push('\n');
            }
            for (month, chunk) in by_month {
                use std::io::Write;
                let path = events_dir.join(format!("{month}.jsonl"));
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                f.write_all(chunk.as_bytes())?;
            }
        }

        worktree::run_git(&self.dir, &["add", "-A"]).await?;
        // A no-op edit (rewriting a file to content identical to what's
        // already committed) can leave nothing staged even though this
        // flush had pending work; `git commit` would then fail with
        // "nothing to commit", so check first rather than treat that as an
        // error.
        let status = worktree::run_git(&self.dir, &["status", "--porcelain"]).await?;
        if status.trim().is_empty() {
            return Ok(());
        }
        let message = match (files.is_empty(), edges.is_some()) {
            (false, true) => format!("{} task update(s), edges", files.len()),
            (false, false) => format!("{} task update(s)", files.len()),
            (true, true) => "edges update".to_string(),
            // Neither files nor edges changed, so this flush is only events
            // (or, since `flush_now` already returned early with nothing
            // pending at all, unreachable in practice); keep the old wording.
            (true, false) => "1 task update(s)".to_string(),
        };
        worktree::run_git(
            &self.dir,
            &[
                "-c",
                "user.email=bridle@localhost",
                "-c",
                "user.name=bridle",
                "commit",
                "-q",
                "-m",
                &message,
            ],
        )
        .await?;
        Ok(())
    }

    /// Reads a task's file back from the worktree, if it exists. A plain
    /// file read, not part of the batched write path; used to hydrate the
    /// in-memory cache on daemon startup.
    pub fn read_task(&self, id: &str) -> Option<Task> {
        let path = self.dir.join("tasks").join(format!("{id}.md"));
        let text = std::fs::read_to_string(path).ok()?;
        parse_task(&text).ok()
    }
}

#[derive(Serialize, Deserialize)]
struct Frontmatter {
    id: String,
    title: String,
    kind: String,
    state: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// `+++`-delimited TOML frontmatter (the same convention Hugo uses, picked
/// so it reads unambiguously as "not YAML" next to a markdown body), then
/// the body, then, if the task has any thread entries, a `## Thread`
/// section with one `### <kind> · <from> · <timestamp>` heading per entry.
/// Only `note` entries are produced by this build; `question`/`answer`/
/// `handoff`/`conflict`/`system` slot into the same heading shape later.
fn render_task(task: &Task) -> Result<String, StateBranchError> {
    let fm = Frontmatter {
        id: task.id.clone(),
        title: task.title.clone(),
        kind: task.kind.as_str().to_string(),
        state: task.state.as_str().to_string(),
        created_at: task.created_at,
        updated_at: task.updated_at,
    };
    let toml = toml::to_string_pretty(&fm)?;
    let mut out = String::new();
    out.push_str("+++\n");
    out.push_str(&toml);
    out.push_str("+++\n\n");
    out.push_str(task.body.trim_end());
    out.push('\n');
    if !task.thread.is_empty() {
        out.push_str("\n## Thread\n");
        for e in &task.thread {
            out.push_str(&format!(
                "\n### {} \u{b7} {} \u{b7} {}\n{}\n",
                e.kind.as_str(),
                e.from,
                e.at.to_rfc3339_opts(SecondsFormat::Millis, true),
                e.body.trim_end(),
            ));
        }
    }
    Ok(out)
}

#[derive(Serialize)]
struct EdgeRecord<'a> {
    from: &'a str,
    to: &'a str,
    kind: &'static str,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct EdgesFile<'a> {
    edge: Vec<EdgeRecord<'a>>,
}

/// The whole edge set as one `edges.toml`, an array of tables (`[[edge]]`),
/// mirroring `render_task`'s TOML-frontmatter convention. There's no parser
/// back: SQLite already has everything an `Edge` carries, so unlike a task's
/// body/thread, nothing here is read back from the state branch — this file
/// exists for git history and recovery, not as bridle's read path.
fn render_edges(edges: &[Edge]) -> Result<String, StateBranchError> {
    let file = EdgesFile {
        edge: edges
            .iter()
            .map(|e| EdgeRecord {
                from: &e.from,
                to: &e.to,
                kind: e.kind.as_str(),
                created_at: e.created_at,
            })
            .collect(),
    };
    Ok(toml::to_string_pretty(&file)?)
}

/// The inverse of [`render_task`]. A known limitation: the body or an
/// entry's text containing the literal sequences `"\n## Thread\n"` or
/// `"\n### "` confuses the split; not guarded against in this build (noted
/// in docs/design/storage.md).
fn parse_task(text: &str) -> Result<Task, StateBranchError> {
    let after_open = text
        .strip_prefix("+++\n")
        .ok_or_else(|| StateBranchError::Parse("missing opening +++ frontmatter fence".into()))?;
    let (fm_str, rest) = after_open
        .split_once("\n+++\n")
        .ok_or_else(|| StateBranchError::Parse("missing closing +++ frontmatter fence".into()))?;
    let fm: Frontmatter = toml::from_str(fm_str)?;
    let kind: TaskKind = fm
        .kind
        .parse()
        .map_err(|e: String| StateBranchError::Parse(e))?;
    let state: TaskState = fm
        .state
        .parse()
        .map_err(|e: String| StateBranchError::Parse(e))?;

    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let (body, thread_str) = match rest.split_once("\n## Thread\n") {
        Some((b, t)) => (b, t),
        None => (rest, ""),
    };

    let mut thread = Vec::new();
    for chunk in thread_str.split("\n### ") {
        if chunk.is_empty() {
            continue;
        }
        let (header, entry_body) = chunk
            .split_once('\n')
            .ok_or_else(|| StateBranchError::Parse(format!("malformed thread entry: {chunk:?}")))?;
        let mut parts = header.split(" \u{b7} ");
        let (kind_s, from, at_s) = match (parts.next(), parts.next(), parts.next()) {
            (Some(k), Some(f), Some(a)) => (k, f, a),
            _ => {
                return Err(StateBranchError::Parse(format!(
                    "malformed thread heading: {header:?}"
                )));
            }
        };
        let kind: ThreadEntryKind = kind_s
            .parse()
            .map_err(|e: String| StateBranchError::Parse(e))?;
        let at = DateTime::parse_from_rfc3339(at_s)
            .map_err(|e| StateBranchError::Parse(format!("bad thread timestamp {at_s:?}: {e}")))?
            .with_timezone(&Utc);
        thread.push(ThreadEntry {
            kind,
            from: from.to_string(),
            body: entry_body.trim_end_matches('\n').to_string(),
            at,
        });
    }

    Ok(Task {
        id: fm.id,
        title: fm.title,
        kind,
        state,
        body: body.trim_end_matches('\n').to_string(),
        thread,
        created_at: fm.created_at,
        updated_at: fm.updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridle_api::types::{ThreadEntry, ThreadEntryKind};
    use tokio::process::Command;

    async fn init_repo(dir: &Path) {
        std::fs::create_dir_all(dir).expect("mkdir repo");
        let run = |args: &'static [&'static str]| {
            let dir = dir.to_path_buf();
            async move {
                let out = Command::new("git")
                    .arg("-C")
                    .arg(&dir)
                    .args(args)
                    .output()
                    .await
                    .expect("run git");
                assert!(
                    out.status.success(),
                    "git {args:?}: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        };
        run(&["init", "-q"]).await;
        run(&[
            "-c",
            "user.email=t@e.com",
            "-c",
            "user.name=T",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            "init",
        ])
        .await;
    }

    fn sample_task(id: &str) -> Task {
        let now = Utc::now();
        Task {
            id: id.to_string(),
            title: "Add foo".to_string(),
            kind: TaskKind::Feature,
            state: TaskState::Open,
            body: "Some description.\n\nWith a second paragraph.".to_string(),
            thread: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn render_then_parse_round_trips_without_a_thread() {
        let task = sample_task("tw-7fa2");
        let rendered = render_task(&task).expect("render");
        assert!(rendered.starts_with("+++\n"));
        let parsed = parse_task(&rendered).expect("parse");
        assert_eq!(parsed.id, task.id);
        assert_eq!(parsed.title, task.title);
        assert_eq!(parsed.kind, task.kind);
        assert_eq!(parsed.state, task.state);
        assert_eq!(parsed.body, task.body);
        assert!(parsed.thread.is_empty());
        assert_eq!(parsed.created_at, task.created_at);
        assert_eq!(parsed.updated_at, task.updated_at);
    }

    #[test]
    fn render_then_parse_round_trips_with_a_thread() {
        let mut task = sample_task("tw-c0f1");
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: "human".to_string(),
            body: "dropped: budget cut".to_string(),
            at: Utc::now(),
        });
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: "agent:w1".to_string(),
            body: "multi-line\nnote body".to_string(),
            at: Utc::now(),
        });
        let rendered = render_task(&task).expect("render");
        let parsed = parse_task(&rendered).expect("parse");
        assert_eq!(parsed.thread.len(), 2);
        assert_eq!(parsed.thread[0].from, "human");
        assert_eq!(parsed.thread[0].body, "dropped: budget cut");
        assert_eq!(parsed.thread[1].from, "agent:w1");
        assert_eq!(parsed.thread[1].body, "multi-line\nnote body");
    }

    #[tokio::test]
    async fn open_creates_the_branch_and_worktree_only_once() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");

        let sb = StateBranch::open(&repo, &dir).await.expect("open");
        assert!(dir.join(".git").exists());

        // Re-opening (as a restarted daemon would) is a no-op, not an
        // error, and doesn't fail because the branch/worktree already exist.
        let sb2 = StateBranch::open(&repo, &dir).await.expect("reopen");
        drop(sb);
        drop(sb2);
    }

    #[tokio::test]
    async fn flush_now_is_a_no_op_with_nothing_pending() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");

        sb.flush_now().await.expect("flush");
        // No commit was made: HEAD@bridle is still the orphan's first commit.
        let log = worktree::run_git(&dir, &["log", "--oneline"])
            .await
            .expect("log");
        assert_eq!(log.lines().count(), 1);
    }

    #[tokio::test]
    async fn flush_now_writes_the_file_and_commits_once() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");

        let task = sample_task("tw-7fa2");
        sb.enqueue_task(&task).expect("enqueue");
        sb.enqueue_event("tw-7fa2", "", "open", "human", Utc::now());
        sb.flush_now().await.expect("flush");

        let path = dir.join("tasks/tw-7fa2.md");
        assert!(path.is_file());
        let read_back = sb.read_task("tw-7fa2").expect("read back");
        assert_eq!(read_back.title, task.title);

        let month = Utc::now().format("%Y-%m").to_string();
        let events_path = dir.join(format!("events/{month}.jsonl"));
        let events_text = std::fs::read_to_string(&events_path).expect("read events");
        assert_eq!(events_text.lines().count(), 1);
        let parsed: serde_json::Value =
            serde_json::from_str(events_text.lines().next().unwrap()).expect("json");
        assert_eq!(parsed["task"], "tw-7fa2");
        assert_eq!(parsed["to"], "open");

        let log = worktree::run_git(&dir, &["log", "--oneline"])
            .await
            .expect("log");
        assert_eq!(
            log.lines().count(),
            2,
            "one commit beyond the orphan's first"
        );
    }

    #[tokio::test]
    async fn a_second_edit_before_flush_collapses_into_one_file_version() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");

        let mut task = sample_task("tw-7fa2");
        sb.enqueue_task(&task).expect("enqueue 1");
        task.title = "Add foo, better".to_string();
        sb.enqueue_task(&task).expect("enqueue 2");
        sb.flush_now().await.expect("flush");

        let read_back = sb.read_task("tw-7fa2").expect("read back");
        assert_eq!(read_back.title, "Add foo, better");
        let log = worktree::run_git(&dir, &["log", "--oneline"])
            .await
            .expect("log");
        assert_eq!(
            log.lines().count(),
            2,
            "two enqueues, one flush, one commit"
        );
    }

    #[tokio::test]
    async fn enqueue_edges_writes_the_whole_set_and_a_second_call_replaces_it() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");

        let now = Utc::now();
        let edge = Edge {
            from: "tw-0001".to_string(),
            to: "tw-0002".to_string(),
            kind: bridle_api::types::EdgeKind::Blocks,
            created_at: now,
        };
        sb.enqueue_edges(std::slice::from_ref(&edge))
            .expect("enqueue");
        sb.flush_now().await.expect("flush");

        let contents = std::fs::read_to_string(dir.join("edges.toml")).expect("read edges.toml");
        assert!(contents.contains("tw-0001"));
        assert!(contents.contains("tw-0002"));
        assert!(contents.contains("blocks"));

        // Removing the edge means the next enqueue passes an empty set,
        // which overwrites the file rather than appending to it.
        sb.enqueue_edges(&[]).expect("enqueue empty");
        sb.flush_now().await.expect("flush");
        let contents = std::fs::read_to_string(dir.join("edges.toml")).expect("read edges.toml");
        assert!(!contents.contains("tw-0001"));
    }

    /// The safety property the design calls out explicitly: writing to the
    /// state branch must never touch the main checkout's working tree or
    /// index, however many tasks are enqueued or flushed.
    #[tokio::test]
    async fn state_branch_writes_never_touch_the_main_checkout() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        std::fs::write(repo.join("app.txt"), "unrelated project file\n").expect("write app file");
        let out = Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(["add", "app.txt"])
            .output()
            .await
            .expect("git add");
        assert!(out.status.success());
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");

        // Snapshot the main checkout's raw index bytes rather than calling
        // `git status`/`git add` again on `repo`: git's own "racy git" stat
        // cache refresh means a *second* `status` call on an unchanged repo
        // can itself rewrite the index (and bump its mtime) even with zero
        // real changes, which would make this assertion flaky for reasons
        // that have nothing to do with the code under test.
        let index_before = std::fs::read(repo.join(".git/index")).expect("read index");
        let files_before = list_files(&repo);

        for i in 0..3 {
            let task = sample_task(&format!("tw-000{i}"));
            sb.enqueue_task(&task).expect("enqueue");
            sb.enqueue_event(&task.id, "", "open", "human", Utc::now());
            sb.flush_now().await.expect("flush");
        }

        let index_after = std::fs::read(repo.join(".git/index")).expect("read index");
        assert_eq!(
            index_before, index_after,
            "main checkout's index must not be touched"
        );
        let files_after = list_files(&repo);
        assert_eq!(
            files_before, files_after,
            "main checkout's working tree must not change"
        );

        // And the main checkout's HEAD branch never moved to the state
        // branch (`git init`'s default branch name varies, master vs main,
        // so this just checks it's still whatever it started as).
        let head = worktree::run_git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"])
            .await
            .expect("head");
        assert_ne!(head.trim(), BRANCH);
    }

    /// Every regular file under `dir`, excluding `.git`, with its contents,
    /// so the comparison catches a stray write anywhere in the checkout.
    fn list_files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        for entry in walkdir(dir) {
            if entry
                .strip_prefix(dir)
                .is_ok_and(|rel| rel.starts_with(".git"))
            {
                continue;
            }
            if entry.is_file() {
                out.push((entry.clone(), std::fs::read(&entry).expect("read file")));
            }
        }
        out.sort();
        out
    }

    fn walkdir(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&d) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    out.push(path);
                }
            }
        }
        out
    }
}
