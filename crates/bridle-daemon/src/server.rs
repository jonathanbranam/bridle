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
    AddQueueTierRequest, Agent, AllocPortRequest, AnswerQuestionRequest, ApiErrorResponse,
    AskQuestionRequest, BudgetHoldRequest, BudgetOverrideRequest, BudgetStatus, Conflict,
    DoneTaskRequest, DropTaskRequest, Edge, EdgeKind, EditTaskRequest, ErrorBody, Event,
    EventQuery, Health, HoldStatus, ImpactCheckRequest, ImpactReport, InteractiveUsageRow,
    InterruptRequest, MaxWorkersRequest, MergeProbe, Message, MessageKind, MessageQuery,
    MessageState, NewEdgeRequest, NewTaskRequest, NoteTaskRequest, OpenQuestion, OverlapLevel,
    PortAllocation, PrincipalKind, ProbeOutcome, ProbeRequest, ProbeResult, Queue, RateLimit,
    RemoveEdgeQuery, RemoveQuery, RenewRequest, ResolveConflictRequest, ResumeRequest,
    ScheduleOverrideStatus, SendRequest, SetImpactRequest, SetQueueRequest, SetSummaryRequest,
    SpawnRequest, Status, StatusLineReport, StopRequest, Task, TaskQuery, TaskState,
    TokenCreateRequest, TokenCreated, TokenInfo, TranscriptLine, TranscriptQuery, Usage,
    UsageBreakdown, UsageBreakdownQuery, UsageGroupBy, When, WindowStatus, event_kind,
};
use chrono::Utc;
use futures::Stream;
use serde::Deserialize;
use tokio::sync::watch;

use crate::events::Emitter;
use crate::paths::Workspace;
use crate::store::{Principal, Store, StoreError};
use crate::supervisor::{AgentManager, SupervisorError, ToTarget};
use crate::tasks::{TaskError, TaskManager};

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
    pub ci: crate::ci::CiWatcher,
    pub tasks: TaskManager,
    pub ports: crate::config::PortsConfig,
    /// `[branches] integration`, the branch `probe` merges against.
    pub integration: String,
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
        .route("/v1/agents/{id}/renew", post(renew_agent))
        .route("/v1/agents/{id}/transcript", get(transcript))
        .route("/v1/messages", get(list_messages).post(send_message))
        .route("/v1/messages/{id}/read", post(mark_read))
        .route("/v1/events", get(list_events))
        .route("/v1/events/stream", get(events_stream))
        .route("/v1/usage", get(usage))
        .route("/v1/usage/breakdown", get(usage_breakdown))
        .route("/v1/statusline", post(report_statusline))
        .route("/v1/budget", get(budget))
        .route("/v1/budget/hold", post(budget_hold))
        .route("/v1/budget/release", post(budget_release))
        .route("/v1/budget/override", post(budget_override))
        .route("/v1/budget/override/clear", post(budget_override_clear))
        .route("/v1/budget/max-workers", post(budget_max_workers))
        .route("/v1/tokens", get(list_tokens).post(create_token))
        .route("/v1/tokens/{name}", axum::routing::delete(revoke_token))
        .route("/v1/tasks", get(list_tasks).post(new_task))
        .route("/v1/tasks/{id}", get(get_task).patch(edit_task))
        .route("/v1/tasks/{id}/plan", post(plan_task))
        .route("/v1/tasks/{id}/drop", post(drop_task))
        .route("/v1/tasks/{id}/done", post(done_task))
        .route("/v1/tasks/{id}/summary", post(set_summary))
        .route("/v1/tasks/{id}/impact", post(set_impact))
        .route("/v1/impact/check", post(impact_check))
        .route("/v1/probe", post(probe))
        .route("/v1/ports", get(list_ports).post(alloc_port))
        .route("/v1/ports/{port}/release", post(release_port))
        .route("/v1/conflicts", get(list_conflicts))
        .route("/v1/conflicts/{id}/resolve", post(resolve_conflict))
        .route("/v1/tasks/{id}/reopen", post(reopen_task))
        .route("/v1/tasks/{id}/ask", post(ask_task))
        .route("/v1/tasks/{id}/answer", post(answer_task))
        .route("/v1/tasks/{id}/note", post(note_task))
        .route("/v1/tasks/{id}/claim", post(claim_task))
        .route("/v1/tasks/{id}/release", post(release_task))
        .route("/v1/questions", get(list_open_questions))
        .route(
            "/v1/edges",
            get(list_edges).post(add_edge).delete(remove_edge),
        )
        .route("/v1/queue", get(get_queue).post(set_queue))
        .route("/v1/queue/tiers", post(add_queue_tier))
        .route("/v1/rebuild", post(rebuild))
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
            StoreError::ShuttingDown => ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "shutting_down",
                "daemon is shutting down".to_string(),
            ),
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

