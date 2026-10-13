//! Login and sessions (docs/design/human-web-ui.md section 3): one user, an argon2 hash in the
//! machine config, sessions behind an `HttpOnly`, `SameSite=Strict` cookie. Sessions last 30
//! days, sliding, and are kept on disk (hashed) so a gateway restart or a reboot keeps the login.

use std::collections::HashMap;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash};
use axum::Json;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use ts_rs::TS;

use crate::config::Login;

pub const COOKIE_NAME: &str = "bridle_session";
const SESSION_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);
/// A used session slides its expiry in memory at once, but the file is rewritten at most this
/// often, so reads don't hammer the disk. A restart can lose up to this much of the slide.
const SAVE_EVERY_SECS: u64 = 60 * 60;
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
    sessions: Arc<Mutex<Sessions>>,
}

#[derive(Serialize, Deserialize)]
struct Session {
    username: String,
    /// Unix seconds.
    expires: u64,
}

/// Sessions keyed by the sha256 of the cookie value, so a leaked file can't be replayed.
struct Sessions {
    map: HashMap<String, Session>,
    path: Option<PathBuf>,
    last_saved: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn hash_id(id: &str) -> String {
    Sha256::digest(id.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

impl Sessions {
    /// A missing, empty or corrupt file means no sessions; expired ones are dropped.
    fn load(path: Option<PathBuf>, now: u64) -> Self {
        let mut map: HashMap<String, Session> = path
            .as_deref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        map.retain(|_, s| s.expires > now);
        Self {
            map,
            path,
            last_saved: now,
        }
    }

    fn save(&mut self, now: u64) {
        self.last_saved = now;
        let Some(path) = &self.path else { return };
        if let Err(e) = write_private(path, &self.map) {
            tracing::warn!(path = %path.display(), error = %e, "could not save the sessions");
        }
    }

    fn insert(&mut self, id: &str, username: String, now: u64) {
        let expires = now + SESSION_TTL.as_secs();
        self.map.insert(hash_id(id), Session { username, expires });
        self.save(now);
    }

    fn remove(&mut self, id: &str, now: u64) {
        if self.map.remove(&hash_id(id)).is_some() {
            self.save(now);
        }
    }

    /// The session's user, sliding its expiry.
    fn touch(&mut self, id: &str, now: u64) -> Option<String> {
        self.map.retain(|_, s| s.expires > now);
        let found = self.map.get_mut(&hash_id(id)).map(|s| {
            s.expires = now + SESSION_TTL.as_secs();
            s.username.clone()
        });
        if found.is_some() && now.saturating_sub(self.last_saved) >= SAVE_EVERY_SECS {
            self.save(now);
        }
        found
    }
}

/// Atomic (temp file, then rename) and mode 0600.
fn write_private(path: &Path, map: &HashMap<String, Session>) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&tmp)?;
    f.write_all(&serde_json::to_vec(map).map_err(std::io::Error::other)?)?;
    f.sync_all()?;
    std::fs::rename(&tmp, path)
}

impl Auth {
    /// `sessions_path`: where sessions persist; `None` keeps them in memory only.
    pub fn new(login: Option<Login>, sessions_path: Option<PathBuf>) -> Self {
        Self {
            login: login.map(Arc::new),
            sessions: Arc::new(Mutex::new(Sessions::load(sessions_path, now_secs()))),
        }
    }

    fn session_user(&self, headers: &HeaderMap) -> Option<String> {
        let id = session_id(headers)?;
        self.sessions
            .lock()
            .expect("sessions lock")
            .touch(&id, now_secs())
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

#[derive(Deserialize, TS)]
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
    auth.sessions
        .lock()
        .expect("sessions lock")
        .insert(&id, login.username.clone(), now_secs());
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
        auth.sessions
            .lock()
            .expect("sessions lock")
            .remove(&id, now_secs());
    }
    let cookie = format!("{COOKIE_NAME}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0");
    ([(header::SET_COOKIE, cookie)], StatusCode::NO_CONTENT).into_response()
}

#[derive(Serialize, TS)]
pub struct SessionInfo {
    pub username: String,
}

/// Who the session belongs to; the UI's "am I logged in" probe.
pub async fn session(State(auth): State<Auth>, headers: HeaderMap) -> Response {
    match auth.session_user(&headers) {
        Some(username) => Json(SessionInfo { username }).into_response(),
        None => unauthorized(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u64 = 24 * 60 * 60;

    fn store() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("gateway-sessions.json");
        (dir, path)
    }

    #[test]
    fn a_session_survives_a_restart_and_is_stored_hashed_and_private() {
        let (_d, path) = store();
        let mut a = Sessions::load(Some(path.clone()), 1000);
        a.insert("secret-id", "jo".into(), 1000);
        let text = std::fs::read_to_string(&path).expect("file");
        assert!(!text.contains("secret-id"));
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).expect("meta").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        let mut b = Sessions::load(Some(path), 2000);
        assert_eq!(b.touch("secret-id", 2000).as_deref(), Some("jo"));
        assert_eq!(b.touch("other", 2000), None);
    }

    #[test]
    fn a_session_expires_after_30_days_and_slides_on_use() {
        let (_d, path) = store();
        let mut s = Sessions::load(Some(path.clone()), 0);
        s.insert("id", "jo".into(), 0);
        assert!(s.touch("id", 29 * DAY).is_some());
        // The use at day 29 slid the expiry, so day 58 is still inside it.
        assert!(s.touch("id", 58 * DAY).is_some());
        // The slide was written (more than an hour after the last save), so a restart sees it.
        let mut r = Sessions::load(Some(path), 58 * DAY);
        assert!(r.touch("id", 58 * DAY + 1).is_some());
        assert!(s.touch("id", 58 * DAY + 31 * DAY).is_none());
    }

    #[test]
    fn expired_sessions_are_dropped_on_load() {
        let (_d, path) = store();
        let mut s = Sessions::load(Some(path.clone()), 0);
        s.insert("id", "jo".into(), 0);
        let r = Sessions::load(Some(path), 31 * DAY);
        assert!(r.map.is_empty());
    }

    #[test]
    fn logout_removes_the_session_from_disk() {
        let (_d, path) = store();
        let mut s = Sessions::load(Some(path.clone()), 0);
        s.insert("id", "jo".into(), 0);
        s.remove("id", 10);
        assert!(s.touch("id", 11).is_none());
        let mut r = Sessions::load(Some(path), 20);
        assert!(r.touch("id", 20).is_none());
    }

    #[test]
    fn a_corrupt_or_empty_store_means_no_sessions() {
        let (_d, path) = store();
        for content in ["", "not json {", "[1,2]"] {
            std::fs::write(&path, content).expect("write");
            assert!(Sessions::load(Some(path.clone()), 0).map.is_empty());
        }
        assert!(
            Sessions::load(Some(path.with_extension("none")), 0)
                .map
                .is_empty()
        );
    }
}
