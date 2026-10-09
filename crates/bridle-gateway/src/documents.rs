//! The UI's document view (ticket x8jt, a narrow piece of v8kn): routes over `bridle-docs`, which
//! holds the reading, searching and the guarded write (br-5e4k). A write puts the document under
//! review on the project's daemon. See docs/design/human-web-ui.md section 3.

use std::path::PathBuf;

use axum::Json;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use bridle_api::client::{Client, ClientError};
use bridle_api::types::ReviewAddRequest;
pub use bridle_docs::documents::*;

use crate::actions::resolve as resolve_daemon;
use crate::actions::{ActionError, resolve_target};
use crate::discovery::{PROBE_TIMEOUT, Target, current_targets};

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

/// Where a project's documents are: its working tree on this machine, or its daemon elsewhere
/// (ui-9hq8 B3). The routes keep one shape for both.
pub(crate) enum DocTarget {
    Local(PathBuf),
    Remote(Remote),
}

/// A project's daemon on another machine, reached with the human token for that machine.
pub(crate) struct Remote {
    machine: String,
    client: Client,
}

/// `GET /api/v1/projects/{project}/documents/{*path}`.
pub async fn read_route(
    Path((project, path)): Path<(String, String)>,
) -> Result<Json<Document>, ApiDocError> {
    let doc = match target_of(&project).await? {
        DocTarget::Local(repo) => blocking(move || read_document(&repo, &project, &path)).await?,
        DocTarget::Remote(r) => r.read(&project, &path).await?,
    };
    Ok(Json(doc))
}

/// `PUT /api/v1/projects/{project}/documents/{*path}`.
pub async fn write_route(
    Path((project, path)): Path<(String, String)>,
    Json(req): Json<DocumentWrite>,
) -> Result<Json<DocumentSaved>, ApiDocError> {
    let (saved, client) = match target_of(&project).await? {
        DocTarget::Local(repo) => {
            let (p, rel) = (project.clone(), path.clone());
            let saved = blocking(move || write_document(&repo, &p, &rel, &req)).await?;
            let client = resolve_daemon(&project)
                .await
                .map(|(url, token)| Client::new_with_timeout(url, Some(token), PROBE_TIMEOUT * 5));
            (saved, client.map_err(|e| e.to_string()))
        }
        DocTarget::Remote(r) => {
            let saved = r.write(&project, &path, &req).await?;
            (saved, Ok(r.client))
        }
    };
    // A comment saved from the UI puts the document under review (jrm2). The save is already
    // committed, so a daemon that can't be reached is logged, not an error.
    if let Err(e) = add_to_review(client, &path).await {
        tracing::warn!("{project}: not added to review: {path}: {e}");
    }
    Ok(Json(saved))
}

/// Asks the project's daemon to put the document under review if it has a pending thread.
async fn add_to_review(client: Result<Client, String>, path: &str) -> Result<(), String> {
    client?
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
    let paths = match target_of(&project).await? {
        DocTarget::Local(repo) => blocking(move || Ok(search_documents(&repo, &query.q))).await?,
        DocTarget::Remote(r) => r.search(&query.q).await?,
    };
    Ok(Json(DocumentMatches { project, paths }))
}

/// `POST /api/v1/projects/{project}/links/resolve`: which of the UI's link targets are documents.
pub async fn resolve_links_route(
    Path(project): Path<String>,
    Json(req): Json<LinkResolveRequest>,
) -> Result<Json<ResolvedLinks>, ApiDocError> {
    let links = match target_of(&project).await? {
        DocTarget::Local(repo) => blocking(move || Ok(resolve_links(&repo, &req.targets))).await?,
        DocTarget::Remote(r) => r.resolve_links(&req.targets).await?,
    };
    Ok(Json(ResolvedLinks { project, links }))
}

pub(crate) async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, DocError> + Send + 'static,
) -> Result<T, DocError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| DocError::Internal(e.to_string()))?
}

/// Where the project's documents are, from this machine's registry and machine config.
pub(crate) async fn target_of(project: &str) -> Result<DocTarget, DocError> {
    let t = current_targets()
        .await
        .into_iter()
        .find(|t| t.project == project)
        .ok_or_else(|| DocError::UnknownProject(project.to_string()))?;
    // Reads the token file: blocking work.
    tokio::task::spawn_blocking(move || doc_target(t, crate::actions::human_token))
        .await
        .map_err(|e| DocError::Internal(e.to_string()))?
}

