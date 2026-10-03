//! The TypeScript types for `bridle-ui`, generated from the gateway's wire types by ts-rs and
//! committed under `bindings/`. A type joins the API by deriving `TS` and being listed in
//! [`export_all`]. `just gateway-types` regenerates; the test below fails when they're stale.

use std::path::Path;

use ts_rs::{Config, TS};

use crate::auth::{Credentials, SessionInfo};
use crate::discovery::{ProjectStatus, Projects};

/// Writes every API type (and what it depends on) into `dir`.
pub fn export_all(dir: &Path) -> Result<(), ts_rs::ExportError> {
    let config = Config::default().with_out_dir(dir);
    ProjectStatus::export_all(&config)?;
    Projects::export_all(&config)?;
    Credentials::export_all(&config)?;
    SessionInfo::export_all(&config)?;
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