impl From<TaskError> for ApiError {
    fn from(e: TaskError) -> Self {
        match e {
            TaskError::NotFound(m) => ApiError::not_found(m),
            TaskError::Conflict(m) => ApiError::new(StatusCode::CONFLICT, "conflict", m),
            TaskError::BadRequest(m) => ApiError::bad_request(m),
            TaskError::Internal(m) => {
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
        // The daemon only listens on 127.0.0.1 (docs/design/agent-host/
        // principals.md, "Read access without a token"): a request already
        // on this machine can read without a token. Writes still need one.
        if matches!(
            req.method(),
            &axum::http::Method::GET | &axum::http::Method::HEAD
        ) {
            req.extensions_mut().insert(Principal {
                id: "local".to_string(),
                kind: PrincipalKind::Local,
            });
            return next.run(req).await;
        }
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

/// The queue is PM-owned; the human can override it, and every other
/// principal (the manager included) only reads it
/// (roles-and-lifecycle.md, "the queue"). Unlike [`require_not_worker`]'s
/// blocklist, this is an allowlist: only `human` and an agent whose role is
/// `product-manager` pass.
async fn require_pm_or_human(state: &AppState, principal: &Principal) -> Result<(), ApiError> {
    if principal.kind == PrincipalKind::Human {
        return Ok(());
    }
    let name = principal.id.strip_prefix("agent:").unwrap_or(&principal.id);
    let role = state.store.get_agent(name).await?.map(|a| a.role);
    if role.as_deref() == Some("product-manager") {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "only the product manager or the human may edit the queue",
        ))
    }
}

/// Worker-role agents get no agent lifecycle authority
/// (docs/design/agent-host/roles-and-config.md): they can run arbitrary
/// Bash but must not spawn/interrupt/stop/resume/renew/remove agents via
/// the API. Manager, orchestrator, human and other non-worker principals
/// pass through unchanged.
async fn require_not_worker(state: &AppState, principal: &Principal) -> Result<(), ApiError> {
    if principal.kind != PrincipalKind::Agent {
        return Ok(());
    }
    let name = principal.id.strip_prefix("agent:").unwrap_or(&principal.id);
    let role = state.store.get_agent(name).await?.map(|a| a.role);
    if role.as_deref() == Some("worker") {
        Err(ApiError::forbidden(
            "worker agents cannot use agent lifecycle endpoints",
        ))
    } else {
        Ok(())
    }
}

// ---------- health / status ----------

async fn health(State(state): State<AppState>) -> Result<Json<Health>, ApiError> {
    let agent_count = state.store.list_agents(false).await?.len() as u32;
    Ok(Json(Health {
        ok: true,
        version: state.version.clone(),
        agent_count,
    }))
}

async fn status(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<Json<Status>, ApiError> {
    let agents_by_state = state.store.agents_by_state().await?;
    let unread = state.store.unread_count("human").await?;
    let rate_limits = state.store.rate_limits().await?;
    let claude_version = state.store.get_meta("claude_version").await?;
    let mut merged_leftovers = Vec::new();
    for agent in state.store.list_agents(true).await? {
        if agent.state == bridle_api::types::AgentState::Stopped
            && let Some(branch) = &agent.branch
            && matches!(
                crate::worktree::is_merged(&state.workspace.repo, branch).await,
                Ok(true)
            )
        {
            merged_leftovers.push(agent.name);
        }
    }
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
        ci: state.ci.last(),
        merged_leftovers,
    }))
}

// ---------- budget ----------

async fn budget(State(state): State<AppState>) -> Result<Json<BudgetStatus>, ApiError> {
    let snapshot = state.governor.snapshot();
    let cfg = state.governor.config();
    let rate_limits = state.store.rate_limits().await?;
    let mut windows = Vec::new();
    for window in [
        "five_hour",
        "seven_day",
        "seven_day_opus",
        "seven_day_sonnet",
    ] {
        let rl = rate_limits.iter().find(|r| r.window == window);
        let block = if window == "seven_day_opus" {
            snapshot.per_model.get("opus").cloned().unwrap_or_default()
        } else if window == "seven_day_sonnet" {
            snapshot
                .per_model
                .get("sonnet")
                .cloned()
                .unwrap_or_default()
        } else if snapshot.default.window.as_deref() == Some(window) {
            snapshot.default.clone()
        } else {
            crate::governor::WindowBlock::default()
        };
        let age = rl.map(|r| (Utc::now() - r.observed_at).to_std().unwrap_or_default());
        windows.push(WindowStatus {
            window: window.to_string(),
            state: block.state,
            status: rl.and_then(|r| r.status.clone()),
            utilization: rl.and_then(|r| r.utilization),
            resets_at: rl.and_then(|r| r.resets_at),
            observed_at: rl.map(|r| r.observed_at),
            stale: age.is_none_or(|a| a > cfg.max_staleness),
            age_secs: age.map(|a| a.as_secs()),
        });
    }
    let five_hour = state.governor.applied_five_hour();
    let mut thresholds = cfg.to_wire();
    thresholds
        .hold_at
        .insert("five_hour".into(), five_hour.hold_at);
    thresholds
        .wind_down_at
        .insert("five_hour".into(), five_hour.wind_down_at);
    thresholds
        .stop_at
        .insert("five_hour".into(), five_hour.stop_at);
    let reasons = budget_reasons(&windows, &thresholds);
    Ok(Json(BudgetStatus {
        state: snapshot.default.state,
        windows,
        thresholds,
        five_hour,
        schedule: state.governor.schedule_info(),
        reasons,
        human_hold: state
            .governor
            .hold_status()
            .map(|until| HoldStatus { until }),
        schedule_override: state
            .governor
            .schedule_override_status()
            .map(|(period, until)| ScheduleOverrideStatus { period, until }),
        max_workers_override: state.manager.max_workers_override(),
    }))
}

/// One line per window above `normal`: the threshold it crossed or the
/// `status` reported with it (`allowed_warning` is shown but forces nothing,
/// ticket kv7d).
fn budget_reasons(
    windows: &[WindowStatus],
    t: &bridle_api::types::BudgetThresholds,
) -> Vec<String> {
    let mut out = Vec::new();
    for w in windows {
        if w.state == bridle_api::types::GovernorState::Normal {
            continue;
        }
        let pct = w.utilization.map(|u| u * 100.0);
        let get = |m: &std::collections::BTreeMap<String, f64>| {
            m.get(&w.window).or_else(|| m.get("default")).copied()
        };
        let crossed = pct.and_then(|p| {
            [
                ("stop_at", get(&t.stop_at)),
                ("wind_down_at", get(&t.wind_down_at)),
                ("hold_at", get(&t.hold_at)),
            ]
            .into_iter()
            .find(|(_, v)| v.is_some_and(|v| p >= v))
        });
        let status = w.status.as_deref().filter(|s| *s != "allowed");
        let mut line = format!("{} is {}", w.window, w.state);
        if let (Some((name, Some(v))), Some(p)) = (crossed, pct) {
            line.push_str(&format!(": {p:.0}% >= {name} {v:.0}%"));
        }
        if let Some(s) = status {
            line.push_str(&format!("; status {s}"));
        }
        out.push(line);
    }
    out
}

