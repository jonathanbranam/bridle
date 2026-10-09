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
    RestartRequest { upgrade: true }
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
async fn self_upgrade_restarts_only_after_the_mid_turn_agent_finishes() {
    // The build waits for a go file, so the upgrade (and its drain) cannot begin until the agent
    // is seen mid-turn. Without it the drain could start while the spawn was in flight, hold the
    // agent's first prompt, and leave nothing to wait on (the flake of ticket h7gt, run
    // 37954619068).
    let build = "while [ ! -f \"$CARGO_TARGET_DIR.go\" ]; do sleep 0.05; done; \
                 echo ran > \"$CARGO_TARGET_DIR.txt\"";
    let (daemon, _tmp) =
        support::start_daemon_with_config(Some(auto(build)), Some(SELF_UPGRADE)).await;
    let agent = daemon
        .client
        .spawn(&worker("SLEEP 3"))
        .await
        .expect("spawn");
    support::wait_for("the agent to work", || async {
        let a = daemon.client.get_agent(&agent.id).await.ok()?;
        (a.state == bridle_api::types::AgentState::Working).then_some(())
    })
    .await;
    std::fs::write(daemon.workspace.join(".bridle/upgrade-target.go"), "").expect("go file");
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

fn spawn_named(name: &str, prompt: &str) -> bridle_api::types::SpawnRequest {
    let mut r = worker(prompt);
    r.name = Some(name.to_string());
    r
}

fn say(body: &str) -> bridle_api::types::SendRequest {
    bridle_api::types::SendRequest {
        to: None,
        body: body.to_string(),
        kind: bridle_api::types::MessageKind::Note,
        when: bridle_api::types::When::Now,
        reply_to: None,
        task: None,
    }
}

async fn messages_to(client: &Client, id: &str) -> Vec<bridle_api::types::Message> {
    client
        .list_messages(&MessageQuery {
            to: Some(id.to_string()),
            ..Default::default()
        })
        .await
        .expect("messages")
}

#[tokio::test]
async fn self_upgrade_starts_the_build_while_agents_are_busy() {
    let (daemon, _tmp) =
        support::start_daemon_with_config(Some(auto("sleep 1")), Some(SELF_UPGRADE)).await;
    let agent = daemon
        .client
        .spawn(&worker("SLEEP 6"))
        .await
        .expect("spawn");
    support::wait_for_state(
        &daemon.client,
        &agent.id,
        bridle_api::types::AgentState::Working,
    )
    .await;
    support::wait_for("the build to start", || async {
        upgrade_events(&daemon)
            .await
            .contains(&"upgrade.building".to_string())
            .then_some(())
    })
    .await;
    let a = daemon.client.get_agent(&agent.id).await.expect("agent");
    assert_eq!(a.state, bridle_api::types::AgentState::Working);
    assert!(!daemon.running.restart_requested());
    support::wait_for("the restart after the turn", || async {
        daemon.running.restart_requested().then_some(())
    })
    .await;
    daemon.running.join().await.expect("join");
}

/// A drain holds the new turns (messages to a busy and to an idle agent), refuses spawns and
/// claims, shows what it waits on, restarts the moment the turn ends with no timeout, and the
/// held messages are delivered after the resume, in order, none lost or duplicated.
#[tokio::test]
async fn a_drain_holds_new_turns_and_delivers_them_after_the_restart() {
    let (daemon, tmp) = support::start_daemon(Some(hooks("success", "true"))).await;
    let busy = daemon
        .client
        .spawn(&spawn_named("busy", "SLEEP 60"))
        .await
        .expect("spawn");
    let idle = daemon
        .client
        .spawn(&spawn_named("idle", "hi"))
        .await
        .expect("spawn");
    support::wait_for_state(
        &daemon.client,
        &busy.id,
        bridle_api::types::AgentState::Working,
    )
    .await;
    support::wait_for_state(
        &daemon.client,
        &idle.id,
        bridle_api::types::AgentState::Idle,
    )
    .await;
    daemon.client.restart(&upgrade()).await.expect("reply");
    let status = support::wait_for("the drain in status", || async {
        let s = daemon.client.status().await.ok()?;
        s.draining.then_some(s)
    })
    .await;
    assert!(status.upgrade_waiting.is_some());
    assert_eq!(status.draining_on, vec!["busy".to_string()]);

    // Refused while draining: spawns and claims.
    let err = daemon
        .client
        .spawn(&spawn_named("late", "hi"))
        .await
        .expect_err("refused");
    assert!(err.to_string().contains("draining"), "{err}");
    let err = daemon
        .client
        .claim_task("t-0000")
        .await
        .expect_err("refused");
    assert!(err.to_string().contains("draining"), "{err}");

    // New turns are held, not delivered.
    let m1 = daemon
        .client
        .send_to_agent(&busy.id, &say("first for busy"))
        .await
        .expect("send");
    let m2 = daemon
        .client
        .send_to_agent(&idle.id, &say("for idle"))
        .await
        .expect("send");
    let m3 = daemon
        .client
        .send_to_agent(&busy.id, &say("second for busy"))
        .await
        .expect("send");
    for m in [&m1, &m2, &m3] {
        assert_eq!(m.state, bridle_api::types::MessageState::Held, "{m:?}");
    }
    assert!(!daemon.running.restart_requested());

    // The turn ends (interrupted, so a loaded machine can't end it before the drain checks
    // above); no timeout, the restart follows by itself.
    daemon
        .client
        .interrupt(
            &busy.id,
            &bridle_api::types::InterruptRequest { drop_held: false },
        )
        .await
        .expect("interrupt");
    support::wait_for("the restart", || async {
        daemon.running.restart_requested().then_some(())
    })
    .await;
    let (workspace, repo) = (daemon.workspace.clone(), daemon.repo.clone());
    daemon.running.join().await.expect("join");

    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = hooks("success", "true");
    overrides.bridle_home = Some(support::machine_home_dir(tmp.path()));
    let running = bridle_daemon::start(opts, overrides).await.expect("start");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = Client::new(running.url.clone(), Some(token.trim().to_string()));
    let delivered = |s: &bridle_api::types::MessageState| {
        !matches!(
            s,
            bridle_api::types::MessageState::Held | bridle_api::types::MessageState::Pending
        )
    };
    let on_busy = support::wait_for("the held messages reach busy", || async {
        let ms: Vec<_> = messages_to(&client, &busy.id)
            .await
            .into_iter()
            .filter(|m| m.body.contains("for busy"))
            .collect();
        (ms.len() == 2 && ms.iter().all(|m| delivered(&m.state))).then_some(ms)
    })
    .await;
    // Once each, and in the order they were sent.
    assert_eq!(on_busy[0].id, m1.id);
    assert_eq!(on_busy[1].id, m3.id);
    assert!(on_busy[0].written_at <= on_busy[1].written_at);
    support::wait_for("the held message reaches idle", || async {
        let ms: Vec<_> = messages_to(&client, &idle.id)
            .await
            .into_iter()
            .filter(|m| m.id == m2.id)
            .collect();
        (ms.len() == 1 && delivered(&ms[0].state)).then_some(())
    })
    .await;
    running.shutdown();
    running.join().await.expect("join");
}

/// The drain's one-hour wake (a short stand-in for the hour here) fires once, naming who is
/// still in a turn.
#[tokio::test]
async fn a_long_drain_wakes_the_orchestrator_once() {
    let mut o = hooks("success", "true");
    o.drain_wake_after = Duration::from_secs(1);
    let (daemon, _tmp) = support::start_daemon(Some(o)).await;
    let agent = daemon
        .client
        .spawn(&spawn_named("slow", "SLEEP 60"))
        .await
        .expect("spawn");
    support::wait_for_state(
        &daemon.client,
        &agent.id,
        bridle_api::types::AgentState::Working,
    )
    .await;
    daemon.client.restart(&upgrade()).await.expect("reply");
    let orch = daemon.external_client("orchestrator").await;
    // The upgrade builds before it drains, so the drain's clock starts late on a loaded machine:
    // wait for the wake itself (generous bound), not for the restart.
    let mut seen = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while seen.is_empty() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "no upgrade_draining wake"
        );
        let wakes = orch.orchestrator_wake(Some(1)).await.expect("wake").wakes;
        seen.extend(wakes.into_iter().filter(|w| w.reason == "upgrade_draining"));
    }
    // End the turn (interrupted, not timed) so the restart follows; no second wake may come.
    daemon
        .client
        .interrupt(
            &agent.id,
            &bridle_api::types::InterruptRequest { drop_held: false },
        )
        .await
        .expect("interrupt");
    while !daemon.running.restart_requested() {
        let wakes = orch.orchestrator_wake(Some(1)).await.expect("wake").wakes;
        seen.extend(wakes.into_iter().filter(|w| w.reason == "upgrade_draining"));
    }
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert!(seen[0].text.contains("slow"), "{:?}", seen[0]);
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_daemon_raised_wake_also_comes_through_agent_wake() {
    let (daemon, _tmp) =
        support::start_daemon(Some(hooks("success", "echo boom >&2; exit 3"))).await;
    daemon.client.restart(&upgrade()).await.expect("reply");
    let orch = daemon.external_client("orchestrator").await;
    let got = orch
        .principal_wake(&bridle_api::types::PrincipalWakeQuery {
            principal: "external:orchestrator".to_string(),
            timeout_secs: Some(5),
            session: None,
        })
        .await
        .expect("wake")
        .reasons;
    let failed = got.iter().find(|r| r.reason == "upgrade_failed");
    assert!(failed.is_some_and(|r| r.detail.is_some()), "{got:?}");
}

