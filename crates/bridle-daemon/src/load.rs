//! Machine load watch: on a timer, reads the 1-minute load average, normalises it by core
//! count, and while it is over `[machine] load_per_core` holds new agent spawns (like the
//! budget hold; they resume when the load falls). The orchestrator is messaged once per
//! crossing, naming the load and the top CPU consumers (ticket 58c9). Running agents are
//! left alone: this slice only stops adding work.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use bridle_api::types::{LoadStatus, MessageKind, When};

use crate::supervisor::{AgentManager, ToTarget};

pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(30);
/// 70 on 16 cores (the incident behind the ticket) is 4.4; a busy but healthy build box
/// sits around 1 to 2.
pub const DEFAULT_LOAD_PER_CORE: f64 = 2.5;

/// What a sample needs from the machine; tests inject a fake.
pub trait LoadSource: Send + Sync {
    /// 1-minute load average and the core count.
    fn sample(&self) -> Option<(f64, usize)>;
    /// The top CPU consumers as a short human string, or empty if unknown.
    fn top_consumers(&self) -> String;
}

/// Reads `/proc/loadavg` (Linux) or `sysctl vm.loadavg` (macOS): no libc call, so no unsafe.
pub struct SystemLoad;

impl LoadSource for SystemLoad {
    fn sample(&self) -> Option<(f64, usize)> {
        let raw = std::fs::read_to_string("/proc/loadavg").ok().or_else(|| {
            let out = std::process::Command::new("sysctl")
                .args(["-n", "vm.loadavg"])
                .output()
                .ok()?;
            String::from_utf8(out.stdout).ok()
        })?;
        let cores = std::thread::available_parallelism().ok()?.get();
        Some((parse_load1(&raw)?, cores))
    }

    fn top_consumers(&self) -> String {
        let Ok(out) = std::process::Command::new("ps")
            .args(["-A", "-o", "pcpu=,comm="])
            .output()
        else {
            return String::new();
        };
        top_from_ps(&String::from_utf8_lossy(&out.stdout), 3)
    }
}

/// The first number of `0.52 0.58 0.59 1/467 8223` or `{ 3.17 2.90 2.85 }`.
fn parse_load1(raw: &str) -> Option<f64> {
    raw.split_whitespace()
        .find_map(|w| w.trim_matches(['{', '}']).parse::<f64>().ok())
}

