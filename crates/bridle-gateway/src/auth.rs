//! Login and sessions (docs/design/human-web-ui.md section 3): one user, an argon2 hash in the
//! machine config, in-memory sessions behind an `HttpOnly`, `SameSite=Strict` cookie.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash};
use axum::Json;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;

use crate::config::Login;

pub const COOKIE_NAME: &str = "bridle_session";
const SESSION_TTL: Duration = Duration::from_secs(12 * 60 * 60);
/// The only brake on guessing: a failed login costs the caller this long.
const FAILURE_DELAY: Duration = Duration::from_millis(500);

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())?
        .to_string())
}

pub fn is_valid_hash(hash: &str) -> bool {
    PasswordHash::new(hash).is_ok()
}

#[derive(Clone)]
pub struct Auth {
    login: Option<Arc<Login>>,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

struct Session {
    username: String,
    expires: Instant,
}

impl Auth {
    pub fn new(login: Option<Login>) -> Self {
        Self {
            login: login.map(Arc::new),
            sessions: Arc::default(),
        }
    }

    fn session_user(&self, headers: &HeaderMap) -> Option<String> {
        let id = session_id(headers)?;
        let mut sessions = self.sessions.lock().expect("sessions lock");
        let now = Instant::now();
        sessions.retain(|_, s| s.expires > now);
        sessions.get(&id).map(|s| s.username.clone())
    }
}

fn session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE_NAME)
        .map(|(_, value)| value.to_string())
}

fn new_session_id() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("the OS random source");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "login required" })),
    )
        .into_response()
}

/// Refuses every route it wraps without a live session.
pub async fn require_session(State(auth): State<Auth>, req: Request, next: Next) -> Response {
    if auth.session_user(req.headers()).is_none() {
        return unauthorized();
    }
    next.run(req).await
}

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

pub async fn login(State(auth): State<Auth>, Json(creds): Json<Credentials>) -> Response {
    let Some(login) = auth.login.clone() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "no login is configured in [gateway]" })),
        )
            .into_response();
    };
    let hash = login.password_hash.clone();
    let password = creds.password;
    // Verify even for a wrong username, so the two failures cost the same.
    let verified = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash)
            .map(|h| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &h)
                    .is_ok()
            })
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false);
    if !verified || creds.username != login.username {
        tokio::time::sleep(FAILURE_DELAY).await;
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "wrong username or password" })),
        )
            .into_response();
    }
    let id = new_session_id();
    auth.sessions.lock().expect("sessions lock").insert(
        id.clone(),
        Session {
            username: login.username.clone(),
            expires: Instant::now() + SESSION_TTL,
        },
    );
    let cookie = format!(
        "{COOKIE_NAME}={id}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        SESSION_TTL.as_secs()
    );
    (
        [(header::SET_COOKIE, cookie)],
        Json(json!({ "username": login.username })),
    )
        .into_response()
}

pub async fn logout(State(auth): State<Auth>, headers: HeaderMap) -> Response {
    if let Some(id) = session_id(&headers) {
        auth.sessions.lock().expect("sessions lock").remove(&id);
    }
    let cookie = format!("{COOKIE_NAME}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0");
    ([(header::SET_COOKIE, cookie)], StatusCode::NO_CONTENT).into_response()
}

/// Who the session belongs to; the UI's "am I logged in" probe.
pub async fn session(State(auth): State<Auth>, headers: HeaderMap) -> Response {
    match auth.session_user(&headers) {
        Some(username) => Json(json!({ "username": username })).into_response(),
        None => unauthorized(),
    }
}
