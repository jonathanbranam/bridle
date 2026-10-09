//! The human's to-dos and task questions from every reachable project, merged: what
//! `GET /api/v1/items` returns. Types are the gateway's own (plain serde, no daemon types), so
//! the daemons' API can change without moving the UI's contract. See
//! docs/design/human-web-ui.md sections 1, 2 and 5.

use std::collections::HashMap;
use std::time::Duration;

use bridle_api::client::Client;
use bridle_api::discovery::resolve_token;
use bridle_api::types::{Task, TaskPriority, TaskState, sort_by_priority};
use serde::Serialize;
use ts_rs::TS;

use crate::discovery::{PROBE_TIMEOUT, ProjectStatus, Target, current_targets};

/// Declaration order is the sort order: high first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Critical,
    Urgent,
    High,
    Normal,
    Low,
}

impl From<TaskPriority> for Priority {
    fn from(p: TaskPriority) -> Self {
        match p {
            TaskPriority::Critical => Self::Critical,
            TaskPriority::Urgent => Self::Urgent,
            TaskPriority::High => Self::High,
            TaskPriority::Normal => Self::Normal,
            TaskPriority::Low => Self::Low,
        }
    }
}

/// A task the human holds: check it off or decline it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Todo {
    pub task_id: String,
    pub title: String,
    pub body: String,
    pub priority: Priority,
    /// RFC 3339, UTC.
    pub created_at: String,
}

/// A task's open question: answer it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Decision {
    pub task_id: String,
    pub title: String,
    pub asked_by: String,
    pub question: String,
    /// The task's priority; the question itself has none.
    pub priority: Priority,
    /// RFC 3339, UTC.
    pub asked_at: String,
}

/// One reachable project's items. Both lists are ordered: high priority first, then oldest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ProjectItems {
    pub project: String,
    pub machine: Option<String>,
    pub decisions: Vec<Decision>,
    pub todos: Vec<Todo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Items {
    /// Reachable projects with something for the human, those with decisions first.
    pub projects: Vec<ProjectItems>,
    /// Projects that didn't answer (or whose items couldn't be read), with the reason.
    pub unreachable: Vec<ProjectStatus>,
}

/// A daemon to read from, with the token to read it with.
pub(crate) struct Source {
    pub project: String,
    pub machine: Option<String>,
    pub url: Option<String>,
    pub token: Option<String>,
    /// Set when the config or the token lookup already says it can't be reached.
    pub problem: Option<String>,
}

/// Reads every daemon this machine knows of.
pub async fn items() -> Items {
    let targets = current_targets().await;
    // Token lookup reads files: blocking work.
    let sources = tokio::task::spawn_blocking(move || targets.into_iter().map(source).collect())
        .await
        .unwrap_or_default();
    items_from(sources, PROBE_TIMEOUT).await
}

pub(crate) fn source(t: Target) -> Source {
    let (token, token_problem) = match resolve_token(
        None,
        t.workspace.as_deref().map(std::path::Path::new),
        Some(&t.project),
        t.machine.as_deref(),
        &crate::discovery::HumanEnv,
        true,
    ) {
        Ok(token) => (token, None),
        Err(e) => (None, Some(e.to_string())),
    };
    Source {
        project: t.project,
        machine: t.machine,
        url: t.url,
        token,
        problem: t.problem.or(token_problem),
    }
}

pub(crate) async fn items_from(sources: Vec<Source>, timeout: Duration) -> Items {
    let reads = sources.into_iter().map(|s| read_project(s, timeout));
    let results = futures::future::join_all(reads).await;
    let mut projects = Vec::new();
    let mut unreachable = Vec::new();
    for r in results {
        match r {
            Ok(p) if p.decisions.is_empty() && p.todos.is_empty() => {}
            Ok(p) => projects.push(p),
            Err(status) => unreachable.push(status),
        }
    }
    // `sort_by_key` is stable, so equal groups stay alphabetical.
    projects.sort_by(|a, b| a.project.cmp(&b.project));
    projects.sort_by_key(|p| p.decisions.is_empty());
    unreachable.sort_by(|a, b| a.project.cmp(&b.project));
    Items {
        projects,
        unreachable,
    }
}

