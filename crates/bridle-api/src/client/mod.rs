//! An async HTTP/SSE client for the bridle daemon API. See
//! docs/design/agent-host/api.md for the endpoint list this mirrors.

mod sse;

use bytes::Bytes;
use futures::{Stream, StreamExt};
use reqwest::{Method, RequestBuilder, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::types::{
    AddQueueTierRequest, Agent, AllocPortRequest, AnswerQuestionRequest, ApiErrorResponse,
    AskQuestionRequest, BudgetHoldRequest, BudgetOverrideRequest, BudgetStatus, Conflict,
    DoneTaskRequest, DropTaskRequest, Edge, EditTaskRequest, Event, EventQuery, Handover,
    HandoverDone, Health, ImpactCheckRequest, ImpactReport, Interaction, InteractionsQuery,
    InterruptRequest, InterruptResponse, LandRequest, LandResult, MaxWorkersRequest, Message,
    MessageQuery, NewEdgeRequest, NewTaskRequest, NoteTaskRequest, OpenQuestion,
    OrchestratorWakeQuery, PortAllocation, PrincipalWakeQuery, PrincipalWakeResponse, ProbeRequest,
    ProbeResult, Queue, RemoveEdgeQuery, RemoveQuery, RenewRequest, ResolveConflictRequest,
    RestartRequest, RestartResponse, ResumeRequest, ReviewAddRequest, ReviewAddResponse,
    ReviewNowRequest, ReviewNowResponse, SendRequest, SessionEnd, SessionInfo, SessionKeep,
    SessionRegister, SetImpactRequest, SetKindRequest, SetPriorityRequest, SetQueueRequest,
    SetSummaryRequest, ShutdownResponse, SkipSettleRequest, SpawnRequest, Status, StatusLineReport,
    StopRequest, StopWakeRequest, StopWakeResponse, SubmitTaskRequest, Task, TaskQuery,
    TokenCreateRequest, TokenCreated, TokenInfo, TranscriptLine, TranscriptQuery, Usage,
    UsageBreakdown, UsageBreakdownQuery, WakeResponse, WriteHandoverRequest,
};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("could not reach the daemon: {0}")]
    Unreachable(String),
    #[error("{code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },
    #[error("failed to decode response: {0}")]
    Decode(String),
    #[error("{0}")]
    Other(String),
}

/// A thin, cloneable HTTP client for one daemon.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
    /// A named advisor's label (`BRIDLE_ADVISOR_NAME`), sent as [`ADVISOR_HEADER`].
    advisor: Option<String>,
}

/// Header a named advisor's CLI adds so the daemon signs its messages `external:advisor/<name>`.
pub const ADVISOR_HEADER: &str = "x-bridle-advisor";

impl Client {
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Self {
        let mut base_url = base_url.into();
        while base_url.ends_with('/') {
            base_url.pop();
        }
        Self {
            http: reqwest::Client::new(),
            base_url,
            token,
            advisor: None,
        }
    }

    /// Like [`Client::new`], but every request gives up after `timeout`
    /// instead of waiting on reqwest's (much longer) default. For callers
    /// that must never hang, e.g. `bridle statusline` on the human's
    /// interactive prompt.
    pub fn new_with_timeout(
        base_url: impl Into<String>,
        token: Option<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let mut base_url = base_url.into();
        while base_url.ends_with('/') {
            base_url.pop();
        }
        Self {
            http: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .expect("building a plain http client with a timeout never fails"),
            base_url,
            token,
            advisor: None,
        }
    }

    /// Sends as the named advisor `name` (a label on the shared advisor token, not proof).
    pub fn with_advisor(mut self, name: Option<String>) -> Self {
        self.advisor = name.filter(|n| !n.is_empty());
        self
    }

    fn build_url(&self, segments: &[&str]) -> Result<reqwest::Url, ClientError> {
        let mut url = reqwest::Url::parse(&self.base_url)
            .map_err(|e| ClientError::Other(format!("invalid base url {}: {e}", self.base_url)))?;
        {
            let mut segs = url.path_segments_mut().map_err(|()| {
                ClientError::Other(format!("base url {} cannot be a base", self.base_url))
            })?;
            segs.extend(segments);
        }
        Ok(url)
    }

    fn request(&self, method: Method, segments: &[&str]) -> Result<RequestBuilder, ClientError> {
        let url = self.build_url(segments)?;
        let mut req = self.http.request(method, url);
        if let Some(t) = &self.token {
            req = req.bearer_auth(t);
        }
        if let Some(a) = &self.advisor {
            req = req.header(ADVISOR_HEADER, a);
        }
        Ok(req)
    }

    async fn get_json<T: DeserializeOwned>(&self, segments: &[&str]) -> Result<T, ClientError> {
        let req = self.request(Method::GET, segments)?;
        self.send_json(req).await
    }

    async fn get_json_query<T: DeserializeOwned, Q: Serialize + ?Sized>(
        &self,
        segments: &[&str],
        query: &Q,
    ) -> Result<T, ClientError> {
        let req = self.request(Method::GET, segments)?.query(query);
        self.send_json(req).await
    }

