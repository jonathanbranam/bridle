//! The bridle gateway: one versioned HTTP API (`/api/v1`) for the human's web UI.
//! See docs/design/human-web-ui.md. It never touches a daemon's start-up path.

pub mod auth;
pub mod config;
pub mod discovery;
pub mod types;

use axum::{
    Json, Router, middleware,
    routing::{get, post},
};
use serde_json::{Value, json};
use tokio::net::TcpListener;

pub use config::{ConfigError, GatewayConfig, Login};

/// Every route lives under this prefix from the start.
pub const API_PREFIX: &str = "/api/v1";

/// The API version the generated types describe; `bridle-ui` records the one it was built
/// against and the gateway compares them (task 8). Bump with `API_PREFIX`.
pub const API_VERSION: u32 = 1;

/// Only health and login are open; everything else, even a path that doesn't exist yet, needs
/// a session. With no `login` nobody can get one, so only health answers.
pub fn router(login: Option<Login>) -> Router {
    let auth = auth::Auth::new(login);
    let protected = Router::new()
        .route("/session", get(auth::session))
        .route("/projects", get(projects))
        .fallback(|| async { axum::http::StatusCode::NOT_FOUND })
        .layer(middleware::from_fn_with_state(
            auth.clone(),
            auth::require_session,
        ));
    let v1 = Router::new()
        .route("/health", get(health))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .merge(protected)
        .with_state(auth);
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

pub async fn serve(listener: TcpListener, login: Option<Login>) -> std::io::Result<()> {
    if let Ok(addr) = listener.local_addr() {
        tracing::info!(%addr, "gateway listening");
    }
    axum::serve(listener, router(login)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_answers_under_api_v1() {
        let config = GatewayConfig {
            bind: "127.0.0.1:0".parse().expect("addr"),
            login: None,
        };
        let listener = bind(&config).await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(serve(listener, None));
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

    async fn start(login: Option<Login>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(serve(listener, login));
        format!("http://{addr}{API_PREFIX}")
    }

    fn login() -> Login {
        Login {
            username: "jo".into(),
            password_hash: auth::hash_password("right").expect("hash"),
        }
    }

    async fn post_login(base: &str, user: &str, password: &str) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{base}/login"))
            .json(&json!({ "username": user, "password": password }))
            .send()
            .await
            .expect("post")
    }

    fn cookie_pair(r: &reqwest::Response) -> String {
        let c = r.headers()[reqwest::header::SET_COOKIE]
            .to_str()
            .expect("str");
        c.split(';').next().expect("pair").to_string()
    }

    async fn session_status(base: &str, cookie: Option<&str>) -> u16 {
        let mut req = reqwest::Client::new().get(format!("{base}/session"));
        if let Some(c) = cookie {
            req = req.header(reqwest::header::COOKIE, c);
        }
        req.send().await.expect("get").status().as_u16()
    }

    #[tokio::test]
    async fn no_cookie_is_refused_but_health_is_open() {
        let base = start(Some(login())).await;
        assert_eq!(session_status(&base, None).await, 401);
        let nowhere = reqwest::get(format!("{base}/nowhere")).await.expect("get");
        assert_eq!(nowhere.status(), 401);
        let health = reqwest::get(format!("{base}/health")).await.expect("get");
        assert_eq!(health.status(), 200);
        assert_eq!(
            session_status(&base, Some("bridle_session=forged")).await,
            401
        );
    }

    #[tokio::test]
    async fn wrong_password_or_username_is_refused() {
        let base = start(Some(login())).await;
        let r = post_login(&base, "jo", "wrong").await;
        assert_eq!(r.status(), 401);
        assert!(r.headers().get(reqwest::header::SET_COOKIE).is_none());
        assert_eq!(post_login(&base, "eve", "right").await.status(), 401);
    }

    #[tokio::test]
    async fn right_password_sets_a_strict_httponly_cookie_that_works() {
        let base = start(Some(login())).await;
        let r = post_login(&base, "jo", "right").await;
        assert_eq!(r.status(), 200);
        let set = r.headers()[reqwest::header::SET_COOKIE]
            .to_str()
            .expect("str")
            .to_string();
        assert!(set.contains("HttpOnly"), "{set}");
        assert!(set.contains("SameSite=Strict"), "{set}");
        assert_eq!(session_status(&base, Some(&cookie_pair(&r))).await, 200);
    }

    #[tokio::test]
    async fn logout_invalidates_the_session() {
        let base = start(Some(login())).await;
        let cookie = cookie_pair(&post_login(&base, "jo", "right").await);
        let r = reqwest::Client::new()
            .post(format!("{base}/logout"))
            .header(reqwest::header::COOKIE, &cookie)
            .send()
            .await
            .expect("post");
        assert_eq!(r.status(), 204);
        assert_eq!(session_status(&base, Some(&cookie)).await, 401);
    }

    #[tokio::test]
    async fn no_login_configured_serves_only_health() {
        let base = start(None).await;
        assert_eq!(post_login(&base, "jo", "right").await.status(), 503);
        assert_eq!(session_status(&base, None).await, 401);
        let health = reqwest::get(format!("{base}/health")).await.expect("get");
        assert_eq!(health.status(), 200);
    }
}