async fn budget_hold(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<BudgetHoldRequest>,
) -> Result<Json<BudgetStatus>, ApiError> {
    require_human(&principal)?;
    state.governor.hold(req.until);
    state.governor.recompute().await;
    budget(State(state)).await
}

async fn budget_release(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<Json<BudgetStatus>, ApiError> {
    require_human(&principal)?;
    state.governor.release();
    state.governor.recompute().await;
    budget(State(state)).await
}

async fn budget_override(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<BudgetOverrideRequest>,
) -> Result<Json<BudgetStatus>, ApiError> {
    require_human(&principal)?;
    state.governor.set_schedule_override(req.period, req.until);
    state.governor.recompute().await;
    budget(State(state)).await
}

async fn budget_override_clear(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<Json<BudgetStatus>, ApiError> {
    require_human(&principal)?;
    state.governor.clear_schedule_override();
    state.governor.recompute().await;
    budget(State(state)).await
}

async fn budget_max_workers(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<MaxWorkersRequest>,
) -> Result<Json<BudgetStatus>, ApiError> {
    require_human(&principal)?;
    state.governor.forget_preset_cap();
    state.manager.set_max_workers_override(req.max_workers);
    budget(State(state)).await
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
    require_not_worker(&state, &principal).await?;
    let mut req = req;
    req.components = if req.components.is_empty() {
        // Default to the spawner's claimed task's scope (components.md).
        state
            .tasks
            .list_tasks()
            .into_iter()
            .find(|t| t.claimed_by.as_deref() == Some(principal.id.as_str()))
            .map(|t| t.components)
            .unwrap_or_default()
    } else {
        state.manager.normalize_components(&req.components)?
    };
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
    require_not_worker(&state, &principal).await?;
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
    require_not_worker(&state, &principal).await?;
    Ok(Json(state.manager.stop(&id, req.now, &principal).await?))
}

async fn resume_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<ResumeRequest>,
) -> Result<Json<Agent>, ApiError> {
    require_not_worker(&state, &principal).await?;
    Ok(Json(
        state
            .manager
            .resume(&id, req.ignore_budget, &principal)
            .await?,
    ))
}

async fn renew_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    // `RenewRequest.ignore_budget` is accepted and ignored: renew never waits on a hold.
    Json(_req): Json<RenewRequest>,
) -> Result<Json<Agent>, ApiError> {
    require_not_worker(&state, &principal).await?;
    Ok(Json(state.manager.renew(&id, &principal).await?))
}