    async fn post_json<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        segments: &[&str],
        body: &B,
    ) -> Result<T, ClientError> {
        let req = self.request(Method::POST, segments)?.json(body);
        self.send_json(req).await
    }

    async fn post_empty<T: DeserializeOwned>(&self, segments: &[&str]) -> Result<T, ClientError> {
        let req = self.request(Method::POST, segments)?;
        self.send_json(req).await
    }

    async fn patch_json<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        segments: &[&str],
        body: &B,
    ) -> Result<T, ClientError> {
        let req = self.request(Method::PATCH, segments)?.json(body);
        self.send_json(req).await
    }

    async fn send_json<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<T, ClientError> {
        let resp = req.send().await.map_err(map_send_err)?;
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ClientError::Decode(e.to_string()))?;
        if status.is_success() {
            serde_json::from_slice(&bytes).map_err(|e| ClientError::Decode(e.to_string()))
        } else {
            Err(parse_api_error(status, &bytes))
        }
    }

    async fn send_unit(&self, req: RequestBuilder) -> Result<(), ClientError> {
        let resp = req.send().await.map_err(map_send_err)?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| ClientError::Decode(e.to_string()))?;
            Err(parse_api_error(status, &bytes))
        }
    }

    // ---------- health / status ----------

    pub async fn health(&self) -> Result<Health, ClientError> {
        self.get_json(&["v1", "health"]).await
    }

    pub async fn status(&self) -> Result<Status, ClientError> {
        self.get_json(&["v1", "status"]).await
    }

    // ---------- agents ----------

    pub async fn list_agents(&self) -> Result<Vec<Agent>, ClientError> {
        self.get_json(&["v1", "agents"]).await
    }

    pub async fn spawn(&self, req: &SpawnRequest) -> Result<Agent, ClientError> {
        self.post_json(&["v1", "agents"], req).await
    }

    pub async fn get_agent(&self, id: &str) -> Result<Agent, ClientError> {
        self.get_json(&["v1", "agents", id]).await
    }

    pub async fn send_to_agent(&self, id: &str, req: &SendRequest) -> Result<Message, ClientError> {
        self.post_json(&["v1", "agents", id, "messages"], req).await
    }

    pub async fn interrupt(
        &self,
        id: &str,
        req: &InterruptRequest,
    ) -> Result<InterruptResponse, ClientError> {
        self.post_json(&["v1", "agents", id, "interrupt"], req)
            .await
    }

    pub async fn stop(&self, id: &str, req: &StopRequest) -> Result<Agent, ClientError> {
        self.post_json(&["v1", "agents", id, "stop"], req).await
    }

    pub async fn resume(&self, id: &str, req: &ResumeRequest) -> Result<Agent, ClientError> {
        self.post_json(&["v1", "agents", id, "resume"], req).await
    }

    pub async fn renew(&self, id: &str, req: &RenewRequest) -> Result<Agent, ClientError> {
        self.post_json(&["v1", "agents", id, "renew"], req).await
    }

    pub async fn remove(&self, id: &str, query: &RemoveQuery) -> Result<(), ClientError> {
        let req = self
            .request(Method::DELETE, &["v1", "agents", id])?
            .query(query);
        self.send_unit(req).await
    }

    pub async fn transcript(
        &self,
        id: &str,
        since: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<TranscriptLine>, ClientError> {
        let query = TranscriptQuery { since, limit };
        self.get_json_query(&["v1", "agents", id, "transcript"], &query)
            .await
    }

    // ---------- messages ----------

    pub async fn list_messages(&self, query: &MessageQuery) -> Result<Vec<Message>, ClientError> {
        self.get_json_query(&["v1", "messages"], query).await
    }

    /// Returns one message per recipient: one for `human` or a single
    /// agent, or one per matching live agent for a `role:<name>` target.
    pub async fn send(&self, req: &SendRequest) -> Result<Vec<Message>, ClientError> {
        self.post_json(&["v1", "messages"], req).await
    }

    pub async fn mark_read(&self, id: &str) -> Result<Message, ClientError> {
        self.post_empty(&["v1", "messages", id, "read"]).await
    }

    pub async fn mark_unread(&self, id: &str) -> Result<Message, ClientError> {
        self.post_empty(&["v1", "messages", id, "unread"]).await
    }

    // ---------- events ----------

    pub async fn events(&self, query: &EventQuery) -> Result<Vec<Event>, ClientError> {
        self.get_json_query(&["v1", "events"], query).await
    }

    /// `GET /v1/events/stream?since=` as SSE, reconnecting transparently.
    ///
    /// The daemon's `events_stream` ends the SSE response cleanly (no error)
    /// whenever the broadcast channel lags or closes (see
    /// `bridle-daemon::server::events_stream`), so a long-lived consumer
    /// must reconnect on its own. This resumes with `since` set to the last
    /// seq actually yielded, after a short backoff, whether the inner
    /// stream ended in an `Err` or just stopped. The request isn't sent
    /// until the returned stream is first polled.
    pub fn events_stream(
        &self,
        since: Option<i64>,
    ) -> impl Stream<Item = Result<Event, ClientError>> + use<> {
        type EventStream = std::pin::Pin<Box<dyn Stream<Item = Result<Event, ClientError>> + Send>>;
        struct State {
            client: Client,
            cursor: Option<i64>,
            inner: Option<EventStream>,
        }
        let state = State {
            client: self.clone(),
            cursor: since,
            inner: None,
        };
        futures::stream::unfold(state, |mut st| async move {
            loop {
                if st.inner.is_none() {
                    match open_event_stream(st.client.clone(), st.cursor).await {
                        Ok(s) => st.inner = Some(Box::pin(s)),
                        Err(_) => {
                            tokio::time::sleep(RECONNECT_BACKOFF).await;
                            continue;
                        }
                    }
                }
                let next = st.inner.as_mut().expect("set above when None").next().await;
                match next {
                    Some(Ok(ev)) => {
                        st.cursor = Some(ev.seq);
                        return Some((Ok(ev), st));
                    }
                    Some(Err(_)) | None => {
                        st.inner = None;
                        tokio::time::sleep(RECONNECT_BACKOFF).await;
                    }
                }
            }
        })
    }
}

