//! br-btdn / incident y455: `AgentManager::spawning()` must be false once every spawn has
//! returned, whatever happened to the agent meanwhile. A stuck-true flag makes every restart and
//! self-upgrade wait for a quiet point that never comes ("still busy: a spawning agent").
//!
//! The postmortem (docs/tickets/open/incident-bridle-s-daemon-couldn-t-restart-or-self-upgrade-fo-y455.md)
//! found that a stop during spawn is NOT the leak: this passes today and guards that path. The
//! leak itself needs a spawn future that neither finishes nor is dropped, which no test here
//! reproduces yet.

mod support;

use bridle_api::types::{RestartRequest, SpawnRequest, StopRequest, Workdir};

#[tokio::test]
async fn a_stop_during_spawn_leaves_no_spawning_flag() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let req = SpawnRequest {
        components: Vec::new(),
        role: "worker".to_string(),
        name: Some("stopped-early".to_string()),
        prompt: Some("hello".to_string()),
        workdir: Some(Workdir::Repo),
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
    };
    // Stop it the moment it exists, as manager-2 did 3 s after agent.spawned.
    let stopper = {
        let client = daemon.client.clone();
        async move {
            let id = support::wait_for("the agent row", || {
                let client = client.clone();
                async move {
                    client
                        .list_agents()
                        .await
                        .ok()?
                        .into_iter()
                        .find(|a| a.name == "stopped-early")
                        .map(|a| a.id)
                }
            })
            .await;
            let _ = client.stop(&id, &StopRequest::default()).await;
        }
    };
    let (spawned, ()) = tokio::join!(daemon.client.spawn(&req), stopper);
    let _ = spawned;

    let orch = daemon.external_client("orchestrator").await;
    let outcome = orch.restart(&RestartRequest::default()).await;
    assert!(
        outcome.is_ok(),
        "restart refused after the spawn returned: {outcome:?}"
    );
    daemon.running.join().await.expect("join");
}
