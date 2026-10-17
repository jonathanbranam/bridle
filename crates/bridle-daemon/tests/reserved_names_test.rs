//! Role names are reserved, by prefix, for agent names (ticket 2vja): only the human, the
//! daemon and the orchestrator create an agent under one; resume and renew never block.

mod support;

use bridle_api::types::{AgentState, RenewRequest, ResumeRequest, SpawnRequest, StopRequest};
use support::{start_daemon, wait_for_state};

fn req(role: &str, name: &str) -> SpawnRequest {
    SpawnRequest {
        components: Vec::new(),
        role: role.to_string(),
        name: Some(name.to_string()),
        prompt: None,
        workdir: None,
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
        ticket: None,
    }
}

fn is_403<T: std::fmt::Debug>(r: Result<T, bridle_api::ClientError>) -> String {
    match r {
        Err(bridle_api::ClientError::Api {
            status: 403,
            message,
            ..
        }) => message,
        other => panic!("expected 403, got {other:?}"),
    }
}

#[tokio::test]
async fn the_orchestrator_and_human_may_create_reserved_names() {
    let (daemon, _tmp) = start_daemon(None).await;
    let orch = daemon.external_client("orchestrator").await;
    orch.spawn(&req("manager", "manager-2"))
        .await
        .expect("orchestrator spawns a manager");
    orch.spawn(&req("manager", "advisor-x"))
        .await
        .expect("orchestrator spawns advisor-x");
    daemon
        .client
        .spawn(&req("manager", "pm-1"))
        .await
        .expect("human spawns pm-1");
}

#[tokio::test]
async fn agents_and_other_externals_may_not_create_reserved_names() {
    let (daemon, _tmp) = start_daemon(None).await;
    let m = daemon
        .client
        .spawn(&req("manager", "manager-9"))
        .await
        .expect("spawn manager");
    let mc = daemon.agent_client(&m.id);
    for name in ["advisor-x", "manager-3", "pm-2", "orchestrator", "Worker-1"] {
        let msg = is_403(mc.spawn(&req("worker", name)).await);
        assert!(msg.contains("is reserved for the"), "{msg}");
    }
    let other = daemon.external_client("someone").await;
    is_403(other.spawn(&req("worker", "advisor-jim")).await);
}

#[tokio::test]
async fn names_in_use_today_still_spawn_for_a_manager() {
    let (daemon, _tmp) = start_daemon(None).await;
    let m = daemon
        .client
        .spawn(&req("manager", "manager-9"))
        .await
        .expect("spawn manager");
    let mc = daemon.agent_client(&m.id);
    for name in [
        "send-hint",
        "completions",
        "parses-fast",
        "deny-write",
        "worker1",
        "managers",
        "pmx",
        "advisors",
    ] {
        // Past the name gate is what's tested; max_workers may still say 409.
        if let Err(e) = mc.spawn(&req("worker", name)).await {
            assert!(
                matches!(e, bridle_api::ClientError::Api { status: 409, .. }),
                "{name}: {e:?}"
            );
        }
    }
}

#[tokio::test]
async fn resume_and_renew_of_reserved_names_are_not_blocked() {
    let (daemon, _tmp) = start_daemon(None).await;
    let orch = daemon.external_client("orchestrator").await;
    for name in ["manager-2", "pm-1"] {
        let a = daemon
            .client
            .spawn(&req("manager", name))
            .await
            .expect("spawn");
        wait_for_state(&daemon.client, &a.id, AgentState::Idle).await;
        daemon
            .client
            .stop(&a.id, &StopRequest { now: true })
            .await
            .expect("stop");
        wait_for_state(&daemon.client, &a.id, AgentState::Stopped).await;
        orch.renew(
            &a.id,
            &RenewRequest {
                ignore_budget: false,
            },
        )
        .await
        .expect("renew");
        wait_for_state(&daemon.client, &a.id, AgentState::Idle).await;
        daemon
            .client
            .stop(&a.id, &StopRequest { now: true })
            .await
            .expect("stop");
        wait_for_state(&daemon.client, &a.id, AgentState::Stopped).await;
        daemon
            .client
            .resume(
                &a.id,
                &ResumeRequest {
                    ignore_budget: false,
                },
            )
            .await
            .expect("resume");
    }
}