async fn remove_agent(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Query(q): Query<RemoveQuery>,
) -> Result<StatusCode, ApiError> {
    require_not_worker(&state, &principal).await?;
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
    let entries =
        bridle_claude::transcript::read_lines(&path, q.since, q.limit.unwrap_or(500) as usize)
            .map_err(|e| {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            })?;
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
) -> Result<Json<Vec<Message>>, ApiError> {
    let Some(to_raw) = req.to.as_deref() else {
        return Err(ApiError::bad_request("`to` is required"));
    };
    let targets = if to_raw == "human" {
        vec![ToTarget::Human]
    } else if let Some(name) = to_raw.strip_prefix("external:") {
        if !state.store.external_exists(name).await? {
            return Err(ApiError::not_found(format!("no such recipient: {to_raw}")));
        }
        vec![ToTarget::External(to_raw.to_string())]
    } else if let Some(role) = to_raw.strip_prefix("role:") {
        let matching: Vec<ToTarget> = state
            .store
            .list_agents(false)
            .await?
            .into_iter()
            .filter(|a| a.role == role)
            .map(|a| ToTarget::Agent(a.id))
            .collect();
        if matching.is_empty() {
            return Err(ApiError::not_found(format!(
                "no live agents with role: {role}"
            )));
        }
        matching
    } else {
        let agent = state
            .store
            .get_agent(to_raw)
            .await?
            .ok_or_else(|| ApiError::not_found(format!("no such recipient: {to_raw}")))?;
        vec![ToTarget::Agent(agent.id)]
    };
    // The full text goes on the task's thread (an unknown task fails here,
    // before anything is sent); recipients get a short pointer to it.
    let body = match req.task.as_deref() {
        Some(task_id) => {
            let task = state
                .tasks
                .note_task(task_id, &principal.id, &req.body)
                .await?;
            let _ = state
                .emitter
                .emit(
                    event_kind::TASK_NOTE_ADDED,
                    principal.id.clone(),
                    None,
                    serde_json::json!({"task": task.id}),
                )
                .await;
            let first = req
                .body
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("");
            format!("{}: note added\n{first}", task.id)
        }
        None => req.body.clone(),
    };
    let mut msgs = Vec::with_capacity(targets.len());
    for target in targets {
        let msg = state
            .manager
            .send(
                principal.id.clone(),
                target,
                req.kind,
                body.clone(),
                req.when,
                req.reply_to.clone(),
            )
            .await?;
        msgs.push(msg);
    }
    Ok(Json(msgs))
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
    shutdown_rx: watch::Receiver<bool>,
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
    // No cursor at all (fresh `--follow`, no `--since`, no `Last-Event-ID`):
    // start at the tail and only stream events that arrive from here, rather
    // than replaying the whole history. A cursor (reconnect, or an explicit
    // `--since`) still backfills from it.
    let backfill = if since.is_some() {
        state
            .store
            .list_events(EventQuery {
                since,
                agent: None,
                kind: None,
                limit: Some(1_000_000),
            })
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let init = StreamState {
        backfill: backfill.into_iter(),
        rx,
        shutdown_rx: state.shutdown_tx.subscribe(),
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
            // The emitter keeps a sender alive, so `Closed` never comes; end
            // the stream on shutdown or graceful shutdown waits on the client.
            let recv = tokio::select! {
                r = st.rx.recv() => r,
                _ = st.shutdown_rx.wait_for(|v| *v) => return None,
            };
            match recv {
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

async fn usage_breakdown(
    State(state): State<AppState>,
    Query(q): Query<UsageBreakdownQuery>,
) -> Result<Json<UsageBreakdown>, ApiError> {
    let by = q.by.unwrap_or(UsageGroupBy::Agent);
    Ok(Json(state.store.usage_breakdown(q.since, by).await?))
}

/// `bridle statusline`'s report from an interactive session bridle doesn't
/// host: rate-limit windows go through the same `upsert_rate_limit` path
/// every other source feeds, and the rest becomes one `interactive_usage`
/// row (docs/design/usage-and-budget.md, "Where bridle can see usage").
async fn report_statusline(
    State(state): State<AppState>,
    Json(req): Json<StatusLineReport>,
) -> Result<StatusCode, ApiError> {
    let observed_at = Utc::now();
    for reading in &req.rate_limits {
        state
            .store
            .upsert_rate_limit(RateLimit {
                window: reading.window.clone(),
                status: None,
                utilization: reading.utilization,
                resets_at: reading.resets_at,
                observed_at,
            })
            .await?;
    }
    state
        .store
        .record_interactive_usage(InteractiveUsageRow {
            observed_at,
            session_id: req.session_id,
            model: req.model,
            cost_usd: req.cost_usd,
            context_used_percentage: req.context_used_percentage,
            context_used_tokens: req.context_used_tokens,
            context_max_tokens: req.context_max_tokens,
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------- tasks ----------

/// `?claimed_by=me` resolves to the caller's own id, same as `resolve_to`'s
/// `"me"` case for messages; anything else is matched against `Task::claimed_by`
/// verbatim (that field is the claimant's `PrincipalId`, e.g. `agent:w1`, not
/// the stable agent id `resolve_to`/`resolve_from` resolve names to).
async fn resolve_claimed_by(
    store: &Store,
    principal: &Principal,
    raw: &str,
) -> Result<String, ApiError> {
    Ok(match raw {
        "me" => principal.id.clone(),
        "human" => "human".to_string(),
        other => match store.get_agent(other).await? {
            Some(a) => format!("agent:{}", a.name),
            None => other.to_string(),
        },
    })
}

async fn list_tasks(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Query(q): Query<TaskQuery>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let tasks = if q.top_tier.unwrap_or(false) {
        state.tasks.highest_startable_tier()
    } else if q.ready.unwrap_or(false) {
        state.tasks.ready_tasks()
    } else {
        state.tasks.list_tasks()
    };
    let tasks: Vec<Task> = match q.component.as_deref() {
        Some(id) => tasks
            .into_iter()
            .filter(|t| state.manager.components_match(&t.components, id))
            .collect(),
        None => tasks,
    };
    let tasks = match q.claimed_by.as_deref() {
        Some(raw) => {
            let claimed_by = resolve_claimed_by(&state.store, &principal, raw).await?;
            tasks
                .into_iter()
                .filter(|t| t.claimed_by.as_deref() == Some(claimed_by.as_str()))
                .collect()
        }
        None => tasks,
    };
    Ok(Json(tasks))
}

async fn new_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<NewTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let components = state.manager.normalize_components(&req.components)?;
    let task = state
        .tasks
        .new_task(&req.title, req.kind, req.body, components, req.size)
        .await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_CREATED,
            principal.id,
            None,
            serde_json::json!({"task": task.id, "kind": task.kind, "state": task.state}),
        )
        .await;
    let open = state
        .tasks
        .list_tasks()
        .iter()
        .filter(|t| t.state == bridle_api::types::TaskState::Open)
        .count();
    state
        .manager
        .note_task_filed(&task.id, &task.title, open)
        .await;
    Ok(Json(task))
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    state
        .tasks
        .get_task(&id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("no such task: {id}")))
}

async fn edit_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<EditTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let components = req
        .components
        .as_deref()
        .map(|c| state.manager.normalize_components(c))
        .transpose()?;
    let task = state
        .tasks
        .edit_task(&id, req.title, req.body, components, req.size)
        .await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_EDITED,
            principal.id,
            None,
            serde_json::json!({"task": task.id}),
        )
        .await;
    Ok(Json(task))
}

async fn plan_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    let task = state.tasks.plan_task(&id, &principal.id).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_STATE,
            principal.id,
            None,
            serde_json::json!({"task": task.id, "to": task.state}),
        )
        .await;
    Ok(Json(task))
}

async fn drop_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<DropTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let task = state
        .tasks
        .drop_task(&id, &req.reason, &principal.id)
        .await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_STATE,
            principal.id,
            None,
            serde_json::json!({"task": task.id, "to": task.state}),
        )
        .await;
    Ok(Json(task))
}

async fn set_summary(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SetSummaryRequest>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(state.tasks.set_summary(&id, &req.text).await?))
}

async fn set_impact(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SetImpactRequest>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(state.tasks.set_impact(&id, req.impact).await?))
}

async fn impact_check(
    State(state): State<AppState>,
    Json(req): Json<ImpactCheckRequest>,
) -> Result<Json<ImpactReport>, ApiError> {
    let overlaps = crate::impact::check(&state.tasks.list_tasks(), &req.spec_map);
    let mut opened = Vec::new();
    for o in overlaps
        .iter()
        .filter(|o| o.level == OverlapLevel::Conflict)
    {
        if let Some(c) = state.store.open_conflict(&o.tasks, &o.kind, &o.key).await? {
            announce_conflict(&state, &c).await;
            opened.push(c.id);
        }
    }
    let probes = merge_probes(&state).await;
    Ok(Json(ImpactReport {
        overlaps,
        opened,
        probes,
    }))
}

