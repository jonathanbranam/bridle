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
use std::time::{Duration, Instant};

use bridle_api::types::{
    Edge, EdgeKind, Handover, Impact, StatePushStatus, Task, TaskKind, TaskSize, TaskState,
    ThreadEntry, ThreadEntryKind,
};
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

enum PushFailure {
    /// Rejected as a non-fast-forward.
    Diverged(String),
    Other(String),
}

/// `git push origin refs/heads/bridle/state:refs/heads/bridle/state` from the state
/// branch's own worktree: never `--force`, never a `+` refspec.
async fn run_push(dir: &Path, timeout: Duration) -> Result<(), PushFailure> {
    let refspec = format!("refs/heads/{BRANCH}:refs/heads/{BRANCH}");
    let mut cmd = tokio::process::Command::new("git");
    cmd.arg("-C")
        .arg(dir)
        .args(["push", "--porcelain", REMOTE, &refspec])
        // Fail rather than wait on a credential prompt nobody will answer.
        .env("GIT_TERMINAL_PROMPT", "0")
        .kill_on_drop(true);
    let out = match tokio::time::timeout(timeout, cmd.output()).await {
        Err(_) => return Err(PushFailure::Other("push timed out".into())),
        Ok(Err(e)) => return Err(PushFailure::Other(format!("running git: {e}"))),
        Ok(Ok(out)) => out,
    };
    if out.status.success() {
        return Ok(());
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let reason = text
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| l.starts_with("fatal:") || l.starts_with("error:") || l.starts_with('!'))
        .unwrap_or("git push failed")
        .to_string();
    if text.contains("non-fast-forward") || text.contains("fetch first") {
        Err(PushFailure::Diverged(format!(
            "{BRANCH} on {REMOTE} has commits this clone lacks (non-fast-forward)"
        )))
    } else {
        Err(PushFailure::Other(reason))
    }
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
    /// The full rendered contents of `claims.toml`, replaced wholesale on
    /// every [`StateBranch::enqueue_claims`] call, the same reasoning as
    /// `edges` above: claims are durable now (storage.md, "claims"), but
    /// there's still no per-claim file to key a targeted write on.
    claims: Option<String>,
    /// The full rendered contents of `queue.toml`, replaced wholesale on
    /// every [`StateBranch::enqueue_queue`] call: the queue is one small,
    /// PM-owned record (roles-and-lifecycle.md, "the queue"), not per-task
    /// files.
    queue: Option<String>,
    /// handover id -> rendered `handovers/<id>.md`. One file per note, never rewritten
    /// once flushed, so the newest by seq (the current note) is the last of the history.
    handovers: HashMap<String, String>,
}

/// A handle onto the state branch's worktree. Cheap to clone (an `Arc`
/// around the pending-write buffer); every clone shares the same buffer and
/// the same on-disk worktree.
#[derive(Clone)]
pub struct StateBranch {
    /// The state branch's own worktree, e.g. `<workspace>/.bridle/state`.
    dir: PathBuf,
    pending: Arc<Mutex<Pending>>,
    /// `None` unless `[state] push = true`: then nothing below ever runs.
    push: Option<Arc<Pusher>>,
}

/// How long after one push attempt the next may start (`[state]` has no key for it: a
/// constant is enough until someone needs otherwise).
pub const PUSH_DEBOUNCE: Duration = Duration::from_secs(60);
/// A push that hangs (a stuck credential prompt, a dead network) is abandoned after this.
const PUSH_TIMEOUT: Duration = Duration::from_secs(120);
const REMOTE: &str = "origin";

/// Best-effort push of the state branch to `origin` (ticket we2r). Fast-forward only: the
/// command never carries a force flag or a `+` refspec.
struct Pusher {
    debounce: Duration,
    inner: Mutex<PushState>,
}

#[derive(Default)]
struct PushState {
    /// A commit landed that the remote hasn't seen.
    dirty: bool,
    in_flight: bool,
    last_attempt: Option<Instant>,
    last_pushed_at: Option<DateTime<Utc>>,
    /// The latest failure's reason; cleared by a successful push. Logged at WARN only when
    /// it changes.
    failing: Option<String>,
    /// The remote refused a non-fast-forward: someone else wrote the branch. No more pushes.
    diverged: bool,
}

