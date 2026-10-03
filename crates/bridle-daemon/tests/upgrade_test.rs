//! `restart --upgrade` (docs/design/agent-host/daemon.md, "Upgrade"), with canned CI status and
//! a stand-in build command: never a real `cargo install`.

mod support;

use std::sync::Arc;
use std::time::Duration;

use bridle_api::Client;
use bridle_api::types::{MessageQuery, RestartRequest};
use bridle_daemon::UpgradeHooks;
use bridle_daemon::ci::{Gh, Run};

/// Every commit has one finished run with this conclusion.
struct FakeGh(&'static str);

impl Gh for FakeGh {
    fn remote_tip(&self, _: &str) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn runs(&self, _: &str) -> Result<Vec<Run>, String> {
        Ok(vec![Run {
            database_id: 1,
            status: "completed".to_string(),
            conclusion: self.0.to_string(),
            url: String::new(),
        }])
    }
    fn failed_jobs(&self, _: u64) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
}

fn hooks(conclusion: &'static str, build: &str) -> bridle_daemon::Overrides {
    let mut o = support::default_overrides();
    o.upgrade = UpgradeHooks {
        gh: Some(Arc::new(FakeGh(conclusion))),
        build: Some(vec!["sh".to_string(), "-c".to_string(), build.to_string()]),
        preflight: None,
    };
    o
}

fn upgrade() -> RestartRequest {
    RestartRequest {
        wait_secs: Some(30),
        upgrade: true,
    }
}

#[tokio::test]
async fn red_ci_means_nothing_to_build() {
    let marker = "echo ran > \"$CARGO_TARGET_DIR.txt\"";
    let (daemon, _tmp) = support::start_daemon(Some(hooks("failure", marker))).await;
    let reply = daemon.client.restart(&upgrade()).await.expect("reply");
    assert!(!reply.restarting);
    assert!(reply.message.unwrap().contains("nothing to upgrade"));
    assert!(!daemon.running.restart_requested());
    assert!(!daemon.workspace.join(".bridle/upgrade-target.txt").exists());
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_green_commit_is_built_then_restarted_and_not_built_twice() {
    // Records the commit it was run on, so the test sees the build got that checkout.
    let build = "git rev-parse HEAD > \"$CARGO_TARGET_DIR.txt\"";
    let (daemon, tmp) = support::start_daemon(Some(hooks("success", build))).await;
    let reply = daemon.client.restart(&upgrade()).await.expect("reply");
    assert!(!reply.restarting, "the reply comes before the build");
    assert!(reply.message.unwrap().starts_with("building "));
    support::wait_for("the restart", || async {
        daemon.running.restart_requested().then_some(())
    })
    .await;
    let head = tokio::process::Command::new("git")
        .arg("-C")
        .arg(&daemon.repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .await
        .expect("git");
    let built = std::fs::read_to_string(daemon.workspace.join(".bridle/upgrade-target.txt"))
        .expect("the build ran");
    assert_eq!(built.trim(), String::from_utf8_lossy(&head.stdout).trim());
    assert!(!daemon.workspace.join(".bridle/upgrade-src").exists());

    let (workspace, repo) = (daemon.workspace.clone(), daemon.repo.clone());
    daemon.running.join().await.expect("join");
    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = hooks("success", build);
    overrides.bridle_home = Some(support::machine_home_dir(tmp.path()));
    let running = bridle_daemon::start(opts, overrides).await.expect("start");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));
    let reply = client.restart(&upgrade()).await.expect("reply");
    assert!(reply.message.unwrap().contains("nothing to upgrade"));
    running.shutdown();
    running.join().await.expect("join");
}

#[tokio::test]
async fn a_failed_build_leaves_the_daemon_running_and_tells_the_human() {
    let (daemon, _tmp) =
        support::start_daemon(Some(hooks("success", "echo boom >&2; exit 3"))).await;
    let reply = daemon.client.restart(&upgrade()).await.expect("reply");
    assert!(reply.message.unwrap().starts_with("building "));
    let note = support::wait_for("the failure note", || async {
        daemon
            .client
            .list_messages(&MessageQuery {
                to: Some("human".to_string()),
                ..Default::default()
            })
            .await
            .ok()?
            .into_iter()
            .find(|m| m.body.contains("build of") && m.body.contains("boom"))
    })
    .await;
    assert!(note.body.contains("the daemon is unchanged"));
    // A real failure still wakes the orchestrator.
    let orch = daemon.external_client("orchestrator").await;
    let wakes = orch.orchestrator_wake(Some(5)).await.expect("wake").wakes;
    assert!(
        wakes.iter().any(|w| w.reason == "upgrade_failed"),
        "{wakes:?}"
    );
    let ev = upgrade_events(&daemon).await;
    assert!(ev.contains(&"upgrade.building".to_string()), "{ev:?}");
    assert!(ev.contains(&"upgrade.failed".to_string()), "{ev:?}");
    assert!(!daemon.running.restart_requested());
    daemon.client.health().await.expect("still serving");
    // The slot is free again: a second try builds (and fails) rather than being refused.
    tokio::time::sleep(Duration::from_millis(100)).await;
    let again = daemon.client.restart(&upgrade()).await.expect("reply");
    assert!(again.message.unwrap().starts_with("building "));
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

/// `hooks` with the CI tick (which carries the self-upgrade check) running every 100 ms.
fn auto(build: &str) -> bridle_daemon::Overrides {
    let mut o = hooks("success", build);
    o.ci_tick_interval = Duration::from_millis(100);
    o
}

const SELF_UPGRADE: &str = "[daemon]\nself_upgrade = true\n";

fn worker(prompt: &str) -> bridle_api::types::SpawnRequest {
    bridle_api::types::SpawnRequest {
        components: Vec::new(),
        role: "worker".to_string(),
        name: Some("w".to_string()),
        prompt: Some(prompt.to_string()),
        workdir: Some(bridle_api::types::Workdir::Repo),
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
    }
}

#[tokio::test]
async fn self_upgrade_off_does_nothing() {
    let marker = "echo ran > \"$CARGO_TARGET_DIR.txt\"";
    let (daemon, _tmp) = support::start_daemon(Some(auto(marker))).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(!daemon.running.restart_requested());
    assert!(!daemon.workspace.join(".bridle/upgrade-target.txt").exists());
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn self_upgrade_at_a_quiet_point_builds_and_restarts_once() {
    // Appends, so a second build would show as a second line.
    let build = "echo ran >> \"$CARGO_TARGET_DIR.txt\"";
    let (daemon, _tmp) =
        support::start_daemon_with_config(Some(auto(build)), Some(SELF_UPGRADE)).await;
    support::wait_for("the restart", || async {
        daemon.running.restart_requested().then_some(())
    })
    .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let log = std::fs::read_to_string(daemon.workspace.join(".bridle/upgrade-target.txt"))
        .expect("the build ran");
    assert_eq!(log.lines().count(), 1);
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn self_upgrade_waits_while_an_agent_is_mid_turn() {
    let marker = "echo ran > \"$CARGO_TARGET_DIR.txt\"";
    let (daemon, _tmp) =
        support::start_daemon_with_config(Some(auto(marker)), Some(SELF_UPGRADE)).await;
    // Spawned in the same instant the first tick may fire (the tick is 100 ms): the quiet check
    // must count the spawn itself, before the agent reads as working.
    let agent = daemon
        .client
        .spawn(&worker("SLEEP 3"))
        .await
        .expect("spawn");
    support::wait_for_state(
        &daemon.client,
        &agent.id,
        bridle_api::types::AgentState::Working,
    )
    .await;
    // Polls until the turn ends. The flag is read before the state, so a restart seen alongside a
    // still-working agent really did happen mid-turn, however slow the machine is.
    loop {
        let restarted = daemon.running.restart_requested();
        let a = daemon.client.get_agent(&agent.id).await.expect("agent");
        assert!(
            !(restarted && a.state == bridle_api::types::AgentState::Working),
            "restarted while a turn was running"
        );
        if a.state != bridle_api::types::AgentState::Working {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    support::wait_for("the restart after the turn", || async {
        daemon.running.restart_requested().then_some(())
    })
    .await;
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn failed_preflight_leaves_the_daemon_untouched() {
    let mut o = hooks("success", "true");
    o.upgrade.preflight = Some(vec![
        "sh".into(),
        "-c".into(),
        "echo bad config >&2; exit 1".into(),
    ]);
    let (daemon, _tmp) = support::start_daemon(Some(o)).await;
    let reply = daemon.client.restart(&upgrade()).await.expect("reply");
    assert!(reply.message.unwrap().starts_with("building "));
    let note = support::wait_for("the refusal note", || async {
        daemon
            .client
            .list_messages(&MessageQuery {
                to: Some("human".to_string()),
                ..Default::default()
            })
            .await
            .ok()?
            .into_iter()
            .find(|m| m.body.contains("self-check failed"))
    })
    .await;
    assert!(note.body.contains("bad config"));
    assert!(!daemon.running.restart_requested());
    assert!(
        !daemon
            .workspace
            .join(".bridle/upgrade-pending.json")
            .exists()
    );
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_waiting_build_refuses_new_workers_until_it_gives_up() {
    let (daemon, _tmp) = support::start_daemon(Some(hooks("success", "true"))).await;
    let busy = daemon
        .client
        .spawn(&worker("SLEEP 8"))
        .await
        .expect("spawn");
    support::wait_for_state(
        &daemon.client,
        &busy.id,
        bridle_api::types::AgentState::Working,
    )
    .await;
    daemon
        .client
        .restart(&RestartRequest {
            wait_secs: Some(2),
            upgrade: true,
        })
        .await
        .expect("reply");
    support::wait_for("the refusal in status", || async {
        daemon.client.status().await.ok()?.upgrade_waiting
    })
    .await;
    let mut second = worker("hi");
    second.name = Some("w2".to_string());
    let err = daemon.client.spawn(&second).await.expect_err("refused");
    assert!(
        err.to_string().contains("waiting for a quiet point"),
        "{err}"
    );
    // The wait gives up (the worker is still busy): the refusal lifts.
    support::wait_for("the refusal to lift", || async {
        daemon
            .client
            .status()
            .await
            .ok()?
            .upgrade_waiting
            .is_none()
            .then_some(())
    })
    .await;
    assert!(!daemon.running.restart_requested());
    daemon.client.spawn(&second).await.expect("spawn allowed");
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

async fn upgrade_events(daemon: &support::TestDaemon) -> Vec<String> {
    daemon
        .client
        .events(&bridle_api::types::EventQuery {
            kind: Some("upgrade.".to_string()),
            ..Default::default()
        })
        .await
        .expect("events")
        .into_iter()
        .map(|e| e.kind)
        .collect()
}

#[tokio::test]
async fn a_docs_only_commit_is_an_event_and_wakes_no_one() {
    let build = "true";
    let (daemon, tmp) = support::start_daemon(Some(hooks("success", build))).await;
    daemon.client.restart(&upgrade()).await.expect("reply");
    support::wait_for("the restart", || async {
        daemon.running.restart_requested().then_some(())
    })
    .await;
    let (workspace, repo) = (daemon.workspace.clone(), daemon.repo.clone());
    daemon.running.join().await.expect("join");
    std::fs::write(repo.join("NOTES.md"), "docs\n").expect("write");
    for args in [
        vec!["add", "NOTES.md"],
        vec![
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-m",
            "docs",
        ],
    ] {
        let out = tokio::process::Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(&args)
            .output()
            .await
            .expect("git");
        assert!(out.status.success(), "{args:?}");
    }
    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = hooks("success", build);
    overrides.bridle_home = Some(support::machine_home_dir(tmp.path()));
    let running = bridle_daemon::start(opts, overrides).await.expect("start");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));
    let reply = client.restart(&upgrade()).await.expect("reply");
    assert!(reply.message.unwrap().starts_with("building "));
    let kinds = support::wait_for("the skipped event", || async {
        let ev = client
            .events(&bridle_api::types::EventQuery {
                kind: Some("upgrade.skipped".to_string()),
                ..Default::default()
            })
            .await
            .ok()?;
        (!ev.is_empty()).then_some(ev)
    })
    .await;
    assert!(kinds[0].data["commit"].is_string());
    assert!(kinds[0].data["reason"].is_string());
    assert!(!running.restart_requested());
    let orch = Client::new(
        running.url.clone(),
        Some(
            client
                .create_token(&bridle_api::types::TokenCreateRequest {
                    name: "orchestrator".to_string(),
                    machine: None,
                })
                .await
                .expect("token")
                .token,
        ),
    );
    let wakes = orch.orchestrator_wake(Some(1)).await.expect("wake").wakes;
    assert!(
        wakes.iter().all(|w| !w.reason.starts_with("upgrade")),
        "{wakes:?}"
    );
    running.shutdown();
    running.join().await.expect("join");
}

#[tokio::test]
async fn automatic_no_quiet_point_is_silent_and_retried() {
    let mut o = auto("sleep 1");
    o.self_upgrade_wait = Duration::from_secs(1);
    let (daemon, _tmp) = support::start_daemon_with_config(Some(o), Some(SELF_UPGRADE)).await;
    // Busy from during the build, so the post-build wait finds no quiet point.
    support::wait_for("the build to start", || async {
        upgrade_events(&daemon)
            .await
            .contains(&"upgrade.building".to_string())
            .then_some(())
    })
    .await;
    let agent = daemon
        .client
        .spawn(&worker("SLEEP 5"))
        .await
        .expect("spawn");
    support::wait_for("the give-up", || async {
        upgrade_events(&daemon)
            .await
            .contains(&"upgrade.waiting".to_string())
            .then_some(())
    })
    .await;
    assert!(!daemon.running.restart_requested());
    let notes = daemon
        .client
        .list_messages(&MessageQuery {
            to: Some("human".to_string()),
            ..Default::default()
        })
        .await
        .expect("messages");
    assert!(
        notes.iter().all(|m| !m.body.contains("upgrade")),
        "{notes:?}"
    );
    let orch = daemon.external_client("orchestrator").await;
    let wakes = orch.orchestrator_wake(Some(1)).await.expect("wake").wakes;
    assert!(
        wakes.iter().all(|w| !w.reason.starts_with("upgrade")),
        "{wakes:?}"
    );
    let _ = agent;
    // Once the worker is idle the next tick builds again and restarts.
    // Read the events before the restart: once it's requested the server shuts
    // down, and an HTTP read races that. The second build precedes the restart.
    support::wait_for("the retry's build", || async {
        let kinds = upgrade_events(&daemon).await;
        (kinds.iter().filter(|k| *k == "upgrade.building").count() == 2).then_some(())
    })
    .await;
    support::wait_for("the retry's restart", || async {
        daemon.running.restart_requested().then_some(())
    })
    .await;
    daemon.running.join().await.expect("join");
}
