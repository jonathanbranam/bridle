//! Read-only views of a project's tasks for the UI's Tasks page (ticket s6cj): the list, and one
//! task with its thread and edges. Types are the gateway's own, like `items.rs`. Reads with the
//! gateway's per-project credentials; nothing here changes a task. See
//! docs/design/human-web-ui.md section 2.

use axum::Json;
use axum::extract::{Path, Query};
use bridle_api::client::{Client, ClientError};
use bridle_api::types::{EdgeKind, Ticket, TicketState};
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
    fn keeps(self, state: TicketState) -> bool {
        let closed = matches!(state, TicketState::Integrated | TicketState::Dropped);
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
#[ts(rename = "TaskSummary")]
pub struct TicketSummary {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub state: String,
    pub priority: String,
    pub claimed_by: Option<String>,
    /// A human's to-do (claimed by `human`): the UI opens the to-do page for it, not the ticket
    /// page (br-bnfs).
    pub todo: bool,
    /// Set when `claimed_by` is `agent:<name>` and the daemon knows that agent.
    pub agent: Option<ClaimAgent>,
    /// RFC 3339, UTC.
    pub updated: String,
    /// While `held`: who paused it, why, and until when (free text). Null otherwise.
    pub held_by: Option<String>,
    pub held_reason: Option<String>,
    pub held_until: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "TaskList")]
pub struct TicketListByTasks {
    pub project: String,
    /// Most recently updated first.
    pub tasks: Vec<TicketSummary>,
}

