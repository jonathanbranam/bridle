//! `bridle daemon restart --cancel` / `--postpone` (br-cvfu): hold a pending drain while a land
//! runs. Plain restarts use a busy agent as the thing the drain waits on; the upgrade case uses
//! the canned CI and build of `upgrade_test`.

mod support;

use std::sync::Arc;
use std::time::Duration;

use bridle_api::types::{
    AgentState, MessageKind, MessageState, RestartHoldRequest, RestartRequest, SendRequest,
    SpawnRequest, When, Workdir,
};
use bridle_daemon::UpgradeHooks;
use bridle_daemon::ci::{Gh, Run};

fn worker(prompt: &str) -> SpawnRequest {
    SpawnRequest {
        ticket: None,
        components: Vec::new(),
        role: "worker".to_string(),
        name: Some("busy".to_string()),
        prompt: Some(prompt.to_string()),
        workdir: Some(Workdir::Repo),
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
    }
}

fn cancel() -> RestartHoldRequest {
    RestartHoldRequest {
        cancel: true,
        postpone_secs: None,
    }
}

fn postpone(secs: u64) -> RestartHoldRequest {
    RestartHoldRequest {
        cancel: false,
        postpone_secs: Some(secs),
    }
}

#[tokio::test]
async fn cancel_ends_a_pending_drain_without_restarting() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&worker("SLEEP 3"))
        .await
        .expect("spawn");
    support::wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;
    let orch = daemon.external_client("orchestrator").await;
    let o2 = orch.clone();
    let asked = tokio::spawn(async move { o2.restart(&RestartRequest::default()).await });
    support::wait_for("the drain", || async {
        daemon.client.status().await.ok()?.draining.then_some(())
    })
    .await;
    orch.restart_hold(&cancel()).await.expect("cancel accepted");
    assert!(
        asked.await.expect("task").is_err(),
        "the restart was cancelled"
    );
    support::wait_for("the drain to lift", || async {
        (!daemon.client.status().await.ok()?.draining).then_some(())
    })
    .await;
    assert!(!daemon.running.restart_requested());
    // Nothing is pending now.
    assert!(orch.restart_hold(&cancel()).await.is_err());
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn only_the_human_orchestrator_and_managers_may_hold() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let other = daemon.external_client("someone-else").await;
    assert!(other.restart_hold(&postpone(60)).await.is_err());
    let w = daemon.client.spawn(&worker("")).await.expect("spawn");
    assert!(
        daemon
            .agent_client(&w.id)
            .restart_hold(&postpone(60))
            .await
            .is_err()
    );
    assert!(
        daemon
            .client
            .status()
            .await
            .expect("status")
            .postponed_until
            .is_none()
    );
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_postponement_refuses_new_restarts_and_shows_in_status() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let orch = daemon.external_client("orchestrator").await;
    orch.restart_hold(&postpone(2)).await.expect("postponed");
    assert!(
        daemon
            .client
            .status()
            .await
            .expect("status")
            .postponed_until
            .is_some()
    );
    let err = orch
        .restart(&RestartRequest::default())
        .await
        .expect_err("refused while postponed");
    assert!(err.to_string().contains("postponed"), "{err}");
    tokio::time::sleep(Duration::from_millis(2200)).await;
    assert!(
        daemon
            .client
            .status()
            .await
            .expect("status")
            .postponed_until
            .is_none()
    );
    orch.restart(&RestartRequest::default())
        .await
        .expect("allowed once expired");
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_postponement_delays_a_pending_drain_past_its_quiet_point() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&worker("SLEEP 2"))
        .await
        .expect("spawn");
    support::wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;
    let orch = daemon.external_client("orchestrator").await;
    let o2 = orch.clone();
    let asked = tokio::spawn(async move { o2.restart(&RestartRequest::default()).await });
    support::wait_for("the drain", || async {
        daemon.client.status().await.ok()?.draining.then_some(())
    })
    .await;
    orch.restart_hold(&postpone(5)).await.expect("postponed");
    support::wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert!(!daemon.running.restart_requested(), "quiet but postponed");
    asked.await.expect("task").expect("restart after expiry");
    assert!(daemon.running.restart_requested());
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_postponed_drain_still_delivers_messages_to_an_idle_agent() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let busy = daemon
        .client
        .spawn(&worker("SLEEP 4"))
        .await
        .expect("spawn");
    support::wait_for_state(&daemon.client, &busy.id, AgentState::Working).await;
    let mut idle_req = worker("");
    idle_req.name = Some("idle".to_string());
    let idle = daemon.client.spawn(&idle_req).await.expect("spawn");
    support::wait_for_state(&daemon.client, &idle.id, AgentState::Idle).await;
    let orch = daemon.external_client("orchestrator").await;
    let o2 = orch.clone();
    let asked = tokio::spawn(async move { o2.restart(&RestartRequest::default()).await });
    support::wait_for("the drain", || async {
        daemon.client.status().await.ok()?.draining.then_some(())
    })
    .await;
    orch.restart_hold(&postpone(30)).await.expect("postponed");
    let sent = daemon
        .client
        .send_to_agent(
            &idle.id,
            &SendRequest {
                to: None,
                body: "built".to_string(),
                kind: MessageKind::Note,
                when: When::Idle,
                reply_to: None,
                task: None,
            },
        )
        .await
        .expect("send");
    assert_ne!(
        sent.state,
        MessageState::Held,
        "a postponed drain holds nothing"
    );
    orch.restart_hold(&cancel()).await.expect("cancel");
    let _ = asked.await;
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

