//! Sessions with their start and end reasons, startup cost and context over turns (br-eapt, the
//! UI's context and cost pages): `GET /api/v1/usage/sessions?since=` asks every project's
//! daemon and lists each session with the project it ran in; a daemon that can't answer is
//! listed as unreachable, never counted as zero. The turns of one session come from its own
//! project's daemon: `GET /api/v1/projects/{project}/usage/sessions/{agent_id}/{session_id}/turns`.

use std::time::Duration;

use axum::Json;
use axum::extract::{Path, Query};
use bridle_api::client::Client;
use bridle_api::types::{UsageSession, UsageSessionsQuery};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::actions::ActionError;
use crate::discovery::{PROBE_TIMEOUT, ProjectStatus, current_targets};
use crate::items::{Source, source};
use crate::system::{api_error, client_for, rfc};

#[derive(Debug, Deserialize)]
pub struct SessionsParams {
    /// RFC 3339; only sessions with a turn or report at or after it.
    pub since: Option<DateTime<Utc>>,
}

/// One session and where it ran.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct SessionRow {
    pub project: String,
    pub machine: Option<String>,
    /// The hosted agent id; the identity for an external session.
    pub agent_id: String,
    pub session_id: String,
    pub name: String,
    pub role: String,
    pub model: String,
    pub external: bool,
    /// RFC 3339, UTC.
    pub started_at: String,
    /// Null while open, and for an external session.
    pub ended_at: Option<String>,
    /// `spawn`, `resume` or `renewal`; null for an external session or an old one.
    pub start_reason: Option<String>,
    pub end_reason: Option<String>,
    /// The context renewal limit (a fraction) in force at the start.
    pub renewal_limit: Option<f64>,
    /// Null for an external session.
    pub turns: Option<u32>,
    #[ts(type = "number")]
    pub input_tokens: u64,
    #[ts(type = "number")]
    pub output_tokens: u64,
    #[ts(type = "number")]
    pub cache_read_tokens: u64,
    #[ts(type = "number")]
    pub cache_write_tokens: u64,
    pub cost_usd: f64,
    #[ts(type = "number | null")]
    pub busy_seconds: Option<u64>,
    /// The first turn: what startup eats, and whether it was cached. Null for an external one.
    pub startup_cost_usd: Option<f64>,
    #[ts(type = "number | null")]
    pub startup_context_tokens: Option<u64>,
    #[ts(type = "number | null")]
    pub startup_cache_read_tokens: Option<u64>,
    #[ts(type = "number | null")]
    pub startup_cache_write_tokens: Option<u64>,
    #[ts(type = "number | null")]
    pub peak_context_tokens: Option<u64>,
    #[ts(type = "number | null")]
    pub end_context_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountSessions {
    /// Every project asked, with `reachable` and the reason it wasn't.
    pub projects: Vec<ProjectStatus>,
    pub since: Option<String>,
    /// Newest first, across the reachable projects.
    pub sessions: Vec<SessionRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct SessionTurnView {
    pub n: u32,
    /// RFC 3339, UTC.
    pub started_at: String,
    /// Null for a turn recorded before context sizes were.
    #[ts(type = "number | null")]
    pub context_tokens: Option<u64>,
    #[ts(type = "number")]
    pub input_tokens: u64,
    #[ts(type = "number")]
    pub output_tokens: u64,
    #[ts(type = "number")]
    pub cache_read_tokens: u64,
    #[ts(type = "number")]
    pub cache_write_tokens: u64,
    pub cost_usd: f64,
}

/// One session's turns, oldest first: the context growth line.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct SessionTurns {
    pub project: String,
    pub agent_id: String,
    pub session_id: String,
    pub turns: Vec<SessionTurnView>,
}

fn row(project: &str, machine: &Option<String>, s: UsageSession) -> SessionRow {
    let (startup_cost, startup_ctx, startup_read, startup_write) = match s.startup {
        Some(u) => (
            Some(u.cost_usd),
            u.context_tokens,
            Some(u.cache_read),
            Some(u.cache_write),
        ),
        None => (None, None, None, None),
    };
    SessionRow {
        project: project.to_string(),
        machine: machine.clone(),
        agent_id: s.agent_id,
        session_id: s.session_id,
        name: s.name,
        role: s.role,
        model: s.model,
        external: s.external,
        started_at: rfc(s.started_at),
        ended_at: s.ended_at.map(rfc),
        start_reason: s.start_reason,
        end_reason: s.end_reason,
        renewal_limit: s.renewal_limit,
        turns: s.turns,
        input_tokens: s.tokens.input,
        output_tokens: s.tokens.output,
        cache_read_tokens: s.tokens.cache_read,
        cache_write_tokens: s.tokens.cache_write,
        cost_usd: s.cost_usd,
        busy_seconds: s.busy_seconds,
        startup_cost_usd: startup_cost,
        startup_context_tokens: startup_ctx,
        startup_cache_read_tokens: startup_read,
        startup_cache_write_tokens: startup_write,
        peak_context_tokens: s.peak_context_tokens,
        end_context_tokens: s.end_context_tokens,
    }
}

