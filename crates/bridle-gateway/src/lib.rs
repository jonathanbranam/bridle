//! The bridle gateway: one versioned HTTP API (`/api/v1`) for the human's web UI.
//! See docs/design/human-web-ui.md. It never touches a daemon's start-up path.

pub mod config;
pub mod discovery;

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use tokio::net::TcpListener;

pub use config::{ConfigError, GatewayConfig};

/// Every route lives under this prefix from the start.
pub const API_PREFIX: &str = "/api/v1";

pub fn router() -> Router {
    let v1 = Router::new()
        .route("/health", get(health))
        .route("/projects", get(projects));
    Router::new().nest(API_PREFIX, v1)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn projects() -> Json<discovery::Projects> {
    Json(discovery::discover().await)
}

/// Binds the configured address; split from [`serve`] so a caller (and a test) can learn the
/// real port when the config asks for port 0.
pub async fn bind(config: &GatewayConfig) -> std::io::Result<TcpListener> {
    TcpListener::bind(config.bind).await
}

pub async fn serve(listener: TcpListener) -> std::io::Result<()> {
    if let Ok(addr) = listener.local_addr() {
        tracing::info!(%addr, "gateway listening");
    }
    axum::serve(listener, router()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_answers_under_api_v1() {
        let config = GatewayConfig {
            bind: "127.0.0.1:0".parse().expect("addr"),
        };
        let listener = bind(&config).await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(serve(listener));
        let body: Value = reqwest::get(format!("http://{addr}/api/v1/health"))
            .await
            .expect("get")
            .json()
            .await
            .expect("json");
        assert_eq!(body["status"], "ok");
        let old = reqwest::get(format!("http://{addr}/health"))
            .await
            .expect("get");
        assert_eq!(old.status(), 404);
        server.abort();
    }
}
