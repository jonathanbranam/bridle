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

/// A pending row, and its version 2 ticket file with `extra` frontmatter lines.
async fn v2_ticket(daemon: &support::TestDaemon, extra: &str) -> String {
    let row = daemon
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
        .expect("new task");
    let tail = row.id.rsplit('-').next().expect("tail").to_string();
    let dir = daemon.repo.join("docs/tickets/open");
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(
        dir.join(format!("prepare-me-{tail}.md")),
        format!(
            "---\nid: {}\nversion: 2\nstages:\n  proposal: {{}}\n  design: {{requires: [proposal]}}\n{extra}---\n\nbody\n",
            row.id
        ),
    )
    .expect("write");
    row.id
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
            && got[0].contains("timed out"),
        "got {got:?}"
    );
    assert_eq!(designers(&daemon).await.len(), 1, "no retry");
}
