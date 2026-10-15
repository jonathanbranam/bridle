//! The settle hold covers spawning an agent for a ticket, in any state (br-85hs).

mod support;
use support::ClientExt as _;

use bridle_api::types::{NewTicketRequest, SpawnRequest, TicketKind, Workdir};
use support::start_daemon_with_config;

fn spawn_for(ticket: &str, name: &str) -> SpawnRequest {
    SpawnRequest {
        role: "designer".to_string(),
        name: Some(name.to_string()),
        workdir: Some(Workdir::Repo),
        ticket: Some(ticket.to_string()),
        ..Default::default()
    }
}

#[tokio::test]
async fn a_spawn_for_an_unsettled_pending_ticket_is_held_and_says_when_it_ends() {
    let (daemon, _repo) = start_daemon_with_config(
        None,
        Some("[roles.manager]\nautostart = false\n[tasks]\nsettle = \"10m\"\n"),
    )
    .await;
    let id = daemon
        .client
        .new_open_task(&NewTicketRequest {
            ticket: None,
            parent: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: "fresh".to_string(),
            kind: TicketKind::Feature,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new ticket")
        .id;

    let other = daemon.external_client("pm").await;
    let err = other.spawn(&spawn_for(&id, "des")).await.expect_err("held");
    assert!(err.to_string().contains("settling until"), "{err}");

    // The human's own spawn is not held.
    daemon
        .client
        .spawn(&spawn_for(&id, "des-human"))
        .await
        .expect("human spawn");
}
