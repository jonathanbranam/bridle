//! The restart drain and lands (br-d7r5, br-eggc): a land past the cap gets exactly one incident
//! and its check is stopped; a land refused while the drain is closed runs once it is open again.
//! The exec itself can't happen in a test process (see restart_in_place_test.rs), and a new
//! daemon's `Lands` starts open, so reopening stands for the restart.

mod support;
use support::ClientExt as _;

use bridle_api::types::{EventQuery, LandRequest, NewTicketRequest, RestartRequest, TicketKind};
use std::time::Duration;

fn git(dir: &std::path::Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

/// A branch off main with one commit, and a feature ticket to land it with.
async fn branch_and_task(d: &support::TestDaemon, branch: &str) -> String {
    git(&d.repo, &["branch", branch, "main"]);
    let wt = d.workspace.join(format!("scratch-{branch}"));
    git(
        &d.repo,
        &["worktree", "add", wt.to_str().expect("utf8"), branch],
    );
    std::fs::write(wt.join("f.txt"), "x\n").expect("write");
    git(&wt, &["add", "."]);
    git(&wt, &["commit", "-qm", "work"]);
    git(
        &d.repo,
        &["worktree", "remove", "--force", wt.to_str().expect("utf8")],
    );
    d.client
        .new_open_task(&NewTicketRequest {
            ticket: None,
            parent: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: "t".to_string(),
            kind: TicketKind::Feature,
            body: String::new(),
            size: None,
        })
        .await
        .expect("task")
        .id
}

fn req(branch: &str, check: Option<&str>) -> LandRequest {
    LandRequest {
        branch: Some(branch.to_string()),
        check_cmd: check.map(str::to_string),
        checked_commit: None,
    }
}

#[tokio::test]
async fn a_land_past_the_cap_gets_one_incident_and_its_check_is_stopped() {
    let (d, tmp) = support::start_daemon(None).await;
    d.running.lands.set_cap(Duration::from_millis(500));
    let id = branch_and_task(&d, "b1").await;
    let land = {
        let client = d.client.clone();
        let id = id.clone();
        tokio::spawn(async move { client.land_task(&id, &req("b1", Some("sleep 300"))).await })
    };
    tokio::time::sleep(Duration::from_millis(1500)).await;
    d.client
        .restart(&RestartRequest::default())
        .await
        .expect("restart accepted");
    let landed = tokio::time::timeout(Duration::from_secs(30), land)
        .await
        .expect("the check was stopped")
        .expect("joined");
    assert!(landed.is_err(), "a stopped check is a failed land");
    let (workspace, repo) = (d.workspace.clone(), d.repo.clone());
    d.running.join().await.expect("join");

    // Read what it recorded from a daemon serving the same data again, as after a failed exec.
    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = support::default_overrides();
    overrides.bridle_home = Some(support::machine_home_dir(tmp.path()));
    let running = bridle_daemon::start_after_failed_restart(opts, overrides, "test")
        .await
        .expect("serves again");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));
    let events = client.events(&EventQuery::default()).await.expect("events");
    running.shutdown();
    running.join().await.expect("join");
    let kinds = |k: &str| events.iter().filter(|e| e.kind == k).count();
    assert_eq!(kinds("restart.land_forced"), 1);
    assert_eq!(
        kinds(bridle_api::types::event_kind::ORCHESTRATOR_INCIDENT),
        1,
        "one incident, not one per drain poll"
    );
}

#[tokio::test]
async fn a_land_refused_during_the_drain_runs_once_it_is_open_again() {
    let (d, _tmp) = support::start_daemon(None).await;
    let id = branch_and_task(&d, "b1").await;
    d.running
        .lands
        .close_if_quiet(Duration::from_secs(60))
        .expect("quiet");
    let err = d
        .client
        .land_task(&id, &req("b1", None))
        .await
        .expect_err("refused while closed");
    let text = err.to_string();
    assert!(
        text.contains("draining") && text.contains("retry"),
        "{text}"
    );
    let main_before = std::process::Command::new("git")
        .args(["-C"])
        .arg(&d.repo)
        .args(["rev-parse", "main"])
        .output()
        .expect("git")
        .stdout;

    d.running.lands.reopen();
    d.client
        .land_task(&id, &req("b1", None))
        .await
        .expect("the retried land runs");
    let main_after = std::process::Command::new("git")
        .args(["-C"])
        .arg(&d.repo)
        .args(["rev-parse", "main"])
        .output()
        .expect("git")
        .stdout;
    assert_ne!(main_before, main_after, "the land moved main");
}
