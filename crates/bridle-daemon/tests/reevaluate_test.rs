//! Landing an arch-revision opens one re-evaluate task per capability with suspect
//! requirements, once (traceability.md).

mod support;

use bridle_api::types::{DoneTaskRequest, NewTaskRequest, TaskKind};
use support::start_daemon;

const SPEC: &str = "## Requirements\n\n### Requirement: R {#r-000N traces=a-12cd@0000}\nSHALL.\n\n#### Scenario: S {#s-000N}\n\n*Verification*: **non-executable**\n\nprose\n";

fn git(repo: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8(out.stdout)
        .expect("utf8")
        .trim()
        .to_string()
}

async fn land(daemon: &support::TestDaemon, kind: TaskKind, title: &str) -> String {
    let t = daemon
        .client
        .new_task(&NewTaskRequest {
            for_human: false,
            components: Vec::new(),
            title: title.to_string(),
            kind,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new task");
    let commit = git(&daemon.repo, &["rev-parse", "HEAD"]);
    daemon
        .client
        .done_task(
            &t.id,
            &DoneTaskRequest {
                commit,
                branch: None,
            },
        )
        .await
        .expect("done");
    t.id
}

async fn reevaluate_titles(daemon: &support::TestDaemon) -> Vec<String> {
    let mut v: Vec<String> = daemon
        .client
        .list_tasks()
        .await
        .expect("list")
        .into_iter()
        .filter(|t| t.kind == TaskKind::ReEvaluate)
        .map(|t| t.title)
        .collect();
    v.sort();
    v
}

#[tokio::test]
async fn arch_revision_opens_one_task_per_capability_once() {
    let (daemon, _tmp) = start_daemon(None).await;
    let d = daemon.repo.join("design");
    for sub in ["goals", "architecture", "specs"] {
        std::fs::create_dir_all(d.join(sub)).expect("mkdir");
    }
    std::fs::write(
        d.join("architecture/a.md"),
        "## Engine {#a-12cd}\nDecides.\n",
    )
    .expect("arch");
    for (cap, n) in [("alpha", "1"), ("beta", "2")] {
        std::fs::write(d.join(format!("specs/{cap}.md")), SPEC.replace('N', n)).expect("spec");
    }
    git(&daemon.repo, &["add", "."]);
    git(&daemon.repo, &["commit", "-m", "design"]);

    // A non-arch landing opens nothing.
    land(&daemon, TaskKind::Feature, "feature").await;
    assert!(reevaluate_titles(&daemon).await.is_empty());

    let arch = land(&daemon, TaskKind::ArchRevision, "revise").await;
    assert_eq!(
        reevaluate_titles(&daemon).await,
        vec![
            format!("re-evaluate alpha after {arch}"),
            format!("re-evaluate beta after {arch}"),
        ]
    );
    let tasks = daemon.client.list_tasks().await.expect("list");
    let t = tasks
        .iter()
        .find(|t| t.title.starts_with("re-evaluate alpha"))
        .expect("alpha");
    assert!(t.body.contains("r-0001"), "body: {}", t.body);
    assert!(t.body.contains("bridle trace confirm"));
}
