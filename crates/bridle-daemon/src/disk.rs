//! Disk usage monitor: on a timer, reads free space on the workspace volume and the
//! sizes of what bridle's work grows (the clone's `target/`, `wt/` worktrees, `.bridle/`),
//! logs it, and records it as a `disk.checked` event. Only free space under
//! `[disk] min_free_gb` reaches the human's inbox (ticket m3wq; kp3f: the inbox is for
//! things the human must act on, so a routine reading never goes there).

use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bridle_api::types::{MessageKind, When, event_kind};
use serde_json::json;

use crate::events::Emitter;
use crate::paths::Workspace;
use crate::supervisor::{AgentManager, ToTarget};

pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(60 * 60);
pub const DEFAULT_MIN_FREE_GB: u64 = 20;
const GB: u64 = 1 << 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub target_bytes: u64,
    pub worktrees_bytes: u64,
    pub data_bytes: u64,
}

/// Disk space actually allocated under `path` (like `du`), not following symlinks.
/// Missing or unreadable entries count as zero.
pub fn dir_size(path: &Path) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    let own = meta.blocks() * 512;
    if !meta.is_dir() {
        return own;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return own;
    };
    own + entries.flatten().map(|e| dir_size(&e.path())).sum::<u64>()
}

/// Blocking: walks three directory trees.
pub fn measure(ws: &Workspace) -> std::io::Result<Reading> {
    let st = nix::sys::statvfs::statvfs(&ws.workspace).map_err(std::io::Error::from)?;
    let frag = st.fragment_size() as u64;
    Ok(Reading {
        free_bytes: st.blocks_available() as u64 * frag,
        total_bytes: st.blocks() as u64 * frag,
        target_bytes: dir_size(&ws.repo.join("target")),
        worktrees_bytes: dir_size(&ws.wt_dir()),
        data_bytes: dir_size(&ws.state_dir()),
    })
}

#[derive(Clone)]
pub struct DiskMonitor(Arc<Inner>);

struct Inner {
    ws: Workspace,
    min_free_bytes: u64,
    emitter: Emitter,
    manager: AgentManager,
    /// Whether the last tick was under the threshold, so the human is told once per
    /// dip, not every hour it stays low.
    low: Mutex<bool>,
}

impl DiskMonitor {
    pub fn new(ws: Workspace, min_free_gb: u64, emitter: Emitter, manager: AgentManager) -> Self {
        DiskMonitor(Arc::new(Inner {
            ws,
            min_free_bytes: min_free_gb * GB,
            emitter,
            manager,
            low: Mutex::new(false),
        }))
    }

    pub async fn tick(&self) {
        let ws = self.0.ws.clone();
        let r = match tokio::task::spawn_blocking(move || measure(&ws)).await {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => return tracing::warn!(error = %e, "disk check failed"),
            Err(e) => return tracing::warn!(error = %e, "disk check panicked"),
        };
        tracing::info!(
            free_gb = r.free_bytes / GB,
            target_gb = r.target_bytes / GB,
            worktrees_gb = r.worktrees_bytes / GB,
            data_gb = r.data_bytes / GB,
            "disk usage"
        );
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::DISK_CHECKED,
                "system".to_string(),
                None,
                json!({
                    "free_bytes": r.free_bytes,
                    "total_bytes": r.total_bytes,
                    "target_bytes": r.target_bytes,
                    "worktrees_bytes": r.worktrees_bytes,
                    "data_bytes": r.data_bytes,
                }),
            )
            .await;
        let now_low = r.free_bytes < self.0.min_free_bytes;
        let was_low = std::mem::replace(&mut *self.0.low.lock().expect("disk low lock"), now_low);
        if !(now_low && !was_low) {
            return;
        }
        let body = format!(
            "Low disk space: {} GB free of {} GB (threshold {} GB). Largest: clone target/ {} GB, worktrees {} GB, .bridle {} GB. \
             Suggested: `cargo clean` in the clone and in idle worktrees (`git worktree remove` finished ones).",
            r.free_bytes / GB,
            r.total_bytes / GB,
            self.0.min_free_bytes / GB,
            r.target_bytes / GB,
            r.worktrees_bytes / GB,
            r.data_bytes / GB,
        );
        let _ = self
            .0
            .manager
            .send(
                "system".to_string(),
                ToTarget::Human,
                MessageKind::Note,
                body,
                When::Now,
                None,
            )
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_size_counts_files_and_ignores_missing() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("sub")).unwrap();
        std::fs::write(tmp.path().join("sub/f"), vec![1u8; 100_000]).unwrap();
        assert!(dir_size(tmp.path()) >= 100_000);
        assert_eq!(dir_size(&tmp.path().join("nope")), 0);
    }

    #[test]
    fn measure_reads_the_three_trees_and_the_volume() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path().join("repo"), Some(tmp.path().to_path_buf()));
        std::fs::create_dir_all(ws.repo.join("target")).unwrap();
        std::fs::write(ws.repo.join("target/x"), vec![1u8; 200_000]).unwrap();
        std::fs::create_dir_all(ws.wt_dir()).unwrap();
        let r = measure(&ws).unwrap();
        assert!(r.target_bytes >= 200_000);
        assert!(r.total_bytes > 0 && r.free_bytes <= r.total_bytes);
        assert_eq!(r.data_bytes, 0);
    }
}
