//! The TypeScript types for `bridle-ui`, generated from the gateway's wire types by ts-rs and
//! committed under `bindings/`. A type joins the API by deriving `TS` and being listed in
//! [`export_all`]. `just gateway-types` regenerates; the test below fails when they're stale.

use std::path::Path;

use ts_rs::{Config, TS};

use crate::actions::{ActionRequest, ActionResult, ReviewRequest, ReviewResult};
use crate::auth::{Credentials, SessionInfo};
use crate::discovery::{ProjectStatus, Projects};
use crate::documents::{
    Document, DocumentMatches, DocumentSaved, DocumentWrite, LinkResolveRequest, ResolvedLinks,
};
use crate::interactions::{DayReport, HoursReport, InteractionReport, IntervalsReport};
use crate::items::{Decision, Items, Priority, ProjectItems, Todo};
use crate::system::{AgentList, SystemView};
use crate::tasks::{ClaimAgent, TaskDetail, TaskList, TaskSummary, ThreadItem};
use crate::ui::UiHealth;

/// Writes every API type (and what it depends on) into `dir`.
pub fn export_all(dir: &Path) -> Result<(), ts_rs::ExportError> {
    let config = Config::default().with_out_dir(dir);
    ProjectStatus::export_all(&config)?;
    Projects::export_all(&config)?;
    Items::export_all(&config)?;
    ProjectItems::export_all(&config)?;
    Decision::export_all(&config)?;
    LinkResolveRequest::export_all(&config)?;
    ResolvedLinks::export_all(&config)?;
    Todo::export_all(&config)?;
    Priority::export_all(&config)?;
    Credentials::export_all(&config)?;
    SessionInfo::export_all(&config)?;
    UiHealth::export_all(&config)?;
    ActionRequest::export_all(&config)?;
    ActionResult::export_all(&config)?;
    InteractionReport::export_all(&config)?;
    DayReport::export_all(&config)?;
    HoursReport::export_all(&config)?;
    IntervalsReport::export_all(&config)?;
    Document::export_all(&config)?;
    DocumentWrite::export_all(&config)?;
    DocumentSaved::export_all(&config)?;
    DocumentMatches::export_all(&config)?;
    ReviewRequest::export_all(&config)?;
    ReviewResult::export_all(&config)?;
    TaskList::export_all(&config)?;
    TaskSummary::export_all(&config)?;
    TaskDetail::export_all(&config)?;
    ThreadItem::export_all(&config)?;
    ClaimAgent::export_all(&config)?;
    SystemView::export_all(&config)?;
    AgentList::export_all(&config)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn read_ts(dir: &Path) -> Vec<(String, String)> {
        let mut files: Vec<_> = fs::read_dir(dir)
            .expect("read dir")
            .map(|e| {
                let path = e.expect("entry").path();
                let name = path
                    .file_name()
                    .expect("name")
                    .to_string_lossy()
                    .into_owned();
                (name, fs::read_to_string(&path).expect("read"))
            })
            .collect();
        files.sort();
        files
    }

    #[test]
    fn committed_types_are_current() {
        let committed = Path::new(env!("CARGO_MANIFEST_DIR")).join("bindings");
        if std::env::var_os("BRIDLE_UPDATE_TYPES").is_some() {
            fs::remove_dir_all(&committed).ok();
            export_all(&committed).expect("export");
            return;
        }
        let fresh = tempfile::tempdir().expect("tempdir");
        export_all(fresh.path()).expect("export");
        assert!(
            read_ts(fresh.path()) == read_ts(&committed),
            "gateway TypeScript types are stale: run `just gateway-types` and commit bindings/"
        );
    }
}
