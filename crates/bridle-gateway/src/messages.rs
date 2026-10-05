//! The gateway's one agent-facing write: a message from the human to one agent or the
//! orchestrator of a project (ticket rk7k). Sent with the human's token through the daemon's
//! ordinary send endpoint, so the sender is recorded as `human` and the message is in the audit
//! trail. Nothing else about agents is writable. See docs/design/human-web-ui.md section 2.

use axum::Json;
use axum::extract::Path;
use bridle_api::client::Client;
use bridle_api::types::{MessageKind, SendRequest, When};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::actions::{ActionError, resolve};
use crate::discovery::PROBE_TIMEOUT;
use crate::system::api_error;

/// Longest message the gateway will pass on, in characters.
pub const MAX_TEXT: usize = 8000;

/// The orchestrator's name in a recipient list; the daemon's principal is `external:orchestrator`.
const ORCHESTRATOR: &str = "orchestrator";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Recipients {
    pub project: String,
    /// Running agents' names, then `orchestrator`.
    pub recipients: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct MessageRequest {
    /// An agent name from the recipients list, or `orchestrator`.
    pub to: String,
    pub text: String,
    /// Thread the message on this task.
    #[serde(default)]
    pub task: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MessageSent {
    pub project: String,
    pub to: String,
}

async fn recipients(client: &Client, project: &str) -> Result<Recipients, ActionError> {
    let agents = client.list_agents().await.map_err(api_error)?;
    let mut recipients: Vec<String> = agents
        .into_iter()
        .filter(|a| a.state.is_running())
        .map(|a| a.name)
        .collect();
    recipients.sort();
    recipients.push(ORCHESTRATOR.to_string());
    Ok(Recipients {
        project: project.to_string(),
        recipients,
    })
}

async fn send(
    client: &Client,
    project: &str,
    req: &MessageRequest,
) -> Result<MessageSent, ActionError> {
    let text = req.text.trim();
    if text.is_empty() {
        return Err(ActionError::MissingText("message"));
    }
    if text.chars().count() > MAX_TEXT {
        return Err(ActionError::Invalid(format!(
            "the message is longer than {MAX_TEXT} characters"
        )));
    }
    let known = recipients(client, project).await?;
    if !known.recipients.contains(&req.to) {
        return Err(ActionError::Invalid(format!(
            "'{}' is not an agent of '{project}' or the orchestrator",
            req.to
        )));
    }
    let to = if req.to == ORCHESTRATOR {
        format!("external:{ORCHESTRATOR}")
    } else {
        req.to.clone()
    };
    client
        .send(&SendRequest {
            to: Some(to),
            body: text.to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: req.task.clone().filter(|t| !t.is_empty()),
        })
        .await
        .map_err(api_error)?;
    Ok(MessageSent {
        project: project.to_string(),
        to: req.to.clone(),
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

/// `GET /api/v1/projects/{project}/recipients`.
pub async fn recipients_route(
    Path(project): Path<String>,
) -> Result<Json<Recipients>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(recipients(&client, &project).await?))
}

/// `POST /api/v1/projects/{project}/messages`.
pub async fn send_route(
    Path(project): Path<String>,
    Json(req): Json<MessageRequest>,
) -> Result<Json<MessageSent>, ActionError> {
    let client = client_for(&project).await?;
    Ok(Json(send(&client, &project, &req).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get, routing::post};
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};

    fn agent(name: &str, state: &str) -> Value {
        json!({
            "id": format!("a-{name}"), "name": name, "role": "worker", "state": state,
            "model": "m", "session_id": "s", "pid": 1, "cwd": "/c", "worktree": "/w",
            "branch": "b", "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z", "last_event_at": null, "turns": 0,
            "turn_started_at": null, "cost_usd_total": 0.0, "context_tokens": null,
            "exit": null, "created_by": "human", "held_messages": 0, "unacked_messages": 0,
        })
    }

    /// A daemon that records the bodies posted to /v1/messages.
    async fn fake() -> (Client, Arc<Mutex<Vec<Value>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        let agents = json!([agent("w1", "working"), agent("old", "stopped")]);
        let app = Router::new()
            .route("/v1/agents", get(move || async move { Json(agents) }))
            .route(
                "/v1/messages",
                post(move |Json(b): Json<Value>| async move {
                    log.lock().expect("lock").push(b);
                    Json(json!([]))
                }),
            );
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        (Client::new_with_timeout(url, None, PROBE_TIMEOUT * 5), seen)
    }

    fn req(to: &str, text: &str, task: Option<&str>) -> MessageRequest {
        MessageRequest {
            to: to.into(),
            text: text.into(),
            task: task.map(String::from),
        }
    }

    #[tokio::test]
    async fn recipients_are_running_agents_and_the_orchestrator() {
        let (c, _) = fake().await;
        let r = recipients(&c, "p").await.expect("recipients");
        assert_eq!(r.recipients, ["w1", "orchestrator"]);
    }

    #[tokio::test]
    async fn send_to_an_agent_threads_on_the_task() {
        let (c, seen) = fake().await;
        send(&c, "p", &req("w1", " hello ", Some("br-1")))
            .await
            .expect("sent");
        let b = seen.lock().expect("lock")[0].clone();
        assert_eq!(
            (b["to"].as_str(), b["body"].as_str()),
            (Some("w1"), Some("hello"))
        );
        assert_eq!(b["task"], "br-1");
    }

    #[tokio::test]
    async fn send_to_the_orchestrator_uses_its_principal() {
        let (c, seen) = fake().await;
        send(&c, "p", &req("orchestrator", "hi", None))
            .await
            .expect("sent");
        assert_eq!(seen.lock().expect("lock")[0]["to"], "external:orchestrator");
    }

    #[tokio::test]
    async fn unknown_or_stopped_recipient_is_refused() {
        let (c, seen) = fake().await;
        for to in ["nobody", "old", "human", "external:advisor"] {
            let e = send(&c, "p", &req(to, "hi", None)).await.expect_err(to);
            assert!(matches!(e, ActionError::Invalid(_)), "{to}: {e}");
        }
        assert!(seen.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn empty_or_long_text_is_refused() {
        let (c, seen) = fake().await;
        let e = send(&c, "p", &req("w1", "  \n", None))
            .await
            .expect_err("empty");
        assert!(matches!(e, ActionError::MissingText(_)));
        let long = "x".repeat(MAX_TEXT + 1);
        let e = send(&c, "p", &req("w1", &long, None))
            .await
            .expect_err("long");
        assert!(matches!(e, ActionError::Invalid(_)));
        assert!(seen.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn unknown_project_is_404() {
        use axum::response::IntoResponse;
        let e = send_route(
            Path("no-such-project-rk7k".into()),
            Json(req("w1", "hi", None)),
        )
        .await
        .expect_err("unknown");
        assert_eq!(
            e.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }
}