/// Merges two branches in memory and describes the outcome. A git failure (a branch that
/// doesn't exist) is a bad request.
async fn probe_branches(
    state: &AppState,
    branch: &str,
    against: &str,
) -> Result<ProbeResult, ApiError> {
    use crate::worktree::MergeProbe as P;
    let repo = &state.workspace.repo;
    let (outcome, paths) = match crate::worktree::merge_probe(repo, against, branch)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?
    {
        P::Clean => (ProbeOutcome::Clean, Vec::new()),
        P::Conflicts(p) => (ProbeOutcome::Conflict, p),
        P::Unsupported => (ProbeOutcome::Unsupported, Vec::new()),
    };
    Ok(ProbeResult {
        branch: branch.to_string(),
        against: against.to_string(),
        outcome,
        paths,
    })
}

/// The branch a claimed task's claimant works on.
async fn claimed_branch(state: &AppState, task: &Task) -> Option<String> {
    let name = task.claimed_by.as_deref()?.strip_prefix("agent:")?;
    state.store.get_agent(name).await.ok()??.branch
}

/// Probes every claimed task's branch against the integration branch (`conflict`), and each
/// pair of them against each other (`warn`). Clean merges are left out, and a probe that
/// can't run (no such branch) is skipped.
async fn merge_probes(state: &AppState) -> Vec<MergeProbe> {
    let mut claimed = Vec::new();
    for t in state.tasks.list_tasks() {
        if t.state == TaskState::Claimed
            && let Some(b) = claimed_branch(state, &t).await
        {
            claimed.push((t.id, b));
        }
    }
    claimed.sort();
    let mut out = Vec::new();
    for (i, (id, branch)) in claimed.iter().enumerate() {
        if let Ok(r) = probe_branches(state, branch, &state.integration).await
            && r.outcome != ProbeOutcome::Clean
        {
            out.push(MergeProbe {
                level: OverlapLevel::Conflict,
                tasks: vec![id.clone()],
                result: r,
            });
        }
        for (other_id, other) in &claimed[i + 1..] {
            if let Ok(r) = probe_branches(state, branch, other).await
                && r.outcome == ProbeOutcome::Conflict
            {
                out.push(MergeProbe {
                    level: OverlapLevel::Warn,
                    tasks: vec![id.clone(), other_id.clone()],
                    result: r,
                });
            }
        }
    }
    out
}

/// `bridle probe`: does a task's (or agent's) branch, or a named branch, merge cleanly
/// into the integration branch.
async fn probe(
    State(state): State<AppState>,
    Json(req): Json<ProbeRequest>,
) -> Result<Json<ProbeResult>, ApiError> {
    let branch = match (&req.branch, &req.target) {
        (Some(b), None) => b.clone(),
        (None, Some(t)) => {
            let task = state.tasks.get_task(t);
            let by_task = match &task {
                Some(task) => claimed_branch(&state, task).await,
                None => None,
            };
            match by_task {
                Some(b) => b,
                None => state
                    .store
                    .get_agent(t)
                    .await?
                    .and_then(|a| a.branch)
                    .ok_or_else(|| ApiError::not_found(format!("no branch for {t}")))?,
            }
        }
        _ => return Err(ApiError::bad_request("give exactly one of target, branch")),
    };
    Ok(Json(
        probe_branches(&state, &branch, &state.integration).await?,
    ))
}

/// Injects the conflict into each task's claimant, or the running managers for an
/// unclaimed task, and notes it on both task threads. Best effort: a recipient that
/// can't be reached doesn't fail the check.
async fn announce_conflict(state: &AppState, c: &Conflict) {
    for (i, task_id) in c.tasks.iter().enumerate() {
        let other = &c.tasks[1 - i];
        let body = format!(
            "{}: conflict {} with {other} over {} {}. Settle it with the other task's claimant, \
             then record it: bridle conflict resolve {} --compatible '<why>' | --order A,B | \
             --merge-into <task>",
            task_id, c.id, c.kind, c.key, c.id
        );
        let _ = state
            .tasks
            .note_task(task_id, &"system".to_string(), &body)
            .await;
        let claimant = state.tasks.get_task(task_id).and_then(|t| t.claimed_by);
        let mut to = Vec::new();
        if let Some(name) = claimant.as_deref().and_then(|p| p.strip_prefix("agent:"))
            && let Ok(Some(a)) = state.store.get_agent(name).await
        {
            to.push(a.id);
        } else if let Ok(agents) = state.store.list_agents(false).await {
            to.extend(
                agents
                    .into_iter()
                    .filter(|a| a.role == "manager" && a.state.is_running())
                    .map(|a| a.id),
            );
        }
        for id in to {
            let _ = state
                .manager
                .send(
                    "system".to_string(),
                    ToTarget::Agent(id),
                    MessageKind::Note,
                    body.clone(),
                    When::Now,
                    None,
                )
                .await;
        }
    }
}

async fn alloc_port(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<AllocPortRequest>,
) -> Result<Json<PortAllocation>, ApiError> {
    // An agent owns its ports by stable id, so they're freed when it exits.
    let agent = if principal.kind == PrincipalKind::Agent {
        let name = principal.id.strip_prefix("agent:").unwrap_or(&principal.id);
        match state.store.get_agent(name).await? {
            Some(a) => a.id,
            None => principal.id.clone(),
        }
    } else {
        principal.id.clone()
    };
    let task = state
        .tasks
        .list_tasks()
        .into_iter()
        .find(|t| t.claimed_by.as_deref() == Some(principal.id.as_str()))
        .map(|t| t.id);
    let p = crate::ports::alloc(&state.store, &state.ports, agent, task, req.pid, req.label)
        .await?
        .ok_or_else(|| {
            ApiError::from(StoreError::Conflict(format!(
                "no free port in {}-{}",
                state.ports.range.0, state.ports.range.1
            )))
        })?;
    Ok(Json(p))
}

