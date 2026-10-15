//! The task record's other durable copy: the `bridle` state branch
//! (docs/design/storage.md, "The state branch"). One markdown file per task
//! under `tracking/<id>.md` (`tasks/<id>.md` before br-95t9; TOML frontmatter + body + thread) and an
//! append-only `events/<YYYY-MM>.jsonl`.
//!
//! Writes are batched, not committed one at a time: callers enqueue a
//! task's latest rendering and any event lines, and a periodic tick (or a
//! future immediate-flush case) calls [`StateBranch::flush_now`], which
//! writes whatever is pending and makes one commit. A crash between an
//! enqueue and the next flush loses that pending write (storage.md notes
//! this on `Ticket::body`); the database's own copy of `title`/`kind`/`state`
//! is unaffected, since that write already went to SQLite synchronously.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bridle_api::types::{
    Edge, EdgeKind, Handover, Impact, StatePushStatus, ThreadEntry, ThreadEntryKind, Ticket,
    TicketKind, TicketPriority, TicketSize, TicketState, UNKNOWN_CREATOR,
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
    #[error("`{0}` is not the recorded owner of this project")]
    NotOwner(String),
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
    /// ticket id -> rendered `tickets/<id>.md`, the ticket's thread (22ab section 1). Only
    /// for a task whose `ticket` is its own id, the one row of its ticket; same collapsing
    /// as `files`.
    threads: HashMap<String, String>,
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
    /// The full rendered `owner.toml` (hw6c), written by [`StateBranch::claim_owner`].
    owner: Option<String>,
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

    /// Pushes now and confirms: after the push, origin's `bridle/state` must be this branch's
    /// tip. For the take-over, which must not report success on a claim origin doesn't have.
    pub async fn push_confirmed(&self) -> Result<(), String> {
        let reason = |f: PushFailure| match f {
            PushFailure::Diverged(m) | PushFailure::Other(m) => m,
        };
        run_push(&self.dir, PUSH_TIMEOUT).await.map_err(reason)?;
        let local = local_sha(&self.dir, BRANCH).await;
        let remote = worktree::run_git(
            &self.dir,
            &["ls-remote", REMOTE, &format!("refs/heads/{BRANCH}")],
        )
        .await
        .map_err(|e| format!("asking origin for {BRANCH}: {e}"))?;
        match remote.split_whitespace().next() {
            Some(sha) if sha == local => Ok(()),
            other => Err(format!(
                "origin's {BRANCH} is {} after the push, not {local}",
                other.unwrap_or("missing")
            )),
        }
    }

    /// Renders `task` and queues it to overwrite `tasks/<id>.md` at the
    /// next flush.
    pub fn enqueue_task(&self, task: &Ticket) -> Result<(), StateBranchError> {
        let rendered = render_task(task)?;
        let mut pending = self.pending.lock().expect("state branch pending lock");
        if owns_thread_file(task) && !task.thread.is_empty() {
            pending
                .threads
                .insert(task.id.clone(), render_thread_file(task));
        }
        pending.files.insert(task.id.clone(), rendered);
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

    /// Records `host` as the project's owner in `owner.toml` (written at the next flush) unless
    /// the worktree already says so. `since` is when this host took the project over, not the
    /// daemon's latest start, so an ordinary restart makes no commit.
    pub fn claim_owner(&self, host: &str, now: DateTime<Utc>) -> Result<(), StateBranchError> {
        let mut owner = match read_owner_file(&self.dir) {
            Some(o) if o.host == host && o.released.is_none() => return Ok(()),
            // Serving again after a stop takes the release back.
            Some(o) if o.host == host => o,
            _ => Owner {
                host: host.to_string(),
                since: now.to_rfc3339_opts(SecondsFormat::Secs, true),
                released: None,
            },
        };
        owner.released = None;
        let rendered = toml::to_string(&owner)?;
        self.pending
            .lock()
            .expect("state branch pending lock")
            .owner = Some(rendered);
        Ok(())
    }

    /// Marks `host`'s ownership released in `owner.toml` (written at the next flush). Refuses
    /// unless `host` is the recorded owner.
    pub fn release_owner(&self, host: &str, now: DateTime<Utc>) -> Result<(), StateBranchError> {
        let Some(mut owner) = read_owner_file(&self.dir).filter(|o| o.host == host) else {
            return Err(StateBranchError::NotOwner(host.to_string()));
        };
        owner.released = Some(now.to_rfc3339_opts(SecondsFormat::Secs, true));
        self.pending
            .lock()
            .expect("state branch pending lock")
            .owner = Some(toml::to_string(&owner)?);
        Ok(())
    }

    /// The fetch-first half of starting a served project (hw6c): fetches `origin`'s
    /// `bridle/state` (adopting it into a missing or untouched local branch) and refuses when
    /// its `owner.toml` names a host other than one of `hosts`, unless `take_over`. `state_dir` is the
    /// state worktree, used when it exists. An unreachable origin or one without the branch
    /// doesn't block: there is nothing to conflict with.
    pub async fn fetch_and_check_owner(
        repo: &Path,
        state_dir: &Path,
        project: &str,
        hosts: &[String],
        take_over: bool,
    ) -> Result<FetchOutcome, OwnerConflict> {
        let dir = if state_dir.join(".git").exists() {
            state_dir
        } else {
            repo
        };
        let outcome = Self::fetch_from_origin(dir).await;
        if matches!(
            outcome,
            FetchOutcome::NoRemoteBranch | FetchOutcome::Failed(_)
        ) {
            return Ok(outcome);
        }
        // `fetch_from_origin` leaves the fetched tip in FETCH_HEAD, whatever it did to the branch.
        let shown = worktree::run_git(dir, &["show", &format!("FETCH_HEAD:{OWNER_FILE}")]).await;
        let owner = shown.ok().and_then(|t| toml::from_str::<Owner>(&t).ok());
        match owner {
            Some(o) if !hosts.contains(&o.host) && !take_over => Err(OwnerConflict {
                project: project.to_string(),
                host: o.host,
                since: o.since,
            }),
            _ => Ok(outcome),
        }
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
            // A failed fetch is "no such branch" only if origin answers and lacks it.
            Ok(Err(e)) => {
                return Ok(match origin_has_state_branch(dir).await {
                    Ok(false) => FetchOutcome::NoRemoteBranch,
                    Ok(true) => FetchOutcome::Failed(e.to_string()),
                    Err(reason) => FetchOutcome::Failed(reason),
                });
            }
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
        // The state worktree is the only checkout of the branch. From anywhere else (the
        // project's checkout, on `main`) only the ref moves: a hard reset there would clobber it.
        let checked_out = worktree::run_git(dir, &["symbolic-ref", "--short", "-q", "HEAD"])
            .await
            .is_ok_and(|b| b.trim() == BRANCH);
        if checked_out {
            worktree::run_git(dir, &["reset", "--hard", "-q", remote]).await?;
        } else {
            worktree::run_git(dir, &["branch", "-f", BRANCH, remote]).await?;
        }
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
            threads,
            events,
            edges,
            claims,
            queue,
            handovers,
            owner,
        } = {
            let mut guard = self.pending.lock().expect("state branch pending lock");
            std::mem::take(&mut *guard)
        };
        if files.is_empty()
            && threads.is_empty()
            && events.is_empty()
            && edges.is_none()
            && claims.is_none()
            && queue.is_none()
            && handovers.is_empty()
            && owner.is_none()
        {
            return Ok(false);
        }

        let tasks_dir = self.dir.join(ITEMS_DIR);
        std::fs::create_dir_all(&tasks_dir)?;
        for (id, contents) in &files {
            std::fs::write(tasks_dir.join(format!("{id}.md")), contents)?;
            // A touched item moves out of the old folder; untouched ones stay until touched.
            match std::fs::remove_file(self.dir.join(LEGACY_ITEMS_DIR).join(format!("{id}.md"))) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        if !threads.is_empty() {
            let dir = self.dir.join(THREADS_DIR);
            std::fs::create_dir_all(&dir)?;
            for (id, contents) in &threads {
                std::fs::write(dir.join(format!("{id}.md")), contents)?;
            }
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
        if let Some(contents) = &owner {
            std::fs::write(self.dir.join(OWNER_FILE), contents)?;
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
        if !threads.is_empty() {
            parts.push(format!("{} ticket thread(s)", threads.len()));
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
        if owner.is_some() {
            parts.push("owner".to_string());
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
    pub fn read_task(&self, id: &str) -> Option<Ticket> {
        let text = self.read_item_file(id)?;
        let mut task = parse_task(&text).ok()?;
        self.overlay_thread(&mut task);
        Some(task)
    }

    /// An item's file text: from `tracking/`, else from the `tasks/` folder of a state branch
    /// written before br-95t9 (read for one release; no bulk rewrite).
    fn read_item_file(&self, id: &str) -> Option<String> {
        [ITEMS_DIR, LEGACY_ITEMS_DIR]
            .iter()
            .find_map(|d| std::fs::read_to_string(self.dir.join(d).join(format!("{id}.md"))).ok())
    }

    /// A task that owns a ticket thread file takes its thread from there; the inline
    /// `## Thread` of a file written before the move is the fallback.
    fn overlay_thread(&self, task: &mut Ticket) {
        let _ = self.overlay_thread_strict(task);
    }

    /// [`Self::overlay_thread`], reporting an unparseable thread file (named, with line) for
    /// `bridle rebuild`; a missing file is not an error.
    fn overlay_thread_strict(&self, task: &mut Ticket) -> Result<(), String> {
        if !owns_thread_file(task) {
            return Ok(());
        }
        let name = format!("{}/{}.md", THREADS_DIR, task.id);
        if let Ok(text) = std::fs::read_to_string(self.dir.join(&name)) {
            task.thread = parse_thread_file(&text).map_err(|e| format!("{name}: {e}"))?;
        }
        Ok(())
    }

    /// Whether the task's file has a watchers record: false for one written before watchers
    /// existed, which `TicketManager::open` then backfills.
    pub fn task_records_watchers(&self, id: &str) -> bool {
        self.read_item_file(id).is_some_and(|text| {
            text.strip_prefix("+++\n")
                .and_then(|t| t.split_once("\n+++\n"))
                .and_then(|(fm, _)| toml::from_str::<Frontmatter>(fm).ok())
                .is_some_and(|fm| fm.watchers.is_some())
        })
    }

    /// Every task file under `tasks/`, parsed, sorted by id for a
    /// deterministic order. Unlike [`StateBranch::read_task`] (which treats
    /// a missing or unparseable file as "nothing to hydrate", tolerating a
    /// crash between an enqueue and the next flush), this is strict: it's
    /// only used by `bridle rebuild`, reconstructing the database from this
    /// branch alone, where a task file that exists but won't parse must
    /// surface as an error rather than silently vanish from the rebuilt set.
    pub fn list_tasks(&self) -> Result<Vec<Ticket>, StateBranchError> {
        let mut ids = Vec::new();
        for d in [ITEMS_DIR, LEGACY_ITEMS_DIR] {
            match std::fs::read_dir(self.dir.join(d)) {
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
        }
        ids.sort();
        ids.dedup();
        // Every bad file is reported in one pass, each named with its path.
        let mut tasks = Vec::new();
        let mut errors = Vec::new();
        for id in ids {
            let text = self.read_item_file(&id).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, format!("{id}.md vanished"))
            })?;
            let parsed = parse_task(&text)
                .map_err(|e| format!("tracking/{id}.md: {e}"))
                .and_then(|mut task| {
                    self.overlay_thread_strict(&mut task)?;
                    Ok(task)
                });
            match parsed {
                Ok(task) => tasks.push(task),
                Err(e) => errors.push(e),
            }
        }
        if errors.is_empty() {
            Ok(tasks)
        } else {
            Err(StateBranchError::Parse(format!(
                "{} unreadable file(s):\n{}",
                errors.len(),
                errors.join("\n")
            )))
        }
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
    /// durable claims). Used both to hydrate `TicketManager::open`'s claims
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

const OWNER_FILE: &str = "owner.toml";

/// `owner.toml` on the state branch: the one host whose daemon serves the project (hw6c).
#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub host: String,
    pub since: String,
    /// When the owner's `project stop` confirmed its pushes and gave the project up (53gy B).
    /// Absent means not released: a take-over refuses without `--force`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub released: Option<String>,
}

/// Whether the state worktree's `owner.toml` names `host`.
pub fn owned_by(dir: &Path, hosts: &[String]) -> bool {
    read_owner_file(dir).is_some_and(|o| hosts.contains(&o.host))
}

fn read_owner_file(dir: &Path) -> Option<Owner> {
    toml::from_str(&std::fs::read_to_string(dir.join(OWNER_FILE)).ok()?).ok()
}

/// Another host owns the project's state branch on origin.
#[derive(Debug, thiserror::Error)]
#[error(
    "project `{project}` is owned by host `{host}` (since {since}); run `bridle project stop` \
     on it, then `bridle project take-over` here"
)]
pub struct OwnerConflict {
    pub project: String,
    pub host: String,
    pub since: String,
}

/// A take-over refuses to start rather than run on a history it hasn't seen or has forked
/// from. The message names both SHAs; there is no override, the human sorts it out by hand.
#[derive(Debug, thiserror::Error)]
pub enum TakeOverError {
    #[error(
        "take-over refused: `{branch}` has diverged from origin (local {local}, origin \
         {origin}); sort it out by hand, bridle never rebases or forces"
    )]
    Diverged {
        branch: String,
        local: String,
        origin: String,
    },
    #[error(
        "take-over refused: `{branch}` not fetched from origin (local {local}, origin \
         unknown): {reason}"
    )]
    NotFetched {
        branch: String,
        local: String,
        reason: String,
    },
    #[error(
        "take-over refused: local `{branch}` ({local}) and origin's ({origin}) have diverged: \
         each has commits the other lacks. Merge or rebase by hand so one contains the other \
         (bridle never rebases or forces), then run take-over again"
    )]
    NotAncestor {
        branch: String,
        local: String,
        origin: String,
    },
    #[error(
        "take-over refused: the checkout of `{branch}` at {dir} has uncommitted changes, so \
         it can't be fast-forwarded to origin ({origin}); commit or discard them first"
    )]
    Dirty {
        branch: String,
        dir: String,
        origin: String,
    },
    #[error(
        "take-over refused: `{host}` has not released {project}. Run `bridle project stop` on \
         `{host}` first; if that machine is dead, run this again with --force"
    )]
    NotReleased { project: String, host: String },
    #[error(
        "take-over has nothing to take over: `{branch}` is not on origin (local {local}); this \
         is a new project: run `bridle project start`"
    )]
    NewProject { branch: String, local: String },
    #[error("take-over refused: {0}")]
    Git(#[from] WorktreeError),
}

