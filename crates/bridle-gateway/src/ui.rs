//! Serves the UI folder (`~/.bridle/ui/` by default) at `/` and checks the API version its
//! build recorded in `api-version`. See docs/design/human-web-ui.md.
//!
//! The files are open, not behind the session: the page has to load to show its login form,
//! and it holds no data. Every API route is still guarded.

use std::path::{Component, Path, PathBuf};

use axum::{
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use ts_rs::TS;

use crate::API_VERSION;

/// What to do when the folder's build targets another API version.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OnMismatch {
    #[default]
    Warn,
    Refuse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiConfig {
    pub dir: PathBuf,
    pub on_mismatch: OnMismatch,
}

impl UiConfig {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            on_mismatch: OnMismatch::Warn,
        }
    }
}

/// The UI folder's state as `/api/v1/health` reports it.
#[derive(Debug, Clone, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UiStatus {
    Missing,
    /// The folder has no readable `api-version`, so nothing can be compared.
    Unversioned,
    Ok,
    Mismatch,
}

#[derive(Debug, Clone, Serialize, TS)]
pub struct UiHealth {
    pub status: UiStatus,
    pub expected_api_version: u32,
    pub api_version: Option<u32>,
}

pub fn check(config: &UiConfig) -> UiHealth {
    let recorded = std::fs::read_to_string(config.dir.join("api-version"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok());
    let status = if !config.dir.is_dir() {
        UiStatus::Missing
    } else {
        match recorded {
            None => UiStatus::Unversioned,
            Some(v) if v == API_VERSION => UiStatus::Ok,
            Some(_) => UiStatus::Mismatch,
        }
    };
    UiHealth {
        status,
        expected_api_version: API_VERSION,
        api_version: recorded,
    }
}

/// Logs once at start-up what [`check`] found.
pub fn log_status(config: &UiConfig) {
    let h = check(config);
    match h.status {
        UiStatus::Ok => tracing::info!(dir = %config.dir.display(), "serving the UI"),
        UiStatus::Missing => {
            tracing::warn!(dir = %config.dir.display(), "UI folder not found; / explains")
        }
        UiStatus::Unversioned => {
            tracing::warn!(dir = %config.dir.display(), "UI folder has no readable api-version file")
        }
        UiStatus::Mismatch => tracing::warn!(
            built_for = ?h.api_version,
            expected = API_VERSION,
            refuse = config.on_mismatch == OnMismatch::Refuse,
            "UI was built for another API version"
        ),
    }
}

/// The router's fallback: any path no API route claimed.
pub async fn serve_ui(config: UiConfig, uri: Uri) -> Response {
    let health = check(&config);
    match health.status {
        UiStatus::Missing => {
            return notice(
                StatusCode::NOT_FOUND,
                &format!(
                    "No UI installed: {} does not exist. The API at /api/v1 still works.",
                    config.dir.display()
                ),
            );
        }
        UiStatus::Mismatch if config.on_mismatch == OnMismatch::Refuse => {
            return notice(
                StatusCode::SERVICE_UNAVAILABLE,
                &format!(
                    "The UI in {} was built for API version {}, but this gateway serves {}. \
                     Install a matching UI build.",
                    config.dir.display(),
                    health.api_version.map_or("?".into(), |v| v.to_string()),
                    API_VERSION
                ),
            );
        }
        _ => {}
    }
    let Some(rel) = safe_relative(uri.path()) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let wanted = config.dir.join(&rel);
    // A path with an extension is a file the page asked for; anything else is a client-side
    // route, so it gets the index.
    let (path, body) = match tokio::fs::read(&wanted).await {
        Ok(b) if wanted.is_file() => (wanted, b),
        _ if rel.extension().is_some() => return StatusCode::NOT_FOUND.into_response(),
        _ => {
            let index = config.dir.join("index.html");
            match tokio::fs::read(&index).await {
                Ok(b) => (index, b),
                Err(_) => return StatusCode::NOT_FOUND.into_response(),
            }
        }
    };
    ([(header::CONTENT_TYPE, content_type(&path))], body).into_response()
}

fn notice(status: StatusCode, text: &str) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        format!("{text}\n"),
    )
        .into_response()
}

/// The request path as a relative path of plain names only; `None` for `..` or anything else
/// that could leave the folder (percent-encoded dots included, as they're decoded first).
fn safe_relative(request_path: &str) -> Option<PathBuf> {
    let decoded = percent_decode(request_path)?;
    let mut out = PathBuf::new();
    for c in Path::new(decoded.trim_start_matches('/')).components() {
        match c {
            Component::Normal(n) => out.push(n),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

fn percent_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hex = std::str::from_utf8(b.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}