/// How long to wait before reissuing `GET /v1/events/stream` after the
/// previous attempt ended, so a persistently-down server doesn't spin hot.
const RECONNECT_BACKOFF: std::time::Duration = std::time::Duration::from_millis(500);

async fn open_event_stream(
    client: Client,
    since: Option<i64>,
) -> Result<impl Stream<Item = Result<Event, ClientError>> + use<>, ClientError> {
    let mut req = client.request(Method::GET, &["v1", "events", "stream"])?;
    if let Some(s) = since {
        req = req.query(&[("since", s)]);
    }
    let resp = req.send().await.map_err(map_send_err)?;
    let status = resp.status();
    if !status.is_success() {
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ClientError::Decode(e.to_string()))?;
        return Err(parse_api_error(status, &bytes));
    }
    let byte_stream: std::pin::Pin<Box<dyn Stream<Item = Result<Bytes, ClientError>> + Send>> =
        Box::pin(resp.bytes_stream().map(|r| r.map_err(map_send_err)));
    Ok(sse::parse_event_stream(byte_stream))
}

impl Client {
    // ---------- usage / tokens / shutdown ----------

    pub async fn usage(&self) -> Result<Usage, ClientError> {
        self.get_json(&["v1", "usage"]).await
    }

    pub async fn usage_breakdown(
        &self,
        query: &UsageBreakdownQuery,
    ) -> Result<UsageBreakdown, ClientError> {
        self.get_json_query(&["v1", "usage", "breakdown"], query)
            .await
    }

    /// `GET /v1/interactions`: this machine's prompt log, oldest first. `human` and local
    /// readers only.
    pub async fn interactions(
        &self,
        query: &InteractionsQuery,
    ) -> Result<Vec<Interaction>, ClientError> {
        self.get_json_query(&["v1", "interactions"], query).await
    }

    /// `POST /v1/handovers`: any principal; the note is keyed by the caller's own identity.
    pub async fn write_handover(&self, body: &str) -> Result<Handover, ClientError> {
        let req = WriteHandoverRequest {
            body: body.to_string(),
        };
        self.post_json(&["v1", "handovers"], &req).await
    }

    /// `GET /v1/handovers`: newest first; `role` limits it to one identity's notes.
    pub async fn list_handovers(&self, role: Option<&str>) -> Result<Vec<Handover>, ClientError> {
        self.get_json_query(&["v1", "handovers"], &[("role", role)])
            .await
    }

    /// `GET /v1/handovers/latest`: the newest note (of `role`, if given), if any.
    pub async fn latest_handover(
        &self,
        role: Option<&str>,
    ) -> Result<Option<Handover>, ClientError> {
        self.get_json_query(&["v1", "handovers", "latest"], &[("role", role)])
            .await
    }

    /// `GET /v1/handovers/{id}`.
    pub async fn get_handover(&self, id: &str) -> Result<Handover, ClientError> {
        self.get_json(&["v1", "handovers", id]).await
    }

    /// `GET /v1/orchestrator/wake`: holds until a wake is pending (or `timeout_secs`, default
    /// 25 minutes, are up, answering empty). `external:orchestrator` only.
    pub async fn orchestrator_wake(
        &self,
        timeout_secs: Option<u64>,
    ) -> Result<WakeResponse, ClientError> {
        self.get_json_query(
            &["v1", "orchestrator", "wake"],
            &OrchestratorWakeQuery { timeout_secs },
        )
        .await
    }

    /// `GET /v1/wake`: holds until the daemon decides `query.principal` should wake; empty
    /// when `timeout_secs` ran out first. The caller must be that principal (or the human).
    pub async fn principal_wake(
        &self,
        query: &PrincipalWakeQuery,
    ) -> Result<PrincipalWakeResponse, ClientError> {
        self.get_json_query(&["v1", "wake"], query).await
    }

    /// `POST /v1/wake/stop`: end open waits (see [`StopWakeRequest`]).
    pub async fn stop_wake(&self, req: &StopWakeRequest) -> Result<StopWakeResponse, ClientError> {
        self.post_json(&["v1", "wake", "stop"], req).await
    }

    /// `POST /v1/orchestrator/handover`: the orchestrator has written its state; the daemon
    /// stops the session and relaunches it. Human and `external:orchestrator` only.
    pub async fn handover_done(&self) -> Result<HandoverDone, ClientError> {
        self.post_empty(&["v1", "orchestrator", "handover"]).await
    }