/// Asks origin whether it has `bridle/state`. `Err` means origin could not be asked (no remote,
/// unreachable, timed out), which is not the same as "no".
pub async fn origin_has_state_branch(dir: &Path) -> Result<bool, String> {
    let asked = tokio::time::timeout(
        PUSH_TIMEOUT,
        worktree::run_git(
            dir,
            &[
                "ls-remote",
                "--heads",
                REMOTE,
                &format!("refs/heads/{BRANCH}"),
            ],
        ),
    )
    .await;
    match asked {
        Ok(Ok(out)) => Ok(!out.trim().is_empty()),
        Ok(Err(e)) => Err(format!("origin unreachable: {e}")),
        Err(_) => Err("origin unreachable: ls-remote timed out".into()),
    }
}

/// A brand-new project: `bridle/state` exists neither here nor on a reachable origin. The first
/// `serve` creates it and claims the owner, so `project start` may go ahead.
pub async fn is_new_project(repo: &Path) -> Result<bool, String> {
    if local_sha(repo, BRANCH).await != "none" {
        return Ok(false);
    }
    origin_has_state_branch(repo).await.map(|has| !has)
}

async fn local_sha(dir: &Path, branch: &str) -> String {
    worktree::run_git(
        dir,
        &[
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/heads/{branch}"),
        ],
    )
    .await
    .map(|s| s.trim().to_string())
    .unwrap_or_else(|_| "none".into())
}

