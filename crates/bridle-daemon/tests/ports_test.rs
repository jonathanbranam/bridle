//! The port registry (docs/design/worktrees-and-ports.md): alloc skips reserved, taken
//! and listening ports, release frees one, and an agent's ports go when it exits.

mod support;

use std::net::TcpListener;
use std::time::Duration;

use bridle_api::types::{
    AgentState, AllocPortRequest, NewTaskRequest, SpawnRequest, StopRequest, TaskKind, Workdir,
};
use support::{default_overrides, start_daemon_with_config, wait_for, wait_for_state};

/// The start of a run of six ports that are all free right now.
fn free_base() -> u16 {
    loop {
        let start = TcpListener::bind("127.0.0.1:0")
            .expect("bind")
            .local_addr()
            .expect("addr")
            .port();
        let held: Vec<_> = (start..start.saturating_add(6))
            .filter_map(|p| TcpListener::bind(("127.0.0.1", p)).ok())
            .collect();
        if held.len() == 6 {
            return start;
        }
    }
}

fn req(pid: Option<i32>) -> AllocPortRequest {
    AllocPortRequest { pid, label: None }
}

#[tokio::test]
async fn alloc_skips_reserved_taken_and_listening_ports_and_release_frees() {
    let base = free_base();
    let cfg = format!(
        "[ports]\nrange = [{base}, {}]\nreserved = [{base}]\n",
        base.saturating_add(3)
    );
    let (d, _tmp) = start_daemon_with_config(None, Some(&cfg)).await;
    let c = &d.client;
    // Someone else is listening on base+1.
    let _busy = TcpListener::bind(("127.0.0.1", base + 1)).expect("listen");

    // Other tests' daemons may grab a port in the range meanwhile, so no exact numbers.
    let a = c.alloc_port(&req(None)).await.expect("alloc");
    assert!(
        a.port != base && a.port != base + 1,
        "skips the reserved and the listening port, got {}",
        a.port
    );
    if let Ok(b) = c.alloc_port(&req(None)).await {
        assert_ne!(b.port, a.port, "skips the allocated one");
        assert!(b.port != base && b.port != base + 1);
    }
    while c.alloc_port(&req(None)).await.is_ok() {}
    assert!(c.alloc_port(&req(None)).await.is_err(), "range exhausted");
    assert!(c.list_ports().await.expect("list").len() <= 2);

    let freed = c.release_port(a.port).await.expect("release");
    assert_eq!(freed.port, a.port);
    assert!(c.release_port(a.port).await.is_err(), "already free");
    c.alloc_port(&req(None)).await.expect("alloc after release");
}

#[tokio::test]
async fn an_agents_ports_are_freed_when_it_exits() {
    let base = free_base();
    let cfg = format!("[ports]\nrange = [{base}, {}]\n", base.saturating_add(5));
    let (d, _tmp) = start_daemon_with_config(None, Some(&cfg)).await;
    let c = &d.client;
    let t = c
        .new_task(&NewTaskRequest {
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
    let cfg = format!("[ports]\nrange = [{base}, {}]\n", base.saturating_add(5));
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
