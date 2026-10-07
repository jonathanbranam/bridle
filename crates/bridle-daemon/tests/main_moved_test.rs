//! A landing tells the other running workers that main moved
//! (impact-and-conflicts.md, conflict protocol step 4): one message each,
//! none to the agent whose task landed.

mod support;
use support::ClientExt as _;

use bridle_api::types::{
    AgentState, DoneTaskRequest, MessageQuery, NewTaskRequest, SpawnRequest, TaskKind, Workdir,
};
use support::{start_daemon, wait_for_state};

async fn worker(daemon: &support::TestDaemon, name: &str) -> String {
    let a = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some(name.to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &a.id, AgentState::Idle).await;
    a.id
}

async fn notices(daemon: &support::TestDaemon, agent: &str) -> Vec<String> {
    daemon
        .client
        .list_messages(&MessageQuery {
            to: Some(agent.to_string()),
            from: Some("system".to_string()),
            unread: false,
            limit: None,
            ..Default::default()
        })
        .await
        .expect("messages")
        .into_iter()
        .map(|m| m.body)
        .collect()
}

#[tokio::test]
async fn landing_notifies_the_other_worker_once_and_not_the_lander() {
    let (daemon, _tmp) = start_daemon(None).await;
    let lander = worker(&daemon, "w1").await;
    let other = worker(&daemon, "w2").await;
    let task = daemon
        .client
        .new_open_task(&NewTaskRequest {
            ticket: None,
            parent: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: "Landed".to_string(),
            kind: TaskKind::Feature,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new task");
    let head = String::from_utf8(
        std::process::Command::new("git")
            .arg("-C")
            .arg(&daemon.repo)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("rev-parse")
            .stdout,
    )
    .expect("utf8")
    .trim()
    .to_string();
    daemon
        .client
        .done_task(
            &task.id,
            &DoneTaskRequest {
                commit: head.clone(),
                branch: Some("bridle/w1".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("done");

    let bodies = notices(&daemon, &other).await;
    assert_eq!(bodies.len(), 1, "got {bodies:?}");
    assert!(bodies[0].contains("main moved"), "got {bodies:?}");
    assert!(bodies[0].contains(&task.id), "got {bodies:?}");
    assert!(bodies[0].contains(&head), "got {bodies:?}");
    assert!(notices(&daemon, &lander).await.is_empty());
}

fn git(daemon: &support::TestDaemon, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(&daemon.repo)
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8(out.stdout)
        .expect("utf8")
        .trim()
        .to_string()
}

#[tokio::test]
async fn landing_names_the_overlap_only_to_the_worker_whose_impact_overlaps() {
    use bridle_api::types::{Impact, SetImpactRequest};
    let (daemon, _tmp) = start_daemon(None).await;
    let w2 = worker(&daemon, "w2").await;
    let w3 = worker(&daemon, "w3").await;
    let mut tasks = Vec::new();
    for (title, glob, agent) in [
        ("Lands", "", None),
        ("A", "src/a/**", Some(&w2)),
        ("B", "docs/**", Some(&w3)),
    ] {
        let t = daemon
            .client
            .new_open_task(&NewTaskRequest {
                ticket: None,
                parent: None,
                for_human: false,
                priority: None,
                components: Vec::new(),
                title: title.to_string(),
                kind: TaskKind::Feature,
                body: String::new(),
                size: None,
            })
            .await
            .expect("new task");
        if let Some(agent) = agent {
            let impact = Impact {
                files: vec![glob.to_string()],
                ..Impact::default()
            };
            daemon
                .client
                .set_task_impact(&t.id, &SetImpactRequest { impact })
                .await
                .expect("impact");
            daemon.client.plan_task(&t.id).await.expect("plan");
            daemon
                .agent_client(agent)
                .claim_task(&t.id)
                .await
                .expect("claim");
        }
        tasks.push(t);
    }
    std::fs::create_dir_all(daemon.repo.join("src/a")).expect("mkdir");
    std::fs::write(daemon.repo.join("src/a/x.rs"), "fn x() {}\n").expect("write");
    git(&daemon, &["add", "."]);
    git(&daemon, &["commit", "-m", "touch src/a"]);
    let head = git(&daemon, &["rev-parse", "HEAD"]);
    daemon
        .client
        .done_task(
            &tasks[0].id,
            &DoneTaskRequest {
                commit: head,
                branch: None,
                ..Default::default()
            },
        )
        .await
        .expect("done");

    let overlapping = notices(&daemon, &w2).await;
    assert_eq!(overlapping.len(), 1, "got {overlapping:?}");
    assert!(
        overlapping[0].contains("spec changed under you: src/a/x.rs"),
        "got {overlapping:?}"
    );
    let other = notices(&daemon, &w3).await;
    assert_eq!(other.len(), 1, "got {other:?}");
    assert!(other[0].contains("main moved"), "got {other:?}");
    assert!(
        !other[0].contains("spec changed under you"),
        "got {other:?}"
    );
}