async fn list_ports(State(state): State<AppState>) -> Result<Json<Vec<PortAllocation>>, ApiError> {
    Ok(Json(state.store.list_ports().await?))
}

async fn release_port(
    State(state): State<AppState>,
    Path(port): Path<u16>,
) -> Result<Json<PortAllocation>, ApiError> {
    state
        .store
        .release_port(port)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("port {port} is not allocated")))
}

async fn list_conflicts(State(state): State<AppState>) -> Result<Json<Vec<Conflict>>, ApiError> {
    Ok(Json(state.store.list_conflicts().await?))
}

async fn resolve_conflict(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<ResolveConflictRequest>,
) -> Result<Json<Conflict>, ApiError> {
    let c = state
        .store
        .get_conflict(&id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("no such conflict: {id}")))?;
    if c.state != "open" {
        return Err(ApiError::from(StoreError::Conflict(format!(
            "conflict {id} is already resolved"
        ))));
    }
    let in_pair = |t: &str| c.tasks.iter().any(|x| x == t);
    let resolution = match (&req.compatible, &req.order, &req.merge_into) {
        (Some(why), None, None) if !why.trim().is_empty() => format!("compatible: {why}"),
        (None, Some([a, b]), None) => {
            if !in_pair(a) || !in_pair(b) || a == b {
                return Err(ApiError::bad_request(format!(
                    "--order must name the conflict's two tasks: {}, {}",
                    c.tasks[0], c.tasks[1]
                )));
            }
            state.tasks.add_edge(a, b, EdgeKind::Blocks).await?;
            format!("order: {a} blocks {b}")
        }
        (None, None, Some(t)) => {
            if !in_pair(t) {
                return Err(ApiError::bad_request(format!(
                    "--merge-into must be one of {}, {}",
                    c.tasks[0], c.tasks[1]
                )));
            }
            format!("merge-into: {t}")
        }
        _ => {
            return Err(ApiError::bad_request(
                "give exactly one of compatible (with a reason), order, merge_into",
            ));
        }
    };
    state.store.resolve_conflict(&id, &resolution).await?;
    for t in &c.tasks {
        let _ = state
            .tasks
            .note_task(
                t,
                &principal.id,
                &format!("conflict {id} resolved, {resolution}"),
            )
            .await;
    }
    let c = state
        .store
        .get_conflict(&id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("no such conflict: {id}")))?;
    Ok(Json(c))
}

async fn done_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<DoneTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let branch = req
        .branch
        .as_deref()
        .map(str::trim)
        .filter(|b| !b.is_empty());
    // Landing removes the branch's agents, worktree and branch, so refuse
    // before recording anything unless the work really is on the integration
    // branch.
    if branch.is_some() {
        let commit = req.commit.trim();
        if !crate::worktree::is_on_head(&state.workspace.repo, commit)
            .await
            .map_err(SupervisorError::from)?
        {
            return Err(TaskError::Conflict(format!(
                "commit {commit} is not on the integration branch; land it before marking the task done"
            ))
            .into());
        }
    }
    let claimant = state.tasks.get_task(&id).and_then(|t| t.claimed_by);
    let mut task = state
        .tasks
        .done_task(&id, &req.commit, req.branch.as_deref(), &principal.id)
        .await?;
    notify_main_moved(&state, &task, claimant.as_deref(), branch).await;
    if task.kind == bridle_api::types::TaskKind::ArchRevision {
        open_reevaluate_tasks(&state, &task).await;
    }
    if let Some(branch) = branch {
        let report = clean_up_landed_branch(&state, branch, &principal).await;
        task = state.tasks.note_task(&id, &principal.id, &report).await?;
    }
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_STATE,
            principal.id,
            None,
            serde_json::json!({"task": task.id, "to": task.state}),
        )
        .await;
    Ok(Json(task))
}

/// After an `arch-revision` lands, opens one `re-evaluate` task per capability with suspect
/// requirements (once per arch task and capability) and tells the manager. Failures only log:
/// the landing itself has succeeded.
async fn open_reevaluate_tasks(state: &AppState, arch: &Task) {
    let repo = state.workspace.repo.clone();
    let suspects =
        match tokio::task::spawn_blocking(move || crate::reevaluate::suspects_by_capability(&repo))
            .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                tracing::warn!(task = %arch.id, "re-evaluate: {e}");
                return;
            }
            Err(_) => return,
        };
    let existing: std::collections::HashSet<String> = state
        .tasks
        .list_tasks()
        .into_iter()
        .filter(|t| t.kind == bridle_api::types::TaskKind::ReEvaluate)
        .map(|t| t.title)
        .collect();
    let mut opened = Vec::new();
    for (cap, ids) in suspects {
        let title = crate::reevaluate::title(&cap, &arch.id);
        if existing.contains(&title) {
            continue;
        }
        match state
            .tasks
            .new_task(
                &title,
                bridle_api::types::TaskKind::ReEvaluate,
                crate::reevaluate::body(&ids),
                Vec::new(),
                None,
            )
            .await
        {
            Ok(t) => opened.push(t.id),
            Err(e) => tracing::warn!(task = %arch.id, "re-evaluate: {e}"),
        }
    }
    if opened.is_empty() {
        return;
    }
    let manager = match state.store.list_agents(false).await {
        Ok(agents) => agents
            .into_iter()
            .find(|a| a.role == "manager" && a.state.is_running()),
        Err(_) => None,
    };
    let to = manager.map_or(ToTarget::Human, |a| ToTarget::Agent(a.id));
    let _ = state
        .manager
        .send(
            "system".to_string(),
            to,
            bridle_api::types::MessageKind::Note,
            format!(
                "arch-revision {} landed with suspect requirements; opened re-evaluate tasks: {}",
                arch.id,
                opened.join(", ")
            ),
            bridle_api::types::When::Now,
            None,
        )
        .await;
}

