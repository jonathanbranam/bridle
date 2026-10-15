//! Read-only views of a project's daemon for the UI's System page (ticket 7sd9): its status and
//! its agents. Types are the gateway's own, like `tasks.rs`, so nothing the daemon holds that is
//! secret or machine-local can leak by being added to its wire types: only the fields below pass.
//! Dropped on purpose: the principal, the daemon's workspace, repo and URL, an agent's session
//! id, pid, cwd and worktree, and a session's pid, pane and Claude session id. The daemon's token
//! is never in a status or agent record. See docs/design/human-web-ui.md section 2.

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::{Path, Query};
use bridle_api::client::{Client, ClientError};
use bridle_api::types::{
    Agent, Status, TicketState, UsageBucket, UsageHistoryQuery, UsageInstancesQuery,
    UsageSeriesQuery,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::actions::{ActionError, resolve};
use crate::discovery::PROBE_TIMEOUT;

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct RateLimitView {
    pub window: String,
    pub status: Option<String>,
    pub utilization: Option<f64>,
    /// RFC 3339, UTC.
    pub resets_at: Option<String>,
}

/// One reading in a window's history; a flat stretch is one point (see the daemon's
/// `/v1/usage/history`).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct UsagePoint {
    pub utilization: Option<f64>,
    /// RFC 3339, UTC.
    pub resets_at: Option<String>,
    /// RFC 3339, UTC.
    pub observed_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct UsageHistory {
    pub project: String,
    pub window: String,
    /// Oldest first.
    pub points: Vec<UsagePoint>,
}

#[derive(Debug, Deserialize)]
pub struct UsageHistoryParams {
    pub window: String,
    /// RFC 3339; only readings at or after it.
    pub since: Option<chrono::DateTime<chrono::Utc>>,
}

/// One agent instance with usage in a period (the daemon's `/v1/usage/agents`).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct UsageInstanceView {
    pub agent_id: String,
    pub name: String,
    pub role: String,
    pub model: String,
    pub removed: bool,
    /// An external (unbridled) session: `agent_id` is its identity, `turns` is null.
    pub external: bool,
    pub turns: Option<u32>,
    pub cost_usd: f64,
    /// RFC 3339, UTC.
    pub first_turn_at: String,
    /// RFC 3339, UTC.
    pub last_turn_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct UsageInstances {
    pub project: String,
    /// Latest turn first.
    pub instances: Vec<UsageInstanceView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct UsageSeriesBucket {
    /// RFC 3339, UTC: the start of the bucket.
    pub start: String,
    /// Null for an external session (not measured).
    pub turns: Option<u32>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub cost_usd: f64,
    /// Null for an external session (not measured).
    pub busy_seconds: Option<u64>,
}

/// One instance's usage by hour or day (the daemon's `/v1/usage/agents/{id}/series`). Empty
/// buckets are absent.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct UsageSeriesView {
    pub project: String,
    pub agent_id: String,
    /// `hour` or `day`.
    pub bucket: String,
    /// Oldest first.
    pub points: Vec<UsageSeriesBucket>,
}

#[derive(Debug, Deserialize)]
pub struct UsageSinceParams {
    /// RFC 3339; only turns started at or after it.
    pub since: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UsageSeriesParams {
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    /// `hour` or `day` (default `day`).
    pub bucket: Option<UsageBucket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CiView {
    pub sha: String,
    pub conclusion: String,
    pub url: Option<String>,
    pub completed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct IncidentView {
    pub id: String,
    pub title: String,
    pub since: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SessionView {
    pub identity: String,
    pub started_at: String,
    #[ts(type = "number | null")]
    pub tokens: Option<u64>,
    pub last_activity: Option<String>,
    pub project: Option<String>,
    pub machine: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct SystemStatus {
    pub pid: i32,
    pub version: String,
    /// RFC 3339, UTC.
    pub started_at: String,
    pub claude_version: Option<String>,
    /// `normal`, `holding`, `winding_down` or `paused`.
    pub budget_state: String,
    pub rate_limits: Vec<RateLimitView>,
    pub ci: Option<CiView>,
    pub incidents: Vec<IncidentView>,
    pub sessions: Vec<SessionView>,
    /// Agent counts keyed by state.
    pub agents_by_state: BTreeMap<String, u32>,
    pub unread_human_messages: u32,
    /// Short sha of a green build waiting for a quiet point to restart into.
    pub upgrade_waiting: Option<String>,
}

/// `reachable: false` (with `error`) when the daemon can't be asked: the UI shows "unreachable",
/// not a failure of the gateway.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct SystemView {
    pub project: String,
    pub reachable: bool,
    pub error: Option<String>,
    pub status: Option<SystemStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AgentView {
    pub name: String,
    pub role: String,
    pub state: String,
    /// Not running any more (stopped, exited, crashed or lost): the UI may hide these.
    pub stopped: bool,
    pub model: String,
    /// The task this agent holds, when one is claimed by `agent:<name>`.
    pub task: Option<String>,
    pub branch: Option<String>,
    #[ts(type = "number | null")]
    pub context_tokens: Option<u64>,
    pub cost_usd_total: f64,
    pub turns: u32,
    /// Why it stopped, when it has.
    pub exit_reason: Option<String>,
    pub updated: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AgentList {
    pub project: String,
    /// Running agents first, then stopped ones; most recently updated first within each.
    pub agents: Vec<AgentView>,
}

pub(crate) fn rfc(t: chrono::DateTime<chrono::Utc>) -> String {
    t.to_rfc3339()
}

fn system_status(s: Status) -> SystemStatus {
    SystemStatus {
        pid: s.daemon.pid,
        version: s.daemon.version,
        started_at: rfc(s.daemon.started_at),
        claude_version: s.claude_version,
        budget_state: s.budget_state.as_str().to_string(),
        rate_limits: s
            .rate_limits
            .into_iter()
            .map(|r| RateLimitView {
                window: r.window,
                status: r.status,
                utilization: r.utilization,
                resets_at: r.resets_at.map(rfc),
            })
            .collect(),
        ci: s.ci.map(|c| CiView {
            sha: c.sha,
            conclusion: c.conclusion,
            url: c.url,
            completed_at: rfc(c.completed_at),
        }),
        incidents: s
            .incidents
            .into_iter()
            .map(|i| IncidentView {
                id: i.id,
                title: i.title,
                since: rfc(i.since),
            })
            .collect(),
        sessions: s
            .sessions
            .into_iter()
            .map(|x| SessionView {
                identity: x.identity,
                started_at: rfc(x.started_at),
                tokens: x.tokens,
                last_activity: x.last_activity.map(rfc),
                project: x.project,
                machine: x.machine,
            })
            .collect(),
        agents_by_state: s.agents_by_state,
        unread_human_messages: s.unread_human_messages,
        upgrade_waiting: s.upgrade_waiting,
    }
}

pub(crate) fn api_error(e: ClientError) -> ActionError {
    match e {
        ClientError::Api {
            status, message, ..
        } => ActionError::Refused { status, message },
        other => ActionError::Daemon(other.to_string()),
    }
}

pub(crate) async fn system(client: &Client, project: &str) -> SystemView {
    let (status, error) = match client.status().await {
        Ok(s) => (Some(system_status(s)), None),
        Err(e) => (None, Some(e.to_string())),
    };
    SystemView {
        project: project.to_string(),
        reachable: status.is_some(),
        error,
        status,
    }
}

pub(crate) async fn agents(client: &Client, project: &str) -> Result<AgentList, ActionError> {
    let mut agents: Vec<Agent> = client.list_agents().await.map_err(api_error)?;
    // Best effort: the list is still useful without who holds what.
    let tasks = client.list_tasks().await.unwrap_or_default();
    agents.sort_by_key(|a| (!a.state.is_running(), std::cmp::Reverse(a.updated_at)));
    let agents = agents
        .into_iter()
        .map(|a| {
            let me = format!("agent:{}", a.name);
            let task = tasks
                .iter()
                .find(|t| {
                    t.claimed_by.as_deref() == Some(me.as_str())
                        && !matches!(t.state, TicketState::Integrated | TicketState::Dropped)
                })
                .map(|t| t.id.clone());
            AgentView {
                stopped: !a.state.is_running(),
                state: a.state.as_str().to_string(),
                name: a.name,
                role: a.role,
                model: a.model,
                task,
                branch: a.branch,
                context_tokens: a.context_tokens,
                cost_usd_total: a.cost_usd_total,
                turns: a.turns,
                exit_reason: a.exit.map(|e| e.reason),
                updated: rfc(a.updated_at),
            }
        })
        .collect();
    Ok(AgentList {
        project: project.to_string(),
        agents,
    })
}

pub(crate) async fn usage_history(
    client: &Client,
    project: &str,
    params: UsageHistoryParams,
) -> Result<UsageHistory, ActionError> {
    let query = UsageHistoryQuery {
        window: params.window.clone(),
        since: params.since,
    };
    let mut points = client.usage_history(&query).await.map_err(api_error)?;
    points.sort_by_key(|p| p.observed_at);
    Ok(UsageHistory {
        project: project.to_string(),
        window: params.window,
        points: points
            .into_iter()
            .map(|p| UsagePoint {
                utilization: p.utilization,
                resets_at: p.resets_at.map(rfc),
                observed_at: rfc(p.observed_at),
            })
            .collect(),
    })
}

pub(crate) async fn usage_instances(
    client: &Client,
    project: &str,
    params: UsageSinceParams,
) -> Result<UsageInstances, ActionError> {
    let list = client
        .usage_instances(&UsageInstancesQuery {
            since: params.since,
        })
        .await
        .map_err(api_error)?;
    Ok(UsageInstances {
        project: project.to_string(),
        instances: list
            .into_iter()
            .map(|i| UsageInstanceView {
                agent_id: i.agent_id,
                name: i.name,
                role: i.role,
                model: i.model,
                removed: i.removed,
                external: i.external,
                turns: i.turns,
                cost_usd: i.cost_usd,
                first_turn_at: rfc(i.first_turn_at),
                last_turn_at: rfc(i.last_turn_at),
            })
            .collect(),
    })
}

pub(crate) async fn usage_series(
    client: &Client,
    project: &str,
    agent_id: &str,
    params: UsageSeriesParams,
) -> Result<UsageSeriesView, ActionError> {
    let s = client
        .usage_series(
            agent_id,
            &UsageSeriesQuery {
                since: params.since,
                bucket: params.bucket,
            },
        )
        .await
        .map_err(api_error)?;
    Ok(UsageSeriesView {
        project: project.to_string(),
        agent_id: s.agent_id,
        bucket: match s.bucket {
            UsageBucket::Hour => "hour",
            UsageBucket::Day => "day",
        }
        .to_string(),
        points: s
            .points
            .into_iter()
            .map(|p| UsageSeriesBucket {
                start: rfc(p.start),
                turns: p.turns,
                input_tokens: p.tokens.input,
                output_tokens: p.tokens.output,
                cache_read_tokens: p.tokens.cache_read,
                cache_write_tokens: p.tokens.cache_write,
                cost_usd: p.cost_usd,
                busy_seconds: p.busy_seconds,
            })
            .collect(),
    })
}

pub(crate) async fn client_for(project: &str) -> Result<Client, ActionError> {
    let (url, token) = resolve(project).await?;
    Ok(Client::new_with_timeout(
        url,
        Some(token),
        PROBE_TIMEOUT * 5,
    ))
}

/// `GET /api/v1/projects/{project}/system`. An unknown project is 404; a daemon that is down
/// (or whose address or token can't be found) is a 200 with `reachable: false`.
pub async fn system_route(Path(project): Path<String>) -> Result<Json<SystemView>, ActionError> {
    match client_for(&project).await {
        Ok(client) => Ok(Json(system(&client, &project).await)),
        Err(e @ ActionError::UnknownProject(_)) => Err(e),
        Err(e) => Ok(Json(SystemView {
            project,
            reachable: false,
            error: Some(e.to_string()),
            status: None,
        })),
    }
}

/// `GET /api/v1/projects/{project}/agents`, stopped agents included (flagged).
pub async fn agents_route(Path(project): Path<String>) -> Result<Json<AgentList>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(agents(&client, &project).await?))
}

/// `GET /api/v1/projects/{project}/usage/history?window=&since=`: the account's rate-limit
/// readings for one window, oldest first. The limits are account-wide; the project only picks
/// which daemon is asked.
pub async fn usage_history_route(
    Path(project): Path<String>,
    Query(params): Query<UsageHistoryParams>,
) -> Result<Json<UsageHistory>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(usage_history(&client, &project, params).await?))
}

/// `GET /api/v1/projects/{project}/usage/agents?since=`: the agent instances (live and removed)
/// with usage in the period, for the chart's picker.
pub async fn usage_instances_route(
    Path(project): Path<String>,
    Query(params): Query<UsageSinceParams>,
) -> Result<Json<UsageInstances>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(usage_instances(&client, &project, params).await?))
}

/// `GET /api/v1/projects/{project}/usage/agents/{agent_id}/series?since=&bucket=`.
pub async fn usage_series_route(
    Path((project, agent_id)): Path<(String, String)>,
    Query(params): Query<UsageSeriesParams>,
) -> Result<Json<UsageSeriesView>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(
        usage_series(&client, &project, &agent_id, params).await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use serde_json::{Value, json};

    fn agent(name: &str, state: &str, updated: &str) -> Value {
        json!({
            "id": format!("a-{name}"), "name": name, "role": "worker", "state": state,
            "model": "m", "session_id": "SECRET-SESSION", "pid": 4242, "cwd": "/secret/cwd",
            "worktree": "/secret/wt", "branch": "bridle/x", "created_at": updated,
            "updated_at": updated, "last_event_at": null, "turns": 3,
            "turn_started_at": null, "cost_usd_total": 1.5, "context_tokens": 1000,
            "exit": null, "created_by": "human", "held_messages": 0, "unacked_messages": 0,
        })
    }

    fn status() -> Value {
        json!({
            "daemon": {"project": "p", "workspace": "/secret/ws", "repo": "/secret/repo",
                "url": "http://127.0.0.1:1", "pid": 99,
                "started_at": "2026-01-01T00:00:00Z", "version": "1.2.3"},
            "principal": "human", "agents_by_state": {"working": 1}, "unread_human_messages": 2,
            "rate_limits": [{"window": "five_hour", "status": "allowed", "utilization": 0.5,
                "resets_at": null, "observed_at": "2026-01-01T00:00:00Z"}],
            "budget_state": "holding", "upgrade_waiting": "abc1234",
            "incidents": [{"id": "i-1", "title": "down", "since": "2026-01-01T00:00:00Z"}],
            "sessions": [{"identity": "advisor", "pid": 7, "pane": "%SECRET-PANE",
                "claude_session_id": "SECRET-CLAUDE", "started_at": "2026-01-01T00:00:00Z"}],
        })
    }

    async fn serve(app: Router) -> Client {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        Client::new_with_timeout(url, None, PROBE_TIMEOUT * 5)
    }

    async fn fake() -> Client {
        let agents = json!([
            agent("old", "stopped", "2026-01-05T00:00:00Z"),
            agent("w1", "working", "2026-01-02T00:00:00Z"),
        ]);
        let tasks = json!([{
            "id": "t-1", "title": "x", "kind": "feature", "state": "claimed", "body": "",
            "thread": [], "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z", "claimed_by": "agent:w1", "watchers": [],
        }]);
        serve(
            Router::new()
                .route("/v1/status", get(|| async { Json(status()) }))
                .route("/v1/agents", get(move || async move { Json(agents) }))
                .route("/v1/tickets", get(move || async move { Json(tasks) })),
        )
        .await
    }

    #[tokio::test]
    async fn status_passes_through() {
        let v = system(&fake().await, "p").await;
        assert!(v.reachable);
        let s = v.status.expect("status");
        assert_eq!((s.pid, s.version.as_str()), (99, "1.2.3"));
        assert_eq!(s.budget_state, "holding");
        assert_eq!(s.upgrade_waiting.as_deref(), Some("abc1234"));
        assert_eq!(s.rate_limits[0].utilization, Some(0.5));
        assert_eq!(s.incidents[0].id, "i-1");
        assert_eq!(s.sessions[0].identity, "advisor");
        assert_eq!(s.unread_human_messages, 2);
    }

    #[tokio::test]
    async fn down_daemon_is_unreachable_not_an_error() {
        // Nothing listens on port 1.
        let c = Client::new_with_timeout("http://127.0.0.1:1".to_string(), None, PROBE_TIMEOUT);
        let v = system(&c, "p").await;
        assert!(!v.reachable);
        assert!(v.status.is_none());
        assert!(v.error.is_some());
    }

    #[tokio::test]
    async fn agents_list_flags_stopped_and_names_the_task() {
        let l = agents(&fake().await, "p").await.expect("agents");
        let names: Vec<_> = l.agents.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["w1", "old"]);
        assert!(!l.agents[0].stopped && l.agents[1].stopped);
        assert_eq!(l.agents[0].task.as_deref(), Some("t-1"));
        assert_eq!(l.agents[1].task, None);
        assert_eq!(l.agents[0].context_tokens, Some(1000));
    }

    #[tokio::test]
    async fn secrets_and_machine_paths_are_stripped() {
        let c = fake().await;
        let out = format!(
            "{}{}",
            serde_json::to_string(&system(&c, "p").await).expect("json"),
            serde_json::to_string(&agents(&c, "p").await.expect("agents")).expect("json"),
        );
        for secret in ["SECRET", "/secret", "4242", "principal", "127.0.0.1"] {
            assert!(!out.contains(secret), "{secret} leaked: {out}");
        }
    }

    #[tokio::test]
    async fn usage_history_is_time_ordered_and_passes_since() {
        let seen = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let seen2 = seen.clone();
        let c = serve(Router::new().route(
            "/v1/usage/history",
            get(
                move |axum::extract::RawQuery(q): axum::extract::RawQuery| async move {
                    *seen2.lock().expect("lock") = q.unwrap_or_default();
                    Json(json!([
                        {"window": "five_hour", "utilization": 0.9, "resets_at": null,
                            "observed_at": "2026-01-02T00:00:00Z"},
                        {"window": "five_hour", "utilization": 0.2,
                            "resets_at": "2026-01-01T05:00:00Z",
                            "observed_at": "2026-01-01T00:00:00Z"},
                    ]))
                },
            ),
        ))
        .await;
        let params = UsageHistoryParams {
            window: "five_hour".into(),
            since: Some("2026-01-01T00:00:00Z".parse().expect("time")),
        };
        let h = usage_history(&c, "p", params).await.expect("history");
        let u: Vec<_> = h.points.iter().map(|p| p.utilization).collect();
        assert_eq!(u, [Some(0.2), Some(0.9)]);
        assert_eq!(h.window, "five_hour");
        let q = seen.lock().expect("lock").clone();
        assert!(
            q.contains("window=five_hour") && q.contains("since=2026-01-01"),
            "{q}"
        );
    }

    #[tokio::test]
    async fn usage_series_and_instances_map_the_daemon_shapes() {
        let c = serve(
            Router::new()
                .route(
                    "/v1/usage/agents",
                    get(|| async {
                        Json(json!([{
                            "agent_id": "a-1", "name": "w1", "role": "worker", "model": "m",
                            "removed": true, "turns": 2, "cost_usd": 1.5,
                            "first_turn_at": "2026-01-01T00:00:00Z",
                            "last_turn_at": "2026-01-02T00:00:00Z"
                        }]))
                    }),
                )
                .route(
                    "/v1/usage/agents/{id}/series",
                    get(|| async {
                        Json(json!({
                            "agent_id": "a-1", "bucket": "hour",
                            "points": [{
                                "start": "2026-01-01T00:00:00Z", "turns": 2,
                                "tokens": {"input": 10, "output": 1, "cache_read": 5,
                                    "cache_write": 2},
                                "cost_usd": 1.5, "busy_seconds": 30
                            }]
                        }))
                    }),
                ),
        )
        .await;
        let l = usage_instances(&c, "p", UsageSinceParams { since: None })
            .await
            .expect("instances");
        assert!(l.instances[0].removed);
        assert_eq!(l.instances[0].last_turn_at, "2026-01-02T00:00:00+00:00");
        let s = usage_series(
            &c,
            "p",
            "a-1",
            UsageSeriesParams {
                since: None,
                bucket: Some(UsageBucket::Hour),
            },
        )
        .await
        .expect("series");
        assert_eq!(s.bucket, "hour");
        assert_eq!(s.points[0].cache_read_tokens, 5);
        assert_eq!(s.points[0].busy_seconds, Some(30));
    }

    #[tokio::test]
    async fn usage_history_unknown_project_is_404() {
        use axum::response::IntoResponse;
        let params = UsageHistoryParams {
            window: "five_hour".into(),
            since: None,
        };
        let err = usage_history_route(Path("no-such-project-yydm".into()), Query(params))
            .await
            .expect_err("unknown");
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn unknown_project_is_404() {
        use axum::response::IntoResponse;
        let err = system_route(Path("no-such-project-7sd9".into()))
            .await
            .expect_err("unknown");
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }
}
