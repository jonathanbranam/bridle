//! The UI's read-only spec view: a route over `bridle-docs` (br-5e4k).

use axum::Json;
use axum::extract::Path;

use crate::documents::{ApiDocError, blocking, repo_of};
pub use bridle_docs::specs::*;

/// `GET /api/v1/projects/{project}/specs`.
pub async fn specs_route(Path(project): Path<String>) -> Result<Json<ProjectSpecs>, ApiDocError> {
    let repo = repo_of(&project).await?;
    let specs = blocking(move || Ok(load(&repo))).await?;
    Ok(Json(ProjectSpecs { project, specs }))
}