    /// `POST /v1/sessions`: register an interactive session, or update the one with this pid.
    pub async fn session_register(
        &self,
        req: &SessionRegister,
    ) -> Result<SessionInfo, ClientError> {
        self.post_json(&["v1", "sessions"], req).await
    }

    /// `POST /v1/sessions/end`.
    pub async fn session_end(&self, pid: i32) -> Result<(), ClientError> {
        let _: serde_json::Value = self
            .post_json(&["v1", "sessions", "end"], &SessionEnd { pid })
            .await?;
        Ok(())
    }

    /// `POST /v1/sessions/keep`: carry on past the session's current step.
    pub async fn session_keep(&self, identity: &str) -> Result<(), ClientError> {
        let _: serde_json::Value = self
            .post_json(
                &["v1", "sessions", "keep"],
                &SessionKeep {
                    identity: identity.to_string(),
                },
            )
            .await?;
        Ok(())
    }

    /// `POST /v1/review/now`.
    pub async fn review_now(
        &self,
        req: &ReviewNowRequest,
    ) -> Result<ReviewNowResponse, ClientError> {
        self.post_json(&["v1", "review", "now"], req).await
    }

    /// `POST /v1/review/add`.
    pub async fn review_add(
        &self,
        req: &ReviewAddRequest,
    ) -> Result<ReviewAddResponse, ClientError> {
        self.post_json(&["v1", "review", "add"], req).await
    }

    /// `GET /v1/sessions`.
    pub async fn sessions(&self) -> Result<Vec<SessionInfo>, ClientError> {
        self.get_json(&["v1", "sessions"]).await
    }

    pub async fn budget(&self) -> Result<BudgetStatus, ClientError> {
        self.get_json(&["v1", "budget"]).await
    }

    pub async fn budget_hold(&self, req: &BudgetHoldRequest) -> Result<BudgetStatus, ClientError> {
        self.post_json(&["v1", "budget", "hold"], req).await
    }

    pub async fn budget_release(&self) -> Result<BudgetStatus, ClientError> {
        self.post_empty(&["v1", "budget", "release"]).await
    }

    pub async fn budget_override(
        &self,
        req: &BudgetOverrideRequest,
    ) -> Result<BudgetStatus, ClientError> {
        self.post_json(&["v1", "budget", "override"], req).await
    }

    pub async fn budget_override_clear(&self) -> Result<BudgetStatus, ClientError> {
        self.post_empty(&["v1", "budget", "override", "clear"])
            .await
    }

    pub async fn budget_max_workers(
        &self,
        req: &MaxWorkersRequest,
    ) -> Result<BudgetStatus, ClientError> {
        self.post_json(&["v1", "budget", "max-workers"], req).await
    }

    pub async fn create_token(
        &self,
        req: &TokenCreateRequest,
    ) -> Result<TokenCreated, ClientError> {
        self.post_json(&["v1", "tokens"], req).await
    }

    pub async fn list_tokens(&self) -> Result<Vec<TokenInfo>, ClientError> {
        self.get_json(&["v1", "tokens"]).await
    }

    pub async fn revoke_token(&self, name: &str) -> Result<(), ClientError> {
        let req = self.request(Method::DELETE, &["v1", "tokens", name])?;
        self.send_unit(req).await
    }

    // ---------- tasks ----------

    pub async fn list_tasks(&self) -> Result<Vec<Task>, ClientError> {
        self.get_json(&["v1", "tasks"]).await
    }

    /// `GET /v1/tasks?ready=true` (roles-and-lifecycle.md, "ready is computed").
    pub async fn ready_tasks(&self) -> Result<Vec<Task>, ClientError> {
        let query = TaskQuery {
            ready: Some(true),
            ..Default::default()
        };
        self.get_json_query(&["v1", "tasks"], &query).await
    }

    /// `GET /v1/tasks?top_tier=true`: just the highest queue tier with a
    /// startable task (roles-and-lifecycle.md, "the queue").
    pub async fn top_tier_ready_tasks(&self) -> Result<Vec<Task>, ClientError> {
        let query = TaskQuery {
            top_tier: Some(true),
            ..Default::default()
        };
        self.get_json_query(&["v1", "tasks"], &query).await
    }

    /// `GET /v1/tasks?claimed_by=...`; `claimed_by` may be `me`, `human`, an
    /// agent name, or a full `PrincipalId`.
    pub async fn list_tasks_claimed_by(&self, claimed_by: &str) -> Result<Vec<Task>, ClientError> {
        let query = TaskQuery {
            claimed_by: Some(claimed_by.to_string()),
            ..Default::default()
        };
        self.get_json_query(&["v1", "tasks"], &query).await
    }

    /// `GET /v1/tasks?component=...`: tasks naming `component` or a descendant.
    pub async fn list_tasks_component(&self, component: &str) -> Result<Vec<Task>, ClientError> {
        let query = TaskQuery {
            component: Some(component.to_string()),
            ..Default::default()
        };
        self.get_json_query(&["v1", "tasks"], &query).await
    }