/// `ps -o pcpu=,comm=` lines folded by command name, busiest first, as `name 120%, ...`.
fn top_from_ps(ps: &str, n: usize) -> String {
    let mut by_name: Vec<(String, f64)> = Vec::new();
    for line in ps.lines() {
        let Some((cpu, comm)) = line.trim().split_once(char::is_whitespace) else {
            continue;
        };
        let (Ok(cpu), name) = (
            cpu.parse::<f64>(),
            comm.trim().rsplit('/').next().unwrap_or("").to_string(),
        ) else {
            continue;
        };
        match by_name.iter_mut().find(|(k, _)| *k == name) {
            Some(e) => e.1 += cpu,
            None => by_name.push((name, cpu)),
        }
    }
    by_name.sort_by(|a, b| b.1.total_cmp(&a.1));
    by_name
        .iter()
        .take(n)
        .map(|(k, v)| format!("{k} {v:.0}%"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Clone)]
pub struct LoadWatch(Arc<Inner>);

struct Inner {
    threshold: f64,
    source: Box<dyn LoadSource>,
    manager: AgentManager,
    /// Whether the last tick was over the threshold, so the orchestrator is told once per
    /// crossing, not every tick the load stays high.
    holding: Mutex<bool>,
}

impl LoadWatch {
    pub fn new(threshold: f64, source: Box<dyn LoadSource>, manager: AgentManager) -> Self {
        LoadWatch(Arc::new(Inner {
            threshold,
            source,
            manager,
            holding: Mutex::new(false),
        }))
    }

    pub async fn tick(&self) {
        let this = self.clone();
        let sampled = tokio::task::spawn_blocking(move || this.0.source.sample()).await;
        let Ok(Some((load1, cores))) = sampled else {
            return tracing::warn!("load sample failed");
        };
        let per_core = load1 / cores.max(1) as f64;
        // A threshold of zero or less turns the hold off but keeps `bridle status` fed.
        let over = self.0.threshold > 0.0 && per_core > self.0.threshold;
        self.0.manager.set_load(LoadStatus {
            load1,
            cores: cores as u32,
            per_core,
            threshold: self.0.threshold,
            holding: over,
        });
        let was = std::mem::replace(&mut *self.0.holding.lock().expect("load lock"), over);
        if !(over && !was) {
            return;
        }
        let this = self.clone();
        let top = tokio::task::spawn_blocking(move || this.0.source.top_consumers())
            .await
            .unwrap_or_default();
        let mut body = format!(
            "Machine load is high: {load1:.1} on {cores} cores ({per_core:.1} per core, \
             threshold {:.1}). New agent spawns are held until it falls; the daemon resumes \
             them itself. Don't add work: wait.",
            self.0.threshold
        );
        if !top.is_empty() {
            body.push_str(&format!(" Top consumers: {top}."));
        }
        let _ = self
            .0
            .manager
            .send(
                "system".to_string(),
                ToTarget::External(crate::wake::ORCHESTRATOR.to_string()),
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
    use crate::config::Config;
    use crate::events::Emitter;
    use crate::paths::Workspace;
    use crate::store::{ListMessages, Store};
    use std::collections::VecDeque;

    /// Hands out the scripted loads one per tick.
    struct Fake(Mutex<VecDeque<f64>>);

    impl LoadSource for Fake {
        fn sample(&self) -> Option<(f64, usize)> {
            Some((self.0.lock().unwrap().pop_front()?, 4))
        }
        fn top_consumers(&self) -> String {
            "rustc 300%".to_string()
        }
    }

    async fn fixture(loads: &[f64]) -> (LoadWatch, AgentManager, Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("bridle.db")).await.unwrap();
        let manager = AgentManager::new(
            store.clone(),
            Workspace::new(dir.path().join("repo"), None),
            Config::default(),
            "claude".to_string(),
            "http://127.0.0.1:0".to_string(),
            "test".to_string(),
            Emitter::new(store.clone()),
            Default::default(),
        );
        let fake = Fake(Mutex::new(loads.iter().copied().collect()));
        let watch = LoadWatch::new(2.0, Box::new(fake), manager.clone());
        (watch, manager, store, dir)
    }

    async fn orchestrator_messages(store: &Store) -> Vec<String> {
        store
            .list_messages(ListMessages {
                to: Some(crate::wake::ORCHESTRATOR.to_string()),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.body)
            .collect()
    }

    #[tokio::test]
    async fn holds_over_threshold_releases_when_it_falls_and_tells_once_per_crossing() {
        // 4 cores, threshold 2.0 per core: 8.0 is the line (over means above).
        let (watch, manager, store, _dir) = fixture(&[1.0, 12.0, 20.0, 4.0, 16.0]).await;
        watch.tick().await;
        assert!(!manager.load_status().unwrap().holding);
        assert!(manager.refuse_if_load_held().is_ok());

        watch.tick().await;
        let l = manager.load_status().unwrap();
        assert!(l.holding && l.per_core == 3.0 && l.cores == 4);
        assert!(manager.refuse_if_load_held().is_err());
        watch.tick().await; // still over: no second message
        let msgs = orchestrator_messages(&store).await;
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("12.0 on 4 cores") && msgs[0].contains("rustc 300%"));

        watch.tick().await; // falls: spawns resume
        assert!(manager.refuse_if_load_held().is_ok());
        watch.tick().await; // a new crossing: a second message
        assert_eq!(orchestrator_messages(&store).await.len(), 2);
    }

    #[test]
    fn parses_linux_and_macos_forms() {
        assert_eq!(parse_load1("0.52 0.58 0.59 1/467 8223\n"), Some(0.52));
        assert_eq!(parse_load1("{ 3.17 2.90 2.85 }\n"), Some(3.17));
        assert_eq!(parse_load1("junk"), None);
    }

    #[test]
    fn top_consumers_fold_by_name() {
        let ps = " 80.0 /usr/sbin/syspolicyd\n 30.0 /x/rustc\n 40.0 /y/rustc\n  1.0 /bin/zsh\n";
        assert_eq!(top_from_ps(ps, 2), "syspolicyd 80%, rustc 70%");
    }
}
