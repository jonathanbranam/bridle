//! The port registry (docs/design/worktrees-and-ports.md): alloc skips reserved, taken
//! and listening ports, release frees one, and an agent's ports go when it exits.

mod support;
use support::ClientExt as _;

use std::net::TcpListener;
use std::time::Duration;

use bridle_api::types::{
    AgentState, AllocPortRequest, NewTaskRequest, SpawnRequest, StopRequest, TaskKind, Workdir,
};
use support::{default_overrides, start_daemon_with_config, wait_for, wait_for_state};

const BLOCK: u16 = 16;

/// Listeners holding a run of `BLOCK` ports that were all free. The block sits below 32768,
/// outside every OS's ephemeral range (macOS starts at 49152, Linux at 32768), so no other
/// test's outgoing connection can take a port in it. The scan starts at a pid-derived offset
/// so tests running in parallel processes rarely pick the same block.
fn free_block() -> Vec<TcpListener> {
    let blocks = 10_000 / BLOCK;
    let first = u16::try_from(std::process::id() % u32::from(blocks)).expect("fits");
    for i in 0..blocks {
        let start = 20_000 + ((first + i) % blocks) * BLOCK;
        let held: Vec<_> = (start..start + BLOCK)
            .filter_map(|p| TcpListener::bind(("127.0.0.1", p)).ok())
            .collect();
        if held.len() == usize::from(BLOCK) {
            return held;
        }
    }
    panic!("no free block of {BLOCK} ports in 20000-29999");
}

fn free_base() -> u16 {
    free_block()[0].local_addr().expect("addr").port()
}

fn req(pid: Option<i32>) -> AllocPortRequest {
    AllocPortRequest { pid, label: None }
}

#[tokio::test]
async fn alloc_skips_reserved_taken_and_listening_ports_and_release_frees() {
    // Someone else is listening on base+1: keep that listener, free the rest of the block.
    let mut held = free_block();
    let base = held[0].local_addr().expect("addr").port();
    let _busy = held.swap_remove(1);
    drop(held);
    let last = base + BLOCK - 1;
    let cfg = format!("[ports]\nrange = [{base}, {last}]\nreserved = [{base}]\n");
    let (d, _tmp) = start_daemon_with_config(None, Some(&cfg)).await;
    let c = &d.client;

    let a = c.alloc_port(&req(None)).await.expect("alloc");
    assert!(
        a.port != base && a.port != base + 1,
        "skips the reserved and the listening port, got {}",
        a.port
    );
    let b = c.alloc_port(&req(None)).await.expect("second alloc");
    assert_ne!(b.port, a.port, "skips the allocated one");
    assert!(b.port != base && b.port != base + 1);
    let mut allocated = 2;
    while c.alloc_port(&req(None)).await.is_ok() {
        allocated += 1;
    }
    assert_eq!(
        allocated,
        usize::from(BLOCK) - 2,
        "every free port handed out"
    );
    assert!(c.alloc_port(&req(None)).await.is_err(), "range exhausted");
    assert_eq!(c.list_ports().await.expect("list").len(), allocated);

    let freed = c.release_port(a.port).await.expect("release");
    assert_eq!(freed.port, a.port);
    assert!(c.release_port(a.port).await.is_err(), "already free");
    c.alloc_port(&req(None)).await.expect("alloc after release");
}

#[tokio::test]
async fn an_agents_ports_are_freed_when_it_exits() {
    let base = free_base();
    let cfg = format!("[ports]\nrange = [{base}, {}]\n", base + BLOCK - 1);
    let (d, _tmp) = start_daemon_with_config(None, Some(&cfg)).await;
    let c = &d.client;
    let t = c
        .new_open_task(&NewTaskRequest {
            ticket: None,
            parent: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: "web".to_string(),
            kind: TaskKind::Feature,
            body: String::new(),
            size: None,
        })
        .await
        .expect("task");
    c.plan_task(&t.id).await.expect("plan");
    let w = c
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(c, &w.id, AgentState::Idle).await;
    let me = d.agent_client(&w.id);
    me.claim_task(&t.id).await.expect("claim");

    let p = me.alloc_port(&req(None)).await.expect("alloc");
    assert_eq!(p.agent, w.id);
    assert_eq!(p.task.as_deref(), Some(t.id.as_str()));

    c.stop(&w.id, &StopRequest { now: true })
        .await
        .expect("stop");
    wait_for("the port is freed", || async {
        c.list_ports().await.expect("list").is_empty().then_some(())
    })
    .await;
}

#[tokio::test]
async fn a_dead_pid_frees_its_port_on_the_tick() {
    let base = free_base();
    let cfg = format!("[ports]\nrange = [{base}, {}]\n", base + BLOCK - 1);
    let mut o = default_overrides();
    o.port_check_interval = Duration::from_millis(100);
    let (d, _tmp) = start_daemon_with_config(Some(o), Some(&cfg)).await;
    let c = &d.client;
    let mut child = std::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("sleep");
    let pid = i32::try_from(child.id()).expect("pid");
    c.alloc_port(&req(Some(pid))).await.expect("alloc");
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(c.list_ports().await.expect("list").len(), 1, "pid is alive");

    child.kill().expect("kill");
    child.wait().expect("wait");
    wait_for("the port is freed", || async {
        c.list_ports().await.expect("list").is_empty().then_some(())
    })
    .await;
}