    /// Search for tasks by words in title, body, or summary (all words must match, case-insensitive).
    pub async fn search_tasks(&self, words: &[&str]) -> Result<Vec<Task>, ClientError> {
        let all_tasks = self.list_tasks().await?;
        let search_words: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
        let filtered_tasks: Vec<Task> = all_tasks
            .into_iter()
            .filter(|task| {
                let title_lower = task.title.to_lowercase();
                let body_lower = task.body.to_lowercase();
                let summary_lower = task.summary.as_ref().map(|s| s.to_lowercase());
                search_words.iter().all(|word| {
                    title_lower.contains(word)
                        || body_lower.contains(word)
                        || summary_lower.as_ref().is_some_and(|s| s.contains(word))
                })
            })
            .collect();
        Ok(filtered_tasks)
    }

    pub async fn new_task(&self, req: &NewTaskRequest) -> Result<Task, ClientError> {
        self.post_json(&["v1", "tasks"], req).await
    }

    pub async fn submit_task(&self, req: &SubmitTaskRequest) -> Result<Task, ClientError> {
        self.post_json(&["v1", "tasks", "submit"], req).await
    }

    pub async fn get_task(&self, id: &str) -> Result<Task, ClientError> {
        self.get_json(&["v1", "tasks", id]).await
    }

    pub async fn edit_task(&self, id: &str, req: &EditTaskRequest) -> Result<Task, ClientError> {
        self.patch_json(&["v1", "tasks", id], req).await
    }

    pub async fn drop_task(&self, id: &str, req: &DropTaskRequest) -> Result<Task, ClientError> {
        self.post_json(&["v1", "tasks", id, "drop"], req).await
    }

    pub async fn done_task(&self, id: &str, req: &DoneTaskRequest) -> Result<Task, ClientError> {
        self.post_json(&["v1", "tasks", id, "done"], req).await
    }

    pub async fn set_task_summary(
        &self,
        id: &str,
        req: &SetSummaryRequest,
    ) -> Result<Task, ClientError> {
        self.post_json(&["v1", "tasks", id, "summary"], req).await
    }

    pub async fn set_task_priority(
        &self,
        id: &str,
        req: &SetPriorityRequest,
    ) -> Result<Task, ClientError> {
        self.post_json(&["v1", "tasks", id, "priority"], req).await
    }

    pub async fn set_task_kind(&self, id: &str, req: &SetKindRequest) -> Result<Task, ClientError> {
        self.post_json(&["v1", "tasks", id, "kind"], req).await
    }

    pub async fn set_task_impact(
        &self,
        id: &str,
        req: &SetImpactRequest,
    ) -> Result<Task, ClientError> {
        self.post_json(&["v1", "tasks", id, "impact"], req).await
    }

    pub async fn impact_check(
        &self,
        req: &ImpactCheckRequest,
    ) -> Result<ImpactReport, ClientError> {
        self.post_json(&["v1", "impact", "check"], req).await
    }

    pub async fn land_task(&self, id: &str, req: &LandRequest) -> Result<LandResult, ClientError> {
        self.post_json(&["v1", "tasks", id, "land"], req).await
    }

    pub async fn probe(&self, req: &ProbeRequest) -> Result<ProbeResult, ClientError> {
        self.post_json(&["v1", "probe"], req).await
    }

    pub async fn list_conflicts(&self) -> Result<Vec<Conflict>, ClientError> {
        self.get_json(&["v1", "conflicts"]).await
    }

    pub async fn resolve_conflict(
        &self,
        id: &str,
        req: &ResolveConflictRequest,
    ) -> Result<Conflict, ClientError> {
        self.post_json(&["v1", "conflicts", id, "resolve"], req)
            .await
    }

    pub async fn alloc_port(&self, req: &AllocPortRequest) -> Result<PortAllocation, ClientError> {
        self.post_json(&["v1", "ports"], req).await
    }

    pub async fn list_ports(&self) -> Result<Vec<PortAllocation>, ClientError> {
        self.get_json(&["v1", "ports"]).await
    }

    /// Returns the allocation that was released.
    pub async fn release_port(&self, port: u16) -> Result<PortAllocation, ClientError> {
        self.post_empty(&["v1", "ports", &port.to_string(), "release"])
            .await
    }

    pub async fn reopen_task(&self, id: &str) -> Result<Task, ClientError> {
        self.post_empty(&["v1", "tasks", id, "reopen"]).await
    }

    /// `pending` -> `open`. A 409 means the task isn't `pending`.
    pub async fn ready_task(&self, id: &str) -> Result<Task, ClientError> {
        self.post_empty(&["v1", "tasks", id, "ready"]).await
    }

    /// `open` -> `planned`. A 409 means the task isn't `open`.
    pub async fn plan_task(&self, id: &str) -> Result<Task, ClientError> {
        self.post_empty(&["v1", "tasks", id, "plan"]).await
    }

    pub async fn ask_question(
        &self,
        id: &str,
        body: &str,
        to: Option<&str>,
    ) -> Result<Task, ClientError> {
        self.post_json(
            &["v1", "tasks", id, "ask"],
            &AskQuestionRequest {
                body: body.to_string(),
                to: to.map(str::to_string),
            },
        )
        .await
    }

