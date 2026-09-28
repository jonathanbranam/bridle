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
//! Claims are different: storage.md calls them out as one of "the ephemeral
//! tables … [that] arrive with later tasks" — SQLite-only, with no
//! state-branch file or thread entry. `claim_task`/`release_task` write only
//! to `Store`; the claiming agent's own activity (the same signal
//! `supervisor.rs`'s stall check watches) stands in for a lease renewal, so
//! [`TaskManager::tick_claim_lease_check`] can release a stale claim without
//! a separate heartbeat call.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bridle_api::types::{
    Edge, EdgeKind, MessageKind, MessageState, OpenQuestion, PrincipalId, Task, TaskKind,
    TaskState, ThreadEntry, ThreadEntryKind, When,
};
use chrono::Utc;

use crate::state_branch::{StateBranch, StateBranchError};
use crate::store::{NewMessage, RecipientKind, Store, StoreError};

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
    /// Task id -> claimant, for every currently claimed task. Mirrors
    /// `Store::list_claims`, loaded at `open`. Unlike `open_questions`,
    /// there's no state-branch counterpart at all (storage.md: claims is
    /// SQLite-only).
    claims: Arc<Mutex<HashMap<String, PrincipalId>>>,
    /// How long a claim survives without the claiming agent's own activity
    /// before [`TaskManager::tick_claim_lease_check`] releases it.
    claim_lease_after: chrono::Duration,
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
        let claims = store
            .list_claims()
            .await?
            .into_iter()
            .map(|c| (c.task_id, c.claimed_by))
            .collect();
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
        })
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

    /// Changes `title` and/or `body`. Neither changes the task's state.
    pub async fn edit_task(
        &self,
        id: &str,
        title: Option<String>,
        body: Option<String>,
    ) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        if let Some(title) = &title
            && title.trim().is_empty()
        {
            return Err(TaskError::BadRequest("title must not be empty".to_string()));
        }
        if title.is_none() && body.is_none() {
            return Ok(task);
        }
        if let Some(title) = title {
            self.store.set_task_title(id, &title).await?;
            task.title = title;
        }
        if let Some(body) = body {
            task.body = body;
        }
        task.updated_at = Utc::now();
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `open` or `planned` -> `dropped`, recording `reason` in the thread.
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

    /// `dropped` -> `reopened`. Any other starting state is a conflict.
    pub async fn reopen_task(&self, id: &str, actor: &PrincipalId) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        if task.state != TaskState::Dropped {
            return Err(TaskError::Conflict(format!(
                "task {id} is {}, not dropped; only a dropped task can be reopened",
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

    /// A blocker is unresolved unless it's `dropped`: `integrated` and
    /// `accepted` don't exist yet (P0 hasn't built them), so for now
    /// anything else — including a blocker this manager doesn't know about —
    /// still counts as blocking (roles-and-lifecycle.md, "ready is
    /// computed"). Documented here rather than left implicit, since it's a
    /// deliberate simplification this build makes, not the final rule.
    fn blocker_is_resolved(&self, blocker_id: &str) -> bool {
        self.get_task(blocker_id)
            .is_some_and(|b| b.state == TaskState::Dropped)
    }

    /// `planned`, no open `blocks` edge naming an unresolved blocker, and no
    /// unanswered question.
    pub fn is_ready(&self, task: &Task) -> bool {
        if task.state != TaskState::Planned {
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

    // ---------- claims ----------

    /// Claims `id` for `by`: `planned` -> `claimed`. Fails with `Conflict`
    /// if the task isn't ready right now — not planned, blocked, or already
    /// claimed (claiming again would need it to still be `planned`, which
    /// `is_ready` already requires). Unlike drop/reopen, this writes only to
    /// the database: claims are SQLite-only, with no state-branch file or
    /// thread entry (storage.md, "the ephemeral tables … arrive with later
    /// tasks").
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
            .insert(id.to_string(), by.clone());
        task.state = TaskState::Claimed;
        task.updated_at = now;
        Ok(self.put(task))
    }

    /// Releases `id`'s claim: `claimed` -> `planned`. Fails with `Conflict`
    /// if `by` isn't the current claimant (including if the task isn't
    /// claimed at all).
    pub async fn release_task(&self, id: &str, by: &PrincipalId) -> Result<Task, TaskError> {
        {
            let claims = self.claims.lock().expect("claims lock");
            match claims.get(id) {
                Some(claimant) if claimant == by => {}
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

    /// The shared release path for an explicit `release_task` and automatic
    /// lease expiry: clears the claim and transitions the task back to
    /// `planned`, without touching the state branch.
    async fn release_claim(&self, id: &str) -> Result<Task, TaskError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TaskError::NotFound(format!("no such task: {id}")))?;
        self.store.delete_claim(id).await?;
        self.store.set_task_state(id, TaskState::Planned).await?;
        self.claims.lock().expect("claims lock").remove(id);
        task.state = TaskState::Planned;
        task.updated_at = Utc::now();
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
            .map(|(id, by)| (id.clone(), by.clone()))
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
            let _ = self.release_claim(&task_id).await;
        }
    }

    pub fn ready_tasks(&self) -> Vec<Task> {
        self.list_tasks()
            .into_iter()
            .filter(|t| self.is_ready(t))
            .collect()
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
            .args(["init", "-q"])
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
            .new_task("Add foo", TaskKind::Feature, "a description".to_string())
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
            .new_task("Add foo", TaskKind::Feature, String::new())
            .await
            .expect("new task");

        let edited = tm
            .edit_task(&task.id, Some("Add foo, better".to_string()), None)
            .await
            .expect("edit title");
        assert_eq!(edited.title, "Add foo, better");
        assert_eq!(edited.state, TaskState::Open);

        let edited = tm
            .edit_task(&task.id, None, Some("new body".to_string()))
            .await
            .expect("edit body");
        assert_eq!(edited.body, "new body");
        assert_eq!(edited.title, "Add foo, better");
    }

    #[tokio::test]
    async fn edit_rejects_a_blank_title() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new())
            .await
            .expect("new task");
        let err = tm
            .edit_task(&task.id, Some("   ".to_string()), None)
            .await
            .expect_err("blank title");
        assert!(matches!(err, TaskError::BadRequest(_)));
    }

    #[tokio::test]
    async fn drop_requires_a_reason_and_reopen_only_applies_to_dropped() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new())
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
    async fn add_edge_rejects_self_loops_and_unknown_tasks_and_duplicates() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_task("A", TaskKind::Chore, String::new())
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Chore, String::new())
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
            .new_task("A", TaskKind::Chore, String::new())
            .await
            .expect("new a");
        let b = tm
            .new_task("B", TaskKind::Chore, String::new())
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

    /// There's no `plan` command yet (roles-and-lifecycle.md's `open ->
    /// planned` transition isn't built), so this reaches into the private
    /// cache directly to put a task in `planned` for the purposes of
    /// exercising `is_ready`/`ready_tasks` — legal from within this module,
    /// and simpler than inventing a test-only public API for it.
    fn force_planned(tm: &TaskManager, id: &str) {
        let mut cache = tm.cache.lock().expect("task cache lock");
        cache.get_mut(id).expect("task in cache").state = TaskState::Planned;
    }

    #[tokio::test]
    async fn ready_tasks_excludes_unplanned_and_blocked_tasks() {
        let (tm, _tmp) = manager().await;
        let blocker = tm
            .new_task("Blocker", TaskKind::Chore, String::new())
            .await
            .expect("new blocker");
        let blocked = tm
            .new_task("Blocked", TaskKind::Feature, String::new())
            .await
            .expect("new blocked");
        let unplanned = tm
            .new_task("Unplanned", TaskKind::Feature, String::new())
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
    async fn get_edit_and_drop_on_an_unknown_id_is_not_found() {
        let (tm, _tmp) = manager().await;
        assert!(tm.get_task("tw-nope").is_none());
        assert!(matches!(
            tm.edit_task("tw-nope", Some("x".to_string()), None)
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
            .new_task("Add foo", TaskKind::Feature, "a description".to_string())
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
            .new_task("Add foo", TaskKind::Feature, String::new())
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
            .new_task("Add foo", TaskKind::Feature, String::new())
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
            .new_task("Add foo", TaskKind::Feature, String::new())
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
            .new_task("Add foo", TaskKind::Feature, String::new())
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));

        let claimed = tm
            .claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim task");
        assert_eq!(claimed.state, TaskState::Claimed);
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

    #[tokio::test]
    async fn claim_by_another_agent_is_rejected() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new())
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

    #[tokio::test]
    async fn a_stale_claim_expires_and_releases_the_task_back_to_ready() {
        let (tm, _tmp) = manager().await;
        let store = tm.store.clone();
        let task = tm
            .new_task("Add foo", TaskKind::Feature, String::new())
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
}
