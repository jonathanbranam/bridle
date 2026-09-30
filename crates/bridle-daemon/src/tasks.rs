//! Task records: the SQLite fast index (`crate::store`) and the state
//! branch (`crate::state_branch`) kept in step, plus an in-memory cache that
//! carries the body/thread the database doesn't (docs/design/storage.md).
//! Scoped to this build's states: `open`, `planned`, `claimed`, `dropped`,
//! `reopened`; `in_review`, `integrated` and `accepted` arrive with later
//! tasks.
//!
//! Edges (coordination.md) and `ready` (roles-and-lifecycle.md, "ready is
//! computed") live here too, on the same manager, per that design: an edge
//! is durable the same way a task is (database fast index + state branch,
//! written in the same logical operation), and `ready` needs both the task
//! cache and the edge cache to answer.
//!
//! Claims are durable too now: `claim_task`/`release_task` write to `Store`
//! *and* enqueue the full current claim set to the state branch's
//! `claims.toml`, so `bridle rebuild` restores who's working what (storage.md,
//! "claims"). The claiming agent's own activity (the same signal
//! `supervisor.rs`'s stall check watches) still stands in for a lease
//! renewal, so [`TaskManager::tick_claim_lease_check`] can release a stale
//! claim without a separate heartbeat call.
//!
//! The queue (roles-and-lifecycle.md, "the queue") is a separate record this
//! manager also owns: an ordered list of tiers of task ids, PM-written,
//! durable on the state branch's `queue.toml` alone — there's no SQLite
//! table for it, so [`TaskManager::open`] hydrates the in-memory cache
//! straight from that file, the same way `rebuild_from_state_branch`
//! rebuilds the task/edge/open-question caches from their own files.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bridle_api::types::{
    Edge, EdgeKind, Handover, Impact, MessageKind, MessageState, OpenQuestion, PrincipalId, Task,
    TaskKind, TaskSize, TaskState, ThreadEntry, ThreadEntryKind, When,
};
use chrono::Utc;

/// Shape check only: `r-`/`s-`/`g-`/`a-` plus hex digits.
fn valid_spec_id(id: &str) -> bool {
    id.split_once('-').is_some_and(|(p, hex)| {
        matches!(p, "r" | "s" | "g" | "a")
            && !hex.is_empty()
            && hex.bytes().all(|b| b.is_ascii_hexdigit())
    })
}

use crate::state_branch::{ClaimRecord, StateBranch, StateBranchError};
use crate::store::{NewMessage, RecipientKind, Store, StoreError, TaskRow};

/// A claim's claimant and when it was made; the in-memory mirror of a
/// `claims` row (`Store::list_claims`).
type ClaimEntry = (PrincipalId, chrono::DateTime<Utc>);

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<StoreError> for TaskError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::NotFound(m) => TaskError::NotFound(m),
            StoreError::Conflict(m) => TaskError::Conflict(m),
            other => TaskError::Internal(other.to_string()),
        }
    }
}

impl From<StateBranchError> for TaskError {
    fn from(e: StateBranchError) -> Self {
        TaskError::Internal(e.to_string())
    }
}

#[derive(Clone)]
pub struct TaskManager {
    store: Store,
    state: StateBranch,
    /// The id prefix new tasks get (storage.md: `<prefix>-<4 hex chars>`).
    prefix: String,
    /// Full records, including body/thread, which the database doesn't
    /// carry. Hydrated from the state branch at [`TaskManager::open`], kept
    /// current on every write.
    cache: Arc<Mutex<HashMap<String, Task>>>,
    /// Every edge. Unlike `cache`, the database row *is* the full record
    /// (there's no extra body/thread to hydrate from the state branch), so
    /// this is loaded straight from `Store::list_edges` at `open`.
    edges: Arc<Mutex<Vec<Edge>>>,
    /// Task id -> the id of its open question's message, for the tasks that
    /// currently have one. Mirrors `Store::list_open_questions`, loaded at
    /// `open`, so `has_open_questions` (called from the sync `is_ready`) can
    /// answer without a database round trip.
    open_questions: Arc<Mutex<HashMap<String, String>>>,
    /// Task id -> (claimant, claimed at), for every currently claimed task.
    /// Mirrors `Store::list_claims`, loaded at `open`. Like
    /// `open_questions` it is a SQLite table, but the whole set is also
    /// mirrored to `claims.toml` on the state branch and restored by a
    /// rebuild (storage.md); this is also the source of
    /// `Task::claimed_by`/`claimed_at` on the cached task.
    claims: Arc<Mutex<HashMap<String, ClaimEntry>>>,
    /// How long a claim survives without the claiming agent's own activity
    /// before [`TaskManager::tick_claim_lease_check`] releases it.
    claim_lease_after: chrono::Duration,
    /// The queue: an ordered list of tiers, each a list of task ids
    /// (roles-and-lifecycle.md, "the queue"). Loaded from `queue.toml` at
    /// [`TaskManager::open`]; there's no SQLite table backing it, so this
    /// cache — kept current on every [`TaskManager::set_queue`] — is the
    /// only in-memory copy.
    queue: Arc<Mutex<Vec<Vec<String>>>>,
}

impl TaskManager {
    /// Loads every task the database knows about, hydrating each from its
    /// state-branch file. A task whose file is missing (a crash between
    /// `insert_task` and the first flush that would have written it) falls
    /// back to an empty body/thread built from the database row alone,
    /// rather than failing daemon startup over it.
    pub async fn open(
        store: Store,
        state: StateBranch,
        prefix: String,
        claim_lease_after: std::time::Duration,
    ) -> Result<Self, TaskError> {
        let rows = store.list_tasks().await?;
        let mut cache = HashMap::with_capacity(rows.len());
        for row in rows {
            let task = state.read_task(&row.id).unwrap_or(Task {
                id: row.id,
                title: row.title,
                kind: row.kind,
                state: row.state,
                body: String::new(),
                thread: Vec::new(),
                created_at: row.created_at,
                updated_at: row.updated_at,
                claimed_by: None,
                claimed_at: None,
                components: Vec::new(),
                size: None,
                branch: None,
                commit: None,
                summary: None,
                impact: Impact::default(),
            });
            cache.insert(task.id.clone(), task);
        }
        let edges = store.list_edges().await?;
        let open_questions = store
            .list_open_questions()
            .await?
            .into_iter()
            .map(|q| (q.task_id, q.message_id))
            .collect();
        // A claim on a task that isn't `claimed` (say, left behind by an
        // older `drop`) is stale: drop the row rather than list the task as
        // claimed forever.
        let mut claims: HashMap<String, ClaimEntry> = HashMap::new();
        for c in store.list_claims().await? {
            if cache
                .get(&c.task_id)
                .is_some_and(|t| t.state == TaskState::Claimed)
            {
                claims.insert(c.task_id, (c.claimed_by, c.claimed_at));
            } else {
                store.delete_claim(&c.task_id).await?;
            }
        }
        for (task_id, (claimant, claimed_at)) in &claims {
            if let Some(task) = cache.get_mut(task_id) {
                task.claimed_by = Some(claimant.clone());
                task.claimed_at = Some(*claimed_at);
            }
        }
        let queue = state.read_queue();
        Ok(TaskManager {
            store,
            state,
            prefix,
            cache: Arc::new(Mutex::new(cache)),
            edges: Arc::new(Mutex::new(edges)),
            open_questions: Arc::new(Mutex::new(open_questions)),
            claims: Arc::new(Mutex::new(claims)),
            claim_lease_after: chrono::Duration::from_std(claim_lease_after)
                .unwrap_or_else(|_| chrono::Duration::zero()),
            queue: Arc::new(Mutex::new(queue)),
        })
    }