    pub async fn answer_question(&self, id: &str, body: &str) -> Result<Task, ClientError> {
        self.post_json(
            &["v1", "tasks", id, "answer"],
            &AnswerQuestionRequest {
                body: body.to_string(),
            },
        )
        .await
    }

    pub async fn note_task(&self, id: &str, body: &str) -> Result<Task, ClientError> {
        self.post_json(
            &["v1", "tasks", id, "note"],
            &NoteTaskRequest {
                body: body.to_string(),
            },
        )
        .await
    }

    /// Skips the task's settle period, recording `reason` in its thread.
    pub async fn skip_settle(&self, id: &str, reason: &str) -> Result<Task, ClientError> {
        self.post_json(
            &["v1", "tasks", id, "skip-settle"],
            &SkipSettleRequest {
                reason: reason.to_string(),
            },
        )
        .await
    }

    pub async fn list_open_questions(&self) -> Result<Vec<OpenQuestion>, ClientError> {
        self.get_json(&["v1", "questions"]).await
    }

    /// Claims a ready task for the caller: `planned` -> `claimed`. A 409
    /// means the task isn't ready to claim (not planned, blocked, or
    /// already claimed).
    pub async fn claim_task(&self, id: &str) -> Result<Task, ClientError> {
        self.post_empty(&["v1", "tasks", id, "claim"]).await
    }

    /// Releases the caller's own claim: `claimed` -> `planned`. A 409 means
    /// the caller isn't the current claimant, including if the task isn't
    /// claimed at all.
    /// The calling principal starts watching the task (a no-op if it already does).
    pub async fn watch_task(&self, id: &str) -> Result<Task, ClientError> {
        self.post_empty(&["v1", "tasks", id, "watch"]).await
    }

    /// The calling principal stops watching the task.
    pub async fn unwatch_task(&self, id: &str) -> Result<Task, ClientError> {
        self.post_empty(&["v1", "tasks", id, "unwatch"]).await
    }

    pub async fn release_task(&self, id: &str) -> Result<Task, ClientError> {
        self.post_empty(&["v1", "tasks", id, "release"]).await
    }

    // ---------- queue ----------

    pub async fn get_queue(&self) -> Result<Queue, ClientError> {
        self.get_json(&["v1", "queue"]).await
    }

    /// PM-only (and human): 403 for anyone else (roles-and-lifecycle.md,
    /// "the queue").
    pub async fn set_queue(&self, tiers: Vec<Vec<String>>) -> Result<Queue, ClientError> {
        self.post_json(&["v1", "queue"], &SetQueueRequest { tiers })
            .await
    }

    /// PM-only (and human), same as [`Client::set_queue`].
    pub async fn add_queue_tier(&self, tasks: Vec<String>) -> Result<Queue, ClientError> {
        self.post_json(&["v1", "queue", "tiers"], &AddQueueTierRequest { tasks })
            .await
    }

    /// `POST /v1/migrations`: record an applied project migration as an event.
    pub async fn record_migration(
        &self,
        rec: &crate::types::MigrationRecord,
    ) -> Result<(), ClientError> {
        let _: serde_json::Value = self.post_json(&["v1", "migrations"], rec).await?;
        Ok(())
    }

    // ---------- edges ----------

    pub async fn list_edges(&self) -> Result<Vec<Edge>, ClientError> {
        self.get_json(&["v1", "edges"]).await
    }

    pub async fn add_edge(&self, req: &NewEdgeRequest) -> Result<Edge, ClientError> {
        self.post_json(&["v1", "edges"], req).await
    }

    pub async fn remove_edge(&self, query: &RemoveEdgeQuery) -> Result<(), ClientError> {
        let req = self.request(Method::DELETE, &["v1", "edges"])?.query(query);
        self.send_unit(req).await
    }

    /// Waits (up to `req.wait_secs`) for a quiet point, then the daemon restarts itself. A busy
    /// daemon answers 409 and does nothing. Give the client a timeout longer than the wait.
    pub async fn restart(&self, req: &RestartRequest) -> Result<RestartResponse, ClientError> {
        self.post_json(&["v1", "restart"], req).await
    }

    /// `None` when the daemon is older than the reply body and answered 204.
    pub async fn shutdown(&self) -> Result<Option<ShutdownResponse>, ClientError> {
        let resp = self
            .request(Method::POST, &["v1", "shutdown"])?
            .send()
            .await
            .map_err(map_send_err)?;
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ClientError::Decode(e.to_string()))?;
        if !status.is_success() {
            return Err(parse_api_error(status, &bytes));
        }
        if bytes.is_empty() {
            return Ok(None);
        }
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|e| ClientError::Decode(e.to_string()))
    }

    /// `bridle rebuild`: reconstructs `tasks`/`edges`/`open_questions` from
    /// the project's state branch. Human-only; fails if the database
    /// already has rows in any of those tables.
    /// With `from_origin`, first fetches `origin/bridle/state` (fast-forward only; never
    /// overwrites a local branch) and says what happened in the response.
    pub async fn rebuild(
        &self,
        from_origin: bool,
    ) -> Result<crate::types::RebuildResponse, ClientError> {
        let mut req = self.request(Method::POST, &["v1", "rebuild"])?;
        if from_origin {
            req = req.query(&[("from_origin", "true")]);
        }
        self.send_json(req).await
    }

    /// `bridle statusline`'s only network call. Build the client with
    /// [`Client::new_with_timeout`] so this never hangs the human's prompt.
    pub async fn report_statusline(&self, report: &StatusLineReport) -> Result<(), ClientError> {
        let req = self
            .request(Method::POST, &["v1", "statusline"])?
            .json(report);
        self.send_unit(req).await
    }
}

