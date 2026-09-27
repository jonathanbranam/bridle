//! Process containment: tracking and reaping an agent's descendant
//! processes, and stopping a process group. See docs/design/agent-host/agents.md.

use std::collections::{HashMap, HashSet};
use std::io;
use std::process::Command;
use std::time::Duration;

use nix::sys::signal::{Signal, kill, killpg};
use nix::unistd::Pid;
use tokio::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcInfo {
    pub pid: i32,
    pub ppid: i32,
    pub pgid: i32,
    /// `lstart`-style start time, opaque and only ever compared for
    /// equality: it's how we avoid signalling a reused pid.
    pub start: String,
}

/// Snapshots the process table with `ps`. Works on both macOS and Linux:
/// `lstart` is multi-word, so it's kept last and the rest of the line after
/// the first three numeric fields is taken whole.
pub fn snapshot() -> io::Result<Vec<ProcInfo>> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,ppid=,pgid=,lstart="])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "ps exited with {}",
            output.status
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut procs = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let (Some(pid), Some(ppid), Some(pgid)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        let (Ok(pid), Ok(ppid), Ok(pgid)) = (pid.parse(), ppid.parse(), pgid.parse()) else {
            continue;
        };
        let start = it.collect::<Vec<_>>().join(" ");
        if start.is_empty() {
            continue;
        }
        procs.push(ProcInfo {
            pid,
            ppid,
            pgid,
            start,
        });
    }
    Ok(procs)
}

/// Every process in `snap` transitively parented by `root`, excluding
/// `root` itself.
pub fn descendants(root: i32, snap: &[ProcInfo]) -> Vec<ProcInfo> {
    let mut children: HashMap<i32, Vec<&ProcInfo>> = HashMap::new();
    for p in snap {
        children.entry(p.ppid).or_default().push(p);
    }

    let mut out = Vec::new();
    let mut frontier = vec![root];
    let mut visited: HashSet<i32> = HashSet::new();
    while let Some(pid) = frontier.pop() {
        let Some(kids) = children.get(&pid) else {
            continue;
        };
        for &k in kids {
            if visited.insert(k.pid) {
                out.push(k.clone());
                frontier.push(k.pid);
            }
        }
    }
    out
}

/// The current start time of `pid`, from a fresh snapshot, or `None` if it's
/// not running.
pub fn start_time(pid: i32) -> Option<String> {
    snapshot()
        .ok()?
        .into_iter()
        .find(|p| p.pid == pid)
        .map(|p| p.start)
}

/// Whether `pid` is currently running with exactly this start time. A pid
/// with a *different* start time has been reused by an unrelated process
/// and must never be signalled.
pub fn is_same_process(pid: i32, start: &str) -> bool {
    start_time(pid).as_deref() == Some(start)
}

/// Maps each pid in `snap` to its start time, for O(1) identity checks
/// instead of a linear scan per tracked pid.
fn start_index(snap: &[ProcInfo]) -> HashMap<i32, &str> {
    snap.iter().map(|p| (p.pid, p.start.as_str())).collect()
}

/// Tracks an agent's descendant processes across repeated snapshots, keyed
/// by pid + start time so a reused pid is never confused for the one we saw
/// before (agents.md, Containment).
#[derive(Debug, Clone)]
pub struct Tracker {
    pub root_pid: i32,
    pub root_start: String,
    pub seen: HashMap<i32, String>,
}

impl Tracker {
    pub fn new(root_pid: i32, root_start: String) -> Self {
        Tracker {
            root_pid,
            root_start,
            seen: HashMap::new(),
        }
    }

