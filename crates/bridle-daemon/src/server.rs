//! The axum HTTP + SSE API. Every route in docs/design/agent-host/api.md.

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use bridle_api::types::{
    Agent, ApiErrorResponse, BudgetStatus, ErrorBody, Event, EventQuery, Health, InterruptRequest,
    Message, MessageQuery, MessageState, PrincipalKind, RemoveQuery, SendRequest, SpawnRequest,
    Status, StopRequest, TokenCreateRequest, TokenCreated, TranscriptLine, TranscriptQuery, Usage,
    WindowStatus,
};
use chrono::Utc;
use futures::Stream;
use serde::Deserialize;
use tokio::sync::watch;

use crate::events::Emitter;
use crate::paths::Workspace;
use crate::store::{Principal, Store, StoreError};
use crate::supervisor::{AgentManager, SupervisorError, ToTarget};

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub manager: AgentManager,
    pub emitter: Emitter,
    pub workspace: Workspace,
    pub project: String,
    pub repo: String,
    pub url: String,
    pub version: String,
    pub started_at: chrono::DateTime<Utc>,
    pub pid: i32,
    pub shutdown_tx: watch::Sender<bool>,
    pub governor: crate::governor::Governor,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/status", get(status))
        .route("/v1/agents", get(list_agents).post(spawn_agent))
        .route("/v1/agents/{id}", get(get_agent).delete(remove_agent))
        .route("/v1/agents/{id}/messages", post(send_to_agent))
        .route("/v1/agents/{id}/interrupt", post(interrupt_agent))
        .route("/v1/agents/{id}/stop", post(stop_agent))
        .route("/v1/agents/{id}/resume", post(resume_agent))
        .route("/v1/agents/{id}/transcript", get(transcript))
        .route("/v1/messages", get(list_messages).post(send_message))
        .route("/v1/messages/{id}/read", post(mark_read))
        .route("/v1/events", get(list_events))
        .route("/v1/events/stream", get(events_stream))
        .route("/v1/usage", get(usage))
        .route("/v1/budget", get(budget))
        .route("/v1/tokens", post(create_token))
        .route("/v1/shutdown", post(shutdown))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

// ---------- errors ----------

struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", msg)
    }
    fn forbidden(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", msg)
    }
    fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorResponse {
                error: ErrorBody {
                    code: self.code.to_string(),
                    message: self.message,
                },
            }),
        )
            .into_response()
    }
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::NotFound(m) => ApiError::not_found(m),
            StoreError::Conflict(m) => ApiError::new(StatusCode::CONFLICT, "conflict", m),
            other => ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                other.to_string(),
            ),
        }
    }
}

impl From<SupervisorError> for ApiError {
    fn from(e: SupervisorError) -> Self {
        match e {
            SupervisorError::NotFound(m) => ApiError::not_found(m),
            SupervisorError::Conflict(m) => ApiError::new(StatusCode::CONFLICT, "conflict", m),
            SupervisorError::BadRequest(m) => ApiError::bad_request(m),
            SupervisorError::AgentNotRunning(m) => {
                ApiError::new(StatusCode::CONFLICT, "agent_not_running", m)
            }
            SupervisorError::Internal(m) => {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", m)
            }
        }
    }
}

// ---------- auth ----------

async fn auth_middleware(
    State(state): State<AppState>,
    mut req: axum::extract::Request,
    next: Next,
) -> Response {
    if req.uri().path() == "/v1/health" {
        return next.run(req).await;
    }
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);
    let Some(token) = token else {
        return ApiError::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "missing bearer token",
        )
        .into_response();
    };
    match state.store.authenticate(&token).await {
        Ok(Some(principal)) => {
            req.extensions_mut().insert(principal);
            next.run(req).await
        }
        Ok(None) => {
            ApiError::new(StatusCode::UNAUTHORIZED, "unauthorized", "invalid token").into_response()
        }
        Err(e) => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            .into_response(),
    }
}

fn require_human(principal: &Principal) -> Result<(), ApiError> {
    if principal.kind == PrincipalKind::Human {
        Ok(())
    } else {
        Err(ApiError::forbidden("this endpoint is human-only"))
    }
}

// ---------- health / status ----------

async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        ok: true,
        version: state.version.clone(),
    })
}