/// `TicketListByTasks` under its ticket name (22ab step 8): the same rows, keyed `tickets`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct TicketList {
    pub project: String,
    /// Most recently updated first.
    pub tickets: Vec<TicketSummary>,
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
#[ts(rename = "TaskDetail")]
pub struct TicketDetail {
    pub project: String,
    #[serde(flatten)]
    pub summary: TicketSummary,
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

fn summary(t: &Ticket, agent: Option<ClaimAgent>) -> TicketSummary {
    TicketSummary {
        id: t.id.clone(),
        title: t.title.clone(),
        kind: t.kind.as_str().to_string(),
        state: t.state.as_str().to_string(),
        priority: format!("{:?}", t.priority).to_lowercase(),
        claimed_by: t.claimed_by.clone(),
        todo: t.claimed_by.as_deref() == Some("human") && t.state == TicketState::Claimed,
        agent,
        updated: t.updated_at.to_rfc3339(),
        held_by: t.held_by.as_ref().map(|p| p.to_string()),
        held_reason: t.held_reason.clone(),
        held_until: t.held_until.clone(),
    }
}

pub(crate) async fn list(
    client: &Client,
    project: &str,
    filter: StateFilter,
) -> Result<TicketListByTasks, ActionError> {
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
    Ok(TicketListByTasks {
        project: project.to_string(),
        tasks,
    })
}

/// The row id a link's `id` names (br-wayb). The daemon knows only project-qualified row ids,
/// but links carry a bare id (`btdn`) or a ticket file's id whose row has another id
/// (`7d62` -> `br-10c0`). An id the daemon has as is stays as is; otherwise a unique row whose
/// id tail, or whose ticket's tail, is the given tail wins. No unique match leaves `id` for the
/// daemon to refuse.
pub(crate) async fn resolve_id(client: &Client, id: &str) -> String {
    let Ok(rows) = client.list_tasks().await else {
        return id.to_string();
    };
    if rows.iter().any(|t| t.id == id) {
        return id.to_string();
    }
    let tail = |s: &str| s.split_once('-').map_or(s, |(_, t)| t).to_string();
    let want = tail(id);
    let by_row: Vec<_> = rows.iter().filter(|t| tail(&t.id) == want).collect();
    let by_file: Vec<_> = rows
        .iter()
        .filter(|t| t.ticket.as_deref().is_some_and(|f| tail(f) == want))
        .collect();
    match (by_row.as_slice(), by_file.as_slice()) {
        ([t], _) | ([], [t]) => t.id.clone(),
        _ => id.to_string(),
    }
}

pub(crate) async fn detail(
    client: &Client,
    project: &str,
    id: &str,
) -> Result<TicketDetail, ActionError> {
    let id = &resolve_id(client, id).await;
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
    Ok(TicketDetail {
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
) -> Result<Json<TicketListByTasks>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(list(&client, &project, q.state).await?))
}

/// `GET /api/v1/projects/{project}/tickets?state=...`: `list_route` with the list keyed `tickets`.
pub async fn ticket_list_route(
    project: Path<String>,
    q: Query<ListQuery>,
) -> Result<Json<TicketList>, ActionError> {
    let Json(l) = list_route(project, q).await?;
    Ok(Json(TicketList {
        project: l.project,
        tickets: l.tasks,
    }))
}

/// `GET /api/v1/projects/{project}/tasks/{id}`: any state.
pub async fn detail_route(
    Path((project, id)): Path<(String, String)>,
) -> Result<Json<TicketDetail>, ActionError> {
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
            .route("/v1/tickets", get(move || async move { Json(tasks) }))
            .route(
                "/v1/tickets/{id}",
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

    fn ids(l: &TicketListByTasks) -> Vec<&str> {
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

    #[test]
    fn a_task_the_human_holds_is_marked_a_todo() {
        let of = |state, by| {
            let t: Ticket =
                serde_json::from_value(task("t-9", state, by, "2026-01-02T00:00:00Z")).expect("t");
            summary(&t, None).todo
        };
        assert!(of("claimed", Some("human")));
        assert!(!of("integrated", Some("human")));
        assert!(!of("claimed", Some("agent:w1")));
        assert!(!of("planned", None));
    }

    #[test]
    fn ticket_list_is_the_task_list_keyed_tickets() {
        let l = TicketList {
            project: "p".into(),
            tickets: vec![],
        };
        assert_eq!(
            serde_json::to_value(&l).expect("json"),
            json!({"project": "p", "tickets": []})
        );
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
    async fn links_with_bare_or_file_ids_open_the_row() {
        let mut rows = vec![task("br-btdn", "planned", None, "2026-01-05T00:00:00Z")];
        let mut moved = task("br-10c0", "planned", None, "2026-01-05T00:00:00Z");
        moved["ticket"] = json!("br-7d62");
        rows.push(moved);
        let by_id = rows.clone();
        let app = Router::new()
            .route("/v1/tickets", get(move || async move { Json(rows) }))
            .route(
                "/v1/tickets/{id}",
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
            .route("/v1/edges", get(|| async { Json(json!([])) }))
            .route("/v1/agents", get(|| async { Json(json!([])) }));
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        let c = Client::new_with_timeout(url, None, PROBE_TIMEOUT * 5);
        for (link, row) in [
            ("btdn", "br-btdn"),
            ("br-btdn", "br-btdn"),
            ("7d62", "br-10c0"),
            ("br-7d62", "br-10c0"),
            ("br-10c0", "br-10c0"),
        ] {
            let d = detail(&c, "p", link).await.expect(link);
            assert_eq!(d.summary.id, row, "{link}");
        }
        assert!(detail(&c, "p", "zzzz").await.is_err());
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

    #[tokio::test]
    async fn held_and_built_states_and_held_fields_serialize() {
        let mut held = task("t-h", "held", None, "2026-01-06T00:00:00Z");
        held["held_by"] = json!("human");
        held["held_reason"] = json!("waiting on a design call");
        held["held_until"] = json!("friday");
        let built = task("t-b", "built", None, "2026-01-07T00:00:00Z");
        let (h, b) = (
            serde_json::from_value::<Ticket>(held).expect("held"),
            serde_json::from_value::<Ticket>(built).expect("built"),
        );
        let (h, b) = (summary(&h, None), summary(&b, None));
        assert_eq!((h.state.as_str(), b.state.as_str()), ("held", "built"));
        let v = serde_json::to_value(&h).expect("json");
        assert_eq!(v["held_by"], "human");
        assert_eq!(v["held_reason"], "waiting on a design call");
        assert_eq!(v["held_until"], "friday");
        assert!(serde_json::to_value(&b).expect("json")["held_by"].is_null());
        // Neither is closed, so both stay in the default (open) list.
        assert!(StateFilter::Open.keeps(TicketState::Held));
        assert!(StateFilter::Open.keeps(TicketState::Built));
    }
}