    /// Prunes `seen` to what `snap` confirms still exists (a pid gone, or
    /// reused by a different process, is dropped -- this check is pid+start
    /// identity only, so a process that's re-parented to init is still kept
    /// as long as its pid+start still matches), then adds every current
    /// descendant of the root. Call this periodically and just before any
    /// stop, so short-lived tool process groups are caught before they can
    /// re-parent, and `seen` doesn't grow without bound across an agent's
    /// lifetime.
    pub fn update(&mut self, snap: &[ProcInfo]) {
        let index = start_index(snap);
        self.seen
            .retain(|pid, start| index.get(pid) == Some(&start.as_str()));
        for p in descendants(self.root_pid, snap) {
            self.seen.insert(p.pid, p.start);
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SweepReport {
    pub terminated: u32,
    pub killed: u32,
}

/// SIGTERMs every process in `tracker.seen` that's still alive in `snap`
/// with a matching start time, waits up to `grace`, then SIGKILLs whatever
/// is still around -- checked against the same `snap`, not a fresh one, since
/// staleness here only risks a redundant signal to an already-dead pid, never
/// a signal to a reused one. Never signals a pid whose start time doesn't
/// match. Takes `snap` rather than fetching its own, so a sweep over many
/// tracked pids costs one process-table scan, not one per pid.
pub async fn sweep(tracker: &mut Tracker, snap: &[ProcInfo], grace: Duration) -> SweepReport {
    let index = start_index(snap);
    let matches = |pid: i32, start: &str| index.get(&pid) == Some(&start);

    let mut termed: Vec<(i32, String)> = Vec::new();
    for (&pid, start) in tracker.seen.iter() {
        if matches(pid, start) && kill(Pid::from_raw(pid), Signal::SIGTERM).is_ok() {
            termed.push((pid, start.clone()));
        }
    }
    let terminated = termed.len() as u32;

    if !termed.is_empty() {
        tokio::time::sleep(grace).await;
    }

    let mut killed = 0u32;
    for (pid, start) in termed {
        if matches(pid, &start) && kill(Pid::from_raw(pid), Signal::SIGKILL).is_ok() {
            killed += 1;
        }
    }

    SweepReport { terminated, killed }
}

/// SIGTERMs the process group `pgid`, waits up to `grace` polling for the
/// group to disappear, then SIGKILLs it. Returns whether a SIGKILL was
/// needed.
pub async fn terminate_group(pgid: i32, grace: Duration) -> bool {
    let group_alive = || killpg(Pid::from_raw(pgid), None).is_ok();

    if kill(Pid::from_raw(-pgid), Signal::SIGTERM).is_err() {
        // No such process group: nothing to do.
        return false;
    }

    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if !group_alive() {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    if group_alive() {
        let _ = kill(Pid::from_raw(-pgid), Signal::SIGKILL);
        true
    } else {
        false
    }
}

/// Behind this trait so a future Linux cgroup-v2 implementation can slot in
/// without touching callers.
pub trait Containment {
    fn sweep(
        &self,
        tracker: &mut Tracker,
        snap: &[ProcInfo],
        grace: Duration,
    ) -> impl std::future::Future<Output = SweepReport> + Send;
    fn terminate_group(
        &self,
        pgid: i32,
        grace: Duration,
    ) -> impl std::future::Future<Output = bool> + Send;
}

/// The v1, macOS/Linux-portable implementation: `ps` snapshots and POSIX
/// signals.
pub struct PsContainment;

impl Containment for PsContainment {
    async fn sweep(
        &self,
        tracker: &mut Tracker,
        snap: &[ProcInfo],
        grace: Duration,
    ) -> SweepReport {
        sweep(tracker, snap, grace).await
    }

    async fn terminate_group(&self, pgid: i32, grace: Duration) -> bool {
        terminate_group(pgid, grace).await
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::CommandExt;

    use super::*;

    /// Spawns `sh -c 'sleep 30 & sleep 30'` as the leader of its own process
    /// group (stable, safe `process_group(0)`; no `pre_exec`/`unsafe`, which
    /// workspace lints forbid). The shell backgrounds one sleep and waits on
    /// the other, so it has two live descendants.
    fn spawn_group() -> std::process::Child {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("sleep 30 & sleep 30");
        cmd.process_group(0);
        cmd.spawn().expect("spawn test process group")
    }

    #[tokio::test]
    async fn tracker_sees_descendants_and_sweep_kills_them() {
        let mut child = spawn_group();
        let root_pid = child.id() as i32;

        // Give the shell a moment to fork its two sleeps.
        tokio::time::sleep(Duration::from_millis(200)).await;

        let start = start_time(root_pid).expect("root process visible in ps");
        let mut tracker = Tracker::new(root_pid, start);
        let snap = snapshot().expect("snapshot");
        tracker.update(&snap);

        assert!(
            tracker.seen.len() >= 2,
            "expected at least two descendant sleeps, saw {:?}",
            tracker.seen
        );

        let report = sweep(&mut tracker, &snap, Duration::from_millis(500)).await;
        assert!(report.terminated + report.killed >= 2);

        for pid in tracker.seen.keys() {
            assert!(
                nix::sys::signal::kill(Pid::from_raw(*pid), None).is_err(),
                "pid {pid} should be gone after sweep"
            );
        }

        let _ = child.wait();
    }

    #[tokio::test]
    async fn mismatched_start_time_is_skipped() {
        let mut child = spawn_group();
        let root_pid = child.id() as i32;
        tokio::time::sleep(Duration::from_millis(200)).await;

        let snap = snapshot().expect("snapshot");
        let descendants = descendants(root_pid, &snap);
        let victim = descendants.first().expect("at least one descendant").pid;

        assert!(is_same_process(victim, &start_time(victim).expect("alive")));
        assert!(!is_same_process(victim, "not a real start time"));

        let mut tracker = Tracker::new(root_pid, start_time(root_pid).expect("root alive"));
        tracker
            .seen
            .insert(victim, "not a real start time".to_string());

        let report = sweep(&mut tracker, &snap, Duration::from_millis(300)).await;
        assert_eq!(report.terminated, 0);
        assert_eq!(report.killed, 0);

        // The process is untouched: still alive under its real start time.
        assert!(is_same_process(
            victim,
            &start_time(victim).expect("still alive")
        ));

        // Clean up for real.
        let real_snap = snapshot().expect("snapshot");
        let mut real_tracker = Tracker::new(root_pid, start_time(root_pid).expect("root alive"));
        real_tracker.update(&real_snap);
        sweep(&mut real_tracker, &real_snap, Duration::from_millis(300)).await;
        let _ = child.wait();
    }

    /// Regression for the stop-hangs-forever bug: sweep used to call
    /// `is_same_process` (a full `ps` scan) once per tracked pid, twice over
    /// (once per pass). With thousands of tracked pids that made `stop` take
    /// minutes even though the agent's own process had already exited. Sweep
    /// now takes one snapshot as a parameter and never calls the real,
    /// `ps`-backed `snapshot()` itself, so this test tracks thousands of
    /// fabricated (never-real) pids and asserts it still completes fast.
    #[tokio::test]
    async fn sweep_uses_one_snapshot_regardless_of_tracked_pid_count() {
        let mut tracker = Tracker::new(1, "root-start".to_string());
        for i in 0..5_000 {
            tracker
                .seen
                .insert(1_000_000 + i, format!("fake-start-{i}"));
        }
        // None of the fabricated pids appear here, so nothing should be
        // signalled and no grace sleep should be entered.
        let snap = [ProcInfo {
            pid: 1,
            ppid: 0,
            pgid: 1,
            start: "root-start".to_string(),
        }];

        let started = std::time::Instant::now();
        let report = sweep(&mut tracker, &snap, Duration::from_millis(500)).await;
        let elapsed = started.elapsed();

        assert_eq!(report.terminated, 0);
        assert_eq!(report.killed, 0);
        assert!(
            elapsed < Duration::from_millis(200),
            "sweep over {} tracked pids took {elapsed:?}; a ps call per pid would take far longer",
            tracker.seen.len()
        );
    }

    #[tokio::test]
    async fn terminate_group_kills_the_whole_group() {
        let mut child = spawn_group();
        let pgid = child.id() as i32; // leader's pid == pgid (process_group(0))
        tokio::time::sleep(Duration::from_millis(200)).await;

        let needed_kill = terminate_group(pgid, Duration::from_millis(500)).await;
        // A plain `sleep` handles SIGTERM by dying immediately, so SIGKILL
        // usually isn't needed, but either outcome is a pass so long as the
        // group is gone afterward.
        let _ = needed_kill;
        assert!(
            killpg(Pid::from_raw(pgid), None).is_err(),
            "process group should be gone"
        );

        let _ = child.wait();
    }
}