    /// Reconstructs `tasks`, `edges`, `open_questions` and `claims` from the
    /// state branch alone: the migration path for a fresh clone with no
    /// `bridle.db` (docs/design/overview.md, "`bridle rebuild` recreates the
    /// database from the project's state branch"). Refuses — rather than
    /// silently overwriting — if the database already has any rows in these
    /// tables; a rebuild is a from-nothing reconstruction, not a merge. The
    /// queue cache is re-read from `queue.toml` too, the same as any other
    /// startup, since it never lived in SQLite to begin with (there's
    /// nothing to refuse-over for it).
    ///
    /// An open question's `message_id` in the rebuilt `open_questions` row
    /// doesn't point at a real `messages` row: messages don't survive on the
    /// state branch at all (overview.md's durability table lists them as
    /// SQLite-only, "no durability, by design"), only the thread entry that
    /// records the question's body/from/timestamp does. Rather than leave
    /// the column unfillable, this synthesizes a stand-in id from the task
    /// id; nothing downstream looks a message up by this id after a
    /// rebuild, since there's no message row behind it to find.
    pub async fn rebuild_from_state_branch(&self) -> Result<(), TaskError> {
        // Handover notes aren't in the refusal check below (a note alone is no reason to
        // refuse); `restore_handover` skips any already there.
        for h in self.state.list_handovers()? {
            self.store.restore_handover(&h).await?;
        }
        if !self.store.list_tasks().await?.is_empty()
            || !self.store.list_edges().await?.is_empty()
            || !self.store.list_open_questions().await?.is_empty()
            || !self.store.list_claims().await?.is_empty()
        {
            return Err(TaskError::Conflict(
                "database already has tasks, edges, open questions or claims; refusing to rebuild over it"
                    .to_string(),
            ));
        }

        let tasks = self.state.list_tasks()?;
        let mut cache = HashMap::with_capacity(tasks.len());
        let mut open_questions = HashMap::new();
        for task in tasks {
            self.store
                .insert_task_row(&TaskRow {
                    id: task.id.clone(),
                    title: task.title.clone(),
                    kind: task.kind,
                    state: task.state,
                    created_at: task.created_at,
                    updated_at: task.updated_at,
                })
                .await?;
            if let Some(entry) = task
                .thread
                .iter()
                .rev()
                .find(|e| matches!(e.kind, ThreadEntryKind::Question | ThreadEntryKind::Answer))
                && entry.kind == ThreadEntryKind::Question
            {
                let message_id = format!("m-rebuilt-{}", task.id);
                self.store
                    .insert_open_question(&task.id, &message_id, &entry.from, entry.at)
                    .await?;
                open_questions.insert(task.id.clone(), message_id);
            }
            cache.insert(task.id.clone(), task);
        }

        let edges = self.state.list_edges()?;
        for edge in &edges {
            self.store.insert_edge_row(edge).await?;
        }

        let claim_records = self.state.list_claims()?;
        let mut claims = HashMap::with_capacity(claim_records.len());
        for c in &claim_records {
            self.store
                .insert_claim(&c.task_id, &c.claimed_by, c.claimed_at)
                .await?;
            // `claim_task` sets `tasks.state = Claimed` synchronously in
            // SQLite, never on the state branch (storage.md, "claims"): the
            // task file this rebuild just restored the row from still says
            // whatever pre-claim state it was last flushed at (`planned`),
            // so this mirrors that same synchronous update here too.
            self.store
                .set_task_state(&c.task_id, TaskState::Claimed)
                .await?;
            if let Some(task) = cache.get_mut(&c.task_id) {
                task.state = TaskState::Claimed;
                task.claimed_by = Some(c.claimed_by.clone());
                task.claimed_at = Some(c.claimed_at);
            }
            claims.insert(c.task_id.clone(), (c.claimed_by.clone(), c.claimed_at));
        }

        *self.cache.lock().expect("task cache lock") = cache;
        *self.edges.lock().expect("edges lock") = edges;
        *self.open_questions.lock().expect("open questions lock") = open_questions;
        *self.claims.lock().expect("claims lock") = claims;
        *self.queue.lock().expect("queue lock") = self.state.read_queue();
        Ok(())
    }

    /// Queues a handover note for the state branch's next flush.
    pub fn enqueue_handover(&self, h: &Handover) -> Result<(), TaskError> {
        Ok(self.state.enqueue_handover(h)?)
    }

    /// Writes every note in SQLite that the state branch doesn't have yet (notes from before
    /// this existed). Idempotent: a note already on the branch is skipped.
    pub async fn backfill_handovers(&self) -> Result<(), TaskError> {
        for h in self.store.list_handovers().await? {
            if !self.state.has_handover(&h.id) {
                self.state.enqueue_handover(&h)?;
            }
        }
        Ok(())
    }

    /// `bridle rebuild --from-origin`: see [`StateBranch::fetch_from_origin`].
    pub async fn fetch_state_from_origin(&self) -> crate::state_branch::FetchOutcome {
        StateBranch::fetch_from_origin(self.state.dir()).await
    }

    fn put(&self, task: Task) -> Task {
        self.cache
            .lock()
            .expect("task cache lock")
            .insert(task.id.clone(), task.clone());
        task
    }

