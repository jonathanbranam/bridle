//! The UI's document view (ticket x8jt, a narrow piece of v8kn): routes over `bridle-docs`, which
//! holds the reading, searching and the guarded write (br-5e4k). A write puts the document under
//! review on the project's daemon. See docs/design/human-web-ui.md section 3.

use std::path::PathBuf;

use axum::Json;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use bridle_api::client::Client;
use bridle_api::types::ReviewAddRequest;
pub use bridle_docs::documents::*;

use crate::actions::resolve as resolve_daemon;
use crate::discovery::{PROBE_TIMEOUT, current_targets};

/// A [`DocError`] as the gateway's HTTP error: its status and `{"error": text}`.
#[derive(Debug)]
pub struct ApiDocError(pub DocError);

impl From<DocError> for ApiDocError {
    fn from(e: DocError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiDocError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}

/// `GET /api/v1/projects/{project}/documents/{*path}`.
pub async fn read_route(
    Path((project, path)): Path<(String, String)>,
) -> Result<Json<Document>, ApiDocError> {
    let repo = repo_of(&project).await?;
    let doc = blocking(move || read_document(&repo, &project, &path)).await?;
    Ok(Json(doc))
}

/// `PUT /api/v1/projects/{project}/documents/{*path}`.
pub async fn write_route(
    Path((project, path)): Path<(String, String)>,
    Json(req): Json<DocumentWrite>,
) -> Result<Json<DocumentSaved>, ApiDocError> {
    let repo = repo_of(&project).await?;
    let (p, rel) = (project.clone(), path.clone());
    let saved = blocking(move || write_document(&repo, &p, &rel, &req)).await?;
    // A comment saved from the UI puts the document under review (jrm2). The save is already
    // committed, so a daemon that can't be reached is logged, not an error.
    if let Err(e) = add_to_review(&project, &path).await {
        tracing::warn!("{project}: not added to review: {path}: {e}");
    }
    Ok(Json(saved))
}

/// Asks the project's daemon to put the document under review if it has a pending thread.
async fn add_to_review(project: &str, path: &str) -> Result<(), String> {
    let (url, token) = resolve_daemon(project).await.map_err(|e| e.to_string())?;
    Client::new_with_timeout(url, Some(token), PROBE_TIMEOUT * 5)
        .review_add(&ReviewAddRequest {
            path: path.to_string(),
            only_if_pending: true,
        })
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// `GET /api/v1/projects/{project}/documents?q=`: the document picker's search. Matches the
/// markdown under `docs/`; open tickets come first, and a bare ticket ID finds its ticket.
pub async fn search_route(
    Path(project): Path<String>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<DocumentMatches>, ApiDocError> {
    let repo = repo_of(&project).await?;
    let paths = blocking(move || Ok(search_documents(&repo, &query.q))).await?;
    Ok(Json(DocumentMatches { project, paths }))
}

/// `POST /api/v1/projects/{project}/links/resolve`: which of the UI's link targets are documents.
pub async fn resolve_links_route(
    Path(project): Path<String>,
    Json(req): Json<LinkResolveRequest>,
) -> Result<Json<ResolvedLinks>, ApiDocError> {
    let repo = repo_of(&project).await?;
    let links = blocking(move || Ok(resolve_links(&repo, &req.targets))).await?;
    Ok(Json(ResolvedLinks { project, links }))
}

pub(crate) async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, DocError> + Send + 'static,
) -> Result<T, DocError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| DocError::Internal(e.to_string()))?
}

/// The working tree of one of this machine's projects.
pub(crate) async fn repo_of(project: &str) -> Result<PathBuf, DocError> {
    let t = current_targets()
        .await
        .into_iter()
        .find(|t| t.project == project)
        .ok_or_else(|| DocError::UnknownProject(project.to_string()))?;
    if let Some(problem) = t.problem {
        return Err(DocError::NotAvailable(problem));
    }
    t.repo.map(PathBuf::from).ok_or_else(|| {
        DocError::NotAvailable(format!(
            "project '{project}' is on another machine: remote documents aren't supported yet"
        ))
    })
}