async fn read_project(
    s: Source,
    params: &SessionsParams,
    timeout: Duration,
) -> (ProjectStatus, Vec<SessionRow>) {
    let mut status = ProjectStatus {
        project: s.project.clone(),
        machine: s.machine.clone(),
        url: s.url.clone(),
        reachable: false,
        reason: None,
    };
    if let Some(problem) = &s.problem {
        status.reason = Some(problem.clone());
        return (status, Vec::new());
    }
    let Some(url) = &s.url else {
        status.reason = Some("no address".to_string());
        return (status, Vec::new());
    };
    let client = Client::new_with_timeout(url.clone(), s.token.clone(), timeout);
    match client
        .usage_sessions(&UsageSessionsQuery {
            since: params.since,
        })
        .await
    {
        Ok(list) => {
            status.reachable = true;
            let rows = list
                .sessions
                .into_iter()
                .map(|x| row(&s.project, &s.machine, x))
                .collect();
            (status, rows)
        }
        Err(e) => {
            status.reason = Some(e.to_string());
            (status, Vec::new())
        }
    }
}

pub(crate) async fn account_sessions_from(
    sources: Vec<Source>,
    params: SessionsParams,
    timeout: Duration,
) -> AccountSessions {
    let reads = sources
        .into_iter()
        .map(|s| read_project(s, &params, timeout));
    let mut reads = futures::future::join_all(reads).await;
    reads.sort_by(|a, b| a.0.project.cmp(&b.0.project));
    let mut projects = Vec::new();
    let mut sessions = Vec::new();
    for (status, rows) in reads {
        projects.push(status);
        sessions.extend(rows);
    }
    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    AccountSessions {
        projects,
        since: params.since.map(rfc),
        sessions,
    }
}

/// `GET /api/v1/usage/sessions?since=`: see the module doc. Never an error for a down daemon.
pub async fn sessions_route(Query(params): Query<SessionsParams>) -> Json<AccountSessions> {
    let targets = current_targets().await;
    let sources = tokio::task::spawn_blocking(move || targets.into_iter().map(source).collect())
        .await
        .unwrap_or_default();
    Json(account_sessions_from(sources, params, PROBE_TIMEOUT * 5).await)
}

pub(crate) async fn session_turns(
    client: &Client,
    project: &str,
    agent_id: &str,
    session_id: &str,
) -> Result<SessionTurns, ActionError> {
    let turns = client
        .usage_session_turns(agent_id, session_id)
        .await
        .map_err(api_error)?;
    Ok(SessionTurns {
        project: project.to_string(),
        agent_id: agent_id.to_string(),
        session_id: session_id.to_string(),
        turns: turns
            .into_iter()
            .map(|t| SessionTurnView {
                n: t.n,
                started_at: rfc(t.started_at),
                context_tokens: t.context_tokens,
                input_tokens: t.tokens.input,
                output_tokens: t.tokens.output,
                cache_read_tokens: t.tokens.cache_read,
                cache_write_tokens: t.tokens.cache_write,
                cost_usd: t.cost_usd,
            })
            .collect(),
    })
}

/// `GET /api/v1/projects/{project}/usage/sessions/{agent_id}/{session_id}/turns`.
pub async fn session_turns_route(
    Path((project, agent_id, session_id)): Path<(String, String, String)>,
) -> Result<Json<SessionTurns>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(
        session_turns(&client, &project, &agent_id, &session_id).await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use serde_json::json;

    async fn daemon(project: &str, started: &str) -> String {
        let body = json!({
            "project": project, "machine": "m",
            "sessions": [{
                "agent_id": "a-1", "session_id": "s-1", "name": "w", "role": "worker",
                "model": "m", "started_at": started, "start_reason": "renewal",
                "end_reason": "exit", "renewal_limit": 0.5, "turns": 2,
                "tokens": {"input": 1, "output": 2, "cache_read": 3, "cache_write": 4},
                "cost_usd": 1.5, "busy_seconds": 9,
                "startup": {"cost_usd": 1.0, "context_tokens": 100, "cache_read": 0,
                            "cache_write": 40},
                "peak_context_tokens": 400, "end_context_tokens": 400,
            }],
        });
        let app = Router::new().route("/v1/usage/sessions", get(move || async move { Json(body) }));
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        url
    }

    fn src(project: &str, url: &str) -> Source {
        Source {
            project: project.into(),
            machine: Some("m".into()),
            url: Some(url.into()),
            token: None,
            problem: None,
        }
    }

    #[tokio::test]
    async fn sessions_keep_their_project_and_a_down_one_is_unreachable() {
        let a = daemon("alpha", "2026-10-10T10:00:00Z").await;
        let b = daemon("beta", "2026-10-11T10:00:00Z").await;
        let sources = vec![
            src("beta", &b),
            src("gamma", "http://127.0.0.1:1"),
            src("alpha", &a),
        ];
        let params = SessionsParams { since: None };
        let r = account_sessions_from(sources, params, Duration::from_secs(2)).await;
        let reach: Vec<_> = r
            .projects
            .iter()
            .map(|p| (p.project.as_str(), p.reachable))
            .collect();
        assert_eq!(reach, [("alpha", true), ("beta", true), ("gamma", false)]);
        assert!(r.projects[2].reason.is_some());
        // Newest first, each with its own project; the down project adds no row.
        let seen: Vec<_> = r.sessions.iter().map(|s| s.project.as_str()).collect();
        assert_eq!(seen, ["beta", "alpha"]);
        let s = &r.sessions[0];
        assert_eq!(s.start_reason.as_deref(), Some("renewal"));
        assert_eq!(s.renewal_limit, Some(0.5));
        assert_eq!(s.startup_cache_write_tokens, Some(40));
        assert_eq!(s.peak_context_tokens, Some(400));
    }
}