    pub async fn new_task(
        &self,
        title: &str,
        kind: TaskKind,
        body: String,
        components: Vec<String>,
        size: Option<TaskSize>,
    ) -> Result<Task, TaskError> {
        if title.trim().is_empty() {
            return Err(TaskError::BadRequest("title must not be empty".to_string()));
        }
        let row = self.store.insert_task(&self.prefix, title, kind).await?;
        let task = Task {
            id: row.id,
            title: row.title,
            kind: row.kind,
            state: row.state,
            body,
            thread: Vec::new(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            claimed_by: None,
            claimed_at: None,
            components,
            size,
            branch: None,
            commit: None,
            summary: None,
            impact: Impact::default(),
        };
        self.state.enqueue_task(&task)?;
        self.state
            .enqueue_event(&task.id, "", task.state.as_str(), "human", task.created_at);
        Ok(self.put(task))
    }

    pub fn get_task(&self, id: &str) -> Option<Task> {
        self.cache.lock().expect("task cache lock").get(id).cloned()
    }

    pub fn list_tasks(&self) -> Vec<Task> {
        let cache = self.cache.lock().expect("task cache lock");
        let mut tasks: Vec<Task> = cache.values().cloned().collect();
        tasks.sort_by_key(|t| t.created_at);
        tasks
    }

    /// Changes `title`, `body`, `components` and/or `size`. Neither changes the task's state.
    pub async fn edit_task(
        &self,
        id: &str,
        title: Option<String>,
        body: Option<String>,
        components: Option<Vec<String>>,
        size: Option<TaskSize>,
    ) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        if let Some(title) = &title
            && title.trim().is_empty()
        {
            return Err(TaskError::BadRequest("title must not be empty".to_string()));
        }
        if title.is_none() && body.is_none() && components.is_none() && size.is_none() {
            return Ok(task);
        }
        if let Some(title) = title {
            self.store.set_task_title(id, &title).await?;
            task.title = title;
        }
        if let Some(body) = body {
            task.body = body;
        }
        if let Some(components) = components {
            task.components = components;
        }
        if let Some(size) = size {
            task.size = if size == TaskSize::None {
                None
            } else {
                Some(size)
            };
        }
        task.updated_at = Utc::now();
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `open` -> `planned`: the PM has decided this task is ready to build.
    /// Fails with `Conflict` if the task isn't `open` (including a task
    /// that's already `planned`, `claimed`, or reachable only by `reopen`
    /// once dropped).
    pub async fn plan_task(&self, id: &str, actor: &PrincipalId) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        // A recurred incident goes active again from `reopened`.
        let recurred = task.kind == TaskKind::Incident && task.state == TaskState::Reopened;
        if task.state != TaskState::Open && !recurred {
            return Err(TaskError::Conflict(format!(
                "task {id} is {}, not open; only an open task can be planned",
                task.state
            )));
        }
        self.transition(&mut task, TaskState::Planned, actor)
            .await?;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `open`, `planned` or `claimed` -> `dropped` (releasing any claim), recording `reason` in the thread.
    pub async fn drop_task(
        &self,
        id: &str,
        reason: &str,
        actor: &PrincipalId,
    ) -> Result<Task, TaskError> {
        if reason.trim().is_empty() {
            return Err(TaskError::BadRequest(
                "dropping a task requires a reason".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        if task.state == TaskState::Dropped {
            return Err(TaskError::Conflict(format!("task {id} is already dropped")));
        }
        if task.state == TaskState::Claimed {
            self.clear_claim(id).await?;
            task.claimed_by = None;
            task.claimed_at = None;
        }
        self.transition(&mut task, TaskState::Dropped, actor)
            .await?;
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: format!("dropped: {reason}"),
            at: task.updated_at,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `open`, `planned` or `claimed` -> `integrated`, recording the merge
    /// `commit` in the thread. A claim is released first. `blocks` edges
    /// out of the task resolve, and it leaves the queue and `ready`.
    pub async fn done_task(
        &self,
        id: &str,
        commit: &str,
        branch: Option<&str>,
        actor: &PrincipalId,
    ) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        // A human to-do has no code to land, so the human finishes it bare.
        let bare_ok = (task.state == TaskState::Claimed
            && task.claimed_by.as_deref() == Some("human"))
            || task.kind == TaskKind::Incident;
        if commit.trim().is_empty() && !bare_ok {
            return Err(TaskError::BadRequest(
                "marking a task done requires a commit".to_string(),
            ));
        }
        match task.state {
            TaskState::Dropped | TaskState::Integrated => {
                return Err(TaskError::Conflict(format!(
                    "task {id} is already {}",
                    task.state
                )));
            }
            TaskState::Claimed => task = self.release_claim(id).await?,
            _ => {}
        }
        self.transition(&mut task, TaskState::Integrated, actor)
            .await?;
        let branch = branch.map(str::trim).filter(|b| !b.is_empty());
        let commit = commit.trim();
        task.commit = (!commit.is_empty()).then(|| commit.to_string());
        task.branch = branch.map(str::to_string);
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: match (commit.is_empty(), branch) {
                (true, _) => "done".to_string(),
                (false, Some(b)) => format!("integrated: {commit} (branch {b})"),
                (false, None) => format!("integrated: {commit}"),
            },
            at: task.updated_at,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Records how the task was implemented, replacing any earlier summary.
    /// Allowed in any state: a worker writes it before the manager marks
    /// the task done.
    pub async fn set_summary(&self, id: &str, text: &str) -> Result<Task, TaskError> {
        if text.trim().is_empty() {
            return Err(TaskError::BadRequest(
                "summary must not be empty".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        task.summary = Some(text.trim().to_string());
        task.updated_at = Utc::now();
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Replaces the task's declared impact (impact-and-conflicts.md). Only
    /// while the task can still be worked: open, planned or claimed.
    pub async fn set_impact(&self, id: &str, impact: Impact) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        if !matches!(
            task.state,
            TaskState::Open | TaskState::Planned | TaskState::Claimed
        ) {
            return Err(TaskError::Conflict(format!(
                "task {id} is {}; impact can only be set on an open, planned or claimed task",
                task.state
            )));
        }
        for id in impact
            .modify
            .iter()
            .chain(&impact.add_under)
            .chain(&impact.remove)
        {
            if !valid_spec_id(id) {
                return Err(TaskError::BadRequest(format!(
                    "bad spec id {id:?}: expected r-, s-, g- or a- plus hex digits"
                )));
            }
        }
        if impact.files.iter().any(|f| f.trim().is_empty()) {
            return Err(TaskError::BadRequest(
                "file globs must not be empty".to_string(),
            ));
        }
        task.impact = impact;
        task.updated_at = Utc::now();
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `dropped` or `integrated` -> `reopened`. Any other starting state is a conflict.
    pub async fn reopen_task(&self, id: &str, actor: &PrincipalId) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        if !matches!(task.state, TaskState::Dropped | TaskState::Integrated) {
            return Err(TaskError::Conflict(format!(
                "task {id} is {}; only a dropped or integrated task can be reopened",
                task.state
            )));
        }
        self.transition(&mut task, TaskState::Reopened, actor)
            .await?;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    async fn transition(
        &self,
        task: &mut Task,
        to: TaskState,
        actor: &PrincipalId,
    ) -> Result<(), TaskError> {
        let from = task.state;
        self.store.set_task_state(&task.id, to).await?;
        task.state = to;
        task.updated_at = Utc::now();
        self.state
            .enqueue_event(&task.id, from.as_str(), to.as_str(), actor, task.updated_at);
        Ok(())
    }

    // ---------- edges ----------

    /// Adds a `from`-`kind`->`to` edge. Both tasks must exist, and an edge
    /// can't connect a task to itself; the underlying unique constraint
    /// (coordination.md's table has no notion of a duplicate edge) turns a
    /// repeat of the same triple into a conflict.
    pub async fn add_edge(&self, from: &str, to: &str, kind: EdgeKind) -> Result<Edge, TaskError> {
        if from == to {
            return Err(TaskError::BadRequest(
                "an edge cannot connect a task to itself".to_string(),
            ));
        }
        if self.get_task(from).is_none() {
            return Err(TaskError::NotFound(format!("no such task: {from}")));
        }
        if self.get_task(to).is_none() {
            return Err(TaskError::NotFound(format!("no such task: {to}")));
        }
        let edge = self.store.insert_edge(from, to, kind).await?;
        let edges = {
            let mut g = self.edges.lock().expect("edges lock");
            g.push(edge.clone());
            g.clone()
        };
        self.state.enqueue_edges(&edges)?;
        Ok(edge)
    }

    /// Removes the edge identified by the `(from, to, kind)` triple; not
    /// found if no such edge exists.
    pub async fn remove_edge(&self, from: &str, to: &str, kind: EdgeKind) -> Result<(), TaskError> {
        self.store.delete_edge(from, to, kind).await?;
        let edges = {
            let mut g = self.edges.lock().expect("edges lock");
            g.retain(|e| !(e.from == from && e.to == to && e.kind == kind));
            g.clone()
        };
        self.state.enqueue_edges(&edges)?;
        Ok(())
    }

    pub fn list_edges(&self) -> Vec<Edge> {
        self.edges.lock().expect("edges lock").clone()
    }

    // ---------- ready ----------

    /// A blocker is unresolved unless it's `dropped` or `integrated`:
    /// `accepted` doesn't exist yet, so for now anything else — including a
    /// blocker this manager doesn't know about — still counts as blocking
    /// (roles-and-lifecycle.md, "ready is computed"). Documented here rather
    /// than left implicit, since it's a deliberate simplification this
    /// build makes, not the final rule.
    fn blocker_is_resolved(&self, blocker_id: &str) -> bool {
        self.get_task(blocker_id)
            .is_some_and(|b| matches!(b.state, TaskState::Dropped | TaskState::Integrated))
    }

    /// `planned`, no open `blocks` edge naming an unresolved blocker, and no
    /// unanswered question.
    pub fn is_ready(&self, task: &Task) -> bool {
        // An incident is nobody's to build or claim (incidents.md).
        if task.state != TaskState::Planned || task.kind == TaskKind::Incident {
            return false;
        }
        if self.has_open_questions(task) {
            return false;
        }
        let edges = self.edges.lock().expect("edges lock");
        !edges.iter().any(|e| {
            e.kind == EdgeKind::Blocks && e.to == task.id && !self.blocker_is_resolved(&e.from)
        })
    }

    /// `true` while `task` has an unanswered question (docs/design/coordination.md,
    /// "Questions do not stop work").
    fn has_open_questions(&self, task: &Task) -> bool {
        self.open_questions
            .lock()
            .expect("open questions lock")
            .contains_key(&task.id)
    }

    // ---------- questions ----------

    /// Asks a question against `task`: appends a `question` thread entry,
    /// inserts the durable message (`to_kind` `task`, so it's addressed to
    /// the task rather than an agent or `human`), and indexes the task as
    /// blocked until answered. Fails with `Conflict` if the task already has
    /// an open question — one at a time, per `open_questions`'s primary key.
    pub async fn ask_question(
        &self,
        id: &str,
        from: &PrincipalId,
        body: &str,
    ) -> Result<Task, TaskError> {
        if body.trim().is_empty() {
            return Err(TaskError::BadRequest(
                "a question requires a body".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        // Checked here too, not just left to the store's unique-constraint
        // conflict below: this cache is the source `is_ready` reads, so a
        // question already recorded here must not have its store insert
        // half-succeed while this half is skipped.
        if self.has_open_questions(&task) {
            return Err(TaskError::Conflict(format!(
                "task {id} already has an open question"
            )));
        }
        let message = self
            .store
            .insert_message(NewMessage {
                from: from.clone(),
                to: id.to_string(),
                to_kind: RecipientKind::Task,
                kind: MessageKind::Question,
                body: body.to_string(),
                reply_to: None,
                when: When::Now,
                // Delivery here means "durably attached to the task", which
                // already happened synchronously (this message *is* the
                // thread entry below); there's no agent stdin to write to
                // and no further transition this state machine models.
                state: MessageState::Delivered,
            })
            .await?;
        self.store
            .insert_open_question(id, &message.id, from, message.created_at)
            .await?;
        self.open_questions
            .lock()
            .expect("open questions lock")
            .insert(id.to_string(), message.id.clone());
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Question,
            from: from.clone(),
            body: body.to_string(),
            at: message.created_at,
        });
        task.updated_at = message.created_at;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Answers `task`'s open question: appends an `answer` thread entry,
    /// clears the open-questions index, and re-enables readiness. Fails
    /// with `Conflict` if the task has no open question.
    pub async fn answer_question(
        &self,
        id: &str,
        from: &PrincipalId,
        body: &str,
    ) -> Result<Task, TaskError> {
        if body.trim().is_empty() {
            return Err(TaskError::BadRequest(
                "an answer requires a body".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        let question_message_id = self
            .open_questions
            .lock()
            .expect("open questions lock")
            .get(id)
            .cloned()
            .ok_or_else(|| TaskError::Conflict(format!("task {id} has no open question")))?;
        let message = self
            .store
            .insert_message(NewMessage {
                from: from.clone(),
                to: id.to_string(),
                to_kind: RecipientKind::Task,
                kind: MessageKind::Answer,
                body: body.to_string(),
                reply_to: Some(question_message_id),
                when: When::Now,
                state: MessageState::Delivered,
            })
            .await?;
        self.store.delete_open_question(id).await?;
        self.open_questions
            .lock()
            .expect("open questions lock")
            .remove(id);
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Answer,
            from: from.clone(),
            body: body.to_string(),
            at: message.created_at,
        });
        task.updated_at = message.created_at;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Adds a plain note to `task`'s thread: no open-question bookkeeping,
    /// no effect on readiness (docs/design/coordination.md, Messages). The
    /// smallest useful message-to-a-task primitive; `ask_question` builds
    /// the blocking variant on the same shape.
    pub async fn note_task(
        &self,
        id: &str,
        from: &PrincipalId,
        body: &str,
    ) -> Result<Task, TaskError> {
        if body.trim().is_empty() {
            return Err(TaskError::BadRequest("a note requires a body".to_string()));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        let message = self
            .store
            .insert_message(NewMessage {
                from: from.clone(),
                to: id.to_string(),
                to_kind: RecipientKind::Task,
                kind: MessageKind::Note,
                body: body.to_string(),
                reply_to: None,
                when: When::Now,
                state: MessageState::Delivered,
            })
            .await?;
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: from.clone(),
            body: body.to_string(),
            at: message.created_at,
        });
        task.updated_at = message.created_at;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    // ---------- claims ----------

    /// Enqueues the full current claim set to the state branch's
    /// `claims.toml`, mirroring [`TaskManager::claims`] the way
    /// `add_edge`/`remove_edge` mirror the edge cache to `edges.toml`. Called
    /// with the claims lock already released, after every claim/release, so
    /// `bridle rebuild` can restore current claims (storage.md, "claims").
    fn enqueue_claims_snapshot(&self) -> Result<(), TaskError> {
        let records: Vec<ClaimRecord> = self
            .claims
            .lock()
            .expect("claims lock")
            .iter()
            .map(|(task_id, (claimed_by, claimed_at))| ClaimRecord {
                task_id: task_id.clone(),
                claimed_by: claimed_by.clone(),
                claimed_at: *claimed_at,
            })
            .collect();
        Ok(self.state.enqueue_claims(&records)?)
    }

    /// Claims `id` for `by`: `planned` -> `claimed`. Fails with `Conflict`
    /// if the task isn't ready right now — not planned, blocked, or already
    /// claimed (claiming again would need it to still be `planned`, which
    /// `is_ready` already requires). Writes synchronously to the database,
    /// like every other task state change, and enqueues the claim set's new
    /// state to the state branch for the next flush (storage.md, "claims").
    pub async fn claim_task(&self, id: &str, by: &PrincipalId) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        if !self.is_ready(&task) {
            return Err(TaskError::Conflict(format!(
                "task {id} is not ready to claim"
            )));
        }
        let now = Utc::now();
        self.store.insert_claim(id, by, now).await?;
        self.store.set_task_state(id, TaskState::Claimed).await?;
        self.claims
            .lock()
            .expect("claims lock")
            .insert(id.to_string(), (by.clone(), now));
        self.enqueue_claims_snapshot()?;
        task.state = TaskState::Claimed;
        task.updated_at = now;
        task.claimed_by = Some(by.clone());
        task.claimed_at = Some(now);
        Ok(self.put(task))
    }

    /// Releases `id`'s claim: `claimed` -> `planned`. Fails with `Conflict`
    /// if `by` isn't the current claimant (including if the task isn't
    /// claimed at all).
    pub async fn release_task(&self, id: &str, by: &PrincipalId) -> Result<Task, TaskError> {
        {
            let claims = self.claims.lock().expect("claims lock");
            match claims.get(id) {
                Some((claimant, _)) if claimant == by => {}
                Some(_) => {
                    return Err(TaskError::Conflict(format!(
                        "task {id} is claimed by another agent"
                    )));
                }
                None => {
                    return Err(TaskError::Conflict(format!("task {id} is not claimed")));
                }
            }
        }
        self.release_claim(id).await
    }

    /// Removes `id`'s claim row, in-memory entry and state-branch record,
    /// leaving the task's state to the caller.
    async fn clear_claim(&self, id: &str) -> Result<(), TaskError> {
        self.store.delete_claim(id).await?;
        self.claims.lock().expect("claims lock").remove(id);
        self.enqueue_claims_snapshot()
    }

    /// The shared release path for an explicit `release_task` and automatic
    /// lease expiry: clears the claim and transitions the task back to
    /// `planned`, enqueuing the claim set's new state to the state branch
    /// the same way [`TaskManager::claim_task`] does.
    async fn release_claim(&self, id: &str) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        if task.state != TaskState::Claimed {
            return Err(TaskError::Conflict(format!("task {id} is not claimed")));
        }
        self.clear_claim(id).await?;
        self.store.set_task_state(id, TaskState::Planned).await?;
        task.state = TaskState::Planned;
        task.updated_at = Utc::now();
        task.claimed_by = None;
        task.claimed_at = None;
        Ok(self.put(task))
    }

    /// Releases any claim whose claiming agent has gone quiet past
    /// `claim_lease_after`, using the same activity signal the supervisor's
    /// stall check watches (`last_event_at.or(turn_started_at)`,
    /// `supervisor.rs::tick_stall_check`) rather than a separate renewal
    /// call: a claim is alive as long as its claimant is. `now` is passed in
    /// rather than read live, so tests can inject a stale claimant without
    /// waiting out the real lease.
    pub async fn tick_claim_lease_check(&self, now: chrono::DateTime<Utc>) {
        let claimed: Vec<(String, PrincipalId)> = self
            .claims
            .lock()
            .expect("claims lock")
            .iter()
            .map(|(id, (by, _))| (id.clone(), by.clone()))
            .collect();
        for (task_id, claimant) in claimed {
            // Agent principals are `agent:<name>` (principals.md); `agents`
            // rows are keyed by id/name, not this prefixed form (mirrors the
            // same lookup in server.rs).
            let name = claimant.strip_prefix("agent:").unwrap_or(&claimant);
            let Ok(Some(agent)) = self.store.get_agent(name).await else {
                // Not an agent claimant (e.g. `human`), or a lookup error:
                // nothing to measure activity against, so leave the claim as
                // is rather than guessing.
                continue;
            };
            let Some(last) = agent.last_event_at.or(agent.turn_started_at) else {
                continue;
            };
            if now - last < self.claim_lease_after {
                continue;
            }
            // Only a `claimed` task moves back to `planned`; anything else
            // (e.g. dropped) has a stale claim that isn't ours to act on.
            let _ = self.release_claim(&task_id).await;
        }
    }

    /// Incidents that are `planned`, oldest first, for `bridle status`.
    pub fn active_incidents(&self) -> Vec<bridle_api::types::IncidentSummary> {
        self.list_tasks()
            .into_iter()
            .filter(|t| t.kind == TaskKind::Incident && t.state == TaskState::Planned)
            .map(|t| bridle_api::types::IncidentSummary {
                id: t.id,
                title: t.title,
                since: t.updated_at,
            })
            .collect()
    }

    pub fn ready_tasks(&self) -> Vec<Task> {
        self.list_tasks()
            .into_iter()
            .filter(|t| self.is_ready(t))
            .collect()
    }

    // ---------- queue ----------

    /// The queue's tiers in rank order, each a list of task ids
    /// (roles-and-lifecycle.md, "the queue"). Empty until the PM sets one; a
    /// task not listed in any tier is backlog.
    pub fn queue_tiers(&self) -> Vec<Vec<String>> {
        self.queue.lock().expect("queue lock").clone()
    }

    /// [`TaskManager::queue_tiers`] without integrated tasks (and any tier
    /// that leaves empty): merged work is done, not queued. The stored
    /// queue keeps them until the PM's next `set_queue`.
    pub fn live_queue_tiers(&self) -> Vec<Vec<String>> {
        self.queue_tiers()
            .into_iter()
            .map(|tier| {
                tier.into_iter()
                    .filter(|id| {
                        self.get_task(id)
                            .is_none_or(|t| t.state != TaskState::Integrated)
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|tier| !tier.is_empty())
            .collect()
    }

    /// Validates `tiers` against the task cache: every id must name a task
    /// that exists, no id may repeat across tiers (a task holds one rank),
    /// and no tier may be empty (an empty tier isn't a rank, it's nothing).
    fn validate_queue(&self, tiers: &[Vec<String>]) -> Result<(), TaskError> {
        let mut seen = std::collections::HashSet::new();
        for tier in tiers {
            if tier.is_empty() {
                return Err(TaskError::BadRequest(
                    "a tier must name at least one task".to_string(),
                ));
            }
            for id in tier {
                let Some(task) = self.get_task(id) else {
                    return Err(TaskError::NotFound(format!("no such task: {id}")));
                };
                if task.kind == TaskKind::Incident {
                    return Err(TaskError::BadRequest(format!(
                        "task {id} is an incident; incidents aren't queued"
                    )));
                }
                if !seen.insert(id) {
                    return Err(TaskError::Conflict(format!(
                        "task {id} appears more than once in the queue"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Replaces the whole queue: the PM's one write primitive, covering
    /// reorder/add/remove alike (resend the full tier list in the shape it
    /// should be). Durable the same way a task edit is: enqueued for the
    /// state branch's next flush, with `actor` recorded on the accompanying
    /// event line, and the in-memory cache updated immediately so a
    /// following read sees it before that flush happens.
    pub async fn set_queue(
        &self,
        tiers: Vec<Vec<String>>,
        actor: &PrincipalId,
    ) -> Result<Vec<Vec<String>>, TaskError> {
        self.validate_queue(&tiers)?;
        self.state.enqueue_queue(&tiers)?;
        self.state.enqueue_queue_event(actor, Utc::now());
        *self.queue.lock().expect("queue lock") = tiers.clone();
        Ok(tiers)
    }

    /// Appends one new tier, ranked after every existing one: sugar over
    /// `set_queue` for the common case of adding work at the back of the
    /// queue without resending the tiers already there.
    pub async fn add_queue_tier(
        &self,
        tasks: Vec<String>,
        actor: &PrincipalId,
    ) -> Result<Vec<Vec<String>>, TaskError> {
        let mut tiers = self.queue_tiers();
        tiers.push(tasks);
        self.set_queue(tiers, actor).await
    }

    /// The highest tier with at least one startable task (ready: planned,
    /// deps met, no open question, and — since `is_ready` requires
    /// `Planned` — necessarily unclaimed), and just that tier's startable
    /// tasks. Empty if no tier has one, including when the queue itself is
    /// empty: the manager takes from the next tier down rather than idling
    /// on a blocked one (roles-and-lifecycle.md, "the queue"), and a task
    /// outside the queue entirely is backlog, never returned here.
    pub fn highest_startable_tier(&self) -> Vec<Task> {
        let ready_ids: std::collections::HashSet<String> =
            self.ready_tasks().into_iter().map(|t| t.id).collect();
        for tier in self.queue_tiers() {
            let startable: Vec<Task> = tier
                .iter()
                .filter(|id| ready_ids.contains(*id))
                .filter_map(|id| self.get_task(id))
                .collect();
            if !startable.is_empty() {
                return startable;
            }
        }
        Vec::new()
    }

    /// Every task with an unanswered question, oldest first: for `bridle
    /// inbox` (coordination.md, "Questions do not stop work"). Reads the
    /// asker/body/age off the cached task's thread rather than the store, the
    /// same way the rest of this manager avoids a database round trip for
    /// data it already holds.
    pub fn list_open_questions(&self) -> Vec<OpenQuestion> {
        let open = self.open_questions.lock().expect("open questions lock");
        let cache = self.cache.lock().expect("task cache lock");
        let mut questions: Vec<OpenQuestion> = open
            .keys()
            .filter_map(|task_id| {
                let task = cache.get(task_id)?;
                let entry = task
                    .thread
                    .iter()
                    .rev()
                    .find(|e| e.kind == ThreadEntryKind::Question)?;
                Some(OpenQuestion {
                    task_id: task_id.clone(),
                    asked_by: entry.from.clone(),
                    body: entry.body.clone(),
                    asked_at: entry.at,
                })
            })
            .collect();
        questions.sort_by_key(|q| q.asked_at);
        questions
    }

    /// Flushes the state branch's pending writes. Exposed so the periodic
    /// tick and, later, an immediate-flush caller (e.g. on `accept`) share
    /// one code path (docs/design/storage.md, "The state branch").
    pub async fn flush_now(&self) -> Result<(), TaskError> {
        Ok(self.state.flush_now().await?)
    }

    /// One last push after the shutdown flush (bounded; see `StateBranch::push_on_shutdown`).
    pub async fn push_on_shutdown(&self, timeout: std::time::Duration) {
        self.state.push_on_shutdown(timeout).await
    }

    pub fn state_push_status(&self) -> Option<bridle_api::types::StatePushStatus> {
        self.state.push_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{NewAgent, Store};
    use std::path::Path;
    use tokio::process::Command;

    async fn init_repo(dir: &Path) {
        std::fs::create_dir_all(dir).expect("mkdir repo");
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["init", "-q", "-b", "main"])
            .output()
            .await
            .expect("git init");
        assert!(out.status.success());
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
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
            .output()
            .await
            .expect("git commit");
        assert!(out.status.success());
    }

    async fn manager() -> (TaskManager, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TaskManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");
        (tm, tmp)
    }

    #[tokio::test]
    async fn new_task_is_open_and_visible_before_any_flush() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task(
                "Add foo",
                TaskKind::Feature,
                "a description".to_string(),
                vec![],
                None,
            )
            .await
            .expect("new task");
        assert!(task.id.starts_with("tw-"));
        assert_eq!(task.state, TaskState::Open);
        assert_eq!(task.body, "a description");

        let fetched = tm.get_task(&task.id).expect("get");
        assert_eq!(fetched.title, "Add foo");
        assert_eq!(tm.list_tasks().len(), 1);
    }

    #[tokio::test]
    async fn edit_changes_title_and_body_without_changing_state() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");

        let edited = tm
            .edit_task(
                &task.id,
                Some("Add foo, better".to_string()),
                None,
                None,
                None,
            )
            .await
            .expect("edit title");
        assert_eq!(edited.title, "Add foo, better");
        assert_eq!(edited.state, TaskState::Open);

        let edited = tm
            .edit_task(&task.id, None, Some("new body".to_string()), None, None)
            .await
            .expect("edit body");
        assert_eq!(edited.body, "new body");
        assert_eq!(edited.title, "Add foo, better");
    }

    #[tokio::test]
    async fn edit_rejects_a_blank_title() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        let err = tm
            .edit_task(&task.id, Some("   ".to_string()), None, None, None)
            .await
            .expect_err("blank title");
        assert!(matches!(err, TaskError::BadRequest(_)));
    }

    #[tokio::test]
    async fn edit_clears_size_with_none() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task(
                "Add foo",
                TaskKind::Feature,
                String::new(),
                vec![],
                Some(TaskSize::M),
            )
            .await
            .expect("new task");
        assert_eq!(task.size, Some(TaskSize::M));

        let edited = tm
            .edit_task(&task.id, None, None, None, Some(TaskSize::None))
            .await
            .expect("clear size");
        assert_eq!(edited.size, None);
    }

    #[tokio::test]
    async fn drop_requires_a_reason_and_reopen_only_applies_to_dropped() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");

        let err = tm
            .drop_task(&task.id, "", &"human".to_string())
            .await
            .expect_err("empty reason");
        assert!(matches!(err, TaskError::BadRequest(_)));

        let err = tm
            .reopen_task(&task.id, &"human".to_string())
            .await
            .expect_err("not dropped yet");
        assert!(matches!(err, TaskError::Conflict(_)));

        let dropped = tm
            .drop_task(&task.id, "budget cut", &"human".to_string())
            .await
            .expect("drop");
        assert_eq!(dropped.state, TaskState::Dropped);
        assert_eq!(dropped.thread.len(), 1);
        assert_eq!(dropped.thread[0].body, "dropped: budget cut");

        let err = tm
            .drop_task(&task.id, "again", &"human".to_string())
            .await
            .expect_err("already dropped");
        assert!(matches!(err, TaskError::Conflict(_)));

        let reopened = tm
            .reopen_task(&task.id, &"human".to_string())
            .await
            .expect("reopen");
        assert_eq!(reopened.state, TaskState::Reopened);

        let err = tm
            .reopen_task(&task.id, &"human".to_string())
            .await
            .expect_err("not dropped any more");
        assert!(matches!(err, TaskError::Conflict(_)));
    }

    #[tokio::test]
    async fn plan_moves_open_to_planned_and_rejects_every_other_state() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        assert_eq!(task.state, TaskState::Open);

        let planned = tm
            .plan_task(&task.id, &"human".to_string())
            .await
            .expect("plan");
        assert_eq!(planned.state, TaskState::Planned);

        // Already planned: not open any more.
        let err = tm
            .plan_task(&task.id, &"human".to_string())
            .await
            .expect_err("already planned");
        assert!(matches!(err, TaskError::Conflict(_)));

        // Claimed: also not open.
        tm.claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim");
        let err = tm
            .plan_task(&task.id, &"human".to_string())
            .await
            .expect_err("claimed, not open");
        assert!(matches!(err, TaskError::Conflict(_)));

        // Dropped and reopened: also not open (reopened, per the lifecycle
        // diagram, not open).
        tm.release_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("release");
        tm.drop_task(&task.id, "no longer needed", &"human".to_string())
            .await
            .expect("drop");
        tm.reopen_task(&task.id, &"human".to_string())
            .await
            .expect("reopen");
        let err = tm
            .plan_task(&task.id, &"human".to_string())
            .await
            .expect_err("reopened, not open");
        assert!(matches!(err, TaskError::Conflict(_)));
    }

    #[tokio::test]
    async fn plan_of_an_unknown_id_is_not_found() {
        let (tm, _tmp) = manager().await;
        let err = tm
            .plan_task("tw-nope", &"human".to_string())
            .await
            .expect_err("no such task");
        assert!(matches!(err, TaskError::NotFound(_)));
    }

    #[tokio::test]
    async fn add_edge_rejects_self_loops_and_unknown_tasks_and_duplicates() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_task("A", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");

        let err = tm
            .add_edge(&a.id, &a.id, EdgeKind::Blocks)
            .await
            .expect_err("self loop");
        assert!(matches!(err, TaskError::BadRequest(_)));

        let err = tm
            .add_edge("tw-nope", &b.id, EdgeKind::Blocks)
            .await
            .expect_err("unknown from");
        assert!(matches!(err, TaskError::NotFound(_)));

        let err = tm
            .add_edge(&a.id, "tw-nope", EdgeKind::Blocks)
            .await
            .expect_err("unknown to");
        assert!(matches!(err, TaskError::NotFound(_)));

        let edge = tm
            .add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("add edge");
        assert_eq!(edge.from, a.id);
        assert_eq!(edge.to, b.id);
        assert_eq!(tm.list_edges(), vec![edge]);

        let err = tm
            .add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect_err("duplicate edge");
        assert!(matches!(err, TaskError::Conflict(_)));
    }

    #[tokio::test]
    async fn remove_edge_drops_it_and_errors_when_missing() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_task("A", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");
        tm.add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("add edge");

        tm.remove_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("remove edge");
        assert!(tm.list_edges().is_empty());

        let err = tm
            .remove_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect_err("already removed");
        assert!(matches!(err, TaskError::NotFound(_)));
    }

    /// A shortcut for tests that don't care about `plan_task` itself, just
    /// getting a task to `planned` to exercise `is_ready`/`ready_tasks`/etc.
    /// on top of it: reaches into the private cache directly rather than
    /// going through `plan_task`'s actor/event bookkeeping, which those
    /// tests have no use for.
    fn force_planned(tm: &TaskManager, id: &str) {
        let mut cache = tm.cache.lock().expect("task cache lock");
        cache.get_mut(id).expect("task in cache").state = TaskState::Planned;
    }

    #[tokio::test]
    async fn integrating_a_blocker_frees_the_blocked_task_and_leaves_the_queue() {
        let (tm, _tmp) = manager().await;
        let human = "human".to_string();
        let blocker = tm
            .new_task("Blocker", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new blocker");
        let blocked = tm
            .new_task("Blocked", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new blocked");
        tm.add_edge(&blocker.id, &blocked.id, EdgeKind::Blocks)
            .await
            .expect("add edge");
        force_planned(&tm, &blocker.id);
        force_planned(&tm, &blocked.id);
        tm.set_queue(
            vec![vec![blocker.id.clone()], vec![blocked.id.clone()]],
            &human,
        )
        .await
        .expect("set queue");
        assert!(!tm.is_ready(&tm.get_task(&blocked.id).expect("blocked")));

        let err = tm
            .done_task(&blocker.id, " ", None, &human)
            .await
            .expect_err("empty commit");
        assert!(matches!(err, TaskError::BadRequest(_)));

        let done = tm
            .done_task(&blocker.id, "abc123", None, &human)
            .await
            .expect("done");
        assert_eq!(done.state, TaskState::Integrated);
        assert_eq!(done.thread[0].body, "integrated: abc123");
        assert!(tm.is_ready(&tm.get_task(&blocked.id).expect("blocked")));
        let ready: Vec<String> = tm.ready_tasks().into_iter().map(|t| t.id).collect();
        assert_eq!(ready, vec![blocked.id.clone()]);
        assert_eq!(tm.live_queue_tiers(), vec![vec![blocked.id.clone()]]);

        let err = tm
            .done_task(&blocker.id, "def456", None, &human)
            .await
            .expect_err("already integrated");
        assert!(matches!(err, TaskError::Conflict(_)));

        let reopened = tm.reopen_task(&blocker.id, &human).await.expect("reopen");
        assert_eq!(reopened.state, TaskState::Reopened);
        assert_eq!(tm.live_queue_tiers().len(), 2);
    }

    #[tokio::test]
    async fn done_records_branch_and_commit_and_summary_replaces() {
        let (tm, _tmp) = manager().await;
        let human = "human".to_string();
        let t = tm
            .new_task("T", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new");
        let err = tm.set_summary(&t.id, "  ").await.expect_err("empty");
        assert!(matches!(err, TaskError::BadRequest(_)));
        tm.set_summary(&t.id, "first").await.expect("summary");
        let s = tm.set_summary(&t.id, "second\n").await.expect("replace");
        assert_eq!(s.summary.as_deref(), Some("second"));
        let done = tm
            .done_task(&t.id, "abc123", Some("bridle/t"), &human)
            .await
            .expect("done");
        assert_eq!(done.branch.as_deref(), Some("bridle/t"));
        assert_eq!(done.commit.as_deref(), Some("abc123"));
        assert_eq!(done.summary.as_deref(), Some("second"));
        assert_eq!(done.thread[0].body, "integrated: abc123 (branch bridle/t)");
    }

    #[tokio::test]
    async fn ready_tasks_excludes_unplanned_and_blocked_tasks() {
        let (tm, _tmp) = manager().await;
        let blocker = tm
            .new_task("Blocker", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new blocker");
        let blocked = tm
            .new_task("Blocked", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new blocked");
        let unplanned = tm
            .new_task("Unplanned", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new unplanned");

        tm.add_edge(&blocker.id, &blocked.id, EdgeKind::Blocks)
            .await
            .expect("add edge");

        // Still open, not planned: none of the three are ready yet.
        assert!(tm.ready_tasks().is_empty());

        force_planned(&tm, &blocker.id);
        force_planned(&tm, &blocked.id);
        force_planned(&tm, &unplanned.id);

        // `blocked` is planned but its blocker isn't dropped, so it's still
        // not ready. `blocker` and `unplanned` (misleadingly named now that
        // it's planned too) have no blocks-edge pointing at them, so both
        // are ready.
        let ready_ids: Vec<String> = tm.ready_tasks().into_iter().map(|t| t.id).collect();
        assert!(!ready_ids.contains(&blocked.id));
        assert!(ready_ids.contains(&unplanned.id));
        assert!(ready_ids.contains(&blocker.id));

        // Dropping the blocker resolves it, which frees the blocked task.
        tm.drop_task(&blocker.id, "done another way", &"human".to_string())
            .await
            .expect("drop blocker");
        let ready_ids: Vec<String> = tm.ready_tasks().into_iter().map(|t| t.id).collect();
        assert!(ready_ids.contains(&blocked.id));
    }

    #[tokio::test]
    async fn set_queue_validates_unknown_ids_duplicates_and_empty_tiers() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_task("A", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");

        let err = tm
            .set_queue(vec![vec!["tw-nope".to_string()]], &"human".to_string())
            .await
            .expect_err("unknown task id");
        assert!(matches!(err, TaskError::NotFound(_)));

        let err = tm
            .set_queue(
                vec![vec![a.id.clone()], vec![a.id.clone()]],
                &"human".to_string(),
            )
            .await
            .expect_err("same task in two tiers");
        assert!(matches!(err, TaskError::Conflict(_)));

        let err = tm
            .set_queue(vec![vec![]], &"human".to_string())
            .await
            .expect_err("empty tier");
        assert!(matches!(err, TaskError::BadRequest(_)));

        let tiers = tm
            .set_queue(vec![vec![a.id.clone(), b.id.clone()]], &"human".to_string())
            .await
            .expect("valid queue");
        assert_eq!(tiers, vec![vec![a.id.clone(), b.id.clone()]]);
        assert_eq!(tm.queue_tiers(), tiers);
    }

    #[tokio::test]
    async fn add_queue_tier_appends_after_whatever_is_already_there() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_task("A", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");

        tm.add_queue_tier(vec![a.id.clone()], &"human".to_string())
            .await
            .expect("first tier");
        let tiers = tm
            .add_queue_tier(vec![b.id.clone()], &"human".to_string())
            .await
            .expect("second tier");
        assert_eq!(tiers, vec![vec![a.id.clone()], vec![b.id.clone()]]);
    }

    /// The core dispatch rule: the manager takes from the highest tier with
    /// a startable task, skipping a tier that's stuck on a dependency rather
    /// than idling on it (roles-and-lifecycle.md, "the queue").
    #[tokio::test]
    async fn highest_startable_tier_skips_a_tier_blocked_on_a_dependency() {
        let (tm, _tmp) = manager().await;
        let blocker = tm
            .new_task("Blocker", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new blocker");
        let blocked = tm
            .new_task("Blocked", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new blocked");
        let next = tm
            .new_task("Next", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new next");
        tm.add_edge(&blocker.id, &blocked.id, EdgeKind::Blocks)
            .await
            .expect("add edge");
        force_planned(&tm, &blocker.id);
        force_planned(&tm, &blocked.id);
        force_planned(&tm, &next.id);

        // Tier 1's only task is blocked on tier 1's own blocker... but the
        // blocker isn't itself in the queue (it's backlog): tier 1 has
        // nothing startable, so the highest startable tier is tier 2.
        tm.set_queue(
            vec![vec![blocked.id.clone()], vec![next.id.clone()]],
            &"human".to_string(),
        )
        .await
        .expect("set queue");
        let top: Vec<String> = tm
            .highest_startable_tier()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(top, vec![next.id.clone()]);

        // Resolving the blocker frees tier 1's task, which now outranks
        // tier 2 again.
        tm.drop_task(&blocker.id, "done another way", &"human".to_string())
            .await
            .expect("drop blocker");
        let top: Vec<String> = tm
            .highest_startable_tier()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(top, vec![blocked.id.clone()]);
    }

    #[tokio::test]
    async fn highest_startable_tier_is_empty_with_no_queue_or_nothing_startable() {
        let (tm, _tmp) = manager().await;
        assert!(tm.highest_startable_tier().is_empty());

        // A task exists and is even ready, but it's backlog: not in any
        // tier, so it's never returned here.
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));
        assert!(tm.highest_startable_tier().is_empty());
    }

    #[tokio::test]
    async fn queue_survives_a_flush_and_a_fresh_manager_hydrating_from_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TaskManager::open(
            store.clone(),
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let a = tm
            .new_task("A", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");
        tm.set_queue(
            vec![vec![a.id.clone()], vec![b.id.clone()]],
            &"human".to_string(),
        )
        .await
        .expect("set queue");
        tm.flush_now().await.expect("flush");

        let tm2 = TaskManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager");
        assert_eq!(
            tm2.queue_tiers(),
            vec![vec![a.id.clone()], vec![b.id.clone()]]
        );
    }

    #[tokio::test]
    async fn get_edit_and_drop_on_an_unknown_id_is_not_found() {
        let (tm, _tmp) = manager().await;
        assert!(tm.get_task("tw-nope").is_none());
        assert!(matches!(
            tm.edit_task("tw-nope", Some("x".to_string()), None, None, None)
                .await
                .expect_err("no such task"),
            TaskError::NotFound(_)
        ));
        assert!(matches!(
            tm.drop_task("tw-nope", "why", &"human".to_string())
                .await
                .expect_err("no such task"),
            TaskError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn state_survives_a_flush_and_a_fresh_manager_hydrating_from_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TaskManager::open(
            store.clone(),
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let task = tm
            .new_task(
                "Add foo",
                TaskKind::Feature,
                "a description".to_string(),
                vec![],
                None,
            )
            .await
            .expect("new task");
        tm.flush_now().await.expect("flush");

        // A fresh manager over the same store and state branch, as a
        // restarted daemon would build, hydrates the full record (title
        // *and* body) from the flushed file, not just the database row.
        let tm2 = TaskManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager");
        let rehydrated = tm2.get_task(&task.id).expect("rehydrated");
        assert_eq!(rehydrated.title, "Add foo");
        assert_eq!(rehydrated.body, "a description");
    }

    #[tokio::test]
    async fn asking_a_question_blocks_ready_and_answering_unblocks_it() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));

        let asked = tm
            .ask_question(&task.id, &"agent:w1".to_string(), "which endpoint?")
            .await
            .expect("ask question");
        assert_eq!(asked.thread.len(), 1);
        assert_eq!(asked.thread[0].kind, ThreadEntryKind::Question);
        assert_eq!(asked.thread[0].from, "agent:w1");
        assert_eq!(asked.thread[0].body, "which endpoint?");
        assert!(
            !tm.ready_tasks().iter().any(|t| t.id == task.id),
            "an open question excludes a task from ready"
        );

        // A second question while one is already open is a conflict.
        let err = tm
            .ask_question(&task.id, &"human".to_string(), "another one?")
            .await
            .expect_err("already has an open question");
        assert!(matches!(err, TaskError::Conflict(_)));

        let answered = tm
            .answer_question(&task.id, &"human".to_string(), "the v1 endpoint")
            .await
            .expect("answer question");
        assert_eq!(answered.thread.len(), 2);
        assert_eq!(answered.thread[1].kind, ThreadEntryKind::Answer);
        assert_eq!(answered.thread[1].from, "human");
        assert_eq!(answered.thread[1].body, "the v1 endpoint");
        assert!(
            tm.ready_tasks().iter().any(|t| t.id == task.id),
            "answering the question re-enables readiness"
        );

        // Answering again with nothing open is a conflict.
        let err = tm
            .answer_question(&task.id, &"human".to_string(), "still there?")
            .await
            .expect_err("no open question left");
        assert!(matches!(err, TaskError::Conflict(_)));
    }

    #[tokio::test]
    async fn ask_and_answer_reject_blank_bodies() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");

        let err = tm
            .ask_question(&task.id, &"human".to_string(), "   ")
            .await
            .expect_err("blank question body");
        assert!(matches!(err, TaskError::BadRequest(_)));

        tm.ask_question(&task.id, &"human".to_string(), "real question")
            .await
            .expect("ask question");
        let err = tm
            .answer_question(&task.id, &"human".to_string(), "")
            .await
            .expect_err("blank answer body");
        assert!(matches!(err, TaskError::BadRequest(_)));
    }

    #[tokio::test]
    async fn note_adds_a_thread_entry_without_affecting_readiness() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));

        let noted = tm
            .note_task(&task.id, &"agent:w1".to_string(), "fyi, started on this")
            .await
            .expect("note task");
        assert_eq!(noted.thread.len(), 1);
        assert_eq!(noted.thread[0].kind, ThreadEntryKind::Note);
        assert_eq!(noted.thread[0].from, "agent:w1");
        assert_eq!(noted.thread[0].body, "fyi, started on this");
        assert!(
            tm.ready_tasks().iter().any(|t| t.id == task.id),
            "a note doesn't block readiness like a question does"
        );

        let noted = tm
            .note_task(&task.id, &"human".to_string(), "sounds good")
            .await
            .expect("note task from human");
        assert_eq!(noted.thread.len(), 2);
        assert_eq!(noted.thread[1].from, "human");
        assert_eq!(noted.thread[1].body, "sounds good");
    }

    #[tokio::test]
    async fn note_rejects_a_blank_body() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        let err = tm
            .note_task(&task.id, &"human".to_string(), "   ")
            .await
            .expect_err("blank note body");
        assert!(matches!(err, TaskError::BadRequest(_)));
    }

    #[tokio::test]
    async fn an_open_question_survives_a_flush_and_a_fresh_manager_hydrating_from_the_database() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TaskManager::open(
            store.clone(),
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        tm.ask_question(&task.id, &"human".to_string(), "which endpoint?")
            .await
            .expect("ask question");
        tm.flush_now().await.expect("flush");

        // A fresh manager, as a restarted daemon would build, hydrates the
        // open-questions index from the database (not the state branch, which
        // has no separate index of its own) and the thread entry from the
        // flushed task file.
        let tm2 = TaskManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager");
        let rehydrated = tm2.get_task(&task.id).expect("rehydrated");
        assert_eq!(rehydrated.thread.len(), 1);
        assert_eq!(rehydrated.thread[0].kind, ThreadEntryKind::Question);
        assert!(
            !tm2.is_ready(&rehydrated),
            "the open question survives a restart and still blocks ready"
        );
    }

    #[tokio::test]
    async fn claim_blocks_ready_and_release_unblocks_it() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));
        assert_eq!(task.claimed_by, None, "unclaimed on creation");

        let claimed = tm
            .claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim task");
        assert_eq!(claimed.state, TaskState::Claimed);
        assert_eq!(claimed.claimed_by.as_deref(), Some("agent:w1"));
        assert!(claimed.claimed_at.is_some());
        assert!(
            !tm.ready_tasks().iter().any(|t| t.id == task.id),
            "a claimed task drops out of ready"
        );

        // Claiming an already-claimed task is a conflict.
        let err = tm
            .claim_task(&task.id, &"agent:w2".to_string())
            .await
            .expect_err("already claimed");
        assert!(matches!(err, TaskError::Conflict(_)));

        let released = tm
            .release_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("release task");
        assert_eq!(released.state, TaskState::Planned);
        assert_eq!(released.claimed_by, None, "cleared on release");
        assert_eq!(released.claimed_at, None);
        assert!(
            tm.ready_tasks().iter().any(|t| t.id == task.id),
            "releasing the claim re-enables readiness"
        );

        // Releasing again with nothing claimed is a conflict.
        let err = tm
            .release_task(&task.id, &"agent:w1".to_string())
            .await
            .expect_err("no longer claimed");
        assert!(matches!(err, TaskError::Conflict(_)));
    }

    /// A human to-do: the lease check never releases the human's claim, and
    /// the human finishes it with no commit.
    #[tokio::test]
    async fn human_claim_survives_the_lease_check_and_is_done_without_a_commit() {
        let (tm, _tmp) = manager().await;
        let t = tm
            .new_task(
                "[at restart] tokens",
                TaskKind::Feature,
                String::new(),
                vec![],
                None,
            )
            .await
            .expect("new");
        force_planned(&tm, &t.id);
        let human = "human".to_string();
        tm.claim_task(&t.id, &human).await.expect("claim");
        tm.tick_claim_lease_check(Utc::now() + chrono::Duration::days(30))
            .await;
        assert_eq!(tm.get_task(&t.id).expect("task").state, TaskState::Claimed);

        let done = tm.done_task(&t.id, "", None, &human).await.expect("done");
        assert_eq!(done.state, TaskState::Integrated);
        assert_eq!(done.commit, None);
    }

    #[tokio::test]
    async fn done_without_a_commit_is_refused_for_an_agent_claim() {
        let (tm, _tmp) = manager().await;
        let t = tm
            .new_task("A", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new");
        force_planned(&tm, &t.id);
        tm.claim_task(&t.id, &"agent:w1".to_string())
            .await
            .expect("claim");
        let err = tm
            .done_task(&t.id, "", None, &"human".to_string())
            .await
            .expect_err("needs a commit");
        assert!(matches!(err, TaskError::BadRequest(_)));
    }

    /// `server.rs::list_tasks` filters `?claimed_by=` by matching
    /// `Task::claimed_by` verbatim against the already-resolved id (`me` is
    /// resolved to the caller's own `PrincipalId` before this filter ever
    /// runs); this exercises that same filter directly against the field.
    #[tokio::test]
    async fn claimed_by_filters_to_the_matching_claimant() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_task("A", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new b");
        force_planned(&tm, &a.id);
        force_planned(&tm, &b.id);

        tm.claim_task(&a.id, &"agent:w1".to_string())
            .await
            .expect("claim a");
        tm.claim_task(&b.id, &"human".to_string())
            .await
            .expect("claim b");

        let claimed_by = |who: &str| -> Vec<String> {
            tm.list_tasks()
                .into_iter()
                .filter(|t| t.claimed_by.as_deref() == Some(who))
                .map(|t| t.id)
                .collect()
        };
        assert_eq!(claimed_by("agent:w1"), vec![a.id.clone()]);
        assert_eq!(claimed_by("human"), vec![b.id.clone()]);
        assert!(claimed_by("agent:w2").is_empty());
    }

    #[tokio::test]
    async fn claim_by_another_agent_is_rejected() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);

        tm.claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim task");

        let err = tm
            .release_task(&task.id, &"agent:w2".to_string())
            .await
            .expect_err("not the claimant");
        assert!(matches!(err, TaskError::Conflict(_)));

        // Still claimed by w1, untouched by w2's rejected release.
        assert!(!tm.ready_tasks().iter().any(|t| t.id == task.id));
    }

    async fn claimed_task(tm: &TaskManager, agent: &str) -> Task {
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(tm, &task.id);
        tm.claim_task(&task.id, &format!("agent:{agent}"))
            .await
            .expect("claim task")
    }

    #[tokio::test]
    async fn dropping_a_claimed_task_releases_its_claim() {
        let (tm, _tmp) = manager().await;
        let task = claimed_task(&tm, "w1").await;

        let dropped = tm
            .drop_task(&task.id, "obsolete", &"human".to_string())
            .await
            .expect("drop claimed task");
        assert_eq!(dropped.state, TaskState::Dropped);
        assert_eq!(dropped.claimed_by, None);
        assert!(tm.store.list_claims().await.expect("claims").is_empty());
        assert!(tm.claims.lock().expect("claims lock").is_empty());
    }

    #[tokio::test]
    async fn the_lease_check_never_moves_a_task_that_is_not_claimed() {
        let (tm, _tmp) = manager().await;
        let task = claimed_task(&tm, "w1").await;
        // A stale claim on a dropped task, as older versions could leave.
        tm.cache
            .lock()
            .expect("task cache lock")
            .get_mut(&task.id)
            .expect("task in cache")
            .state = TaskState::Dropped;
        assert!(matches!(
            tm.release_claim(&task.id).await,
            Err(TaskError::Conflict(_))
        ));
        tm.tick_claim_lease_check(Utc::now() + chrono::Duration::days(1))
            .await;
        assert_eq!(
            tm.get_task(&task.id).expect("task").state,
            TaskState::Dropped
        );
    }

    #[tokio::test]
    async fn open_discards_claims_whose_task_is_not_claimed() {
        let (tm, _tmp) = manager().await;
        let task = claimed_task(&tm, "w1").await;
        tm.store
            .set_task_state(&task.id, TaskState::Dropped)
            .await
            .expect("set state");

        let tm2 = TaskManager::open(
            tm.store.clone(),
            tm.state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager");
        assert!(tm.store.list_claims().await.expect("claims").is_empty());
        assert!(tm2.claims.lock().expect("claims lock").is_empty());
        assert_eq!(tm2.get_task(&task.id).expect("task").claimed_by, None);
    }

    #[tokio::test]
    async fn a_stale_claim_expires_and_releases_the_task_back_to_ready() {
        let (tm, _tmp) = manager().await;
        let store = tm.store.clone();
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);

        let agent = store
            .insert_agent(NewAgent {
                name: "w1".to_string(),
                role: "worker".to_string(),
                model: "sonnet".to_string(),
                session_id: "sess-1".to_string(),
                workdir_kind: "worktree".to_string(),
                cwd: "/ws/wt/w1".to_string(),
                worktree: Some("/ws/wt/w1".to_string()),
                branch: Some("bridle/w1".to_string()),
                created_by: "human".to_string(),
                extra_allowed_tools: Vec::new(),
                extra_env: Vec::new(),
                components: Vec::new(),
            })
            .await
            .expect("insert agent");
        let last_activity = Utc::now();
        store
            .touch_agent(&agent.id, last_activity)
            .await
            .expect("touch agent");

        tm.claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim task");
        assert!(!tm.ready_tasks().iter().any(|t| t.id == task.id));

        // Well within the lease: a lease-check tick leaves the claim alone.
        tm.tick_claim_lease_check(last_activity + chrono::Duration::seconds(1))
            .await;
        assert!(!tm.ready_tasks().iter().any(|t| t.id == task.id));

        // Past the lease with no further activity: the tick releases it.
        tm.tick_claim_lease_check(
            last_activity + tm.claim_lease_after + chrono::Duration::seconds(1),
        )
        .await;
        assert!(
            tm.ready_tasks().iter().any(|t| t.id == task.id),
            "a stale claim's lease expiry re-enables readiness"
        );
    }

    #[tokio::test]
    async fn a_claim_survives_a_flush_and_a_fresh_manager_hydrating_from_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TaskManager::open(
            store,
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        tm.plan_task(&task.id, &"human".to_string())
            .await
            .expect("plan");
        tm.claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim");
        tm.flush_now().await.expect("flush");

        // A fresh database, over the same already-flushed state branch, as
        // `bridle rebuild` runs against: unlike before durable claims, this
        // restores the claim too, not just the task.
        let fresh_store = Store::open(tmp.path().join("bridle2.db"))
            .await
            .expect("open fresh store");
        let tm2 = TaskManager::open(
            fresh_store.clone(),
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open fresh task manager");
        tm2.rebuild_from_state_branch().await.expect("rebuild");

        let rehydrated = tm2.get_task(&task.id).expect("rehydrated");
        assert_eq!(rehydrated.state, TaskState::Claimed);
        assert_eq!(rehydrated.claimed_by.as_deref(), Some("agent:w1"));
        assert!(rehydrated.claimed_at.is_some());

        let db_claims = fresh_store.list_claims().await.expect("db claims");
        assert_eq!(db_claims.len(), 1);
        assert_eq!(db_claims[0].task_id, task.id);
        assert_eq!(db_claims[0].claimed_by, "agent:w1");
    }

    #[tokio::test]
    async fn impact_is_validated_state_gated_and_survives_rebuild() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TaskManager::open(
            store,
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");

        let impact = Impact {
            modify: vec!["s-b310".to_string()],
            add_under: vec!["r-7fa2".to_string()],
            remove: vec!["a-0c".to_string()],
            files: vec!["client/**".to_string()],
        };
        tm.set_impact(&task.id, impact.clone()).await.expect("set");
        let bad = Impact {
            modify: vec!["x-12".to_string()],
            ..Impact::default()
        };
        assert!(matches!(
            tm.set_impact(&task.id, bad).await,
            Err(TaskError::BadRequest(_))
        ));
        assert_eq!(tm.get_task(&task.id).expect("task").impact, impact);

        tm.flush_now().await.expect("flush");
        let fresh_store = Store::open(tmp.path().join("bridle2.db"))
            .await
            .expect("open fresh store");
        let tm2 = TaskManager::open(
            fresh_store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open fresh task manager");
        tm2.rebuild_from_state_branch().await.expect("rebuild");
        assert_eq!(tm2.get_task(&task.id).expect("rebuilt").impact, impact);

        tm2.drop_task(&task.id, "no", &"human".to_string())
            .await
            .expect("drop");
        assert!(matches!(
            tm2.set_impact(&task.id, impact).await,
            Err(TaskError::Conflict(_))
        ));
    }

    #[tokio::test]
    async fn rebuild_reconstructs_tasks_edges_and_open_questions_from_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TaskManager::open(
            store,
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let a = tm
            .new_task("A", TaskKind::Chore, "body a".to_string(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Feature, "body b".to_string(), vec![], None)
            .await
            .expect("new b");
        let c = tm
            .new_task("C", TaskKind::Feature, String::new(), vec![], None)
            .await
            .expect("new c");
        tm.add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("add edge");

        force_planned(&tm, &c.id);
        tm.ask_question(&c.id, &"human".to_string(), "which endpoint?")
            .await
            .expect("ask c");

        force_planned(&tm, &b.id);
        tm.ask_question(&b.id, &"agent:w1".to_string(), "old question?")
            .await
            .expect("ask b");
        tm.answer_question(&b.id, &"human".to_string(), "answered")
            .await
            .expect("answer b");

        tm.flush_now().await.expect("flush");

        // A fresh, empty database over the same (already flushed) state
        // branch, as the migration story describes: clone, start the
        // daemon, `bridle rebuild`.
        let fresh_store = Store::open(tmp.path().join("bridle2.db"))
            .await
            .expect("open fresh store");
        let tm2 = TaskManager::open(
            fresh_store.clone(),
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open fresh task manager");
        tm2.rebuild_from_state_branch().await.expect("rebuild");

        let mut tasks = tm2.list_tasks();
        tasks.sort_by(|x, y| x.id.cmp(&y.id));
        assert_eq!(tasks.len(), 3);

        let got_a = tasks.iter().find(|t| t.id == a.id).expect("a rebuilt");
        assert_eq!(got_a.title, "A");
        assert_eq!(got_a.body, "body a");
        assert_eq!(got_a.state, TaskState::Open);

        let got_b = tasks.iter().find(|t| t.id == b.id).expect("b rebuilt");
        assert_eq!(got_b.state, TaskState::Planned);
        assert_eq!(
            got_b.thread.len(),
            2,
            "b's question and its answer both survive"
        );
        assert!(
            !tm2.list_open_questions().iter().any(|q| q.task_id == b.id),
            "b's question was answered before the rebuild, so it has no open question"
        );

        let got_c = tasks.iter().find(|t| t.id == c.id).expect("c rebuilt");
        assert_eq!(got_c.state, TaskState::Planned);
        assert!(
            !tm2.is_ready(got_c),
            "c's open question survives the rebuild and still blocks ready"
        );

        let edges = tm2.list_edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, a.id);
        assert_eq!(edges[0].to, b.id);
        assert_eq!(edges[0].kind, EdgeKind::Blocks);

        let open_qs = tm2.list_open_questions();
        assert_eq!(open_qs.len(), 1);
        assert_eq!(open_qs[0].task_id, c.id);
        assert_eq!(open_qs[0].asked_by, "human");
        assert_eq!(open_qs[0].body, "which endpoint?");

        // The database rows themselves, not just the in-memory cache this
        // process built while rebuilding.
        let db_tasks = fresh_store.list_tasks().await.expect("db tasks");
        assert_eq!(db_tasks.len(), 3);
        let db_edges = fresh_store.list_edges().await.expect("db edges");
        assert_eq!(db_edges.len(), 1);
        let db_open_qs = fresh_store
            .list_open_questions()
            .await
            .expect("db open questions");
        assert_eq!(db_open_qs.len(), 1);
        assert_eq!(db_open_qs[0].task_id, c.id);
    }

    #[tokio::test]
    async fn rebuild_restores_handovers_with_their_seq_and_new_notes_do_not_collide() {
        let (tm, tmp) = manager().await;
        let store = tm.store.clone();
        for body in ["one", "two", "three"] {
            let h = store
                .insert_handover("orchestrator", "p", body, "human")
                .await
                .expect("insert");
            tm.enqueue_handover(&h).expect("enqueue");
        }
        tm.flush_now().await.expect("flush");

        let fresh = Store::open(tmp.path().join("fresh.db"))
            .await
            .expect("store");
        let state = StateBranch::open(&tmp.path().join("repo"), &tmp.path().join("state"))
            .await
            .expect("state");
        let tm2 = TaskManager::open(
            fresh.clone(),
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open");
        tm2.rebuild_from_state_branch().await.expect("rebuild");
        let list = fresh.list_handovers().await.expect("list");
        let ids: Vec<_> = list.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, ["h-0003", "h-0002", "h-0001"]);
        assert_eq!(list[0].body, "three");
        let next = fresh
            .insert_handover("orchestrator", "p", "four", "human")
            .await
            .expect("insert");
        assert_eq!(next.id, "h-0004");
    }

    #[tokio::test]
    async fn backfill_writes_only_the_missing_notes() {
        let (tm, _tmp) = manager().await;
        let h1 = tm
            .store
            .insert_handover("orchestrator", "p", "one", "human")
            .await
            .expect("insert");
        tm.backfill_handovers().await.expect("backfill");
        tm.flush_now().await.expect("flush");
        assert!(tm.state.has_handover(&h1.id));
        // Nothing missing: nothing pending, so no new commit.
        tm.backfill_handovers().await.expect("backfill again");
        tm.flush_now().await.expect("flush");
        assert_eq!(tm.state.list_handovers().expect("list").len(), 1);
    }

    #[tokio::test]
    async fn rebuild_refuses_against_an_already_populated_database() {
        let (tm, _tmp) = manager().await;
        tm.new_task("A", TaskKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        tm.flush_now().await.expect("flush");

        let err = tm
            .rebuild_from_state_branch()
            .await
            .expect_err("must refuse to rebuild over an already-populated database");
        assert!(matches!(err, TaskError::Conflict(_)));

        // Untouched: still exactly the one task from before the refused
        // rebuild attempt.
        assert_eq!(tm.list_tasks().len(), 1);
    }
}
