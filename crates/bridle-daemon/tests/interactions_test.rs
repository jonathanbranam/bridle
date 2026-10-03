//! `GET /v1/interactions`: docs/design/agent-host/api.md, roles-and-config.md ("Prompt recording").

mod support;

use bridle_api::types::{InteractionEvent, InteractionsQuery};
use chrono::{TimeZone, Utc};

#[tokio::test]
async fn serves_the_prompt_log_skipping_bad_lines_and_filtering_by_since() {
    let (daemon, tmp) = support::start_daemon(None).await;
    let home = support::machine_home_dir(tmp.path());
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::write(
        home.join("prompts.jsonl"),
        concat!(
            r#"{"at":"2026-10-03T10:00:00Z","session":"s1","role":"advisor","machine":"m","project":"bridle"}"#, "\n",
            "not json\n",
            r#"{"at":"2026-10-03T10:05:00Z","session":"s1","role":"advisor","event":"reply"}"#, "\n",
            r#"{"at":"2026-10-03T11:00:00Z","session":null,"role":"orchestrator","machine":null,"project":null}"#, "\n",
            r#"{"session":"no-at"}"#, "\n",
        ),
    )
    .expect("write log");

    let all = daemon
        .client
        .interactions(&InteractionsQuery::default())
        .await
        .expect("interactions");
    assert_eq!(all.len(), 3, "bad lines are skipped");
    assert_eq!(all[0].session.as_deref(), Some("s1"));
    assert_eq!(
        all[0].event,
        InteractionEvent::Prompt,
        "no event field: a prompt"
    );
    assert_eq!(all[1].event, InteractionEvent::Reply);
    assert_eq!(all[2].role.as_deref(), Some("orchestrator"));
    assert_eq!(all[2].session, None);

    let since = daemon
        .client
        .interactions(&InteractionsQuery {
            since: Some(Utc.with_ymd_and_hms(2026, 10, 3, 10, 30, 0).unwrap()),
        })
        .await
        .expect("interactions since");
    assert_eq!(since.len(), 1);
    assert_eq!(since[0].role.as_deref(), Some("orchestrator"));

    // Agents may not read it.
    let ext = daemon.external_client("reader").await;
    assert!(
        ext.interactions(&InteractionsQuery::default())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn no_log_is_an_empty_list() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let all = daemon
        .client
        .interactions(&InteractionsQuery::default())
        .await
        .expect("interactions");
    assert!(all.is_empty());
}