async fn read_project(s: Source, timeout: Duration) -> Result<ProjectItems, ProjectStatus> {
    let down = |reason: String| ProjectStatus {
        project: s.project.clone(),
        machine: s.machine.clone(),
        url: s.url.clone(),
        reachable: false,
        reason: Some(reason),
    };
    if let Some(problem) = &s.problem {
        return Err(down(problem.clone()));
    }
    let Some(url) = &s.url else {
        return Err(down("no address".to_string()));
    };
    let client = Client::new_with_timeout(url.clone(), s.token.clone(), timeout);
    let fetched = tokio::try_join!(
        client.list_tasks_claimed_by("human"),
        client.list_tasks(),
        client.list_open_questions(),
    );
    let (claimed, all, questions) = fetched.map_err(|e| down(e.to_string()))?;

    let by_id: HashMap<&str, &Task> = all.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut held: Vec<Task> = claimed
        .iter()
        .filter(|t| t.state == TaskState::Claimed)
        .cloned()
        .collect();
    sort_by_priority(&mut held);
    let todos: Vec<Todo> = held
        .iter()
        .map(|t| Todo {
            task_id: t.id.clone(),
            title: t.title.clone(),
            body: t.body.clone(),
            priority: t.priority.into(),
            created_at: t.created_at.to_rfc3339(),
        })
        .collect();
    // A withdrawn task can still have its question open in the daemon; the withdrawal stays on
    // the task's record, the human just isn't asked.
    let mut decisions: Vec<_> = questions
        .iter()
        .filter(|q| {
            by_id
                .get(q.task_id.as_str())
                .is_none_or(|t| t.state != TaskState::Dropped)
        })
        .map(|q| {
            let task = by_id.get(q.task_id.as_str());
            let d = Decision {
                task_id: q.task_id.clone(),
                title: task.map(|t| t.title.clone()).unwrap_or_default(),
                asked_by: q.asked_by.to_string(),
                question: q.body.clone(),
                priority: task.map(|t| t.priority.into()).unwrap_or(Priority::Normal),
                asked_at: q.asked_at.to_rfc3339(),
            };
            (d, q.asked_at)
        })
        .collect();
    decisions.sort_by_key(|(d, at)| (d.priority, *at));
    Ok(ProjectItems {
        project: s.project,
        machine: s.machine,
        decisions: decisions.into_iter().map(|(d, _)| d).collect(),
        todos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, routing::get};
    use serde_json::{Value, json};

    fn task(id: &str, title: &str, state: &str, priority: &str, created: &str) -> Value {
        json!({
            "id": id, "title": title, "kind": "feature", "state": state, "body": "",
            "thread": [], "created_at": created, "updated_at": created,
            "claimed_by": "human", "priority": priority,
        })
    }

    fn question(task: &str, at: &str) -> Value {
        json!({"task_id": task, "asked_by": "agent:w", "body": "which?", "asked_at": at})
    }

    async fn fake(claimed: Vec<Value>, all: Vec<Value>, questions: Vec<Value>) -> String {
        let app = Router::new()
            .route(
                "/v1/tasks",
                get(move |axum::extract::RawQuery(q): axum::extract::RawQuery| {
                    let list = if q.is_some_and(|q| q.contains("claimed_by=human")) {
                        claimed.clone()
                    } else {
                        all.clone()
                    };
                    async move { Json(list) }
                }),
            )
            .route("/v1/questions", get(move || async move { Json(questions) }));
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        url
    }

    fn src(project: &str, url: Option<String>, problem: Option<&str>) -> Source {
        Source {
            project: project.into(),
            machine: None,
            url,
            token: None,
            problem: problem.map(str::to_string),
        }
    }

    #[tokio::test]
    async fn lists_are_high_priority_first_then_oldest_and_claimed_only() {
        let claimed = vec![
            task("t-old", "old", "claimed", "normal", "2026-01-01T00:00:00Z"),
            task("t-new", "new", "claimed", "normal", "2026-03-01T00:00:00Z"),
            task("t-hi", "urgent", "claimed", "high", "2026-02-01T00:00:00Z"),
            task(
                "t-done",
                "done",
                "integrated",
                "high",
                "2026-01-01T00:00:00Z",
            ),
        ];
        let all = vec![
            task("q-lo", "low one", "planned", "low", "2026-01-01T00:00:00Z"),
            task(
                "q-hi",
                "high one",
                "planned",
                "high",
                "2026-01-01T00:00:00Z",
            ),
            task(
                "q-old",
                "old one",
                "planned",
                "normal",
                "2026-01-01T00:00:00Z",
            ),
            task(
                "q-new",
                "new one",
                "planned",
                "normal",
                "2026-01-01T00:00:00Z",
            ),
        ];
        let questions = vec![
            question("q-new", "2026-05-01T00:00:00Z"),
            question("q-lo", "2026-01-01T00:00:00Z"),
            question("q-old", "2026-04-01T00:00:00Z"),
            question("q-hi", "2026-06-01T00:00:00Z"),
        ];
        let url = fake(claimed, all, questions).await;
        let got = items_from(vec![src("p", Some(url), None)], Duration::from_secs(2)).await;
        assert!(got.unreachable.is_empty());
        let p = &got.projects[0];
        let todos: Vec<_> = p.todos.iter().map(|t| t.task_id.as_str()).collect();
        assert_eq!(todos, ["t-hi", "t-old", "t-new"]);
        let decisions: Vec<_> = p.decisions.iter().map(|d| d.task_id.as_str()).collect();
        assert_eq!(decisions, ["q-hi", "q-old", "q-new", "q-lo"]);
        assert_eq!(p.decisions[0].title, "high one");
        assert_eq!(p.decisions[0].priority, Priority::High);
    }

    #[tokio::test]
    async fn withdrawn_todo_and_question_are_hidden_but_stay_on_the_record() {
        // The fake daemon's record is the `all` list; the gateway only reads it.
        let record = vec![
            task(
                "t-gone",
                "gone",
                "dropped",
                "normal",
                "2026-01-01T00:00:00Z",
            ),
            task(
                "q-gone",
                "gone q",
                "dropped",
                "normal",
                "2026-01-01T00:00:00Z",
            ),
            task(
                "q-live",
                "live q",
                "planned",
                "normal",
                "2026-01-01T00:00:00Z",
            ),
        ];
        let claimed = vec![record[0].clone()];
        let questions = vec![
            question("q-gone", "2026-01-01T00:00:00Z"),
            question("q-live", "2026-01-02T00:00:00Z"),
        ];
        let url = fake(claimed, record.clone(), questions).await;
        let got = items_from(
            vec![src("p", Some(url.clone()), None)],
            Duration::from_secs(2),
        )
        .await;
        let p = &got.projects[0];
        assert!(p.todos.is_empty());
        let decisions: Vec<_> = p.decisions.iter().map(|d| d.task_id.as_str()).collect();
        assert_eq!(decisions, ["q-live"]);
        // Nothing was deleted: the daemon still serves the withdrawn tasks.
        let all = Client::new_with_timeout(url, None, Duration::from_secs(2))
            .list_tasks()
            .await
            .expect("list");
        assert!(
            all.iter()
                .any(|t| t.id == "t-gone" && t.state == TaskState::Dropped)
        );
        assert!(all.iter().any(|t| t.id == "q-gone"));
    }

    #[tokio::test]
    async fn two_projects_merge_grouped_decisions_first_and_unreachable_reported() {
        let todo_only = fake(
            vec![task(
                "a-1",
                "a",
                "claimed",
                "normal",
                "2026-01-01T00:00:00Z",
            )],
            vec![],
            vec![],
        )
        .await;
        let with_question = fake(
            vec![],
            vec![task(
                "b-1",
                "b",
                "planned",
                "normal",
                "2026-01-01T00:00:00Z",
            )],
            vec![question("b-1", "2026-01-01T00:00:00Z")],
        )
        .await;
        let empty = fake(vec![], vec![], vec![]).await;
        let down = {
            let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
            format!("http://{}", l.local_addr().expect("addr"))
        };
        let sources = vec![
            src("a-todos", Some(todo_only), None),
            src("b-asks", Some(with_question), None),
            src("c-empty", Some(empty), None),
            src("d-down", Some(down), None),
            src("e-nodaemon", None, Some("no daemon is running for it")),
        ];
        let got = items_from(sources, Duration::from_millis(500)).await;
        let order: Vec<_> = got.projects.iter().map(|p| p.project.as_str()).collect();
        assert_eq!(order, ["b-asks", "a-todos"]);
        assert_eq!(got.projects[0].decisions.len(), 1);
        assert_eq!(got.projects[1].todos.len(), 1);
        let down: Vec<_> = got
            .unreachable
            .iter()
            .map(|u| (u.project.as_str(), u.reachable, u.reason.is_some()))
            .collect();
        assert_eq!(down, [("d-down", false, true), ("e-nodaemon", false, true)]);
    }
}
