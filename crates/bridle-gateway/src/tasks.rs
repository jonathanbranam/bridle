//! Read-only views of a project's tasks for the UI's Tasks page (ticket s6cj): the list, and one
//! task with its thread and edges. Types are the gateway's own, like `items.rs`. Reads with the
//! gateway's per-project credentials; nothing here changes a task. See
//! docs/design/human-web-ui.md section 2.

use axum::Json;
use axum::extract::{Path, Query};
use bridle_api::client::{Client, ClientError};
use bridle_api::types::{EdgeKind, Task, TaskState};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::actions::{ActionError, resolve};
use crate::discovery::PROBE_TIMEOUT;

/// Which tasks the list returns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateFilter {
    /// Everything not integrated or dropped.
    #[default]
    Open,
    /// Integrated or dropped.
    Closed,
    All,
}

impl StateFilter {
    fn keeps(self, state: TaskState) -> bool {
        let closed = matches!(state, TaskState::Integrated | TaskState::Dropped);
        match self {
            Self::Open => !closed,
            Self::Closed => closed,
            Self::All => true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    state: StateFilter,
}

/// The agent holding a task, when `claimed_by` names one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ClaimAgent {
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TaskSummary {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub state: String,
    pub priority: String,
    pub claimed_by: Option<String>,
    /// Set when `claimed_by` is `agent:<name>` and the daemon knows that agent.
    pub agent: Option<ClaimAgent>,
    /// RFC 3339, UTC.
    pub updated: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TaskList {
    pub project: String,
    /// Most recently updated first.
    pub tasks: Vec<TaskSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ThreadItem {
    pub kind: String,
    pub from: String,
    pub body: String,
    /// RFC 3339, UTC.
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TaskDetail {
    pub project: String,
    #[serde(flatten)]
    pub summary: TaskSummary,
    pub body: String,
    pub thread: Vec<ThreadItem>,
    pub watchers: Vec<String>,
    pub branch: Option<String>,
    /// The ticket the task was made from, if any.
    pub ticket: Option<String>,
    /// Ids of tasks this one blocks.
    pub blocks: Vec<String>,
    /// Ids of tasks that block this one.
    pub blocked_by: Vec<String>,
}

fn api_error(e: ClientError) -> ActionError {
    match e {
        ClientError::Api {
            status, message, ..
        } => ActionError::Refused { status, message },
        other => ActionError::Daemon(other.to_string()),
    }
}

/// The agent behind `agent:<name>`; an unknown one (or a daemon that can't list) is just absent.
async fn claim_agent(client: &Client, claimed_by: Option<&str>) -> Option<ClaimAgent> {
    let name = claimed_by?.strip_prefix("agent:")?;
    let agents = client.list_agents().await.ok()?;
    agents
        .into_iter()
        .find(|a| a.name == name)
        .map(|a| ClaimAgent {
            name: a.name,
            role: a.role,
        })
}

fn summary(t: &Task, agent: Option<ClaimAgent>) -> TaskSummary {
    TaskSummary {
        id: t.id.clone(),
        title: t.title.clone(),
        kind: t.kind.as_str().to_string(),
        state: t.state.as_str().to_string(),
        priority: format!("{:?}", t.priority).to_lowercase(),
        claimed_by: t.claimed_by.clone(),
        agent,
        updated: t.updated_at.to_rfc3339(),
    }
}

pub(crate) async fn list(
    client: &Client,
    project: &str,
    filter: StateFilter,
) -> Result<TaskList, ActionError> {
    let mut tasks = client.list_tasks().await.map_err(api_error)?;
    tasks.retain(|t| filter.keeps(t.state));
    tasks.sort_by_key(|t| std::cmp::Reverse(t.updated_at));
    // One agents lookup for the whole list.
    let agents = client.list_agents().await.unwrap_or_default();
    let tasks = tasks
        .iter()
        .map(|t| {
            let agent = t
                .claimed_by
                .as_deref()
                .and_then(|c| c.strip_prefix("agent:"))
                .and_then(|n| agents.iter().find(|a| a.name == n))
                .map(|a| ClaimAgent {
                    name: a.name.clone(),
                    role: a.role.clone(),
                });
            summary(t, agent)
        })
        .collect();
    Ok(TaskList {
        project: project.to_string(),
        tasks,
    })
}

pub(crate) async fn detail(
    client: &Client,
    project: &str,
    id: &str,
) -> Result<TaskDetail, ActionError> {
    let t = client.get_task(id).await.map_err(api_error)?;
    let edges = client.list_edges().await.map_err(api_error)?;
    let agent = claim_agent(client, t.claimed_by.as_deref()).await;
    let blocks = |from: bool| {
        edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Blocks)
            .filter_map(|e| match from {
                true if e.from == t.id => Some(e.to.clone()),
                false if e.to == t.id => Some(e.from.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    Ok(TaskDetail {
        project: project.to_string(),
        summary: summary(&t, agent),
        body: t.body.clone(),
        thread: t
            .thread
            .iter()
            .map(|e| ThreadItem {
                kind: e.kind.as_str().to_string(),
                from: e.from.clone(),
                body: e.body.clone(),
                at: e.at.to_rfc3339(),
            })
            .collect(),
        watchers: t.watchers.clone(),
        branch: t.branch.clone(),
        ticket: t.ticket.clone(),
        blocks: blocks(true),
        blocked_by: blocks(false),
    })
}

async fn client_for(project: &str) -> Result<Client, ActionError> {
    let (url, token) = resolve(project).await?;
    Ok(Client::new_with_timeout(
        url,
        Some(token),
        PROBE_TIMEOUT * 5,
    ))
}

/// `GET /api/v1/projects/{project}/tasks?state=open|closed|all`.
pub async fn list_route(
    Path(project): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<Json<TaskList>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(list(&client, &project, q.state).await?))
}

/// `GET /api/v1/projects/{project}/tasks/{id}`: any state.
pub async fn detail_route(
    Path((project, id)): Path<(String, String)>,
) -> Result<Json<TaskDetail>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(detail(&client, &project, &id).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use serde_json::{Value, json};

    fn task(id: &str, state: &str, claimed_by: Option<&str>, updated: &str) -> Value {
        json!({
            "id": id, "title": format!("title {id}"), "kind": "feature", "state": state,
            "body": "the ask", "thread": [{"kind": "note", "from": "human", "body": "hi",
            "at": updated}], "created_at": updated, "updated_at": updated,
            "claimed_by": claimed_by, "branch": "bridle/x", "watchers": ["human"],
        })
    }

    async fn fake() -> Client {
        let tasks = vec![
            task("t-1", "claimed", Some("agent:w1"), "2026-01-02T00:00:00Z"),
            task("t-2", "integrated", None, "2026-01-03T00:00:00Z"),
            task("t-3", "dropped", None, "2026-01-04T00:00:00Z"),
            task("t-4", "planned", None, "2026-01-05T00:00:00Z"),
        ];
        let edges = json!([
            {"from": "t-2", "to": "t-1", "kind": "blocks", "created_at": "2026-01-01T00:00:00Z"},
            {"from": "t-1", "to": "t-4", "kind": "blocks", "created_at": "2026-01-01T00:00:00Z"},
            {"from": "t-1", "to": "t-3", "kind": "related", "created_at": "2026-01-01T00:00:00Z"},
        ]);
        let agent = json!([{
            "id": "a-1", "name": "w1", "role": "worker", "state": "working", "model": "m",
            "session_id": "s", "pid": null, "cwd": "/", "worktree": null, "branch": null,
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z",
            "last_event_at": null, "turns": 0, "turn_started_at": null, "cost_usd_total": 0.0,
            "context_tokens": null, "exit": null, "created_by": "human",
            "held_messages": 0, "unacked_messages": 0,
        }]);
        let by_id = tasks.clone();
        let app = Router::new()
            .route("/v1/tasks", get(move || async move { Json(tasks) }))
            .route(
                "/v1/tasks/{id}",
                get(
                    move |axum::extract::Path(id): axum::extract::Path<String>| async move {
                        by_id
                            .into_iter()
                            .find(|t| t["id"] == id)
                            .map(Json)
                            .ok_or(axum::http::StatusCode::NOT_FOUND)
                    },
                ),
            )
            .route("/v1/edges", get(move || async move { Json(edges) }))
            .route("/v1/agents", get(move || async move { Json(agent) }));
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        Client::new_with_timeout(url, None, PROBE_TIMEOUT * 5)
    }

    fn ids(l: &TaskList) -> Vec<&str> {
        l.tasks.iter().map(|t| t.id.as_str()).collect()
    }

    #[tokio::test]
    async fn list_defaults_to_open_and_names_the_agent() {
        let c = fake().await;
        let l = list(&c, "p", StateFilter::default()).await.expect("list");
        assert_eq!(ids(&l), ["t-4", "t-1"]);
        let agent = l.tasks[1].agent.as_ref().expect("agent");
        assert_eq!((agent.name.as_str(), agent.role.as_str()), ("w1", "worker"));
        assert_eq!(l.tasks[0].agent, None);
    }

    #[tokio::test]
    async fn list_closed_and_all() {
        let c = fake().await;
        let closed = list(&c, "p", StateFilter::Closed).await.expect("closed");
        assert_eq!(ids(&closed), ["t-3", "t-2"]);
        let all = list(&c, "p", StateFilter::All).await.expect("all");
        assert_eq!(all.tasks.len(), 4);
    }

    #[tokio::test]
    async fn detail_of_a_closed_task_and_its_edges() {
        let c = fake().await;
        let d = detail(&c, "p", "t-2").await.expect("closed detail");
        assert_eq!(d.summary.state, "integrated");
        assert_eq!(d.body, "the ask");
        assert_eq!(d.thread.len(), 1);
        assert_eq!(d.blocks, ["t-1"]);
        assert!(d.blocked_by.is_empty());

        let d = detail(&c, "p", "t-1").await.expect("detail");
        assert_eq!(d.blocks, ["t-4"]);
        assert_eq!(d.blocked_by, ["t-2"]);
        assert_eq!(d.branch.as_deref(), Some("bridle/x"));
        assert_eq!(d.summary.agent.expect("agent").role, "worker");
    }

    #[tokio::test]
    async fn unknown_task_passes_the_daemons_404_through() {
        let c = fake().await;
        let err = detail(&c, "p", "nope").await.expect_err("missing");
        assert!(
            matches!(err, ActionError::Refused { status: 404, .. }),
            "{err}"
        );
    }

    #[tokio::test]
    async fn unknown_project_is_404() {
        use axum::response::IntoResponse;
        let err = client_for("no-such-project-s6cj")
            .await
            .expect_err("unknown");
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }
}
