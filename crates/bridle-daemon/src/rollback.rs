//! Rolling back a self-upgrade whose new binary won't start (docs/design/agent-host/daemon.md,
//! "Upgrade"). Before building, the running binary is copied to `.bridle/bridle.prev`; the
//! restart that follows writes a pending-upgrade marker, which the new process clears once it is
//! serving. A new process that finds the marker and fails to start, or finds it already marked
//! as started (the last attempt died before serving), puts the previous binary back and execs it.

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::paths::Workspace;

#[derive(Serialize, Deserialize)]
struct Pending {
    /// Set by the new process when it begins starting; still set on the next start means the
    /// attempt never reached serving.
    started: bool,
}

fn prev_path(ws: &Workspace) -> PathBuf {
    ws.state_dir().join("bridle.prev")
}

fn marker_path(ws: &Workspace) -> PathBuf {
    ws.state_dir().join("upgrade-pending.json")
}

fn notice_path(ws: &Workspace) -> PathBuf {
    ws.state_dir().join("upgrade-rolled-back.txt")
}

/// Copies the running binary aside, before the build replaces it.
pub fn stash_previous(ws: &Workspace, exe: &Path) -> anyhow::Result<()> {
    std::fs::copy(exe, prev_path(ws))
        .with_context(|| format!("copying {} aside", exe.display()))?;
    Ok(())
}

/// Marks the coming restart as an upgrade that may need rolling back.
pub fn arm(ws: &Workspace) -> anyhow::Result<()> {
    anyhow::ensure!(prev_path(ws).exists(), "no previous binary was kept");
    let json = serde_json::to_string(&Pending { started: false })?;
    std::fs::write(marker_path(ws), json).context("writing the pending-upgrade marker")
}

/// Called before starting. `Ok(true)` means an upgrade is pending and this is its first
/// attempt; `Err` means the last attempt never got to serving, so roll back now.
pub fn begin(ws: &Workspace) -> Result<bool, String> {
    let Some(pending) = read(ws) else {
        return Ok(false);
    };
    if pending.started {
        return Err("the upgraded binary died before it was serving".to_string());
    }
    let json = serde_json::to_string(&Pending { started: true }).map_err(|e| e.to_string())?;
    std::fs::write(marker_path(ws), json).map_err(|e| e.to_string())?;
    Ok(true)
}

/// True while an upgrade's marker is on disk.
pub fn is_pending(ws: &Workspace) -> bool {
    marker_path(ws).exists()
}

fn read(ws: &Workspace) -> Option<Pending> {
    let text = std::fs::read_to_string(marker_path(ws)).ok()?;
    // An unreadable marker counts as a failed attempt rather than a clean start.
    Some(serde_json::from_str(&text).unwrap_or(Pending { started: true }))
}

/// The new process is serving: the upgrade stands.
pub fn clear(ws: &Workspace) {
    let _ = std::fs::remove_file(marker_path(ws));
}

/// Puts the kept binary back over `exe` and removes the marker, leaving a notice for the
/// restored daemon to report. The caller then execs `exe`.
pub fn restore(ws: &Workspace, exe: &Path, reason: &str) -> anyhow::Result<()> {
    let tmp = exe.with_extension("rollback");
    std::fs::copy(prev_path(ws), &tmp).context("reading the previous binary")?;
    std::fs::rename(&tmp, exe).with_context(|| format!("restoring {}", exe.display()))?;
    let _ = std::fs::write(notice_path(ws), reason);
    clear(ws);
    Ok(())
}

/// The reason of a rollback that just happened, once.
pub fn take_notice(ws: &Workspace) -> Option<String> {
    let text = std::fs::read_to_string(notice_path(ws)).ok()?;
    let _ = std::fs::remove_file(notice_path(ws));
    Some(text)
}

/// Pre-flight for a freshly built binary: loads the config, nothing else (the database is left
/// alone so a newer schema is never applied before the restart is certain).
pub fn preflight(repo: &Path, bridle_home: Option<&Path>) -> anyhow::Result<()> {
    let config = crate::config::Config::load_with_home(repo, bridle_home)
        .context("loading .bridle/config.toml")?;
    config.workflow_root(repo)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws() -> (tempfile::TempDir, Workspace) {
        let tmp = tempfile::tempdir().expect("tmp");
        let ws = Workspace::new(tmp.path().join("repo"), Some(tmp.path().to_path_buf()));
        ws.ensure_dirs().expect("dirs");
        (tmp, ws)
    }

    #[test]
    fn no_marker_is_a_plain_start() {
        let (_t, ws) = ws();
        assert_eq!(begin(&ws), Ok(false));
    }

    #[test]
    fn first_attempt_then_serving_clears_the_marker() {
        let (t, ws) = ws();
        let exe = t.path().join("bridle");
        std::fs::write(&exe, "old").unwrap();
        stash_previous(&ws, &exe).unwrap();
        arm(&ws).unwrap();
        assert_eq!(begin(&ws), Ok(true));
        clear(&ws);
        assert_eq!(begin(&ws), Ok(false));
    }

    #[test]
    fn a_second_start_with_the_marker_still_set_rolls_back() {
        let (t, ws) = ws();
        let exe = t.path().join("bridle");
        std::fs::write(&exe, "old").unwrap();
        stash_previous(&ws, &exe).unwrap();
        std::fs::write(&exe, "new, broken").unwrap();
        arm(&ws).unwrap();
        assert_eq!(begin(&ws), Ok(true));
        assert!(begin(&ws).is_err());
        restore(&ws, &exe, "it died").unwrap();
        assert_eq!(std::fs::read_to_string(&exe).unwrap(), "old");
        assert_eq!(begin(&ws), Ok(false));
        assert_eq!(take_notice(&ws).as_deref(), Some("it died"));
        assert_eq!(take_notice(&ws), None);
    }

    #[test]
    fn arming_without_a_kept_binary_fails() {
        let (_t, ws) = ws();
        assert!(arm(&ws).is_err());
    }
}