/// What [`take_over_project`] did.
#[derive(Debug, PartialEq, Eq)]
pub enum TakeOverOutcome {
    /// `owner.toml` already names this machine; nothing was written.
    AlreadyOwner,
    /// `owner.toml` names this machine but marks it released (`released`, an RFC 3339 time);
    /// nothing was written. Not owned now: only the wording differs from `AlreadyOwner`.
    ReleasedByThisMachine { released: String },
    /// This machine is now the owner, and origin has it.
    Claimed {
        from: Option<String>,
        main_push: MainPush,
    },
}

/// What a take-over did about local commits on the integration branch that origin lacks (they
/// could not be pushed before the claim: the pre-push hook refuses a non-owner).
#[derive(Debug, PartialEq, Eq)]
pub enum MainPush {
    /// Local was level with, or behind, origin: nothing to push.
    NotNeeded,
    /// Local was ahead; pushed after the claim and origin confirmed it.
    Pushed,
    /// Local was ahead and the push failed; ownership moved anyway, push by hand.
    Failed(String),
}

/// The scheduled messages a released project carries on `bridle/state`.
pub const SCHEDULES_FILE: &str = "schedules.json";
/// The outbox id high-water mark a stop commits with the released mark (d6h6). Absent in a
/// release from before it was carried: the next owner then seeds nothing.
pub const OUTBOX_SEQ_FILE: &str = "outbox-seq";

/// The one-shot take-over (53gy): claims the project for `names[0]` and confirms origin has the
/// claim. Same preconditions as the old `serve --take-over`: origin reached, the state branch
/// level with it, the integration branch fast-forwardable. Starts no daemon and needs none.
pub async fn take_over_project(
    repo: &Path,
    state_dir: &Path,
    project: &str,
    integration: &str,
    names: &[String],
    force: bool,
) -> anyhow::Result<TakeOverOutcome> {
    let dir = if state_dir.join(".git").exists() {
        state_dir
    } else {
        repo
    };
    let outcome = StateBranch::fetch_and_check_owner(repo, state_dir, project, names, true).await?;
    require_state_synced(dir, &outcome).await?;
    let ahead = sync_integration_for_take_over(repo, integration).await?;
    let sb = StateBranch::open(repo, state_dir).await?;
    let owner = read_owner_file(&sb.dir);
    let from = owner.as_ref().map(|o| o.host.clone());
    if from.as_ref().is_some_and(|h| names.contains(h)) {
        return Ok(match owner.and_then(|o| o.released) {
            Some(released) => TakeOverOutcome::ReleasedByThisMachine { released },
            None => TakeOverOutcome::AlreadyOwner,
        });
    }
    if let Some(o) = owner.filter(|o| o.released.is_none() && !force) {
        return Err(TakeOverError::NotReleased {
            project: project.to_string(),
            host: o.host,
        }
        .into());
    }
    sb.claim_owner(&names[0], Utc::now())?;
    sb.flush_now().await?;
    sb.push_confirmed()
        .await
        .map_err(|e| anyhow::anyhow!("claimed locally but origin does not have it: {e}"))?;
    // Only now may this clone push (the hook reads the claim); never forced.
    let main_push = if ahead {
        match push_integration_confirmed(repo, integration).await {
            Ok(()) => MainPush::Pushed,
            Err(e) => MainPush::Failed(e),
        }
    } else {
        MainPush::NotNeeded
    };
    Ok(TakeOverOutcome::Claimed { from, main_push })
}

/// The owner `owner.toml` records on the local `bridle/state` branch, if any. Read-only.
pub async fn local_owner(repo: &Path) -> Option<Owner> {
    let text = worktree::run_git(
        repo,
        &["show", &format!("refs/heads/{BRANCH}:{OWNER_FILE}")],
    )
    .await
    .ok()?;
    toml::from_str(&text).ok()
}

/// The giving-up half of `project stop` (53gy B), run once the daemon is gone: pushes the state
/// branch, then the integration branch's direct commits, confirming after each that origin has
/// them; only then writes the `released` mark and pushes that as the last commit, confirmed too.
/// Any failure returns the reason and leaves no mark. `schedules` (the project's active scheduled
/// messages as JSON) is committed with the mark as `schedules.json`, and `outbox_seq` (the outbox
/// id high-water mark) as `outbox-seq`, for the next owner's take-over.
pub async fn release_project(
    repo: &Path,
    state_dir: &Path,
    integration: &str,
    names: &[String],
    schedules: &str,
    outbox_seq: i64,
) -> anyhow::Result<()> {
    let sb = StateBranch::open(repo, state_dir).await?;
    let owner = read_owner_file(&sb.dir);
    if !owner.as_ref().is_some_and(|o| names.contains(&o.host)) {
        anyhow::bail!(
            "this machine ({}) is not the owner{}",
            names[0],
            owner
                .map(|o| format!(" ({} is)", o.host))
                .unwrap_or_default()
        );
    }
    sb.flush_now().await?;
    sb.push_confirmed()
        .await
        .map_err(|e| anyhow::anyhow!("pushing {BRANCH}: {e}"))?;
    push_integration_confirmed(repo, integration)
        .await
        .map_err(|e| anyhow::anyhow!("pushing {integration}: {e}"))?;
    let host = owner.map(|o| o.host).unwrap_or_default();
    std::fs::write(sb.dir.join(SCHEDULES_FILE), schedules)?;
    std::fs::write(sb.dir.join(OUTBOX_SEQ_FILE), outbox_seq.to_string())?;
    sb.release_owner(&host, Utc::now())?;
    sb.flush_now().await?;
    sb.push_confirmed()
        .await
        .map_err(|e| anyhow::anyhow!("pushing the released mark: {e}"))?;
    Ok(())
}