impl Pusher {
    fn record(&self, st: &mut PushState, result: Result<(), PushFailure>) {
        match result {
            Ok(()) => {
                st.dirty = false;
                st.failing = None;
                st.last_pushed_at = Some(Utc::now());
            }
            Err(PushFailure::Diverged(reason)) => {
                st.diverged = true;
                if st.failing.as_deref() != Some(&reason) {
                    tracing::warn!(%reason, "state branch push refused; no longer pushing");
                }
                st.failing = Some(reason);
            }
            Err(PushFailure::Other(reason)) => {
                if st.failing.as_deref() != Some(&reason) {
                    tracing::warn!(%reason, "pushing the state branch failed; will retry");
                }
                st.failing = Some(reason);
            }
        }
    }
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
            push: None,
        })
    }

    /// Turns on pushing to `origin` after a flush that committed, at most once per `debounce`.
    pub fn with_push(mut self, debounce: Duration) -> Self {
        self.push = Some(Arc::new(Pusher {
            debounce,
            inner: Mutex::new(PushState::default()),
        }));
        self
    }

    /// What `bridle status` shows; `None` when pushing is off.
    pub fn push_status(&self) -> Option<StatePushStatus> {
        let p = self.push.as_ref()?;
        let st = p.inner.lock().expect("state push lock");
        Some(StatePushStatus {
            last_pushed_at: st.last_pushed_at,
            failing: st.failing.clone(),
            diverged: st.diverged,
        })
    }

    /// Starts a push in the background if one is due: something unpushed, none in flight, not
    /// diverged, and the debounce has passed. Never awaits the push, so a flush doesn't wait
    /// on the network.
    fn poke_push(&self, committed: bool) {
        let Some(p) = &self.push else { return };
        {
            let mut st = p.inner.lock().expect("state push lock");
            st.dirty |= committed;
            if !st.dirty
                || st.in_flight
                || st.diverged
                || st.last_attempt.is_some_and(|t| t.elapsed() < p.debounce)
            {
                return;
            }
            st.in_flight = true;
            st.last_attempt = Some(Instant::now());
        }
        let p = p.clone();
        let dir = self.dir.clone();
        tokio::spawn(async move {
            let result = run_push(&dir, PUSH_TIMEOUT).await;
            let mut st = p.inner.lock().expect("state push lock");
            st.in_flight = false;
            p.record(&mut st, result);
        });
    }

    /// Pushes once now if anything is unpushed, ignoring the debounce, bounded by `timeout`.
    /// For daemon shutdown, after the final flush; failure is logged, never returned.
    pub async fn push_on_shutdown(&self, timeout: Duration) {
        let Some(p) = &self.push else { return };
        {
            let st = p.inner.lock().expect("state push lock");
            if !st.dirty || st.diverged {
                return;
            }
        }
        let result = run_push(&self.dir, timeout).await;
        let mut st = p.inner.lock().expect("state push lock");
        p.record(&mut st, result);
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

    /// Queues one `events/<YYYY-MM>.jsonl` line for a queue reprioritisation:
    /// the same append-only history [`StateBranch::enqueue_event`] gives
    /// task transitions, keyed by `"queue"` instead of a task id, so a
    /// reprioritisation's actor survives even though `queue.toml` itself
    /// (wholesale-replaced) carries no history of its own.
    pub fn enqueue_queue_event(&self, actor: &str, at: DateTime<Utc>) {
        let month = at.format("%Y-%m").to_string();
        let line = serde_json::json!({
            "queue": "reprioritized",
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

    /// Queues the full current claim set to overwrite `claims.toml` at the
    /// next flush, mirroring [`StateBranch::enqueue_edges`]'s wholesale
    /// approach. Claims are durable now (storage.md, "claims"): a `bridle
    /// rebuild` restores current claims from this file, unlike before this
    /// existed, when a claim had no state-branch counterpart at all.
    pub fn enqueue_claims(&self, claims: &[ClaimRecord]) -> Result<(), StateBranchError> {
        let rendered = render_claims(claims)?;
        self.pending
            .lock()
            .expect("state branch pending lock")
            .claims = Some(rendered);
        Ok(())
    }

    /// Queues the full current queue record to overwrite `queue.toml` at the
    /// next flush, the same wholesale approach as edges/claims: the whole
    /// record is small, and there's no per-tier id to key a targeted write
    /// on.
    pub fn enqueue_queue(&self, tiers: &[Vec<String>]) -> Result<(), StateBranchError> {
        let rendered = render_queue(tiers)?;
        self.pending
            .lock()
            .expect("state branch pending lock")
            .queue = Some(rendered);
        Ok(())
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Queues a handover note to be written as `handovers/<id>.md` at the next flush.
    pub fn enqueue_handover(&self, h: &Handover) -> Result<(), StateBranchError> {
        let rendered = render_handover(h)?;
        self.pending
            .lock()
            .expect("state branch pending lock")
            .handovers
            .insert(h.id.clone(), rendered);
        Ok(())
    }

    /// Whether `handovers/<id>.md` is already in the worktree, for the idempotent backfill.
    pub fn has_handover(&self, id: &str) -> bool {
        self.dir.join("handovers").join(format!("{id}.md")).exists()
    }

    /// Every handover file, parsed, oldest first (by seq, which is the id's number). Strict,
    /// like [`StateBranch::list_tasks`]: only `bridle rebuild` reads it.
    pub fn list_handovers(&self) -> Result<Vec<Handover>, StateBranchError> {
        let dir = self.dir.join("handovers");
        let mut out = Vec::new();
        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries {
                    let path = entry?.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        out.push(parse_handover(&std::fs::read_to_string(&path)?)?);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        out.sort_by_key(|h| handover_seq(&h.id));
        Ok(out)
    }

    /// Fetches `origin`'s `bridle/state` when that can't lose anything (ticket we2r, shape 3).
    /// Only ever reads from the remote and never overwrites: a missing local branch is created
    /// from it; a local branch that is an ancestor of it (or only the untouched seed commit
    /// [`StateBranch::open`] makes) is fast-forwarded; anything else is left alone and said so.
    /// `dir` is the state worktree when it exists, else the project's checkout.
    pub async fn fetch_from_origin(dir: &Path) -> FetchOutcome {
        match Self::try_fetch(dir).await {
            Ok(o) => o,
            Err(e) => FetchOutcome::Failed(e.to_string()),
        }
    }

    async fn try_fetch(dir: &Path) -> Result<FetchOutcome, WorktreeError> {
        let fetched = tokio::time::timeout(
            PUSH_TIMEOUT,
            worktree::run_git(
                dir,
                &["fetch", "--quiet", REMOTE, &format!("refs/heads/{BRANCH}")],
            ),
        )
        .await;
        match fetched {
            Err(_) => return Ok(FetchOutcome::Failed("fetch timed out".into())),
            Ok(Err(_)) => return Ok(FetchOutcome::NoRemoteBranch),
            Ok(Ok(_)) => {}
        }
        let remote = worktree::run_git(dir, &["rev-parse", "FETCH_HEAD"]).await?;
        let remote = remote.trim();
        if !worktree::branch_exists(dir, BRANCH).await? {
            worktree::run_git(dir, &["branch", BRANCH, remote]).await?;
            return Ok(FetchOutcome::Created);
        }
        let local = worktree::run_git(dir, &["rev-parse", &format!("refs/heads/{BRANCH}")]).await?;
        let local = local.trim();
        if local == remote {
            return Ok(FetchOutcome::UpToDate);
        }
        let seed = worktree::run_git(
            dir,
            &["rev-list", "--count", &format!("refs/heads/{BRANCH}")],
        )
        .await?
        .trim()
            == "1"
            && worktree::run_git(dir, &["ls-tree", "-r", "--name-only", local])
                .await?
                .trim()
                .is_empty();
        let ancestor = worktree::run_git(dir, &["merge-base", "--is-ancestor", local, remote])
            .await
            .is_ok();
        if !seed && !ancestor {
            return Ok(FetchOutcome::Diverged);
        }
        // The state worktree is the only checkout of the branch; without one (never at first
        // start, when the branch is missing) there is nothing else to move.
        worktree::run_git(dir, &["reset", "--hard", "-q", remote]).await?;
        Ok(FetchOutcome::FastForwarded)
    }

    /// Writes every pending file, event line, and the edges/claims/queue
    /// snapshots, then makes one commit, if anything is pending; a pure
    /// no-op (no git calls at all) otherwise. Takes what's pending under the
    /// lock and does the slower file/git work outside it, so an `enqueue_*`
    /// racing this call can't be lost: it either lands in this flush or the
    /// next one.
    pub async fn flush_now(&self) -> Result<(), StateBranchError> {
        let committed = self.flush_and_commit().await;
        // Also on a flush that committed nothing: that's the timer that retries a failed or
        // debounced push.
        self.poke_push(matches!(committed, Ok(true)));
        committed.map(|_| ())
    }

    /// Whether a commit was made.
    async fn flush_and_commit(&self) -> Result<bool, StateBranchError> {
        let Pending {
            files,
            events,
            edges,
            claims,
            queue,
            handovers,
        } = {
            let mut guard = self.pending.lock().expect("state branch pending lock");
            std::mem::take(&mut *guard)
        };
        if files.is_empty()
            && events.is_empty()
            && edges.is_none()
            && claims.is_none()
            && queue.is_none()
            && handovers.is_empty()
        {
            return Ok(false);
        }

        let tasks_dir = self.dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir)?;
        for (id, contents) in &files {
            std::fs::write(tasks_dir.join(format!("{id}.md")), contents)?;
        }
        if !handovers.is_empty() {
            let dir = self.dir.join("handovers");
            std::fs::create_dir_all(&dir)?;
            for (id, contents) in &handovers {
                std::fs::write(dir.join(format!("{id}.md")), contents)?;
            }
        }
        if let Some(contents) = &edges {
            std::fs::write(self.dir.join("edges.toml"), contents)?;
        }
        if let Some(contents) = &claims {
            std::fs::write(self.dir.join("claims.toml"), contents)?;
        }
        if let Some(contents) = &queue {
            std::fs::write(self.dir.join("queue.toml"), contents)?;
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
            return Ok(false);
        }
        // Built from whatever actually changed, rather than a fixed match
        // arm per combination: with edges/claims/queue all independently
        // optional now, enumerating every combination by hand doesn't scale.
        let mut parts = Vec::new();
        if !files.is_empty() {
            parts.push(format!("{} task update(s)", files.len()));
        }
        if edges.is_some() {
            parts.push("edges".to_string());
        }
        if claims.is_some() {
            parts.push("claims".to_string());
        }
        if queue.is_some() {
            parts.push("queue".to_string());
        }
        if !handovers.is_empty() {
            parts.push(format!("{} handover(s)", handovers.len()));
        }
        // Nothing but events changed (or, since `flush_now` already returned
        // early with nothing pending at all, unreachable in practice); keep
        // the old wording.
        let message = if parts.is_empty() {
            "1 task update(s)".to_string()
        } else {
            parts.join(", ")
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
        Ok(true)
    }

    /// Reads a task's file back from the worktree, if it exists. A plain
    /// file read, not part of the batched write path; used to hydrate the
    /// in-memory cache on daemon startup.
    pub fn read_task(&self, id: &str) -> Option<Task> {
        let path = self.dir.join("tasks").join(format!("{id}.md"));
        let text = std::fs::read_to_string(path).ok()?;
        parse_task(&text).ok()
    }

    /// Every task file under `tasks/`, parsed, sorted by id for a
    /// deterministic order. Unlike [`StateBranch::read_task`] (which treats
    /// a missing or unparseable file as "nothing to hydrate", tolerating a
    /// crash between an enqueue and the next flush), this is strict: it's
    /// only used by `bridle rebuild`, reconstructing the database from this
    /// branch alone, where a task file that exists but won't parse must
    /// surface as an error rather than silently vanish from the rebuilt set.
    pub fn list_tasks(&self) -> Result<Vec<Task>, StateBranchError> {
        let tasks_dir = self.dir.join("tasks");
        let mut ids = Vec::new();
        match std::fs::read_dir(&tasks_dir) {
            Ok(entries) => {
                for entry in entries {
                    let path = entry?.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("md")
                        && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    {
                        ids.push(stem.to_string());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        ids.sort();
        ids.into_iter()
            .map(|id| {
                let text = std::fs::read_to_string(tasks_dir.join(format!("{id}.md")))?;
                parse_task(&text)
            })
            .collect()
    }

    /// The current edge set from `edges.toml`, or empty if the file doesn't
    /// exist yet (a project with no edges). Used by `bridle rebuild`; see
    /// [`render_edges`] for why there's normally no read path back from this
    /// file.
    pub fn list_edges(&self) -> Result<Vec<Edge>, StateBranchError> {
        match std::fs::read_to_string(self.dir.join("edges.toml")) {
            Ok(text) => parse_edges(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e.into()),
        }
    }

    /// The current claim set from `claims.toml`, or empty if the file
    /// doesn't exist yet (a project with nothing claimed, or one predating
    /// durable claims). Used both to hydrate `TaskManager::open`'s claims
    /// cache and by `bridle rebuild`.
    pub fn list_claims(&self) -> Result<Vec<ClaimRecord>, StateBranchError> {
        match std::fs::read_to_string(self.dir.join("claims.toml")) {
            Ok(text) => parse_claims(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e.into()),
        }
    }

    /// The current queue record from `queue.toml`, tolerating a missing or
    /// unparseable file as "no queue yet" (empty) rather than failing —
    /// [`StateBranch::read_task`]'s reasoning applies the same way here,
    /// since this is also used to hydrate the in-memory cache on daemon
    /// startup, not just `bridle rebuild`.
    pub fn read_queue(&self) -> Vec<Vec<String>> {
        let text = match std::fs::read_to_string(self.dir.join("queue.toml")) {
            Ok(text) => text,
            Err(_) => return Vec::new(),
        };
        parse_queue(&text).unwrap_or_default()
    }
}

/// What [`StateBranch::fetch_from_origin`] did, for `bridle rebuild --from-origin` to say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchOutcome {
    /// No local branch; created from `origin/bridle/state`.
    Created,
    /// The local branch was behind (or only the seed commit); moved up to origin's.
    FastForwarded,
    UpToDate,
    /// Both exist and differ; nothing changed.
    Diverged,
    NoRemoteBranch,
    Failed(String),
}

impl FetchOutcome {
    pub fn describe(&self) -> String {
        match self {
            Self::Created => "created bridle/state from origin".into(),
            Self::FastForwarded => "fast-forwarded bridle/state from origin".into(),
            Self::UpToDate => "bridle/state already matches origin".into(),
            Self::Diverged => {
                "local bridle/state and origin/bridle/state differ; left both alone".into()
            }
            Self::NoRemoteBranch => "origin has no bridle/state; nothing fetched".into(),
            Self::Failed(e) => format!("fetching bridle/state from origin failed: {e}"),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct HandoverFrontmatter {
    id: String,
    role: String,
    project: String,
    created_at: DateTime<Utc>,
    created_by: String,
}

/// `h-0007` -> 7; unparseable ids sort first.
pub fn handover_seq(id: &str) -> i64 {
    id.strip_prefix("h-")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// `+++` TOML frontmatter (as tasks), then the note's body verbatim.
fn render_handover(h: &Handover) -> Result<String, StateBranchError> {
    let fm = toml::to_string(&HandoverFrontmatter {
        id: h.id.clone(),
        role: h.role.clone(),
        project: h.project.clone(),
        created_at: h.created_at,
        created_by: h.created_by.clone(),
    })?;
    Ok(format!("+++\n{fm}+++\n{}", h.body))
}

fn parse_handover(text: &str) -> Result<Handover, StateBranchError> {
    let after_open = text
        .strip_prefix("+++\n")
        .ok_or_else(|| StateBranchError::Parse("missing opening +++ frontmatter fence".into()))?;
    let (fm_str, body) = after_open
        .split_once("\n+++\n")
        .ok_or_else(|| StateBranchError::Parse("missing closing +++ frontmatter fence".into()))?;
    let fm: HandoverFrontmatter = toml::from_str(fm_str)?;
    Ok(Handover {
        id: fm.id,
        role: fm.role,
        project: fm.project,
        body: body.to_string(),
        created_at: fm.created_at,
        created_by: fm.created_by,
    })
}

/// One row of `claims.toml`: a task id, its claimant, and when it was
/// claimed — the state-branch mirror of `store::Claim`, using a plain
/// `String` rather than `PrincipalId` since this module doesn't otherwise
/// depend on `bridle_api`'s principal alias.
#[derive(Debug, Clone, PartialEq)]
pub struct ClaimRecord {
    pub task_id: String,
    pub claimed_by: String,
    pub claimed_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct Frontmatter {
    id: String,
    title: String,
    kind: String,
    state: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    /// Absent in records written before components existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    components: Vec<String>,
    /// Absent in records written before size existed, or with none set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    size: Option<TaskSize>,
    /// Absent in records written before the landing record existed, or unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    /// Last: TOML needs tables after plain values.
    #[serde(default, skip_serializing_if = "Impact::is_empty")]
    impact: Impact,
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
        components: task.components.clone(),
        size: task.size,
        branch: task.branch.clone(),
        commit: task.commit.clone(),
        summary: task.summary.clone(),
        impact: task.impact.clone(),
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

#[derive(Deserialize)]
struct EdgeRecordOwned {
    from: String,
    to: String,
    kind: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Default)]
struct EdgesFileOwned {
    #[serde(default)]
    edge: Vec<EdgeRecordOwned>,
}

/// The inverse of [`render_edges`], used only by `bridle rebuild`
/// (`render_edges`'s own doc comment notes there's normally no read path
/// back from `edges.toml`, since SQLite already has everything an `Edge`
/// carries — rebuild is the one case reconstructing SQLite from scratch).
fn parse_edges(text: &str) -> Result<Vec<Edge>, StateBranchError> {
    let file: EdgesFileOwned = toml::from_str(text)?;
    file.edge
        .into_iter()
        .map(|e| {
            let kind: EdgeKind = e.kind.parse().map_err(StateBranchError::Parse)?;
            Ok(Edge {
                from: e.from,
                to: e.to,
                kind,
                created_at: e.created_at,
            })
        })
        .collect()
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
        // Not this file's job: `TaskManager` layers the current claim (from
        // `claims.toml`/SQLite, storage.md) on top of what this parses.
        claimed_by: None,
        claimed_at: None,
        components: fm.components,
        size: fm.size,
        branch: fm.branch,
        commit: fm.commit,
        summary: fm.summary,
        impact: fm.impact,
    })
}

#[derive(Serialize)]
struct ClaimRecordFile<'a> {
    task: &'a str,
    claimed_by: &'a str,
    claimed_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct ClaimsFile<'a> {
    claim: Vec<ClaimRecordFile<'a>>,
}

/// The whole claim set as one `claims.toml`, mirroring [`render_edges`]'s
/// wholesale-array-of-tables shape. Unlike edges, this one does have a read
/// path back (`parse_claims`, [`StateBranch::list_claims`]): the task
/// files carry no claim (storage.md), so this file is the only durable copy
/// `bridle rebuild` can restore them from.
fn render_claims(claims: &[ClaimRecord]) -> Result<String, StateBranchError> {
    let file = ClaimsFile {
        claim: claims
            .iter()
            .map(|c| ClaimRecordFile {
                task: &c.task_id,
                claimed_by: &c.claimed_by,
                claimed_at: c.claimed_at,
            })
            .collect(),
    };
    Ok(toml::to_string_pretty(&file)?)
}

#[derive(Deserialize)]
struct ClaimRecordFileOwned {
    task: String,
    claimed_by: String,
    claimed_at: DateTime<Utc>,
}

#[derive(Deserialize, Default)]
struct ClaimsFileOwned {
    #[serde(default)]
    claim: Vec<ClaimRecordFileOwned>,
}

/// The inverse of [`render_claims`].
fn parse_claims(text: &str) -> Result<Vec<ClaimRecord>, StateBranchError> {
    let file: ClaimsFileOwned = toml::from_str(text)?;
    Ok(file
        .claim
        .into_iter()
        .map(|c| ClaimRecord {
            task_id: c.task,
            claimed_by: c.claimed_by,
            claimed_at: c.claimed_at,
        })
        .collect())
}

#[derive(Serialize)]
struct QueueTierRecord<'a> {
    tasks: &'a [String],
}

#[derive(Serialize)]
struct QueueFile<'a> {
    tier: Vec<QueueTierRecord<'a>>,
}

/// The queue as one `queue.toml`, an ordered array of tables (`[[tier]]`):
/// array order *is* rank order (tier 1 first), so there's no separate rank
/// field to keep in sync with position. Has a read path back
/// ([`parse_queue`], [`StateBranch::read_queue`]) since the queue has no
/// SQLite counterpart at all — this file is its only durable copy.
fn render_queue(tiers: &[Vec<String>]) -> Result<String, StateBranchError> {
    let file = QueueFile {
        tier: tiers
            .iter()
            .map(|tasks| QueueTierRecord { tasks })
            .collect(),
    };
    Ok(toml::to_string_pretty(&file)?)
}

#[derive(Deserialize, Default)]
struct QueueTierRecordOwned {
    #[serde(default)]
    tasks: Vec<String>,
}

#[derive(Deserialize, Default)]
struct QueueFileOwned {
    #[serde(default)]
    tier: Vec<QueueTierRecordOwned>,
}

/// The inverse of [`render_queue`].
fn parse_queue(text: &str) -> Result<Vec<Vec<String>>, StateBranchError> {
    let file: QueueFileOwned = toml::from_str(text)?;
    Ok(file.tier.into_iter().map(|t| t.tasks).collect())
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
        run(&["init", "-q", "-b", "main"]).await;
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
            claimed_by: None,
            claimed_at: None,
            components: vec!["client-games".to_string(), "dungeon".to_string()],
            size: Some(TaskSize::S),
            branch: Some("bridle/x".to_string()),
            commit: Some("abc123".to_string()),
            summary: Some("Did a thing.\n\nSecond \"paragraph\".".to_string()),
            impact: Impact {
                modify: vec!["s-b310".to_string()],
                add_under: vec!["r-7fa2".to_string()],
                remove: Vec::new(),
                files: vec!["client/**".to_string()],
            },
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
        assert_eq!(parsed.size, Some(TaskSize::S));
        assert_eq!(parsed.branch, task.branch);
        assert_eq!(parsed.commit, task.commit);
        assert_eq!(parsed.summary, task.summary);
        assert_eq!(parsed.created_at, task.created_at);
        assert_eq!(parsed.updated_at, task.updated_at);
    }

    #[test]
    fn a_record_written_before_components_existed_still_loads() {
        let text = "+++\nid = \"tw-1\"\ntitle = \"Old\"\nkind = \"feature\"\nstate = \"open\"\n\
                    created_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\nbody\n";
        let task = parse_task(text).expect("parse old record");
        assert!(task.components.is_empty());
        assert_eq!(task.size, None);
        assert!(!render_task(&task).expect("render").contains("size"));
        // And an empty list isn't written back out.
        assert!(!render_task(&task).expect("render").contains("components"));
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

    #[tokio::test]
    async fn claims_round_trip_through_flush_and_read_back() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");

        let claim = ClaimRecord {
            task_id: "tw-0001".to_string(),
            claimed_by: "agent:w1".to_string(),
            claimed_at: Utc::now(),
        };
        sb.enqueue_claims(std::slice::from_ref(&claim))
            .expect("enqueue");
        sb.flush_now().await.expect("flush");

        let read_back = sb.list_claims().expect("list claims");
        assert_eq!(read_back.len(), 1);
        assert_eq!(read_back[0].task_id, "tw-0001");
        assert_eq!(read_back[0].claimed_by, "agent:w1");

        // Releasing means the next enqueue passes an empty set, which
        // overwrites the file rather than appending to it.
        sb.enqueue_claims(&[]).expect("enqueue empty");
        sb.flush_now().await.expect("flush");
        assert!(sb.list_claims().expect("list claims").is_empty());
    }

    #[tokio::test]
    async fn queue_round_trips_through_flush_and_read_back_and_defaults_to_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");

        // No queue.toml written yet: reads as an empty queue, not an error.
        assert!(sb.read_queue().is_empty());

        let tiers = vec![
            vec!["tw-0001".to_string(), "tw-0002".to_string()],
            vec!["tw-0003".to_string()],
        ];
        sb.enqueue_queue(&tiers).expect("enqueue");
        sb.flush_now().await.expect("flush");

        assert_eq!(sb.read_queue(), tiers);
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

    // ---- pushing (ticket we2r) ----

    async fn git(dir: &Path, args: &[&str]) -> String {
        worktree::run_git(dir, args).await.expect("git")
    }

    /// A repo with a state branch pushing to a local bare `origin`.
    async fn pushing(
        tmp: &Path,
        debounce: Option<Duration>,
    ) -> (StateBranch, PathBuf, PathBuf, PathBuf) {
        let repo = tmp.join("repo");
        init_repo(&repo).await;
        let origin = tmp.join("origin.git");
        std::fs::create_dir_all(&origin).expect("mkdir origin");
        git(&origin, &["init", "-q", "--bare"]).await;
        git(
            &repo,
            &["remote", "add", "origin", origin.to_str().expect("utf8")],
        )
        .await;
        let dir = tmp.join("state");
        let mut sb = StateBranch::open(&repo, &dir).await.expect("open");
        if let Some(d) = debounce {
            sb = sb.with_push(d);
        }
        (sb, repo, origin, dir)
    }

    async fn edit(sb: &StateBranch, n: u32) {
        sb.enqueue_task(&sample_task(&format!("tw-{n:04x}")))
            .expect("enqueue");
        sb.flush_now().await.expect("flush");
    }

    async fn origin_tip(origin: &Path) -> Option<String> {
        worktree::run_git(
            origin,
            &["rev-parse", "--verify", "-q", "refs/heads/bridle/state"],
        )
        .await
        .ok()
        .map(|s| s.trim().to_string())
    }

    async fn wait_until_idle(sb: &StateBranch) {
        let p = sb.push.as_ref().expect("push on");
        for _ in 0..200 {
            if !p.inner.lock().expect("lock").in_flight {
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("push still in flight");
    }

    #[tokio::test]
    async fn a_flush_that_committed_pushes_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, origin, dir) = pushing(tmp.path(), Some(Duration::from_millis(0))).await;
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        let head = git(&dir, &["rev-parse", "HEAD"]).await;
        assert_eq!(origin_tip(&origin).await.as_deref(), Some(head.trim()));
        let st = sb.push_status().expect("status");
        assert!(st.last_pushed_at.is_some() && st.failing.is_none() && !st.diverged);
    }

    #[tokio::test]
    async fn pushes_are_debounced_and_a_trailing_push_follows() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, origin, dir) = pushing(tmp.path(), Some(Duration::from_secs(3600))).await;
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        let first = origin_tip(&origin).await.expect("pushed");
        edit(&sb, 2).await;
        edit(&sb, 3).await;
        wait_until_idle(&sb).await;
        assert_eq!(
            origin_tip(&origin).await.as_deref(),
            Some(first.as_str()),
            "coalesced"
        );
        // The window ends (backdated rather than slept, so the test is deterministic).
        sb.push
            .as_ref()
            .expect("push on")
            .inner
            .lock()
            .expect("lock")
            .last_attempt = Instant::now().checked_sub(Duration::from_secs(7200));
        // The next tick, with nothing new to commit, still pushes what's waiting.
        sb.flush_now().await.expect("tick");
        wait_until_idle(&sb).await;
        let head = git(&dir, &["rev-parse", "HEAD"]).await;
        assert_eq!(origin_tip(&origin).await.as_deref(), Some(head.trim()));
        // Both edits went in one push: first + init + 2 commits, not one push each.
        assert_ne!(head.trim(), first);
    }

    #[tokio::test]
    async fn a_failed_push_shows_then_is_retried() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, origin, dir) = pushing(tmp.path(), Some(Duration::from_millis(0))).await;
        let away = tmp.path().join("origin-away.git");
        std::fs::rename(&origin, &away).expect("hide origin");
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        let st = sb.push_status().expect("status");
        assert!(st.failing.is_some() && !st.diverged && st.last_pushed_at.is_none());
        std::fs::rename(&away, &origin).expect("restore origin");
        sb.flush_now().await.expect("tick");
        wait_until_idle(&sb).await;
        let st = sb.push_status().expect("status");
        assert!(st.failing.is_none() && st.last_pushed_at.is_some());
        let head = git(&dir, &["rev-parse", "HEAD"]).await;
        assert_eq!(origin_tip(&origin).await.as_deref(), Some(head.trim()));
    }

    #[tokio::test]
    async fn a_non_fast_forward_stops_pushing_and_never_forces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, origin, _dir) = pushing(tmp.path(), Some(Duration::from_millis(0))).await;
        // Someone else writes bridle/state on origin first.
        let other = tmp.path().join("other");
        init_repo(&other).await;
        git(
            &other,
            &["remote", "add", "origin", origin.to_str().expect("utf8")],
        )
        .await;
        git(&other, &["push", "-q", "origin", "main:bridle/state"]).await;
        let theirs = origin_tip(&origin).await.expect("theirs");

        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        let st = sb.push_status().expect("status");
        assert!(st.diverged && st.failing.is_some());
        edit(&sb, 2).await;
        wait_until_idle(&sb).await;
        assert_eq!(origin_tip(&origin).await.as_deref(), Some(theirs.as_str()));
    }

    #[tokio::test]
    async fn push_off_pushes_nothing_and_has_no_status() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, origin, _dir) = pushing(tmp.path(), None).await;
        edit(&sb, 1).await;
        sb.push_on_shutdown(Duration::from_secs(5)).await;
        assert!(sb.push_status().is_none());
        assert_eq!(origin_tip(&origin).await, None);
    }

    #[tokio::test]
    async fn shutdown_pushes_what_the_debounce_held_back() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, origin, dir) = pushing(tmp.path(), Some(Duration::from_secs(3600))).await;
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        edit(&sb, 2).await;
        wait_until_idle(&sb).await;
        let head = git(&dir, &["rev-parse", "HEAD"]).await;
        assert_ne!(origin_tip(&origin).await.as_deref(), Some(head.trim()));
        sb.push_on_shutdown(Duration::from_secs(10)).await;
        assert_eq!(origin_tip(&origin).await.as_deref(), Some(head.trim()));
    }

    // ---- handovers and fetching (ticket we2r, shapes 2 and 3) ----

    fn sample_handover(seq: u32, body: &str) -> Handover {
        Handover {
            id: format!("h-{seq:04}"),
            role: "orchestrator".into(),
            project: "p".into(),
            body: body.into(),
            created_at: Utc::now(),
            created_by: "human".into(),
        }
    }

    #[tokio::test]
    async fn a_handover_becomes_a_file_after_flush_and_reads_back() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, _origin, dir) = pushing(tmp.path(), None).await;
        let h = sample_handover(7, "# Note\n\nbody with\n+++\nfence\n");
        sb.enqueue_handover(&h).expect("enqueue");
        assert!(!dir.join("handovers/h-0007.md").exists());
        sb.flush_now().await.expect("flush");
        let text = std::fs::read_to_string(dir.join("handovers/h-0007.md")).expect("file");
        assert!(text.starts_with("+++\n") && text.contains("created_by = \"human\""));
        assert_eq!(sb.list_handovers().expect("list"), vec![h]);
    }

    /// Origin holding a `bridle/state` with one task, pushed from a first repo; returns
    /// (origin path, a second repo with the origin remote and no state branch).
    async fn origin_with_state(tmp: &Path) -> (PathBuf, PathBuf) {
        let (sb, _repo, origin, _dir) = pushing(tmp, Some(Duration::from_millis(0))).await;
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        assert!(origin_tip(&origin).await.is_some());
        let repo2 = tmp.join("repo2");
        init_repo(&repo2).await;
        git(
            &repo2,
            &["remote", "add", "origin", origin.to_str().expect("utf8")],
        )
        .await;
        (origin, repo2)
    }

    #[tokio::test]
    async fn fetch_creates_a_missing_local_branch_from_origin() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_origin, repo2) = origin_with_state(tmp.path()).await;
        assert_eq!(
            StateBranch::fetch_from_origin(&repo2).await,
            FetchOutcome::Created
        );
        let dir2 = tmp.path().join("state2");
        let sb2 = StateBranch::open(&repo2, &dir2).await.expect("open");
        assert!(sb2.read_task("tw-0001").is_some());
        assert_eq!(
            StateBranch::fetch_from_origin(&dir2).await,
            FetchOutcome::UpToDate
        );
    }

    #[tokio::test]
    async fn open_alone_never_fetches() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_origin, repo2) = origin_with_state(tmp.path()).await;
        let sb2 = StateBranch::open(&repo2, &tmp.path().join("state2"))
            .await
            .expect("open");
        assert!(sb2.read_task("tw-0001").is_none());
    }

    #[tokio::test]
    async fn fetch_never_overwrites_a_local_branch_with_its_own_state() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_origin, repo2) = origin_with_state(tmp.path()).await;
        let dir2 = tmp.path().join("state2");
        let sb2 = StateBranch::open(&repo2, &dir2).await.expect("open");
        edit(&sb2, 2).await;
        let before = git(&dir2, &["rev-parse", "HEAD"]).await;
        assert_eq!(
            StateBranch::fetch_from_origin(&dir2).await,
            FetchOutcome::Diverged
        );
        assert_eq!(git(&dir2, &["rev-parse", "HEAD"]).await, before);
        assert!(sb2.read_task("tw-0002").is_some());
    }

    #[tokio::test]
    async fn fetch_moves_a_seed_only_branch_and_a_behind_branch_forward() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_origin, repo2) = origin_with_state(tmp.path()).await;
        // The daemon's own first start leaves just the seed commit.
        let dir2 = tmp.path().join("state2");
        let sb2 = StateBranch::open(&repo2, &dir2).await.expect("open");
        assert_eq!(
            StateBranch::fetch_from_origin(&dir2).await,
            FetchOutcome::FastForwarded
        );
        assert!(sb2.read_task("tw-0001").is_some());
        // Behind: origin gains a commit.
        let clone = tmp.path().join("clone");
        git(
            tmp.path(),
            &[
                "clone",
                "-q",
                "-b",
                "bridle/state",
                _origin.to_str().expect("utf8"),
                clone.to_str().expect("utf8"),
            ],
        )
        .await;
        std::fs::write(clone.join("more.txt"), "x").expect("write");
        git(&clone, &["add", "-A"]).await;
        git(
            &clone,
            &[
                "-c",
                "user.email=a@b",
                "-c",
                "user.name=a",
                "commit",
                "-q",
                "-m",
                "more",
            ],
        )
        .await;
        git(&clone, &["push", "-q", "origin", "HEAD:bridle/state"]).await;
        assert_eq!(
            StateBranch::fetch_from_origin(&dir2).await,
            FetchOutcome::FastForwarded
        );
        assert!(dir2.join("more.txt").exists());
    }

    #[tokio::test]
    async fn fetch_with_no_remote_branch_says_so() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_sb, repo, _origin, _dir) = pushing(tmp.path(), None).await;
        assert_eq!(
            StateBranch::fetch_from_origin(&repo).await,
            FetchOutcome::NoRemoteBranch
        );
    }

    #[test]
    fn the_push_command_never_forces() {
        let src = include_str!("state_branch.rs");
        let body = &src[src.find("async fn run_push").expect("run_push")..];
        let body = &body[..body.find("\n}\n").expect("end")];
        assert!(!body.contains("force") && !body.contains("\"+"));
    }

    #[tokio::test]
    async fn origin_already_has_bridle_state_fast_forwards() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, origin, dir) = pushing(tmp.path(), Some(Duration::from_millis(0))).await;
        // First edit: push to origin.
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        let first_commit = git(&dir, &["rev-parse", "HEAD"]).await;
        assert_eq!(
            origin_tip(&origin).await.as_deref(),
            Some(first_commit.trim()),
            "first commit pushed"
        );

        // Second edit: push again, should fast-forward (not force).
        edit(&sb, 2).await;
        wait_until_idle(&sb).await;
        let second_commit = git(&dir, &["rev-parse", "HEAD"]).await;
        assert_ne!(first_commit.trim(), second_commit.trim());
        assert_eq!(
            origin_tip(&origin).await.as_deref(),
            Some(second_commit.trim()),
            "second commit fast-forward"
        );
        let st = sb.push_status().expect("status");
        assert!(st.last_pushed_at.is_some() && st.failing.is_none() && !st.diverged);
    }

    #[tokio::test]
    async fn origin_equals_local_no_error_no_force() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, origin, dir) = pushing(tmp.path(), Some(Duration::from_millis(0))).await;
        // Push to origin.
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        let commit = git(&dir, &["rev-parse", "HEAD"]).await;
        assert_eq!(origin_tip(&origin).await.as_deref(), Some(commit.trim()));

        // A second flush when nothing changed should not error or force.
        sb.flush_now().await.expect("second flush");
        wait_until_idle(&sb).await;
        let st = sb.push_status().expect("status");
        assert!(st.last_pushed_at.is_some() && st.failing.is_none() && !st.diverged);
        assert_eq!(origin_tip(&origin).await.as_deref(), Some(commit.trim()));
    }

    #[tokio::test]
    async fn no_remote_configured_stays_quiet_no_warn_spam() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        // No remote added.
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir)
            .await
            .expect("open")
            .with_push(Duration::from_millis(0));

        // Edit and flush should not panic even though there's no remote.
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        // Push status should show a failure reason (e.g., no remote configured).
        let st = sb.push_status();
        assert!(st.is_some(), "push status exists");
        let st = st.expect("status");
        assert!(st.failing.is_some(), "push fails when no remote configured");
    }
}