fn map_send_err(e: reqwest::Error) -> ClientError {
    if e.is_connect() || e.is_timeout() {
        ClientError::Unreachable(e.to_string())
    } else {
        ClientError::Other(e.to_string())
    }
}

fn parse_api_error(status: StatusCode, body: &[u8]) -> ClientError {
    match serde_json::from_slice::<ApiErrorResponse>(body) {
        Ok(e) => ClientError::Api {
            status: status.as_u16(),
            code: e.error.code,
            message: e.error.message,
        },
        Err(_) => ClientError::Api {
            status: status.as_u16(),
            code: "unknown".to_string(),
            message: String::from_utf8_lossy(body).into_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use axum::Json;
    use axum::extract::State;
    use axum::response::sse::{Event as AxumSseEvent, Sse};
    use axum::routing::{get, post};
    use futures::StreamExt;
    use serde_json::json;
    use tokio::net::TcpListener;

    use super::*;
    use crate::types::{AgentState, ErrorBody, PrincipalKind};

    async fn spawn_test_server(app: axum::Router) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        addr
    }

    #[tokio::test]
    async fn health_ok() {
        let app = axum::Router::new().route(
            "/v1/health",
            get(|| async { Json(json!({"ok": true, "version": "0.1.0", "agent_count": 2})) }),
        );
        let addr = spawn_test_server(app).await;
        let client = Client::new(format!("http://{addr}"), None);
        let health = client.health().await.unwrap();
        assert!(health.ok);
        assert_eq!(health.version, "0.1.0");
        assert_eq!(health.agent_count, 2);
    }

    #[tokio::test]
    async fn list_agents_returns_a_json_array() {
        let app = axum::Router::new().route(
            "/v1/agents",
            get(|| async {
                Json(json!([{
                    "id": "a-1", "name": "w1", "role": "worker", "state": "idle",
                    "model": "sonnet", "session_id": "s-1", "pid": null, "cwd": "/ws/wt/w1",
                    "worktree": "/ws/wt/w1", "branch": "bridle/w1",
                    "created_at": "2026-09-27T00:00:00Z", "updated_at": "2026-09-27T00:00:00Z",
                    "last_event_at": null, "turns": 0, "turn_started_at": null,
                    "cost_usd_total": 0.0, "exit": null, "created_by": "human",
                    "held_messages": 0, "unacked_messages": 0
                }]))
            }),
        );
        let addr = spawn_test_server(app).await;
        let client = Client::new(format!("http://{addr}"), None);
        let agents = client.list_agents().await.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "a-1");
        assert_eq!(agents[0].state, AgentState::Idle);
    }

    #[tokio::test]
    async fn sends_bearer_token() {
        let app = axum::Router::new().route(
            "/v1/status",
            get(|headers: axum::http::HeaderMap| async move {
                let auth = headers
                    .get("authorization")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default();
                assert_eq!(auth, "Bearer secret-token");
                Json(json!({
                    "daemon": {
                        "project": "demo", "workspace": "/ws", "repo": "/ws/repo",
                        "url": "http://x", "pid": 1, "started_at": "2026-09-27T00:00:00Z",
                        "version": "0.1.0"
                    },
                    "principal": "human",
                    "agents_by_state": {},
                    "unread_human_messages": 0,
                    "rate_limits": []
                }))
            }),
        );
        let addr = spawn_test_server(app).await;
        let client = Client::new(format!("http://{addr}"), Some("secret-token".to_string()));
        let status = client.status().await.unwrap();
        assert_eq!(status.principal, "human");
        let _ = PrincipalKind::Human; // keep import used across cfg combos
    }

    #[tokio::test]
    async fn api_error_is_decoded_from_error_body() {
        let app = axum::Router::new().route(
            "/v1/agents/missing",
            get(|| async {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    Json(json!({"error": {"code": "not_found", "message": "no such agent"}})),
                )
            }),
        );
        let addr = spawn_test_server(app).await;
        let client = Client::new(format!("http://{addr}"), None);
        let err = client.get_agent("missing").await.unwrap_err();
        match err {
            ClientError::Api {
                status,
                code,
                message,
            } => {
                assert_eq!(status, 404);
                assert_eq!(code, "not_found");
                assert_eq!(message, "no such agent");
            }
            other => panic!("expected ClientError::Api, got {other:?}"),
        }
        // Exercise the ErrorBody type directly too, so it's covered here.
        let body: ErrorBody = serde_json::from_value(json!({"code": "x", "message": "y"})).unwrap();
        assert_eq!(body.code, "x");
    }

    #[tokio::test]
    async fn unreachable_when_nothing_is_listening() {
        // Reserve a port, then drop the listener so nothing answers on it.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let client = Client::new(format!("http://{addr}"), None);
        let err = client.health().await.unwrap_err();
        assert!(
            matches!(err, ClientError::Unreachable(_)),
            "expected Unreachable, got {err:?}"
        );
    }

    #[tokio::test]
    async fn remove_sends_query_params() {
        let app = axum::Router::new().route(
            "/v1/agents/w1",
            axum::routing::delete(
                |axum::extract::Query(q): axum::extract::Query<RemoveQuery>| async move {
                    assert!(q.force);
                    assert!(!q.delete_branch);
                    axum::http::StatusCode::NO_CONTENT
                },
            ),
        );
        let addr = spawn_test_server(app).await;
        let client = Client::new(format!("http://{addr}"), None);
        client
            .remove(
                "w1",
                &RemoveQuery {
                    force: true,
                    delete_branch: false,
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn events_stream_parses_sse_from_a_real_server() {
        #[derive(Clone)]
        struct AppState;

        async fn stream_handler(
            State(_): State<AppState>,
        ) -> Sse<impl futures::Stream<Item = Result<AxumSseEvent, std::convert::Infallible>>>
        {
            let events = vec![
                json!({"seq": 1, "ts": "2026-09-27T00:00:00Z", "kind": "agent.text", "actor": "human", "agent": null, "data": {}}),
                json!({"seq": 2, "ts": "2026-09-27T00:00:01Z", "kind": "agent.text", "actor": "human", "agent": null, "data": {}}),
            ];
            let stream = futures::stream::iter(
                events
                    .into_iter()
                    .map(|e| Ok(AxumSseEvent::default().data(e.to_string()))),
            );
            Sse::new(stream)
        }

        let app = axum::Router::new()
            .route("/v1/events/stream", get(stream_handler))
            .with_state(AppState);
        let addr = spawn_test_server(app).await;
        let client = Client::new(format!("http://{addr}"), None);

        let events: Vec<_> = client.events_stream(None).take(2).collect().await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].as_ref().unwrap().seq, 1);
        assert_eq!(events[1].as_ref().unwrap().seq, 2);
    }

    #[tokio::test]
    async fn events_stream_reconnects_after_the_server_closes_the_stream() {
        #[derive(Clone)]
        struct AppState {
            calls: std::sync::Arc<std::sync::atomic::AtomicU32>,
        }

        async fn stream_handler(
            State(state): State<AppState>,
            axum::extract::Query(q): axum::extract::Query<
                std::collections::HashMap<String, String>,
            >,
        ) -> Sse<impl futures::Stream<Item = Result<AxumSseEvent, std::convert::Infallible>>>
        {
            let call = state
                .calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let since: i64 = q.get("since").and_then(|s| s.parse().ok()).unwrap_or(0);
            // First connection: two events, then the stream ends cleanly
            // (as the daemon does on lag/close), with no error. A
            // reconnecting client resumes with `since` set to the last seq
            // it yielded, so the second call should only be asked for
            // events after seq 2.
            let events: Vec<serde_json::Value> = if call == 0 {
                vec![
                    json!({"seq": 1, "ts": "2026-09-27T00:00:00Z", "kind": "agent.text", "actor": "human", "agent": null, "data": {}}),
                    json!({"seq": 2, "ts": "2026-09-27T00:00:01Z", "kind": "agent.text", "actor": "human", "agent": null, "data": {}}),
                ]
            } else {
                assert_eq!(
                    since, 2,
                    "reconnect should resume from the last seq yielded"
                );
                vec![
                    json!({"seq": 3, "ts": "2026-09-27T00:00:02Z", "kind": "agent.text", "actor": "human", "agent": null, "data": {}}),
                ]
            };
            let stream = futures::stream::iter(
                events
                    .into_iter()
                    .map(|e| Ok(AxumSseEvent::default().data(e.to_string()))),
            );
            Sse::new(stream)
        }

        let state = AppState {
            calls: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
        };
        let app = axum::Router::new()
            .route("/v1/events/stream", get(stream_handler))
            .with_state(state);
        let addr = spawn_test_server(app).await;
        let client = Client::new(format!("http://{addr}"), None);

        let events: Vec<_> = client.events_stream(None).take(3).collect().await;
        let seqs: Vec<i64> = events.into_iter().map(|e| e.unwrap().seq).collect();
        assert_eq!(seqs, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn spawn_posts_body_and_returns_agent() {
        let app = axum::Router::new().route(
            "/v1/agents",
            post(|Json(body): Json<SpawnRequest>| async move {
                assert_eq!(body.role, "worker");
                Json(json!({
                    "id": "a-2", "name": "w2", "role": "worker", "state": "starting",
                    "model": "sonnet", "session_id": "s-2", "pid": null, "cwd": "/ws/wt/w2",
                    "worktree": "/ws/wt/w2", "branch": "bridle/w2",
                    "created_at": "2026-09-27T00:00:00Z", "updated_at": "2026-09-27T00:00:00Z",
                    "last_event_at": null, "turns": 0, "turn_started_at": null,
                    "cost_usd_total": 0.0, "exit": null, "created_by": "human",
                    "held_messages": 0, "unacked_messages": 0
                }))
            }),
        );
        let addr = spawn_test_server(app).await;
        let client = Client::new(format!("http://{addr}"), None);
        let agent = client
            .spawn(&SpawnRequest {
                role: "worker".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(agent.id, "a-2");
    }
}