/// Pushes the integration branch (never forced) and confirms origin's copy contains the local
/// tip. A local branch behind origin is fine: nothing of it would be lost. No local branch is
/// nothing to push.
async fn push_integration_confirmed(repo: &Path, branch: &str) -> Result<(), String> {
    let local = local_sha(repo, branch).await;
    if local == "none" {
        return Ok(());
    }
    let pushed = tokio::time::timeout(
        PUSH_TIMEOUT,
        worktree::run_git(repo, &["push", REMOTE, &format!("refs/heads/{branch}")]),
    )
    .await;
    let push_error = match pushed {
        Ok(Ok(_)) => None,
        Ok(Err(e)) => Some(e.to_string()),
        Err(_) => Some("push timed out".to_string()),
    };
    let fetched = tokio::time::timeout(
        PUSH_TIMEOUT,
        worktree::run_git(
            repo,
            &["fetch", "--quiet", REMOTE, &format!("refs/heads/{branch}")],
        ),
    )
    .await;
    let contains = matches!(fetched, Ok(Ok(_)))
        && worktree::run_git(repo, &["merge-base", "--is-ancestor", &local, "FETCH_HEAD"])
            .await
            .is_ok();
    if contains {
        Ok(())
    } else {
        Err(push_error.unwrap_or_else(|| format!("origin does not have {local}")))
    }
}

/// Turns the start-up fetch's outcome into a take-over verdict: only a state branch that was
/// created, fast-forwarded or already level with a reached origin may proceed. `dir` is where
/// the fetch ran (its FETCH_HEAD holds origin's tip).
pub async fn require_state_synced(dir: &Path, outcome: &FetchOutcome) -> Result<(), TakeOverError> {
    let branch = BRANCH.to_string();
    match outcome {
        FetchOutcome::Created | FetchOutcome::FastForwarded | FetchOutcome::UpToDate => Ok(()),
        FetchOutcome::Diverged => Err(TakeOverError::Diverged {
            origin: worktree::run_git(dir, &["rev-parse", "FETCH_HEAD"])
                .await?
                .trim()
                .to_string(),
            local: local_sha(dir, BRANCH).await,
            branch,
        }),
        FetchOutcome::NoRemoteBranch | FetchOutcome::Failed(_) => {
            let has_origin = worktree::run_git(dir, &["remote", "get-url", REMOTE])
                .await
                .is_ok();
            let reason = match outcome {
                _ if !has_origin => {
                    "no `origin` remote is configured; a take-over needs one".into()
                }
                FetchOutcome::Failed(e) => format!("origin unreachable: {e}"),
                _ => {
                    return Err(TakeOverError::NewProject {
                        branch,
                        local: local_sha(dir, BRANCH).await,
                    });
                }
            };
            Err(TakeOverError::NotFetched {
                local: local_sha(dir, BRANCH).await,
                branch,
                reason,
            })
        }
    }
}