async fn status(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<Json<Status>, ApiError> {
    let agents_by_state = state.store.agents_by_state().await?;
    let unread = state.store.unread_count("human").await?;
    let rate_limits = state.store.rate_limits().await?;
    let claude_version = state.store.get_meta("claude_version").await?;
    Ok(Json(Status {
        daemon: bridle_api::types::DaemonInfo {
            project: state.project.clone(),
            workspace: state.workspace.workspace.to_string_lossy().into_owned(),
            repo: state.repo.clone(),
            url: state.url.clone(),
            pid: state.pid,
            started_at: state.started_at,
            version: state.version.clone(),
        },
        principal: principal.id,
        agents_by_state,
        unread_human_messages: unread,
        rate_limits,
        claude_version,
        budget_state: state.governor.snapshot().default.state,
    }))
}

// ---------- budget ----------

async fn budget(State(state): State<AppState>) -> Result<Json<BudgetStatus>, ApiError> {
    let snapshot = state.governor.snapshot();
    let rate_limits = state.store.rate_limits().await?;
    let mut windows = Vec::new();
    for window in ["five_hour", "seven_day", "seven_day_opus", "seven_day_sonnet"] {
        let rl = rate_limits.iter().find(|r| r.window == window);
        let block = if window == "seven_day_opus" {
            snapshot.per_model.get("opus").cloned().unwrap_or_default()
        } else if window == "seven_day_sonnet" {
            snapshot.per_model.get("sonnet").cloned().unwrap_or_default()
        } else if snapshot.default.window.as_deref() == Some(window) {
            snapshot.default.clone()
        } else {
            crate::governor::WindowBlock::default()
        };
        windows.push(WindowStatus {
            window: window.to_string(),
            state: block.state,
            status: rl.and_then(|r| r.status.clone()),
            utilization: rl.and_then(|r| r.utilization),
            resets_at: rl.and_then(|r| r.resets_at),
            observed_at: rl.map(|r| r.observed_at),
            stale: rl.is_none(),
        });
    }
    let cfg = state.governor.config();
    Ok(Json(BudgetStatus {
        state: snapshot.default.state,
        windows,
        thresholds: cfg.to_wire(),
    }))
}

// ---------- agents ----------

async fn list_agents(State(state): State<AppState>) -> Result<Json<Vec<Agent>>, ApiError> {
    Ok(Json(state.store.list_agents(true).await?))
}

async fn spawn_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<SpawnRequest>,
) -> Result<Json<Agent>, ApiError> {
    Ok(Json(state.manager.spawn(req, &principal).await?))
}

async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Agent>, ApiError> {
    state
        .store
        .get_agent(&id)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("no such agent: {id}")))
}

async fn send_to_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<SendRequest>,
) -> Result<Json<Message>, ApiError> {
    let agent = state
        .store
        .get_agent(&id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("no such agent: {id}")))?;
    let msg = state
        .manager
        .send(
            principal.id,
            ToTarget::Agent(agent.id),
            req.kind,
            req.body,
            req.when,
            req.reply_to,
        )
        .await?;
    Ok(Json(msg))
}

async fn interrupt_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<InterruptRequest>,
) -> Result<Json<bridle_api::types::InterruptResponse>, ApiError> {
    Ok(Json(
        state
            .manager
            .interrupt(&id, req.drop_held, &principal)
            .await?,
    ))
}

async fn stop_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<StopRequest>,
) -> Result<Json<Agent>, ApiError> {
    Ok(Json(state.manager.stop(&id, req.now, &principal).await?))
}

async fn resume_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Agent>, ApiError> {
    Ok(Json(state.manager.resume(&id, &principal).await?))
}

async fn remove_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Query(q): Query<RemoveQuery>,
) -> Result<StatusCode, ApiError> {
    state
        .manager
        .remove(&id, q.force, q.delete_branch, &principal)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn transcript(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<TranscriptQuery>,
) -> Result<Json<Vec<TranscriptLine>>, ApiError> {
    let agent = state
        .store
        .get_agent(&id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("no such agent: {id}")))?;
    let path = state.workspace.transcript(&agent.id);
    let entries = bridle_claude::transcript::read_lines(
        &path,
        q.since.unwrap_or(0),
        q.limit.unwrap_or(500) as usize,
    )
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string()))?;
    Ok(Json(
        entries
            .into_iter()
            .map(|e| TranscriptLine {
                n: e.n as u64,
                t_ms: e.t_ms,
                dir: e.dir,
                line: e.line,
            })
            .collect(),
    ))
}

// ---------- messages ----------

async fn resolve_to(
    store: &Store,
    principal: &Principal,
    raw: Option<&str>,
) -> Result<Option<String>, ApiError> {
    Ok(match raw {
        None => None,
        Some("me") => Some(match principal.kind {
            PrincipalKind::Human => "human".to_string(),
            PrincipalKind::Agent => {
                let name = principal.id.strip_prefix("agent:").unwrap_or(&principal.id);
                store
                    .get_agent(name)
                    .await?
                    .map(|a| a.id)
                    .unwrap_or_else(|| principal.id.clone())
            }
            _ => principal.id.clone(),
        }),
        Some("human") => Some("human".to_string()),
        Some(other) => Some(match store.get_agent(other).await? {
            Some(a) => a.id,
            None => other.to_string(),
        }),
    })
}

async fn resolve_from(store: &Store, raw: Option<&str>) -> Result<Option<String>, ApiError> {
    Ok(match raw {
        None => None,
        Some("human") => Some("human".to_string()),
        Some(other) => Some(match store.get_agent(other).await? {
            Some(a) => format!("agent:{}", a.name),
            None => other.to_string(),
        }),
    })
}