/// The injectable core of `target_of`: `lookup` finds the human token for a remote daemon.
pub(crate) fn doc_target(
    t: Target,
    lookup: impl Fn(Option<&str>, &str, Option<&str>) -> Result<Option<String>, String>,
) -> Result<DocTarget, DocError> {
    if let Some(problem) = t.problem {
        return Err(DocError::NotAvailable(problem));
    }
    if let Some(repo) = t.repo {
        return Ok(DocTarget::Local(PathBuf::from(repo)));
    }
    let machine = t
        .machine
        .clone()
        .unwrap_or_else(|| "another machine".into());
    let (url, token) = resolve_target(t, lookup).map_err(|e| match e {
        ActionError::UnknownProject(p) => DocError::UnknownProject(p),
        other => DocError::NotAvailable(other.to_string()),
    })?;
    Ok(DocTarget::Remote(Remote {
        machine,
        client: Client::new_with_timeout(url, Some(token), PROBE_TIMEOUT * 5),
    }))
}

impl Remote {
    /// A daemon refusal keeps its meaning; one that can't be reached names the machine.
    fn err(&self, e: ClientError) -> DocError {
        match e {
            ClientError::Api {
                status, message, ..
            } => match status {
                400 => DocError::BadPath(message),
                403 => DocError::DetachedHead,
                404 => DocError::NotFound(message),
                409 => DocError::Stale,
                415 => DocError::NotText(message),
                _ => DocError::Internal(message),
            },
            ClientError::Unreachable(m) => {
                DocError::NotAvailable(format!("{} is not answering: {m}", self.machine))
            }
            other => DocError::Internal(other.to_string()),
        }
    }

    async fn read(&self, project: &str, path: &str) -> Result<Document, DocError> {
        let mut doc = self
            .client
            .read_document(path)
            .await
            .map_err(|e| self.err(e))?;
        doc.project = project.to_string();
        Ok(doc)
    }

    async fn write(
        &self,
        project: &str,
        path: &str,
        req: &DocumentWrite,
    ) -> Result<DocumentSaved, DocError> {
        let mut saved = self
            .client
            .write_document(path, req)
            .await
            .map_err(|e| self.err(e))?;
        saved.project = project.to_string();
        Ok(saved)
    }

    async fn search(&self, q: &str) -> Result<Vec<String>, DocError> {
        let m = self
            .client
            .search_documents(q)
            .await
            .map_err(|e| self.err(e))?;
        Ok(m.paths)
    }

    async fn resolve_links(&self, targets: &[String]) -> Result<Vec<ResolvedLink>, DocError> {
        let r = self
            .client
            .resolve_links(targets)
            .await
            .map_err(|e| self.err(e))?;
        Ok(r.links)
    }