/// Tells the other running workers that a landing moved the integration
/// branch: those holding a claim or working on a branch, not the agent whose
/// task landed. A git failure just drops the file list.
async fn notify_main_moved(
    state: &AppState,
    task: &Task,
    claimant: Option<&str>,
    landed_branch: Option<&str>,
) {
    let Ok(agents) = state.store.list_agents(false).await else {
        return;
    };
    let claims: Vec<String> = state
        .tasks
        .list_tasks()
        .into_iter()
        .filter_map(|t| t.claimed_by)
        .collect();
    let recipients: Vec<&bridle_api::types::Agent> = agents
        .iter()
        .filter(|a| a.role == crate::supervisor::WORKER_ROLE && a.state.is_running())
        .filter(|a| {
            let principal = format!("agent:{}", a.name);
            let lander = claimant == Some(principal.as_str())
                || (landed_branch.is_some() && a.branch.as_deref() == landed_branch);
            let busy = a.branch.is_some() || claims.contains(&principal);
            busy && !lander
        })
        .collect();
    if recipients.is_empty() {
        return;
    }
    let sha = task.commit.as_deref().unwrap_or_default();
    let changed = crate::worktree::changed_files(&state.workspace.repo, sha).await;
    let files = match &changed {
        Ok(f) => {
            let mut shown = f.iter().take(15).cloned().collect::<Vec<_>>().join(", ");
            if f.len() > 15 {
                shown.push_str(&format!(", +{} more", f.len() - 15));
            }
            format!("; files changed: {shown}")
        }
        Err(_) => String::new(),
    };
    let landing = format!("task {} ({}) landed at {sha}{files}", task.id, task.title);
    // Agents whose declared impact overlaps the landing get told what, so they re-read
    // before building on stale text; the rest keep the generic notice.
    let changed = changed.unwrap_or_default();
    let ids = landed_spec_ids(&state.workspace.repo, sha, &changed).await;
    let mut by_claimant: std::collections::HashMap<String, bridle_api::types::Impact> =
        std::collections::HashMap::new();
    for t in state.tasks.list_tasks() {
        if let (Some(c), bridle_api::types::TaskState::Claimed) = (t.claimed_by.clone(), t.state) {
            by_claimant.insert(c, t.impact);
        }
    }
    let recipients = recipients
        .into_iter()
        .map(|a| {
            let overlap = by_claimant
                .get(&format!("agent:{}", a.name))
                .and_then(|i| crate::impact::landing_overlap(i, &ids, &changed));
            let landing = match overlap {
                Some(o) => format!("{landing}; spec changed under you: {o}"),
                None => landing.clone(),
            };
            (a.id.clone(), landing)
        })
        .collect();
    state.manager.note_main_moved(recipients).await;
}

/// Spec ids whose text the landed commit changed, from the `design/specs` files it touched.
async fn landed_spec_ids(
    repo: &std::path::Path,
    sha: &str,
    changed: &[String],
) -> std::collections::BTreeSet<String> {
    let mut ids = std::collections::BTreeSet::new();
    for f in changed
        .iter()
        .filter(|f| f.starts_with("design/specs/") && f.ends_with(".md"))
    {
        let show =
            |rev: String| async move { crate::worktree::run_git(repo, &["show", &rev]).await.ok() };
        let new = show(format!("{sha}:{f}")).await;
        let old = show(format!("{sha}^:{f}")).await;
        ids.extend(crate::impact::changed_spec_ids(
            old.as_deref(),
            new.as_deref(),
        ));
    }
    ids
}

/// Removes every agent on `branch` (stopping any still running), their
/// worktree and the branch itself, and reports what happened for the task's
/// thread. One failure is recorded and doesn't stop the rest.
async fn clean_up_landed_branch(state: &AppState, branch: &str, principal: &Principal) -> String {
    let mut removed = Vec::new();
    let mut failed = Vec::new();
    match state.store.list_agents(true).await {
        Ok(agents) => {
            for agent in agents
                .into_iter()
                .filter(|a| a.branch.as_deref() == Some(branch))
            {
                // Forced: the work is on the integration branch, so what's
                // left in the worktree is scratch.
                match state
                    .manager
                    .remove(&agent.id, true, false, principal)
                    .await
                {
                    Ok(()) => removed.push(format!("agent {}", agent.name)),
                    Err(e) => failed.push(format!("agent {}: {e}", agent.name)),
                }
            }
        }
        Err(e) => failed.push(format!("listing agents: {e}")),
    }
    let repo = &state.workspace.repo;
    if crate::worktree::branch_exists(repo, branch)
        .await
        .unwrap_or(false)
    {
        match crate::worktree::delete_branch(repo, branch, true).await {
            Ok(()) => removed.push(format!("branch {branch}")),
            Err(e) => failed.push(format!("branch {branch}: {e}")),
        }
    }
    let mut report = format!("cleanup: removed {}", list_or_none(&removed));
    if !failed.is_empty() {
        report.push_str(&format!("; failed: {}", failed.join("; ")));
    }
    report
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "nothing".to_string()
    } else {
        items.join(", ")
    }
}

async fn reopen_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    let task = state.tasks.reopen_task(&id, &principal.id).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_STATE,
            principal.id,
            None,
            serde_json::json!({"task": task.id, "to": task.state}),
        )
        .await;
    Ok(Json(task))
}

async fn ask_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<AskQuestionRequest>,
) -> Result<Json<Task>, ApiError> {
    let task = state
        .tasks
        .ask_question(&id, &principal.id, &req.body)
        .await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_QUESTION_ASKED,
            principal.id,
            None,
            serde_json::json!({"task": task.id}),
        )
        .await;
    Ok(Json(task))
}

async fn answer_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<AnswerQuestionRequest>,
) -> Result<Json<Task>, ApiError> {
    let task = state
        .tasks
        .answer_question(&id, &principal.id, &req.body)
        .await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_QUESTION_ANSWERED,
            principal.id,
            None,
            serde_json::json!({"task": task.id}),
        )
        .await;
    Ok(Json(task))
}