struct GreenGh;

impl Gh for GreenGh {
    fn branch_runs(&self, _: &str) -> Result<Vec<Run>, String> {
        Ok(Vec::new())
    }
    fn runs(&self, _: &str) -> Result<Vec<Run>, String> {
        Ok(vec![Run {
            database_id: 1,
            status: "completed".to_string(),
            conclusion: "success".to_string(),
            url: String::new(),
            head_sha: String::new(),
        }])
    }
    fn failed_jobs(&self, _: u64) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
}

#[tokio::test]
async fn cancel_ends_an_upgrade_drain_quietly() {
    let mut o = support::default_overrides();
    o.upgrade = UpgradeHooks {
        gh: Some(Arc::new(GreenGh)),
        build: Some(vec!["true".to_string()]),
        preflight: None,
        releases: None,
    };
    o.ci_tick_interval = Duration::from_millis(100);
    let (daemon, _tmp) =
        support::start_daemon_with_config(Some(o), Some("[daemon]\nself_upgrade = true\n")).await;
    // The 100 ms tick would drain before the agent's first turn starts and the agent would never
    // be Working (br-6byg): hold upgrades off until it is, then let the tick through.
    daemon
        .client
        .restart_hold(&postpone(600))
        .await
        .expect("postpone");
    let agent = daemon
        .client
        .spawn(&worker("SLEEP 4"))
        .await
        .expect("spawn");
    support::wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;
    daemon
        .client
        .restart_hold(&postpone(0))
        .await
        .expect("lift postpone");
    support::wait_for("the upgrade drain", || async {
        let s = daemon.client.status().await.ok()?;
        (s.draining && s.upgrade_waiting.is_some()).then_some(())
    })
    .await;
    // The cancelled upgrade's built binary stays for the next cycle, so the 100 ms tick would
    // start a new drain at once and a slow poller could miss the gap (br-6byg). Postpone first so
    // nothing re-drains once the cancel lifts this one.
    daemon
        .client
        .restart_hold(&postpone(600))
        .await
        .expect("postpone");
    daemon.client.restart_hold(&cancel()).await.expect("cancel");
    support::wait_for("the drain to lift", || async {
        let s = daemon.client.status().await.ok()?;
        (!s.draining && s.upgrade_waiting.is_none()).then_some(())
    })
    .await;
    support::wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    assert!(!daemon.running.restart_requested());
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

/// br-2yxm: a manager postpones before every land; the postponement must end with the land, or
/// back-to-back lands keep a pending self-upgrade out indefinitely.
#[tokio::test]
async fn a_land_ends_the_postponement_it_was_made_for() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let orch = daemon.external_client("orchestrator").await;
    for _ in 0..2 {
        orch.restart_hold(&postpone(2700)).await.expect("postponed");
        assert!(
            daemon
                .client
                .status()
                .await
                .expect("status")
                .postponed_until
                .is_some()
        );
        // The land fails (no such ticket); any end of a land lifts the postponement.
        let _ = daemon
            .client
            .land_task("br-nope", &bridle_api::types::LandRequest::default())
            .await;
        assert!(
            daemon
                .client
                .status()
                .await
                .expect("status")
                .postponed_until
                .is_none(),
            "the land's end lifts the postponement"
        );
    }
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}