    pub(crate) async fn specs(
        &self,
        project: &str,
    ) -> Result<bridle_docs::specs::ProjectSpecs, DocError> {
        let mut s = self.client.specs().await.map_err(|e| self.err(e))?;
        s.project = project.to_string();
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::extract::{Json as J, State};
    use axum::http::HeaderMap;
    use axum::routing::{get, post};
    use std::sync::{Arc, Mutex};

    type Seen = Arc<Mutex<Vec<(String, Option<String>)>>>;

    fn note(seen: &Seen, path: String, h: &HeaderMap) {
        let auth = h
            .get("authorization")
            .map(|v| v.to_str().expect("str").to_string());
        seen.lock().expect("lock").push((path, auth));
    }

    /// A fake NUC daemon: `docs/x.md` reads, `bin.png` answers 415, anything else 404; a write
    /// with hash "old" answers 409.
    async fn fake() -> (String, Seen) {
        let seen: Seen = Arc::default();
        let app = Router::new()
            .route(
                "/v1/documents",
                get(|State(s): State<Seen>, h: HeaderMap| async move {
                    note(&s, "search".into(), &h);
                    J(json!({"project": "nuc-name", "paths": ["docs/x.md"]}))
                }),
            )
            .route(
                "/v1/documents/{*path}",
                get(
                    |State(s): State<Seen>, h: HeaderMap, Path(p): Path<String>| async move {
                        note(&s, format!("read {p}"), &h);
                        match p.as_str() {
                            "docs/x.md" => J(json!({"project": "nuc-name", "path": p,
                                "content": "hi", "hash": "h1", "branch": "main"}))
                            .into_response(),
                            "bin.png" => (
                                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                                J(json!({"code": "not_text", "message": "bin.png"})),
                            )
                                .into_response(),
                            _ => (
                                StatusCode::NOT_FOUND,
                                J(json!({"code": "not_found", "message": p})),
                            )
                                .into_response(),
                        }
                    },
                )
                .put(
                    |State(s): State<Seen>,
                     h: HeaderMap,
                     Path(p): Path<String>,
                     J(req): J<DocumentWrite>| async move {
                        note(&s, format!("write {p}"), &h);
                        if req.hash == "old" {
                            return (
                                StatusCode::CONFLICT,
                                J(json!({"code": "stale", "message": "stale"})),
                            )
                                .into_response();
                        }
                        J(json!({"project": "nuc-name", "path": p, "hash": "h2",
                            "branch": "main", "commit": "abc"}))
                        .into_response()
                    },
                ),
            )
            .route(
                "/v1/links/resolve",
                post(|State(s): State<Seen>, h: HeaderMap| async move {
                    note(&s, "links".into(), &h);
                    J(json!({"project": "nuc-name",
                        "links": [{"target": "a", "path": "docs/a.md"}]}))
                }),
            )
            .route(
                "/v1/review/add",
                post(|State(s): State<Seen>, h: HeaderMap| async move {
                    note(&s, "review".into(), &h);
                    J(json!({"path": "docs/x.md", "under_review": true}))
                }),
            )
            .with_state(seen.clone());
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        (url, seen)
    }

    fn remote_target(url: &str) -> Target {
        Target {
            project: "far".into(),
            machine: Some("nuc".into()),
            url: Some(url.into()),
            workspace: None,
            repo: None,
            problem: None,
        }
    }

    fn remote(url: &str) -> Remote {
        let got = doc_target(remote_target(url), |_, _, m| {
            assert_eq!(m, Some("nuc"));
            Ok(Some("nuc-human".into()))
        });
        match got {
            Ok(DocTarget::Remote(r)) => r,
            _ => panic!("expected a remote target"),
        }
    }

    #[tokio::test]
    async fn remote_read_search_and_links_go_to_the_daemon_with_the_machines_token() {
        let (url, seen) = fake().await;
        let r = remote(&url);
        let doc = r.read("far", "docs/x.md").await.expect("read");
        // The project is the gateway's name for it, not the daemon's.
        assert_eq!((doc.project.as_str(), doc.content.as_str()), ("far", "hi"));
        assert_eq!(r.search("x").await.expect("search"), ["docs/x.md"]);
        let links = r.resolve_links(&["a".into()]).await.expect("links");
        assert_eq!(links[0].path.as_deref(), Some("docs/a.md"));
        let seen = seen.lock().expect("lock");
        assert_eq!(seen.len(), 3);
        assert!(
            seen.iter()
                .all(|s| s.1.as_deref() == Some("Bearer nuc-human"))
        );
    }

    #[tokio::test]
    async fn remote_refusals_keep_their_meaning() {
        let (url, _) = fake().await;
        let r = remote(&url);
        assert!(matches!(
            r.read("far", "nope.md").await,
            Err(DocError::NotFound(_))
        ));
        assert!(matches!(
            r.read("far", "bin.png").await,
            Err(DocError::NotText(_))
        ));
        let stale = DocumentWrite {
            content: "c".into(),
            hash: "old".into(),
        };
        assert!(matches!(
            r.write("far", "docs/x.md", &stale).await,
            Err(DocError::Stale)
        ));
    }

    #[tokio::test]
    async fn remote_write_saves_then_the_review_call_reaches_the_same_daemon() {
        let (url, seen) = fake().await;
        let r = remote(&url);
        let req = DocumentWrite {
            content: "c".into(),
            hash: "h1".into(),
        };
        let saved = r.write("far", "docs/x.md", &req).await.expect("write");
        assert_eq!((saved.project.as_str(), saved.hash.as_str()), ("far", "h2"));
        add_to_review(Ok(r.client.clone()), "docs/x.md")
            .await
            .expect("review");
        let seen = seen.lock().expect("lock");
        let paths: Vec<_> = seen.iter().map(|s| s.0.as_str()).collect();
        assert_eq!(paths, ["write docs/x.md", "review"]);
    }

    #[tokio::test]
    async fn an_unreachable_machine_is_named_and_503() {
        let r = remote("http://127.0.0.1:1");
        let err = r.read("far", "docs/x.md").await.expect_err("down");
        assert_eq!(err.status(), 503);
        assert!(err.to_string().contains("nuc"), "{err}");
    }

    #[test]
    fn a_local_project_is_read_from_its_working_tree() {
        let mut t = remote_target("http://x");
        t.repo = Some("/work/far".into());
        let got = doc_target(t, |_, _, _| panic!("no token needed")).ok();
        assert!(matches!(got, Some(DocTarget::Local(p)) if p.as_os_str() == "/work/far"));
    }

    #[test]
    fn a_machine_without_a_token_is_503_by_name() {
        let err = doc_target(remote_target("http://x"), |_, _, m| {
            Err(format!("put the token under [human.{}]", m.unwrap_or("?")))
        })
        .err()
        .expect("no token");
        assert_eq!(err.status(), 503);
        assert!(err.to_string().contains("[human.nuc]"), "{err}");
    }
}
