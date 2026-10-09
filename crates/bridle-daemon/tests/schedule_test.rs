//! Scheduled messages over the API: who may add, list and remove (br-9xze).

mod support;

use bridle_api::types::{AgentState, ScheduleAddRequest, SpawnRequest, Workdir};
use chrono::{Duration, Utc};
use support::{default_overrides, start_daemon, wait_for_state};

async fn spawn_worker(daemon: &support::TestDaemon, name: &str) -> String {
    let a = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some(name.to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
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

fn once(to: Option<&str>) -> ScheduleAddRequest {
    ScheduleAddRequest {
        to: to.map(str::to_string),
        at: Some((Utc::now() + Duration::hours(1)).to_rfc3339()),
        body: "wake up".to_string(),
        ..Default::default()
    }
}

#[tokio::test]
async fn an_agent_schedules_only_to_itself_and_sees_only_its_own() {
    let (daemon, _tmp) = start_daemon(Some(default_overrides())).await;
    let w1 = daemon.agent_client(&spawn_worker(&daemon, "w1").await);
    spawn_worker(&daemon, "w2").await;

    let mine = w1.add_schedule(&once(None)).await.expect("to itself");
    assert_eq!(mine.target, "agent:w1");
    assert_eq!(mine.created_by, "agent:w1");
    assert_eq!(mine.state, "active");
    let err = w1.add_schedule(&once(Some("w2"))).await.expect_err("other");
    assert!(err.to_string().contains("only to itself"), "{err}");

    let theirs = daemon
        .client
        .add_schedule(&once(Some("w2")))
        .await
        .expect("the human may target any agent");
    assert_eq!(theirs.target, "agent:w2");

    let own = w1.list_schedules(false).await.expect("list");
    assert_eq!(own.iter().map(|s| &s.id).collect::<Vec<_>>(), [&mine.id]);
    assert_eq!(daemon.client.list_schedules(false).await.unwrap().len(), 2);

    // Another's schedule is not found, not forbidden.
    let err = w1.remove_schedule(&theirs.id).await.expect_err("not mine");
    assert!(err.to_string().contains("no such schedule"), "{err}");
    w1.remove_schedule(&mine.id).await.expect("own");
    daemon
        .client
        .remove_schedule(&theirs.id)
        .await
        .expect("the human removes any");
    assert!(daemon.client.list_schedules(true).await.unwrap().is_empty());
}

#[tokio::test]
async fn others_are_refused_and_bad_times_are_explained() {
    let (daemon, _tmp) = start_daemon(Some(default_overrides())).await;
    spawn_worker(&daemon, "w1").await;
    let ext = daemon.external_client("orchestrator").await;
    let err = ext.add_schedule(&once(Some("w1"))).await.expect_err("ext");
    assert!(
        err.to_string().contains("only agents and the human"),
        "{err}"
    );

    let past = ScheduleAddRequest {
        at: Some("2020-01-01 09:00".to_string()),
        ..once(Some("w1"))
    };
    let err = daemon.client.add_schedule(&past).await.expect_err("past");
    assert!(err.to_string().contains("in the past"), "{err}");

    let never = ScheduleAddRequest {
        at: None,
        cron: Some("0 0 31 2 *".to_string()),
        ..once(Some("w1"))
    };
    let err = daemon.client.add_schedule(&never).await.expect_err("never");
    assert!(err.to_string().contains("never fires"), "{err}");

    let ok = ScheduleAddRequest {
        at: None,
        cron: Some("30 9 * * 1-5".to_string()),
        tz: Some("Europe/London".to_string()),
        ..once(Some("w1"))
    };
    let s = daemon.client.add_schedule(&ok).await.expect("cron");
    assert_eq!((s.kind.as_str(), s.tz.as_str()), ("cron", "Europe/London"));
    assert!(s.next_fire_at.is_some());
}
