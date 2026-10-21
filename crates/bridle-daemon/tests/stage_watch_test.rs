//! The stage watch: a pending, unclaimed version 2 ticket gets a designer for its unblocked
//! stage; a claimed one does not; a stage not done in time is reported to the product manager
//! (br-ewhy).

mod support;

use std::time::Duration;

use bridle_api::types::{MessageQuery, NewTicketRequest, TicketKind};
use support::{default_overrides, start_daemon_with_config};

const CONFIG: &str = "[roles.manager]\nautostart = false\n";

async fn daemon(extra: &str) -> (support::TestDaemon, tempfile::TempDir) {
    let mut o = default_overrides();
    o.settle_wake_interval = Duration::from_millis(200);
    start_daemon_with_config(Some(o), Some(&format!("{CONFIG}{extra}"))).await
}

async fn new_row(daemon: &support::TestDaemon) -> String {
    daemon
        .client
        .new_task(&NewTicketRequest {
            ticket: None,
            parent: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: "prepare me".to_string(),
            kind: TicketKind::Feature,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new task")
        .id
}

/// A pending row, and its version 2 ticket file with `extra` frontmatter lines.
async fn v2_ticket(daemon: &support::TestDaemon, extra: &str) -> String {
    let id = new_row(daemon).await;
    v2_file(daemon, &id, extra);
    id
}

fn v2_file(daemon: &support::TestDaemon, id: &str, extra: &str) {
    let tail = id.rsplit('-').next().expect("tail").to_string();
    let dir = daemon.repo.join("docs/tickets/open");
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(
        dir.join(format!("prepare-me-{tail}.md")),
        format!(
            "---\nid: {}\nversion: 2\nstages:\n  proposal: {{}}\n  design: {{requires: [proposal]}}\n{extra}---\n\nbody\n",
            id
        ),
    )
    .expect("write");
}

async fn designers(daemon: &support::TestDaemon) -> Vec<String> {
    daemon
        .client
        .list_agents()
        .await
        .expect("agents")
        .into_iter()
        .filter(|a| a.role == "designer")
        .map(|a| a.name)
        .collect()
}

#[tokio::test]
async fn an_unclaimed_stage_gets_a_designer_once() {
    let (daemon, _tmp) = daemon("").await;
    let id = v2_ticket(&daemon, "").await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let tail = id.rsplit('-').next().expect("tail");
    assert_eq!(
        designers(&daemon).await,
        [format!("designer-{tail}-proposal")]
    );
}

#[tokio::test]
async fn a_claimed_ticket_gets_no_designer() {
    let (daemon, _tmp) = daemon("").await;
    v2_ticket(&daemon, "claimed_by: human\n").await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(designers(&daemon).await.is_empty());
}

#[tokio::test]
async fn a_stage_not_written_in_time_goes_to_the_product_manager_once() {
    let (daemon, _tmp) = daemon("[stages]\ntimeout = \"1s\"\n").await;
    let id = v2_ticket(&daemon, "").await;
    tokio::time::sleep(Duration::from_secs(4)).await;
    let got: Vec<String> = daemon
        .client
        .list_messages(&MessageQuery {
            to: Some("external:advisor/product-manager".to_string()),
            from: Some("system".to_string()),
            unread: false,
            limit: None,
            ..Default::default()
        })
        .await
        .expect("messages")
        .into_iter()
        .map(|m| m.body)
        .collect();
    assert_eq!(got.len(), 1, "got {got:?}");
    assert!(
        got[0].contains(&id)
            && got[0].contains("proposal not done by designer-")
            // The fake designer ends its turn (idle) without a stage-done, which is an ended
            // writer since br-43ug; a slow real one would be "timed out".
            && (got[0].contains("timed out") || got[0].contains("ended without")),
        "got {got:?}"
    );
    assert_eq!(designers(&daemon).await.len(), 1, "no retry");
}

#[tokio::test]
async fn done_is_refused_for_an_unprepared_v2_ticket_and_reopen_gives_pending() {
    let (daemon, _tmp) = daemon("").await;
    let c = &daemon.client;
    // Wrongly integrated while it was still a plain row (br-2ae8's state), then its file shows
    // it is version 2 with only the proposal written.
    let id = new_row(&daemon).await;
    let done = bridle_api::types::DoneTicketRequest {
        commit: "abc123".to_string(),
        ..Default::default()
    };
    c.done_task(&id, &done).await.expect("v1-style row done");
    v2_file(&daemon, &id, "written: [proposal]\n");
    let err = c
        .done_task(&id, &done)
        .await
        .expect_err("integrated already, but the guard speaks first");
    assert!(
        err.to_string().contains("still being prepared"),
        "got {err}"
    );
    let row = c.reopen_task(&id).await.expect("reopen");
    assert_eq!(row.state.as_str(), "pending");
    // The stage watch then spawns the next writer (design; proposal is written).
    tokio::time::sleep(Duration::from_secs(2)).await;
    let tail = id.rsplit('-').next().expect("tail");
    assert_eq!(
        designers(&daemon).await,
        [format!("designer-{tail}-design")]
    );
}

#[tokio::test]
async fn done_is_refused_for_a_pending_unprepared_v2_ticket() {
    let (daemon, _tmp) = daemon("").await;
    let id = v2_ticket(&daemon, "claimed_by: human\n").await;
    let done = bridle_api::types::DoneTicketRequest {
        commit: "abc123".to_string(),
        ..Default::default()
    };
    let err = daemon
        .client
        .done_task(&id, &done)
        .await
        .expect_err("refused");
    assert!(
        err.to_string()
            .contains("landing a stage does not finish it")
    );
    let row = daemon.client.get_task(&id).await.expect("get");
    assert_eq!(row.state.as_str(), "pending");
}

#[tokio::test]
async fn a_stage_writers_built_or_done_never_marks_a_prepared_v2_row() {
    let (daemon, _tmp) = daemon("").await;
    let c = &daemon.client;
    // Every stage signed and prepared, as br-2ae8 was: the row must wait for its implementation.
    let id = v2_ticket(
        &daemon,
        "written: [proposal, design]\nsigned: [proposal, design]\nprepared: true\n",
    )
    .await;
    let stage_branch = format!(
        "bridle/designer-{}-design",
        id.rsplit('-').next().expect("tail")
    );
    let built = bridle_api::types::BuiltTicketRequest {
        branch: stage_branch.clone(),
        commit: "abc123".to_string(),
    };
    let err = c.built_task(&id, &built).await.expect_err("built refused");
    assert!(err.to_string().contains("stage branch"), "got {err}");
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&daemon.repo)
        .output()
        .expect("git");
    let done = bridle_api::types::DoneTicketRequest {
        commit: String::from_utf8(head.stdout)
            .expect("utf8")
            .trim()
            .to_string(),
        branch: Some(stage_branch),
        ..Default::default()
    };
    let err = c.done_task(&id, &done).await.expect_err("done refused");
    assert!(err.to_string().contains("stage branch"), "got {err}");
    assert_eq!(
        c.get_task(&id).await.expect("get").state.as_str(),
        "pending"
    );
}
