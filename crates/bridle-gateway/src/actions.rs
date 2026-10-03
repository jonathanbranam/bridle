//! The only things the gateway does to a daemon besides read: check off a to-do, decline one
//! with a reason, answer a task question. Anything else is refused here, so a stolen session
//! can answer and check off, not run work. Acts with the human's token, found the way the CLI
//! finds it (principals.md); this machine's projects only (remote machines are task 9).
//! See docs/design/human-web-ui.md sections 1, 2 and 5.

use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bridle_api::client::{Client, ClientError};
use bridle_api::discovery::{ProcessEnv, resolve_token};
use bridle_api::types::{DoneTaskRequest, DropTaskRequest};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::discovery::{PROBE_TIMEOUT, current_targets};

/// The v1 actions; the path segment after the task id names one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Check off a to-do.
    Done,
    /// Decline a to-do; needs a reason.
    Drop,
    /// Answer a task question; needs the answer.
    Answer,
}

impl Action {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "done" => Some(Self::Done),
            "drop" => Some(Self::Drop),
            "answer" => Some(Self::Answer),
            _ => None,
        }
    }
}

/// The body of an action. `text` is the reason for `drop` and the answer for `answer`;
/// `done` ignores it.
#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
pub struct ActionRequest {
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ActionResult {
    pub project: String,
    pub task_id: String,
    pub action: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("unknown action '{0}': only done, drop and answer are available")]
    UnknownAction(String),
    #[error("no project '{0}' is known on this machine")]
    UnknownProject(String),
    #[error("{0}")]
    NotAvailable(String),
    #[error("no human token for project '{project}': {reason}")]
    NoToken { project: String, reason: String },
    #[error("'{0}' needs a non-empty text")]
    MissingText(&'static str),
    #[error("{0}")]
    Daemon(String),
    /// The daemon refused: its status and message pass through.
    #[error("{message}")]
    Refused { status: u16, message: String },
}

impl IntoResponse for ActionError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::UnknownAction(_) | Self::UnknownProject(_) => StatusCode::NOT_FOUND,
            Self::MissingText(_) => StatusCode::BAD_REQUEST,
            Self::NotAvailable(_) | Self::NoToken { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::Daemon(_) => StatusCode::BAD_GATEWAY,
            Self::Refused { status, .. } => {
                StatusCode::from_u16(*status).unwrap_or(StatusCode::BAD_GATEWAY)
            }
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

/// `POST /api/v1/projects/{project}/tasks/{id}/{action}`.
pub async fn act_route(
    Path((project, id, action)): Path<(String, String, String)>,
    body: Option<Json<ActionRequest>>,
) -> Result<Json<ActionResult>, ActionError> {
    let action = Action::parse(&action).ok_or(ActionError::UnknownAction(action))?;
    let req = body.map(|b| b.0).unwrap_or_default();
    let (url, token) = resolve(&project).await?;
    act(&url, Some(token), action, &id, &req.text).await?;
    Ok(Json(ActionResult {
        project,
        task_id: id,
        action: format!("{action:?}").to_lowercase(),
    }))
}

/// The daemon's address and the human's token for one of this machine's projects.
async fn resolve(project: &str) -> Result<(String, String), ActionError> {
    let targets = current_targets().await;
    let t = targets
        .into_iter()
        .find(|t| t.project == project)
        .ok_or_else(|| ActionError::UnknownProject(project.to_string()))?;
    if let Some(problem) = t.problem {
        return Err(ActionError::NotAvailable(problem));
    }
    let Some(workspace) = t.workspace else {
        return Err(ActionError::NotAvailable(format!(
            "project '{project}' is on another machine: remote actions aren't supported yet"
        )));
    };
    let url = t
        .url
        .ok_or_else(|| ActionError::NotAvailable(format!("no address for '{project}'")))?;
    let name = project.to_string();
    // Reads the token file: blocking work.
    let token = tokio::task::spawn_blocking(move || {
        resolve_token(
            None,
            Some(std::path::Path::new(&workspace)),
            Some(&name),
            None,
            &ProcessEnv,
            false,
        )
    })
    .await
    .map_err(|e| ActionError::Daemon(e.to_string()))?
    .map_err(|e| ActionError::NoToken {
        project: project.to_string(),
        reason: e.to_string(),
    })?
    .ok_or_else(|| ActionError::NoToken {
        project: project.to_string(),
        reason: "none found".to_string(),
    })?;
    Ok((url, token))
}

/// Sends one action to the daemon at `url` with `token`. A missing token is an error here, not
/// an anonymous request: the daemon would take that as `local`, which can't act.
pub(crate) async fn act(
    url: &str,
    token: Option<String>,
    action: Action,
    task_id: &str,
    text: &str,
) -> Result<(), ActionError> {
    let token = token.ok_or_else(|| ActionError::NoToken {
        project: String::new(),
        reason: "none given".to_string(),
    })?;
    let text = text.trim();
    let client = Client::new_with_timeout(url.to_string(), Some(token), PROBE_TIMEOUT * 5);
    let sent = match action {
        Action::Done => client
            .done_task(task_id, &DoneTaskRequest::default())
            .await
            .map(drop),
        Action::Drop => {
            if text.is_empty() {
                return Err(ActionError::MissingText("drop"));
            }
            client
                .drop_task(
                    task_id,
                    &DropTaskRequest {
                        reason: text.to_string(),
                    },
                )
                .await
                .map(drop)
        }
        Action::Answer => {
            if text.is_empty() {
                return Err(ActionError::MissingText("answer"));
            }
            client.answer_question(task_id, text).await.map(drop)
        }
    };
    sent.map_err(|e| match e {
        ClientError::Api {
            status, message, ..
        } => ActionError::Refused { status, message },
        other => ActionError::Daemon(other.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, extract::State, http::HeaderMap, routing::post};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    type Seen = Arc<Mutex<Vec<(String, Option<String>, Value)>>>;

    async fn fake() -> (String, Seen) {
        let seen: Seen = Arc::default();
        let record = |State(seen): State<Seen>,
                      uri: axum::http::Uri,
                      h: HeaderMap,
                      body: String| async move {
            let auth = h
                .get("authorization")
                .map(|v| v.to_str().expect("str").to_string());
            let body = serde_json::from_str(&body).unwrap_or(Value::Null);
            seen.lock()
                .expect("lock")
                .push((uri.path().to_string(), auth, body));
            Json(json!({
                "id": "t-1", "title": "x", "kind": "feature", "state": "integrated", "body": "",
                "thread": [], "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
            }))
        };
        let app = Router::new()
            .route("/v1/tasks/{id}/done", post(record))
            .route("/v1/tasks/{id}/drop", post(record))
            .route("/v1/tasks/{id}/answer", post(record))
            .with_state(seen.clone());
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        (url, seen)
    }

    #[tokio::test]
    async fn each_action_reaches_the_daemon_with_the_humans_token() {
        let (url, seen) = fake().await;
        let tok = || Some("human-tok".to_string());
        act(&url, tok(), Action::Done, "t-1", "")
            .await
            .expect("done");
        act(&url, tok(), Action::Drop, "t-1", "not now")
            .await
            .expect("drop");
        act(&url, tok(), Action::Answer, "t-1", "yes")
            .await
            .expect("answer");
        let seen = seen.lock().expect("lock");
        let paths: Vec<_> = seen.iter().map(|s| s.0.as_str()).collect();
        assert_eq!(
            paths,
            [
                "/v1/tasks/t-1/done",
                "/v1/tasks/t-1/drop",
                "/v1/tasks/t-1/answer"
            ]
        );
        assert!(
            seen.iter()
                .all(|s| s.1.as_deref() == Some("Bearer human-tok"))
        );
        assert_eq!(seen[1].2["reason"], "not now");
        assert_eq!(seen[2].2["body"], "yes");
    }

    #[tokio::test]
    async fn missing_token_is_a_clear_error_and_sends_nothing() {
        let (url, seen) = fake().await;
        let err = act(&url, None, Action::Done, "t-1", "")
            .await
            .expect_err("no token");
        assert!(err.to_string().contains("no human token"), "{err}");
        assert!(seen.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn drop_and_answer_need_text() {
        let (url, seen) = fake().await;
        for a in [Action::Drop, Action::Answer] {
            let err = act(&url, Some("t".into()), a, "t-1", "  ")
                .await
                .expect_err("empty");
            assert!(matches!(err, ActionError::MissingText(_)));
        }
        assert!(seen.lock().expect("lock").is_empty());
    }

    #[test]
    fn only_the_three_actions_parse() {
        assert_eq!(Action::parse("done"), Some(Action::Done));
        assert_eq!(Action::parse("drop"), Some(Action::Drop));
        assert_eq!(Action::parse("answer"), Some(Action::Answer));
        for other in ["land", "shutdown", "claim", "retract", ""] {
            assert_eq!(Action::parse(other), None, "{other}");
        }
    }
}