/// A drain that begins while a spawn is in flight holds the agent's first prompt, so no init will
/// ever come: the spawn must not sit out its readiness wait (8 s), and the restart must come at
/// once, with the held prompt delivered after it (ticket b6mu; no go-file gate, unlike the
/// mid-turn test above).
#[tokio::test]
async fn a_drain_starting_during_a_spawn_restarts_promptly() {
    let config = "[worktrees]\nsetup = \"sleep 2\"\n";
    let (daemon, _tmp) =
        support::start_daemon_with_config(Some(hooks("success", "true")), Some(config)).await;
    let mut req = spawn_named("racer", "hello");
    req.workdir = None;
    let client = daemon.client.clone();
    let spawn = tokio::spawn(async move { client.spawn(&req).await });
    // The spawn is inside its worktree setup (the drain check at its start has passed).
    tokio::time::sleep(Duration::from_millis(700)).await;
    let began = std::time::Instant::now();
    daemon.client.restart(&upgrade()).await.expect("reply");
    support::wait_for("the restart", || async {
        daemon.running.restart_requested().then_some(())
    })
    .await;
    let spawned = spawn.await.expect("join").expect("spawn");
    assert!(
        began.elapsed() < Duration::from_secs(6),
        "the drain waited out the spawn's readiness wait: {:?}",
        began.elapsed()
    );
    let held = messages_to(&daemon.client, &spawned.id).await;
    assert!(held.iter().any(|m| m.body == "hello"), "{held:?}");
    daemon.running.join().await.expect("join");
}
