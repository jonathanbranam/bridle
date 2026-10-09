//! br-6nzj (n4w4 rec 2): an idle daemon's timer loops must not fork or
//! snapshot the process table, and a busy one must not poll faster than its
//! interval. `containment::SNAPSHOTS` counts every process-table snapshot
//! (native read or `ps` fallback); each test runs in its own process under
//! nextest, so the counter is this daemon's alone.
//!
//! Other `Command::new` callers on timers, checked when this was written:
//! `load::SystemLoad` (`sysctl`, `ps` top consumers) runs only under
//! `load_watch`, which is off here and samples at its own interval; the
//! upgrade, CI, port, doc-watch, queue-nudge and settle-wake loops are at
//! 3600 s in the harness. The tracker is the only short-interval timer
//! that touched the process table.

mod support;

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use bridle_api::types::{SpawnRequest, Workdir};
use bridle_daemon::containment::SNAPSHOTS;
use support::{default_overrides, start_daemon};

const TRACKER: Duration = Duration::from_millis(200);

fn short_timers() -> bridle_daemon::Overrides {
    let mut o = default_overrides();
    o.tracker_interval = TRACKER;
    o.stall_check_interval = TRACKER;
    o.governor_interval = TRACKER;
    o
}

#[tokio::test]
async fn idle_daemon_takes_no_process_snapshots() {
    let (_daemon, _tmp) = start_daemon(Some(short_timers())).await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    assert_eq!(SNAPSHOTS.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn tracker_with_a_live_agent_stays_within_its_interval() {
    let (daemon, _tmp) = start_daemon(Some(short_timers())).await;
    daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SLEEP 4".to_string()),
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    let before = SNAPSHOTS.load(Ordering::Relaxed);
    let start = Instant::now();
    tokio::time::sleep(Duration::from_secs(3)).await;
    let taken = SNAPSHOTS.load(Ordering::Relaxed) - before;
    let max = (start.elapsed().as_millis() / TRACKER.as_millis()) as usize + 1;
    assert!(taken >= 1, "tracker never snapshotted a live agent");
    assert!(
        taken <= max,
        "{taken} snapshots in the window, at most {max}"
    );
}