async fn note_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
    Json(req): Json<NoteTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let task = state.tasks.note_task(&id, &principal.id, &req.body).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_NOTE_ADDED,
            principal.id,
            None,
            serde_json::json!({"task": task.id}),
        )
        .await;
    Ok(Json(task))
}

async fn list_open_questions(
    State(state): State<AppState>,
) -> Result<Json<Vec<OpenQuestion>>, ApiError> {
    Ok(Json(state.tasks.list_open_questions()))
}

async fn claim_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    let task = state.tasks.claim_task(&id, &principal.id).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_STATE,
            principal.id,
            None,
            serde_json::json!({"task": task.id, "to": task.state}),
        )
        .await;
    Ok(Json(task))
}

async fn release_task(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    let task = state.tasks.release_task(&id, &principal.id).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::TASK_STATE,
            principal.id,
            None,
            serde_json::json!({"task": task.id, "to": task.state}),
        )
        .await;
    Ok(Json(task))
}

// ---------- edges ----------

async fn list_edges(State(state): State<AppState>) -> Result<Json<Vec<Edge>>, ApiError> {
    Ok(Json(state.tasks.list_edges()))
}

async fn add_edge(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<NewEdgeRequest>,
) -> Result<Json<Edge>, ApiError> {
    let edge = state.tasks.add_edge(&req.from, &req.to, req.kind).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::EDGE_ADDED,
            principal.id,
            None,
            serde_json::json!({"from": edge.from, "to": edge.to, "kind": edge.kind}),
        )
        .await;
    Ok(Json(edge))
}

async fn remove_edge(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Query(q): Query<RemoveEdgeQuery>,
) -> Result<StatusCode, ApiError> {
    state.tasks.remove_edge(&q.from, &q.to, q.kind).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::EDGE_REMOVED,
            principal.id,
            None,
            serde_json::json!({"from": q.from, "to": q.to, "kind": q.kind}),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

// ---------- queue ----------

async fn get_queue(State(state): State<AppState>) -> Result<Json<Queue>, ApiError> {
    Ok(Json(Queue {
        tiers: state.tasks.live_queue_tiers(),
    }))
}

async fn set_queue(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<SetQueueRequest>,
) -> Result<Json<Queue>, ApiError> {
    require_pm_or_human(&state, &principal).await?;
    let tiers = state.tasks.set_queue(req.tiers, &principal.id).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::QUEUE_CHANGED,
            principal.id,
            None,
            serde_json::json!({"tiers": tiers.len()}),
        )
        .await;
    Ok(Json(Queue { tiers }))
}

async fn add_queue_tier(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<AddQueueTierRequest>,
) -> Result<Json<Queue>, ApiError> {
    require_pm_or_human(&state, &principal).await?;
    let tiers = state.tasks.add_queue_tier(req.tasks, &principal.id).await?;
    let _ = state
        .emitter
        .emit(
            event_kind::QUEUE_CHANGED,
            principal.id,
            None,
            serde_json::json!({"tiers": tiers.len()}),
        )
        .await;
    Ok(Json(Queue { tiers }))
}

async fn create_token(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<TokenCreateRequest>,
) -> Result<Json<TokenCreated>, ApiError> {
    require_human(&principal)?;
    Ok(Json(state.store.create_external_token(&req.name).await?))
}

async fn list_tokens(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<Json<Vec<TokenInfo>>, ApiError> {
    require_human(&principal)?;
    Ok(Json(state.store.list_external_tokens().await?))
}

async fn revoke_token(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    require_human(&principal)?;
    state.store.revoke_external_token(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn shutdown(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<StatusCode, ApiError> {
    require_human(&principal)?;
    tracing::warn!(principal = %principal.id, "shutdown requested via POST /v1/shutdown");
    let _ = state.shutdown_tx.send(true);
    Ok(StatusCode::NO_CONTENT)
}

/// `bridle rebuild` (docs/design/overview.md, "`bridle rebuild` recreates
/// the database from the project's state branch"): reconstructs
/// `tasks`/`edges`/`open_questions` from the state branch alone. Human-only,
/// like `shutdown`; refuses (409) rather than overwrites if the database
/// already has rows in any of those tables.
async fn rebuild(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<StatusCode, ApiError> {
    require_human(&principal)?;
    state.tasks.rebuild_from_state_branch().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    //! `resolve_claimed_by` alone, not `list_tasks`'s use of it: reaching a
    //! `claimed` task needs `planned` first, and there's no `plan` endpoint
    //! yet (same gap `tasks_test.rs` notes for claim/release), so the
    //! end-to-end `?claimed_by=` filter is exercised at the `TaskManager`
    //! level instead (`tasks.rs`, `claim_blocks_ready_and_release_unblocks_it`).
    use super::*;

    async fn store() -> (Store, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        (store, tmp)
    }

    #[tokio::test]
    async fn resolve_claimed_by_resolves_me_to_the_caller_and_passes_the_rest_through() {
        let (store, _tmp) = store().await;

        async fn resolve(store: &Store, principal: &Principal, raw: &str) -> String {
            resolve_claimed_by(store, principal, raw)
                .await
                .unwrap_or_else(|_| panic!("resolve_claimed_by({raw:?}) failed"))
        }

        let human = Principal {
            id: "human".to_string(),
            kind: PrincipalKind::Human,
        };
        assert_eq!(resolve(&store, &human, "me").await, "human");

        let agent = Principal {
            id: "agent:w1".to_string(),
            kind: PrincipalKind::Agent,
        };
        assert_eq!(
            resolve(&store, &agent, "me").await,
            "agent:w1",
            "an agent's own claimed_by is its PrincipalId verbatim, not the stable agent id"
        );

        assert_eq!(
            resolve(&store, &human, "agent:w2").await,
            "agent:w2",
            "an explicit claimant id not naming a known agent passes through unchanged"
        );
    }
}