async fn list_messages(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Query(q): Query<MessageQuery>,
) -> Result<Json<Vec<Message>>, ApiError> {
    let to = resolve_to(&state.store, &principal, q.to.as_deref()).await?;
    let from = resolve_from(&state.store, q.from.as_deref()).await?;
    let msgs = state
        .store
        .list_messages(crate::store::ListMessages {
            to,
            from,
            unread: q.unread,
            limit: q.limit,
        })
        .await?;
    Ok(Json(msgs))
}

async fn send_message(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<SendRequest>,
) -> Result<Json<Message>, ApiError> {
    let Some(to_raw) = req.to.as_deref() else {
        return Err(ApiError::bad_request("`to` is required"));
    };
    let target = if to_raw == "human" {
        ToTarget::Human
    } else {
        let agent = state
            .store
            .get_agent(to_raw)
            .await?
            .ok_or_else(|| ApiError::not_found(format!("no such recipient: {to_raw}")))?;
        ToTarget::Agent(agent.id)
    };
    let msg = state
        .manager
        .send(
            principal.id,
            target,
            req.kind,
            req.body,
            req.when,
            req.reply_to,
        )
        .await?;
    Ok(Json(msg))
}

async fn mark_read(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Message>, ApiError> {
    let msg = state
        .store
        .get_message(&id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("no such message: {id}")))?;
    let is_recipient = match principal.kind {
        PrincipalKind::Human => msg.to == "human",
        PrincipalKind::Agent => {
            let name = principal.id.strip_prefix("agent:").unwrap_or(&principal.id);
            state.store.get_agent(name).await?.map(|a| a.id) == Some(msg.to.clone())
        }
        _ => principal.id == msg.to,
    };
    if !is_recipient && principal.kind != PrincipalKind::Human {
        return Err(ApiError::forbidden("not the recipient of this message"));
    }
    state
        .store
        .set_message_state(&id, MessageState::Read, Utc::now())
        .await?;
    let _ = state
        .emitter
        .emit(
            bridle_api::types::event_kind::MESSAGE_READ,
            principal.id,
            None,
            serde_json::json!({"message": id}),
        )
        .await;
    Ok(Json(state.store.get_message(&id).await?.ok_or_else(
        || {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "message vanished",
            )
        },
    )?))
}

// ---------- events ----------

async fn list_events(
    State(state): State<AppState>,
    Query(mut q): Query<EventQuery>,
) -> Result<Json<Vec<Event>>, ApiError> {
    // Events store the agent id; accept a name too, like every other
    // endpoint. A removed agent has no row, so its id is used as given.
    if let Some(agent) = &q.agent
        && let Some(a) = state.store.get_agent(agent).await?
    {
        q.agent = Some(a.id);
    }
    Ok(Json(state.store.list_events(q).await?))
}

#[derive(Debug, Deserialize)]
struct StreamQuery {
    since: Option<i64>,
}

struct StreamState {
    backfill: std::vec::IntoIter<Event>,
    rx: tokio::sync::broadcast::Receiver<Event>,
    last_sent: i64,
}

async fn events_stream(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<StreamQuery>,
    headers: axum::http::HeaderMap,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let since = q.since.or_else(|| {
        headers
            .get("last-event-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
    });
    // Subscribe before backfilling, so nothing appended in between is missed.
    let rx = state.emitter.subscribe();
    let backfill = state
        .store
        .list_events(EventQuery {
            since,
            agent: None,
            kind: None,
            limit: Some(1_000_000),
        })
        .await
        .unwrap_or_default();

    let init = StreamState {
        backfill: backfill.into_iter(),
        rx,
        last_sent: since.unwrap_or(0),
    };
    let stream = futures::stream::unfold(init, |mut st| async move {
        loop {
            if let Some(ev) = st.backfill.next() {
                if ev.seq <= st.last_sent {
                    continue;
                }
                st.last_sent = ev.seq;
                return Some((Ok(to_sse(&ev)), st));
            }
            match st.rx.recv().await {
                Ok(ev) => {
                    if ev.seq <= st.last_sent {
                        continue;
                    }
                    st.last_sent = ev.seq;
                    return Some((Ok(to_sse(&ev)), st));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => return None,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

fn to_sse(ev: &Event) -> SseEvent {
    SseEvent::default()
        .id(ev.seq.to_string())
        .event(ev.kind.clone())
        .data(serde_json::to_string(ev).unwrap_or_default())
}

// ---------- usage / tokens / shutdown ----------

async fn usage(State(state): State<AppState>) -> Result<Json<Usage>, ApiError> {
    Ok(Json(state.store.usage().await?))
}

async fn create_token(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<TokenCreateRequest>,
) -> Result<Json<TokenCreated>, ApiError> {
    require_human(&principal)?;
    Ok(Json(state.store.create_external_token(&req.name).await?))
}

async fn shutdown(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<StatusCode, ApiError> {
    require_human(&principal)?;
    let _ = state.shutdown_tx.send(true);
    Ok(StatusCode::NO_CONTENT)
}
