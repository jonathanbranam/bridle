//! Task records: the SQLite fast index (`crate::store`) and the state
//! branch (`crate::state_branch`) kept in step, plus an in-memory cache that
//! carries the body/thread the database doesn't (docs/design/storage.md).
//! Scoped to this build's four states: `open`, `planned`, `dropped`,
//! `reopened`; edges, questions-block-a-task, claims and the rest of the
//! lifecycle arrive with later tasks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bridle_api::types::{PrincipalId, Task, TaskKind, TaskState, ThreadEntry, ThreadEntryKind};
use chrono::Utc;

use crate::state_branch::{StateBranch, StateBranchError};
use crate::store::{Store, StoreError};

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
}

impl TaskManager {
    /// Loads every task the database knows about, hydrating each from its
    /// state-branch file. A task whose file is missing (a crash between
    /// `insert_task` and the first flush that would have written it) falls
    /// back to an empty body/thread built from the database row alone,
    /// rather than failing daemon startup over it.
    pub async fn open(store: Store, state: StateBranch, prefix: String) -> Result<Self, TaskError> {
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
        Ok(TaskManager {
            store,
            state,
            prefix,
            cache: Arc::new(Mutex::new(cache)),
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
        tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
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
    use crate::store::Store;
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
                "-c", "user.email=t@e.com", "-c", "user.name=T",
                "commit", "--allow-empty", "-q", "-m", "init",
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
        let tm = TaskManager::open(store, state, "tw".to_string())
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
        let tm = TaskManager::open(store.clone(), state.clone(), "tw".to_string())
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
        let tm2 = TaskManager::open(store, state, "tw".to_string())
            .await
            .expect("reopen task manager");
        let rehydrated = tm2.get_task(&task.id).expect("rehydrated");
        assert_eq!(rehydrated.title, "Add foo");
        assert_eq!(rehydrated.body, "a description");
    }
}