/// Fetches `origin`'s integration branch for a take-over and fast-forwards the local one to
/// it. Refuses unless origin is reached and the local branch is an ancestor of (or equal to)
/// origin's; moving a branch that is checked out in `repo` also needs a clean checkout, never
/// a stash. Fast-forward only: no rebase, no reset, no force. A local branch that is ahead of
/// origin's (commits a refused push held back) is left alone and reported as `true`: the caller
/// pushes it once ownership has moved. Only real divergence is refused.
pub async fn sync_integration_for_take_over(
    repo: &Path,
    branch: &str,
) -> Result<bool, TakeOverError> {
    let local = local_sha(repo, branch).await;
    let fetched = tokio::time::timeout(
        PUSH_TIMEOUT,
        worktree::run_git(
            repo,
            &["fetch", "--quiet", REMOTE, &format!("refs/heads/{branch}")],
        ),
    )
    .await;
    let failure = match fetched {
        Ok(Ok(_)) => None,
        Ok(Err(e)) => Some(format!("fetch of origin failed: {e}")),
        Err(_) => Some("fetch timed out".to_string()),
    };
    if let Some(reason) = failure {
        return Err(TakeOverError::NotFetched {
            branch: branch.to_string(),
            local,
            reason,
        });
    }
    let origin = worktree::run_git(repo, &["rev-parse", "FETCH_HEAD"])
        .await?
        .trim()
        .to_string();
    if local == origin {
        return Ok(false);
    }
    let ahead = local != "none"
        && worktree::run_git(repo, &["merge-base", "--is-ancestor", &origin, &local])
            .await
            .is_ok();
    if ahead {
        return Ok(true);
    }
    let ancestor = local != "none"
        && worktree::run_git(repo, &["merge-base", "--is-ancestor", &local, &origin])
            .await
            .is_ok();
    if !ancestor && local != "none" {
        return Err(TakeOverError::NotAncestor {
            branch: branch.to_string(),
            local,
            origin,
        });
    }
    let checked_out = worktree::run_git(repo, &["symbolic-ref", "--short", "-q", "HEAD"])
        .await
        .is_ok_and(|b| b.trim() == branch);
    if checked_out {
        // Untracked files don't stop a fast-forward; git itself refuses if one would be overwritten.
        let status =
            worktree::run_git(repo, &["status", "--porcelain", "--untracked-files=no"]).await?;
        if !status.trim().is_empty() {
            return Err(TakeOverError::Dirty {
                branch: branch.to_string(),
                dir: repo.display().to_string(),
                origin,
            });
        }
        worktree::run_git(repo, &["merge", "--ff-only", "--quiet", &origin]).await?;
    } else {
        worktree::run_git(repo, &["branch", "-f", branch, &origin]).await?;
    }
    Ok(false)
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
    /// Absent in records written before the field existed; `parse_task` then falls back to
    /// the thread's first entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_by: Option<String>,
    /// Absent in records written before watchers existed (`task_records_watchers`); written
    /// even when empty, so a task nobody watches isn't mistaken for one awaiting backfill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    watchers: Option<Vec<String>>,
    /// Absent in records written before components existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    components: Vec<String>,
    /// Absent in records written before size existed, or with none set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    size: Option<TicketSize>,
    /// Absent in records written before priority existed, or normal.
    #[serde(default, skip_serializing_if = "TicketPriority::is_normal")]
    priority: TicketPriority,
    /// Absent in records written before ordering existed, or never set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    priority_at: Option<DateTime<Utc>>,
    /// Absent in records written before the landing record existed, or unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    /// The ticket the task was made from; absent for a task with none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ticket: Option<String>,
    /// The task this one was split from; absent for an ordinary task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    /// Set while the task is `held`; absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    held_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    held_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    held_until: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    held_at: Option<DateTime<Utc>>,
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
fn render_task(task: &Ticket) -> Result<String, StateBranchError> {
    let fm = Frontmatter {
        id: task.id.clone(),
        title: task.title.clone(),
        kind: task.kind.as_str().to_string(),
        state: task.state.as_str().to_string(),
        created_at: task.created_at,
        updated_at: task.updated_at,
        created_by: (task.created_by != UNKNOWN_CREATOR).then(|| task.created_by.clone()),
        watchers: Some(task.watchers.clone()),
        components: task.components.clone(),
        size: task.size,
        priority: task.priority,
        priority_at: task.priority_at,
        branch: task.branch.clone(),
        commit: task.commit.clone(),
        summary: task.summary.clone(),
        ticket: task.ticket.clone(),
        parent: task.parent.clone(),
        held_by: task.held_by.clone(),
        held_reason: task.held_reason.clone(),
        held_until: task.held_until.clone(),
        held_at: task.held_at,
        impact: task.impact.clone(),
    };
    let toml = toml::to_string_pretty(&fm)?;
    let mut out = String::new();
    out.push_str("+++\n");
    out.push_str(&toml);
    out.push_str("+++\n\n");
    out.push_str(task.body.trim_end());
    out.push('\n');
    if !owns_thread_file(task) {
        out.push_str(&render_thread(&task.thread));
    }
    Ok(out)
}

/// Where a ticket's thread lives on the state branch: `tickets/<id>.md` (22ab section 1).
const THREADS_DIR: &str = "tickets";

/// Where an item's tracking file lives (br-95t9). Not `tickets/`: that holds the thread under the
/// same `<id>.md` name.
const ITEMS_DIR: &str = "tracking";
/// The folder before br-95t9: still read for one release, emptied as items are touched.
const LEGACY_ITEMS_DIR: &str = "tasks";

/// Whether the task's thread lives in `tickets/<id>.md`: the row of a ticket with its own id,
/// which is every row `ticket new` made. A row with another ticket's id (an older task, or an
/// extra task of one ticket) keeps its inline thread, since two rows can't share one file.
fn owns_thread_file(task: &Ticket) -> bool {
    task.ticket.as_deref() == Some(task.id.as_str())
}

/// The `## Thread` section, empty text for no entries.
fn render_thread(thread: &[ThreadEntry]) -> String {
    let mut out = String::new();
    if !thread.is_empty() {
        out.push_str("\n## Thread\n");
        for e in thread {
            out.push_str(&format!(
                "\n### {} \u{b7} {} \u{b7} {}\n{}\n",
                e.kind.as_str(),
                e.from,
                e.at.to_rfc3339_opts(SecondsFormat::Millis, true),
                e.body.trim_end(),
            ));
        }
    }
    out
}

fn render_thread_file(task: &Ticket) -> String {
    format!("# Thread of {}\n{}", task.id, render_thread(&task.thread))
}

fn parse_thread_file(text: &str) -> Result<Vec<ThreadEntry>, StateBranchError> {
    match text.split_once("\n## Thread\n") {
        Some((_, t)) => parse_thread(t, line_of(text, t)),
        None => Ok(Vec::new()),
    }
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

/// The inverse of [`render_task`]. A known limitation: a body containing the
/// literal line `## Thread` confuses the split (noted in docs/design/storage.md). Comment
/// text may hold any heading: only an exact entry-header line starts an entry.
fn parse_task(text: &str) -> Result<Ticket, StateBranchError> {
    let after_open = text
        .strip_prefix("+++\n")
        .ok_or_else(|| StateBranchError::Parse("missing opening +++ frontmatter fence".into()))?;
    let (fm_str, rest) = after_open
        .split_once("\n+++\n")
        .ok_or_else(|| StateBranchError::Parse("missing closing +++ frontmatter fence".into()))?;
    let fm: Frontmatter = toml::from_str(fm_str)?;
    let kind: TicketKind = fm
        .kind
        .parse()
        .map_err(|e: String| StateBranchError::Parse(e))?;
    let state: TicketState = fm
        .state
        .parse()
        .map_err(|e: String| StateBranchError::Parse(e))?;

    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let (body, thread_str) = match rest.split_once("\n## Thread\n") {
        Some((b, t)) => (b, t),
        None => (rest, ""),
    };

    let thread = parse_thread(thread_str, line_of(text, thread_str))?;

    let created_by = fm
        .created_by
        .or_else(|| thread_first_author(&thread))
        .unwrap_or_else(|| UNKNOWN_CREATOR.to_string());
    Ok(Ticket {
        id: fm.id,
        title: fm.title,
        kind,
        state,
        body: body.trim_end_matches('\n').to_string(),
        thread,
        created_at: fm.created_at,
        updated_at: fm.updated_at,
        // The first thread entry is the best guess for a record from before the field: the
        // submitter's note, or whoever planned it.
        created_by,
        watchers: fm.watchers.unwrap_or_default(),
        // Not this file's job: `TicketManager` layers the current claim (from
        // `claims.toml`/SQLite, storage.md) on top of what this parses.
        claimed_by: None,
        claimed_at: None,
        components: fm.components,
        size: fm.size,
        priority: fm.priority,
        priority_at: fm.priority_at,
        branch: fm.branch,
        commit: fm.commit,
        summary: fm.summary,
        ticket: fm.ticket,
        parent: fm.parent,
        impact: fm.impact,
        settle_until: None,
        held_by: fm.held_by,
        held_reason: fm.held_reason,
        held_until: fm.held_until,
        held_at: fm.held_at,
    })
}

/// The 1-based line in `text` where `part`, a suffix slice of it, starts.
fn line_of(text: &str, part: &str) -> usize {
    text[..text.len() - part.len()].matches('\n').count() + 1
}

/// Whether `line` is an entry header exactly as [`render_thread`] writes it:
/// `### <kind> \u{b7} <from> \u{b7} <rfc3339>`. Any other line, `##`/`###` headings included, is
/// comment text: a body is free markdown.
fn parse_entry_header(line: &str) -> Option<(ThreadEntryKind, &str, DateTime<Utc>)> {
    let mut parts = line.strip_prefix("### ")?.split(" \u{b7} ");
    let (kind_s, from, at_s) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let kind = kind_s.parse().ok()?;
    let at = DateTime::parse_from_rfc3339(at_s).ok()?.with_timezone(&Utc);
    Some((kind, from, at))
}

/// `line_base` is the 1-based file line of the text's first line, for error messages.
fn parse_thread(thread_str: &str, line_base: usize) -> Result<Vec<ThreadEntry>, StateBranchError> {
    let mut thread: Vec<ThreadEntry> = Vec::new();
    let mut body: Vec<&str> = Vec::new();
    let finish = |thread: &mut Vec<ThreadEntry>, body: &mut Vec<&str>| {
        if let Some(last) = thread.last_mut() {
            last.body = body.join("\n").trim_end_matches('\n').to_string();
        }
        body.clear();
    };
    for (i, line) in thread_str.lines().enumerate() {
        if let Some((kind, from, at)) = parse_entry_header(line) {
            finish(&mut thread, &mut body);
            thread.push(ThreadEntry {
                kind,
                from: from.to_string(),
                body: String::new(),
                at,
            });
        } else if thread.is_empty() {
            if !line.trim().is_empty() {
                return Err(StateBranchError::Parse(format!(
                    "line {}: text before the first thread entry header: {line:?}",
                    line_base + i
                )));
            }
        } else {
            body.push(line);
        }
    }
    finish(&mut thread, &mut body);
    Ok(thread)
}

fn thread_first_author(thread: &[ThreadEntry]) -> Option<String> {
    thread.first().map(|e| e.from.clone())
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

    fn sample_task(id: &str) -> Ticket {
        let now = Utc::now();
        Ticket {
            id: id.to_string(),
            title: "Add foo".to_string(),
            kind: TicketKind::Feature,
            state: TicketState::Open,
            body: "Some description.\n\nWith a second paragraph.".to_string(),
            thread: vec![],
            created_at: now,
            updated_at: now,
            created_by: "human".to_string(),
            watchers: vec![],
            claimed_by: None,
            claimed_at: None,
            components: vec!["client-games".to_string(), "dungeon".to_string()],
            size: Some(TicketSize::S),
            priority: TicketPriority::High,
            priority_at: None,
            branch: Some("bridle/x".to_string()),
            commit: Some("abc123".to_string()),
            summary: Some("Did a thing.\n\nSecond \"paragraph\".".to_string()),
            ticket: Some("7fa2".to_string()),
            parent: None,
            impact: Impact {
                modify: vec!["s-b310".to_string()],
                add_under: vec!["r-7fa2".to_string()],
                remove: Vec::new(),
                files: vec!["client/**".to_string()],
            },
            settle_until: None,
            held_by: None,
            held_reason: None,
            held_until: None,
            held_at: None,
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
        assert_eq!(parsed.size, Some(TicketSize::S));
        assert_eq!(parsed.priority, TicketPriority::High);
        assert_eq!(parsed.branch, task.branch);
        assert_eq!(parsed.commit, task.commit);
        assert_eq!(parsed.summary, task.summary);
        assert_eq!(parsed.ticket.as_deref(), Some("7fa2"));
        assert_eq!(parsed.created_at, task.created_at);
        assert_eq!(parsed.updated_at, task.updated_at);
        assert_eq!(parsed.created_by, "human");
    }

    #[test]
    fn comment_bodies_with_headings_round_trip() {
        let mut task = sample_task("tw-7fa3");
        let body = "## Implementation complete\n\n### Changes made\n- x\n\n### note \u{b7} fake";
        task.thread = vec![
            ThreadEntry {
                kind: ThreadEntryKind::Note,
                from: "agent-1".to_string(),
                body: body.to_string(),
                at: task.created_at,
            },
            ThreadEntry {
                kind: ThreadEntryKind::Note,
                from: "agent-2".to_string(),
                body: "after".to_string(),
                at: task.created_at,
            },
        ];
        let parsed = parse_task(&render_task(&task).expect("render")).expect("parse");
        assert_eq!(parsed.thread.len(), 2);
        assert_eq!(parsed.thread[0].body, body);
        assert_eq!(parsed.thread[1].from, "agent-2");
    }

    #[test]
    fn a_bad_thread_names_the_line_and_its_text() {
        let text = "+++\nid = \"tw-1\"\ntitle = \"T\"\nkind = \"feature\"\nstate = \"open\"\n\
                    created_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\nbody\n\
                    \n## Thread\n\nstray text\n";
        let err = parse_task(text).expect_err("bad").to_string();
        assert!(
            err.contains("line 14") && err.contains("stray text"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn rebuild_reports_every_bad_file_in_one_pass() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");
        sb.enqueue_task(&sample_task("tw-aaaa")).expect("enqueue");
        sb.flush_now().await.expect("flush");
        for id in ["tw-bad1", "tw-bad2"] {
            std::fs::write(dir.join(format!("tracking/{id}.md")), "not a task\n").expect("write");
        }
        let err = sb.list_tasks().expect_err("bad files").to_string();
        assert!(err.contains("tracking/tw-bad1.md"), "{err}");
        assert!(err.contains("tracking/tw-bad2.md"), "{err}");
    }

    #[test]
    fn a_record_without_created_by_falls_back_to_the_first_thread_entry_then_unknown() {
        let head = "+++\nid = \"tw-1\"\ntitle = \"Old\"\nkind = \"feature\"\nstate = \"open\"\n\
                    created_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\nbody\n";
        assert_eq!(parse_task(head).expect("parse").created_by, UNKNOWN_CREATOR);
        let threaded = format!(
            "{head}\n## Thread\n\n### note \u{b7} agent-9 \u{b7} 2026-01-01T00:00:01.000Z\nhi\n"
        );
        assert_eq!(parse_task(&threaded).expect("parse").created_by, "agent-9");
        // And an unknown creator isn't written out.
        let task = parse_task(head).expect("parse");
        assert!(!render_task(&task).expect("render").contains("created_by"));
    }

    #[test]
    fn a_record_written_before_components_existed_still_loads() {
        let text = "+++\nid = \"tw-1\"\ntitle = \"Old\"\nkind = \"feature\"\nstate = \"open\"\n\
                    created_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\nbody\n";
        let task = parse_task(text).expect("parse old record");
        assert!(task.components.is_empty());
        assert_eq!(task.size, None);
        assert_eq!(task.priority, TicketPriority::Normal);
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

        let path = dir.join("tracking/tw-7fa2.md");
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

    /// A state branch from before br-95t9 keeps its items in `tasks/`: they load, and an item
    /// that is touched is written to `tracking/` and leaves the old folder.
    #[tokio::test]
    async fn items_in_the_old_tasks_folder_load_and_move_when_touched() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");
        sb.enqueue_task(&sample_task("tw-aaaa")).expect("enqueue");
        sb.enqueue_task(&sample_task("tw-bbbb")).expect("enqueue");
        sb.flush_now().await.expect("flush");
        std::fs::create_dir_all(dir.join("tasks")).expect("mkdir");
        for id in ["tw-aaaa", "tw-bbbb"] {
            std::fs::rename(
                dir.join(format!("tracking/{id}.md")),
                dir.join(format!("tasks/{id}.md")),
            )
            .expect("move to the old folder");
        }

        assert!(sb.read_task("tw-aaaa").is_some());
        assert!(sb.task_records_watchers("tw-aaaa"));
        let ids: Vec<_> = sb
            .list_tasks()
            .expect("list")
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, ["tw-aaaa", "tw-bbbb"]);

        sb.enqueue_task(&sample_task("tw-aaaa")).expect("enqueue");
        sb.flush_now().await.expect("flush");
        assert!(dir.join("tracking/tw-aaaa.md").exists());
        assert!(!dir.join("tasks/tw-aaaa.md").exists());
        assert!(dir.join("tasks/tw-bbbb.md").exists(), "untouched stays");
        assert_eq!(sb.list_tasks().expect("list").len(), 2);
    }

    #[tokio::test]
    async fn a_tickets_own_row_keeps_its_thread_in_the_ticket_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let dir = tmp.path().join("state");
        let sb = StateBranch::open(&repo, &dir).await.expect("open");

        let entry = |kind, body: &str| ThreadEntry {
            kind,
            from: "human".to_string(),
            body: body.to_string(),
            at: Utc::now(),
        };
        let mut own = sample_task("br-7fa2");
        own.ticket = Some("br-7fa2".to_string());
        own.thread = vec![
            entry(ThreadEntryKind::Note, "first"),
            entry(ThreadEntryKind::Question, "which?"),
        ];
        // A row of an older task for another ticket keeps its thread inline.
        let mut other = sample_task("br-9999");
        other.ticket = Some("br-7fa2".to_string());
        other.thread = vec![entry(ThreadEntryKind::Note, "old")];
        sb.enqueue_task(&own).expect("enqueue");
        sb.enqueue_task(&other).expect("enqueue");
        sb.flush_now().await.expect("flush");

        let row = std::fs::read_to_string(dir.join("tracking/br-7fa2.md")).expect("row");
        assert!(!row.contains("## Thread"), "{row}");
        let file = std::fs::read_to_string(dir.join("tickets/br-7fa2.md")).expect("thread file");
        assert!(file.contains("which?"), "{file}");
        assert!(!dir.join("tickets/br-9999.md").exists());
        let inline = std::fs::read_to_string(dir.join("tracking/br-9999.md")).expect("row");
        assert!(inline.contains("## Thread"), "{inline}");

        let back = sb.read_task("br-7fa2").expect("read back");
        assert_eq!(back.thread.len(), 2);
        assert_eq!(back.thread[1].kind, ThreadEntryKind::Question);
        assert_eq!(sb.read_task("br-9999").expect("read").thread.len(), 1);
        let listed = sb.list_tasks().expect("list");
        assert_eq!(
            listed
                .iter()
                .find(|t| t.id == "br-7fa2")
                .unwrap()
                .thread
                .len(),
            2
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
    async fn origin_owned_by(tmp: &Path, host: &str) -> PathBuf {
        let (sb, _repo, origin, _dir) = pushing(tmp, Some(Duration::from_millis(0))).await;
        sb.claim_owner(host, Utc::now()).expect("claim");
        edit(&sb, 1).await;
        wait_until_idle(&sb).await;
        origin
    }

    async fn second_clone(tmp: &Path, origin: &Path) -> PathBuf {
        let repo2 = tmp.join("repo2");
        init_repo(&repo2).await;
        git(
            &repo2,
            &["remote", "add", "origin", origin.to_str().expect("utf8")],
        )
        .await;
        repo2
    }

    #[tokio::test]
    async fn another_hosts_project_is_refused_and_says_who() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let origin = origin_owned_by(tmp.path(), "laptop").await;
        let repo2 = second_clone(tmp.path(), &origin).await;
        let err = StateBranch::fetch_and_check_owner(
            &repo2,
            &tmp.path().join("state2"),
            "bridle",
            &["nuc".to_string()],
            false,
        )
        .await
        .expect_err("refused");
        assert_eq!(err.host, "laptop");
        assert!(err.to_string().contains("bridle project take-over"));
    }

    #[tokio::test]
    async fn take_over_claims_and_pushes_the_new_owner() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let origin = origin_owned_by(tmp.path(), "laptop").await;
        let repo2 = second_clone(tmp.path(), &origin).await;
        let dir2 = tmp.path().join("state2");
        let outcome =
            StateBranch::fetch_and_check_owner(&repo2, &dir2, "bridle", &["nuc".to_string()], true)
                .await
                .expect("take over");
        assert_eq!(outcome, FetchOutcome::Created);
        let sb2 = StateBranch::open(&repo2, &dir2)
            .await
            .expect("open")
            .with_push(Duration::from_millis(0));
        sb2.claim_owner("nuc", Utc::now()).expect("claim");
        sb2.flush_now().await.expect("flush");
        wait_until_idle(&sb2).await;
        let owner = git(&origin, &["show", "bridle/state:owner.toml"]).await;
        assert!(owner.contains("host = \"nuc\""), "{owner}");
        // ...and the task history came along.
        assert!(sb2.read_task("tw-0001").is_some());
    }

    #[tokio::test]
    async fn the_owning_host_and_a_first_serve_start_unchanged() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let origin = origin_owned_by(tmp.path(), "laptop").await;
        let repo2 = second_clone(tmp.path(), &origin).await;
        StateBranch::fetch_and_check_owner(
            &repo2,
            &tmp.path().join("s2"),
            "p",
            &["laptop".to_string()],
            false,
        )
        .await
        .expect("same host");
        // No origin at all, then an origin with no state branch or owner file.
        let tmp2 = tempfile::tempdir().expect("tempdir");
        let lone = tmp2.path().join("lone");
        init_repo(&lone).await;
        let out = StateBranch::fetch_and_check_owner(
            &lone,
            &tmp2.path().join("s"),
            "p",
            &["h".to_string()],
            false,
        )
        .await
        .expect("no origin");
        assert!(matches!(
            out,
            FetchOutcome::NoRemoteBranch | FetchOutcome::Failed(_)
        ));
        let (_sb, repo, _origin, dir) = pushing(tmp2.path(), None).await;
        StateBranch::fetch_and_check_owner(&repo, &dir, "p", &["h".to_string()], false)
            .await
            .expect("first serve");
    }

    #[tokio::test]
    async fn a_claim_by_the_recorded_host_makes_no_commit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (sb, _repo, _origin, dir) = pushing(tmp.path(), None).await;
        sb.claim_owner("laptop", Utc::now()).expect("claim");
        sb.flush_now().await.expect("flush");
        let tip = git(&dir, &["rev-parse", "HEAD"]).await;
        sb.claim_owner("laptop", Utc::now()).expect("claim again");
        sb.flush_now().await.expect("flush");
        assert_eq!(git(&dir, &["rev-parse", "HEAD"]).await, tip);
    }

    #[tokio::test]
    async fn a_fresh_clone_adopts_origins_state_even_with_the_branch_only_a_ref() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let origin = origin_owned_by(tmp.path(), "laptop").await;
        let repo2 = second_clone(tmp.path(), &origin).await;
        // A seed-only local branch with no worktree (an earlier start that never flushed).
        worktree::ensure_orphan_branch(&repo2, BRANCH, "initial bridle state")
            .await
            .expect("seed");
        let head = git(&repo2, &["rev-parse", "HEAD"]).await;
        let out = StateBranch::fetch_and_check_owner(
            &repo2,
            &tmp.path().join("state2"),
            "bridle",
            &["laptop".to_string()],
            false,
        )
        .await
        .expect("adopt");
        assert_eq!(out, FetchOutcome::FastForwarded);
        assert_eq!(git(&repo2, &["rev-parse", "HEAD"]).await, head);
        let sb2 = StateBranch::open(&repo2, &tmp.path().join("state2"))
            .await
            .expect("open");
        assert!(sb2.read_task("tw-0001").is_some());
    }

    async fn commit_on(dir: &Path, msg: &str) {
        git(
            dir,
            &[
                "-c",
                "user.email=t@e.com",
                "-c",
                "user.name=T",
                "commit",
                "--allow-empty",
                "-q",
                "-m",
                msg,
            ],
        )
        .await;
    }

    /// `repo2` cloned-ish from `origin` with `main` pushed there from `repo`.
    async fn take_over_pair(tmp: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let origin = origin_owned_by(tmp, "laptop").await;
        let repo = tmp.join("repo");
        git(&repo, &["push", "-q", "origin", "main"]).await;
        let repo2 = tmp.join("repo2");
        git(
            tmp,
            &[
                "clone",
                "-q",
                "-b",
                "main",
                origin.to_str().expect("utf8"),
                "repo2",
            ],
        )
        .await;
        (origin, repo, repo2)
    }

    async fn state_verdict(repo: &Path, state: &Path) -> Result<(), TakeOverError> {
        let out = StateBranch::fetch_and_check_owner(repo, state, "p", &["nuc".to_string()], true)
            .await
            .expect("owner");
        let dir = if state.join(".git").exists() {
            state
        } else {
            repo
        };
        require_state_synced(dir, &out).await
    }

    #[tokio::test]
    async fn take_over_happy_path_fast_forwards_both_branches() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_origin, repo, repo2) = take_over_pair(tmp.path()).await;
        commit_on(&repo, "more").await;
        git(&repo, &["push", "-q", "origin", "main"]).await;
        state_verdict(&repo2, &tmp.path().join("s2"))
            .await
            .expect("state");
        sync_integration_for_take_over(&repo2, "main")
            .await
            .expect("main");
        assert_eq!(
            git(&repo2, &["rev-parse", "main"]).await,
            git(&repo, &["rev-parse", "main"]).await
        );
        // Level already: still fine.
        sync_integration_for_take_over(&repo2, "main")
            .await
            .expect("level");
    }

    #[tokio::test]
    async fn take_over_refuses_a_diverged_state_branch_naming_both_shas() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (origin, _repo, repo2) = take_over_pair(tmp.path()).await;
        let s2 = tmp.path().join("s2");
        let sb = StateBranch::open(&repo2, &s2).await.expect("open");
        drop(sb);
        // A local commit on top of what origin has, and another pushed to origin behind it.
        git(&s2, &["checkout", "-q", "--detach"]).await;
        commit_on(&s2, "local only").await;
        let local = git(&s2, &["rev-parse", "HEAD"]).await;
        git(&s2, &["branch", "-f", "bridle/state", "HEAD"]).await;
        git(&s2, &["checkout", "-q", "bridle/state"]).await;
        let other = tmp.path().join("other");
        git(
            tmp.path(),
            &["clone", "-q", origin.to_str().expect("utf8"), "other"],
        )
        .await;
        git(&other, &["checkout", "-q", "bridle/state"]).await;
        commit_on(&other, "origin only").await;
        git(&other, &["push", "-q", "origin", "bridle/state"]).await;
        let theirs = git(&other, &["rev-parse", "HEAD"]).await;
        let err = state_verdict(&repo2, &s2).await.expect_err("refused");
        let msg = err.to_string();
        assert!(matches!(err, TakeOverError::Diverged { .. }), "{msg}");
        assert!(
            msg.contains(local.trim()) && msg.contains(theirs.trim()),
            "{msg}"
        );
    }

    #[tokio::test]
    async fn take_over_refuses_unreachable_missing_and_no_origin() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // Origin exists but has no bridle/state.
        let (_sb, repo, origin, dir) = pushing(tmp.path(), None).await;
        let err = state_verdict(&repo, &dir).await.expect_err("no branch");
        assert!(matches!(err, TakeOverError::NewProject { .. }), "{err}");
        assert!(err.to_string().contains("bridle project start"), "{err}");
        // It has a local state branch, so it is not new; a fresh clone of the same origin is.
        assert!(!is_new_project(&repo).await.expect("reachable"));
        let fresh = tmp.path().join("fresh");
        git(
            tmp.path(),
            &["clone", "-q", origin.to_str().expect("utf8"), "fresh"],
        )
        .await;
        assert!(is_new_project(&fresh).await.expect("reachable"));
        // Unreachable origin.
        std::fs::remove_dir_all(&origin).expect("rm origin");
        let err = state_verdict(&repo, &dir).await.expect_err("unreachable");
        assert!(matches!(err, TakeOverError::NotFetched { .. }), "{err}");
        assert!(is_new_project(&fresh).await.is_err());
        assert!(sync_integration_for_take_over(&repo, "main").await.is_err());
        // No origin configured at all.
        let lone = tmp.path().join("lone");
        init_repo(&lone).await;
        let err = state_verdict(&lone, &tmp.path().join("ls"))
            .await
            .expect_err("none");
        assert!(err.to_string().contains("no `origin` remote"), "{err}");
    }

    #[tokio::test]
    async fn take_over_refuses_a_diverged_or_dirty_integration_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_origin, repo, repo2) = take_over_pair(tmp.path()).await;
        // Dirty checkout, origin ahead.
        commit_on(&repo, "more").await;
        git(&repo, &["push", "-q", "origin", "main"]).await;
        std::fs::write(repo2.join("f"), "x").expect("write");
        git(&repo2, &["add", "f"]).await;
        let before = git(&repo2, &["rev-parse", "main"]).await;
        let err = sync_integration_for_take_over(&repo2, "main")
            .await
            .expect_err("dirty");
        assert!(matches!(err, TakeOverError::Dirty { .. }), "{err}");
        assert_eq!(git(&repo2, &["rev-parse", "main"]).await, before);
        git(&repo2, &["reset", "-q", "--hard"]).await;
        // Level with origin after a fast-forward, then ahead: a local commit origin lacks is
        // accepted (and left for the caller to push).
        assert!(
            !sync_integration_for_take_over(&repo2, "main")
                .await
                .expect("behind is fast-forwarded")
        );
        commit_on(&repo2, "local").await;
        let ahead = git(&repo2, &["rev-parse", "main"]).await;
        assert!(
            sync_integration_for_take_over(&repo2, "main")
                .await
                .expect("ahead is accepted")
        );
        assert_eq!(git(&repo2, &["rev-parse", "main"]).await, ahead);
        // Diverged: origin gains a commit local lacks.
        commit_on(&repo, "origin side").await;
        git(&repo, &["push", "-q", "origin", "main"]).await;
        let local = ahead;
        let err = sync_integration_for_take_over(&repo2, "main")
            .await
            .expect_err("diverged");
        assert!(matches!(err, TakeOverError::NotAncestor { .. }), "{err}");
        let msg = err.to_string();
        assert!(
            msg.contains(local.trim()) && msg.contains("diverged"),
            "{msg}"
        );
    }
}
