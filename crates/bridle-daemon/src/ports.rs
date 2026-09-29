//! The port registry (docs/design/worktrees-and-ports.md): `bridle port alloc` hands out
//! a free port from `[ports] range`, never a reserved one, and records the owner so
//! "stop what you start" can be checked. `tick` frees ports whose owner is gone.

use std::collections::HashSet;
use std::net::TcpListener;

use bridle_api::types::PortAllocation;
use chrono::Utc;
use nix::sys::signal::kill;
use nix::unistd::Pid;

use crate::config::PortsConfig;
use crate::store::{Store, StoreError};

/// True when nothing on this host is bound to `port` on 127.0.0.1.
fn is_free(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Records and returns the lowest port in range that isn't reserved, allocated or
/// listening. `None` when the range is exhausted.
pub async fn alloc(
    store: &Store,
    cfg: &PortsConfig,
    agent: String,
    task: Option<String>,
    pid: Option<i32>,
    label: Option<String>,
) -> Result<Option<PortAllocation>, StoreError> {
    let taken: HashSet<u16> = store.list_ports().await?.iter().map(|p| p.port).collect();
    let (lo, hi) = cfg.range;
    for port in lo..=hi {
        if cfg.reserved.contains(&port) || taken.contains(&port) {
            continue;
        }
        // A bind test is a syscall; too short to be worth a spawn_blocking per port.
        if !is_free(port) {
            continue;
        }
        let rec = PortAllocation {
            port,
            agent: agent.clone(),
            task: task.clone(),
            pid,
            label: label.clone(),
            allocated_at: Utc::now(),
        };
        // Another allocation can win the race between the listing and here.
        if store.insert_port(&rec).await? {
            return Ok(Some(rec));
        }
    }
    Ok(None)
}

fn pid_alive(pid: i32) -> bool {
    match kill(Pid::from_raw(pid), None) {
        Ok(()) => true,
        Err(nix::errno::Errno::EPERM) => true,
        Err(_) => false,
    }
}

/// Frees ports whose recorded pid is dead, or whose owner agent no longer runs.
pub async fn tick(store: &Store) {
    let Ok(ports) = store.list_ports().await else {
        return;
    };
    for p in ports {
        let mut why = None;
        if p.pid.is_some_and(|pid| !pid_alive(pid)) {
            why = Some("pid exited");
        } else if p.agent != "human"
            && let Ok(agent) = store.get_agent(&p.agent).await
            && agent.is_none_or(|a| !a.state.is_running())
        {
            why = Some("owner agent stopped");
        }
        if let Some(why) = why {
            tracing::info!(port = p.port, agent = %p.agent, why, "freeing port");
            let _ = store.release_port(p.port).await;
        }
    }
}
