//! `GET /v1/wake`: the daemon decides when a principal wakes: an unread message, or someone
//! else touching a task it created or claimed (`principal_wake.rs`).

mod support;
use support::ClientExt as _;

use std::time::Duration;

use bridle_api::types::{
    MessageKind, MessageQuery, NewTaskRequest, PrincipalWakeQuery, PrincipalWakeResponse,
    SendRequest, SetKindRequest, SetPriorityRequest, TaskKind, TaskPriority, When,
};
use bridle_api::{Client, ClientError};

fn query(principal: &str, timeout_secs: u64) -> PrincipalWakeQuery {
    PrincipalWakeQuery {
        principal: principal.to_string(),
        timeout_secs: Some(timeout_secs),
        session: None,
    }
}

fn session_query(principal: &str, session: &str) -> PrincipalWakeQuery {
    PrincipalWakeQuery {
        session: Some(session.to_string()),
        ..query(principal, 60)
    }
}

fn superseded(r: &PrincipalWakeResponse) -> bool {
    r.reasons.len() == 1 && r.reasons[0].reason == bridle_api::types::WAIT_SUPERSEDED_WAKE
}

async fn send(client: &Client, to: &str, body: &str) -> String {
    client
        .send(&SendRequest {
            to: Some(to.to_string()),
            body: body.to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect("send")
        .remove(0)
        .id
}

#[tokio::test]
async fn an_unread_message_wakes_at_once() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let id = send(&daemon.client, "external:advisor", "hello").await;
    let got = tokio::time::timeout(
        Duration::from_secs(3),
        advisor.principal_wake(&query("external:advisor", 60)),
    )
    .await
    .expect("returns at once")
    .expect("wake");
    assert_eq!(got.reasons.len(), 1, "{got:?}");
    assert_eq!(got.reasons[0].reason, "message");
    assert_eq!(got.reasons[0].message_ids, vec![id]);
}

#[tokio::test]
async fn a_message_sent_after_the_call_starts_wakes_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let waiting = tokio::spawn(async move {
        advisor
            .principal_wake(&query("external:advisor", 60))
            .await
            .expect("wake")
    });
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!waiting.is_finished());
    let id = send(&daemon.client, "external:advisor", "later").await;
    let got = tokio::time::timeout(Duration::from_secs(3), waiting)
        .await
        .expect("woken by the event")
        .expect("join");
    assert_eq!(got.reasons[0].message_ids, vec![id]);
}

#[tokio::test]
async fn it_times_out_empty_and_ignores_mail_for_others() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let _orch = daemon.external_client("orchestrator").await;
    send(&daemon.client, "external:orchestrator", "not yours").await;
    let got = advisor
        .principal_wake(&query("external:advisor", 1))
        .await
        .expect("wake");
    assert!(got.reasons.is_empty(), "{got:?}");
}

#[tokio::test]
async fn only_that_principal_or_the_human_may_wait() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let anon = Client::new(daemon.running.url.clone(), None);
    let other = daemon.external_client("orchestrator").await;
    for client in [&anon, &other] {
        let err = client
            .principal_wake(&query("external:advisor", 1))
            .await
            .expect_err("forbidden");
        assert!(
            matches!(err, ClientError::Api { status: 403, .. }),
            "got {err:?}"
        );
    }
    daemon
        .client
        .principal_wake(&query("external:advisor", 1))
        .await
        .expect("the human may");
}

async fn new_task(client: &Client) -> String {
    client
        .new_open_task(&NewTaskRequest {
            ticket: None,
            parent: None,
            title: "t".to_string(),
            kind: TaskKind::Feature,
            body: String::new(),
            size: None,
            components: Vec::new(),
            for_human: false,
            priority: None,
        })
        .await
        .expect("new task")
        .id
}

/// Starts the advisor's wake, runs `act` once it is waiting, and returns the answer (after
/// `timeout_secs` at most).
async fn wake_after(
    advisor: &Client,
    timeout_secs: u64,
    act: impl std::future::Future<Output = ()>,
) -> PrincipalWakeResponse {
    let advisor = advisor.clone();
    let waiting = tokio::spawn(async move {
        advisor
            .principal_wake(&query("external:advisor", timeout_secs))
            .await
            .expect("wake")
    });
    tokio::time::sleep(Duration::from_millis(300)).await;
    act.await;
    waiting.await.expect("join")
}

/// The `task_update` lines `to` has received, oldest first, without marking them read.
async fn updates(client: &Client, to: &str) -> Vec<String> {
    client
        .list_messages(&MessageQuery {
            to: Some(to.to_string()),
            ..Default::default()
        })
        .await
        .expect("messages")
        .into_iter()
        .filter(|m| m.kind == MessageKind::TaskUpdate)
        .map(|m| m.body)
        .collect()
}

#[tokio::test]
async fn another_principals_comment_on_a_task_it_created_is_a_message_and_wakes_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let task = new_task(&advisor).await;
    let got = wake_after(&advisor, 10, async {
        daemon.client.note_task(&task, "hi").await.expect("note");
    })
    .await;
    assert_eq!(got.reasons.len(), 1, "{got:?}");
    assert_eq!(got.reasons[0].reason, "message");
    assert_eq!(got.reasons[0].task, None, "task wake reasons are gone");
    let m = &got.reasons[0].messages[0];
    assert_eq!(m.kind, MessageKind::TaskUpdate);
    assert!(m.body.starts_with(&format!("{task} (t): ")), "{}", m.body);
    assert!(m.body.ends_with("comment by human: hi"), "{}", m.body);
    assert!(!m.body.contains('\n'));
}

#[tokio::test]
async fn each_kind_of_change_is_its_own_line() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let task = new_task(&advisor).await;
    let h = &daemon.client;
    h.plan_task(&task).await.expect("plan");
    h.set_task_priority(
        &task,
        &SetPriorityRequest {
            priority: TaskPriority::High,
        },
    )
    .await
    .expect("priority");
    h.ask_question(&task, "which way?\nsecond line", Some("human"))
        .await
        .expect("ask");
    h.answer_question(&task, "that way").await.expect("answer");
    h.note_task(&task, &"x".repeat(300)).await.expect("note");
    let lines = updates(&advisor, "me").await;
    let tails: Vec<_> = lines
        .iter()
        .map(|l| l.split_once("): ").expect("prefix").1.to_string())
        .collect();
    assert_eq!(tails.len(), 5, "{lines:?}");
    assert_eq!(
        tails[0],
        "planned -> planned by human".replace("planned -> ", "open -> ")
    );
    assert_eq!(tails[1], "priority normal -> high by human");
    assert_eq!(tails[2], "question asked by human: which way? second line");
    assert_eq!(tails[3], "question answered by human: that way");
    assert!(
        tails[4].starts_with("comment by human: xxx"),
        "{}",
        tails[4]
    );
    assert_eq!(tails[4].chars().count(), "comment by human: ".len() + 201);
}

#[tokio::test]
async fn a_kind_change_is_a_line() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let task = new_task(&advisor).await;
    daemon
        .client
        .set_task_kind(
            &task,
            &SetKindRequest {
                kind: TaskKind::Bug,
            },
        )
        .await
        .expect("kind");
    let lines = updates(&advisor, "me").await;
    assert!(
        lines[0].ends_with("kind feature -> bug by human"),
        "{lines:?}"
    );
}

#[tokio::test]
async fn the_actor_and_non_watchers_are_not_told_and_unwatching_stops_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let other = daemon.external_client("other").await;
    let task = new_task(&advisor).await;
    advisor.note_task(&task, "mine").await.expect("own note");
    daemon.client.note_task(&task, "one").await.expect("note");
    assert_eq!(updates(&advisor, "me").await.len(), 1, "own change skipped");
    assert!(updates(&other, "me").await.is_empty(), "not a watcher");
    advisor.unwatch_task(&task).await.expect("unwatch");
    daemon.client.note_task(&task, "two").await.expect("note");
    assert_eq!(updates(&advisor, "me").await.len(), 1, "unwatched");
    other.watch_task(&task).await.expect("watch");
    daemon.client.note_task(&task, "three").await.expect("note");
    assert_eq!(updates(&other, "me").await.len(), 1);
}

#[tokio::test]
async fn the_wake_returns_the_text_and_marks_it_read_once() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let id = send(&daemon.client, "external:advisor", "hello there").await;
    let got = advisor
        .principal_wake(&query("external:advisor", 60))
        .await
        .expect("wake");
    let msgs = &got.reasons[0].messages;
    assert_eq!(msgs.len(), 1, "{got:?}");
    assert_eq!(
        (msgs[0].id.as_str(), msgs[0].body.as_str()),
        (&*id, "hello there")
    );
    // Read now, so the next wake has nothing to repeat.
    let again = advisor
        .principal_wake(&query("external:advisor", 1))
        .await
        .expect("wake");
    assert!(again.reasons.is_empty(), "{again:?}");
}

#[tokio::test]
async fn the_humans_wake_leaves_messages_unread() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let _advisor = daemon.external_client("advisor").await;
    send(&daemon.client, "external:advisor", "hello").await;
    let got = daemon
        .client
        .principal_wake(&query("external:advisor", 60))
        .await
        .expect("wake");
    assert!(got.reasons[0].messages.is_empty(), "{got:?}");
    let got = daemon
        .client
        .principal_wake(&query("external:advisor", 60))
        .await
        .expect("wake again");
    assert_eq!(got.reasons.len(), 1, "still unread: {got:?}");
}

#[tokio::test]
async fn a_non_human_inbox_marks_what_it_lists_read_and_cannot_unread() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let a = send(&daemon.client, "external:advisor", "one").await;
    let b = send(&daemon.client, "external:advisor", "two").await;
    // `show` of one message marks only that one.
    let shown = advisor
        .list_messages(&MessageQuery {
            to: Some("me".into()),
            id: Some(a.clone()),
            mark_read: true,
            ..Default::default()
        })
        .await
        .expect("show");
    assert_eq!(shown.len(), 1);
    let unread = |c: Client| async move {
        c.list_messages(&MessageQuery {
            to: Some("me".into()),
            unread: true,
            ..Default::default()
        })
        .await
        .expect("list")
        .into_iter()
        .map(|m| m.id)
        .collect::<Vec<_>>()
    };
    assert_eq!(unread(advisor.clone()).await, vec![b.clone()]);
    // A plain list marks nothing unless asked; the marking list takes the rest.
    advisor
        .list_messages(&MessageQuery {
            to: Some("me".into()),
            unread: true,
            mark_read: true,
            ..Default::default()
        })
        .await
        .expect("inbox");
    assert!(unread(advisor.clone()).await.is_empty());
    // An agent or external principal can't put it back; the human can.
    let err = advisor.mark_unread(&a).await.expect_err("refused");
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );
    daemon.client.mark_unread(&a).await.expect("human unread");
}

#[tokio::test]
async fn a_waiter_open_during_a_restart_is_told_why_and_the_event_counts_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let waiting =
        tokio::spawn(async move { advisor.principal_wake(&query("external:advisor", 60)).await });
    tokio::time::sleep(Duration::from_millis(300)).await;
    daemon
        .client
        .restart(&bridle_api::types::RestartRequest::default())
        .await
        .expect("restart");
    let got = tokio::time::timeout(Duration::from_secs(5), waiting)
        .await
        .expect("ended by the restart, not the timeout")
        .expect("join")
        .expect("wake");
    assert_eq!(got.reasons.len(), 1, "{got:?}");
    assert_eq!(
        got.reasons[0].reason,
        bridle_api::types::DAEMON_STOPPING_WAKE
    );
    let text = got.reasons[0].text.as_deref().expect("reason text");
    assert!(text.contains("restarting by request"), "{text}");
    let ev = support::wait_for_event(&daemon.client, "daemon.stopping", None, |_| true).await;
    assert_eq!(ev.data["waiters_ended"], 1, "{ev:?}");
    assert!(
        ev.data["reason"]
            .as_str()
            .is_some_and(|r| r.contains("restarting by request")),
        "{ev:?}"
    );
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_new_wait_from_the_same_session_replaces_the_old_one() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let old = {
        let advisor = advisor.clone();
        tokio::spawn(async move {
            advisor
                .principal_wake(&session_query("external:advisor", "100"))
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    let new = {
        let advisor = advisor.clone();
        tokio::spawn(async move {
            advisor
                .principal_wake(&session_query("external:advisor", "100"))
                .await
        })
    };
    let got = tokio::time::timeout(Duration::from_secs(5), old)
        .await
        .expect("old wait ended")
        .unwrap()
        .unwrap();
    assert!(superseded(&got));
    // The new wait is still open and gets the message; the superseded one took nothing.
    send(&daemon.client, "external:advisor", "hi").await;
    let got = new.await.unwrap().unwrap();
    assert_eq!(got.reasons[0].reason, "message");
}

#[tokio::test]
async fn sessions_sharing_an_identity_do_not_replace_each_other() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let a = {
        let advisor = advisor.clone();
        tokio::spawn(async move {
            advisor
                .principal_wake(&session_query("external:advisor", "100"))
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    let b = {
        let advisor = advisor.clone();
        tokio::spawn(async move {
            advisor
                .principal_wake(&session_query("external:advisor", "200"))
                .await
        })
    };
    // A wait with no session replaces nothing either.
    let c = {
        let advisor = advisor.clone();
        tokio::spawn(async move { advisor.principal_wake(&query("external:advisor", 60)).await })
    };
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(!a.is_finished() && !b.is_finished() && !c.is_finished());
    let n = advisor
        .stop_wake(&bridle_api::types::StopWakeRequest {
            principal: Some("external:advisor".into()),
            session: None,
        })
        .await
        .unwrap()
        .stopped;
    assert_eq!(n, 3);
    for h in [a, b, c] {
        assert!(superseded(&h.await.unwrap().unwrap()));
    }
}

#[tokio::test]
async fn stop_ends_own_session_refuses_other_identity_and_reports_none() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let stop =
        |principal: Option<&str>, session: Option<&str>| bridle_api::types::StopWakeRequest {
            principal: principal.map(str::to_string),
            session: session.map(str::to_string),
        };
    // None open.
    assert_eq!(
        advisor
            .stop_wake(&stop(None, Some("100")))
            .await
            .unwrap()
            .stopped,
        0
    );
    let w = {
        let advisor = advisor.clone();
        tokio::spawn(async move {
            advisor
                .principal_wake(&session_query("external:advisor", "100"))
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    // Another session's stop finds nothing; another identity's wait may not be stopped.
    assert_eq!(
        advisor
            .stop_wake(&stop(None, Some("999")))
            .await
            .unwrap()
            .stopped,
        0
    );
    let err = advisor.stop_wake(&stop(Some("human"), None)).await;
    assert!(err.is_err(), "stopping another identity's wait is refused");
    assert_eq!(
        advisor
            .stop_wake(&stop(None, Some("100")))
            .await
            .unwrap()
            .stopped,
        1
    );
    assert!(superseded(&w.await.unwrap().unwrap()));
}

/// A task claimed by `worker`, who has stopped watching it: only the claimant link can tell it.
async fn claimed_by_unwatching_worker(daemon: &support::TestDaemon, worker: &Client) -> String {
    let task = new_task(&daemon.client).await;
    daemon.client.plan_task(&task).await.expect("plan");
    worker.claim_task(&task).await.expect("claim");
    worker.unwatch_task(&task).await.expect("unwatch");
    task
}

#[tokio::test]
async fn a_comment_on_a_claimed_task_tells_its_claimant_once() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let worker = daemon.external_client("worker").await;
    let task = claimed_by_unwatching_worker(&daemon, &worker).await;
    daemon
        .client
        .note_task(&task, "send-back")
        .await
        .expect("note");
    let lines = updates(&worker, "me").await;
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].ends_with("comment by human: send-back"),
        "{lines:?}"
    );
    // The claimant's own comment tells it nothing.
    worker.note_task(&task, "mine").await.expect("own note");
    assert_eq!(updates(&worker, "me").await.len(), 1);
}

#[tokio::test]
async fn notifying_the_claimant_or_watching_it_is_still_one_message() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let worker = daemon.external_client("worker").await;
    let task = claimed_by_unwatching_worker(&daemon, &worker).await;
    // `--notify` is `send --task`: its pointer is the one message, not a second watcher line.
    daemon
        .client
        .send(&SendRequest {
            to: Some("external:worker".to_string()),
            body: "merge main".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: Some(task.clone()),
        })
        .await
        .expect("send");
    let all = worker
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            ..Default::default()
        })
        .await
        .expect("messages");
    assert_eq!(all.len(), 1, "{all:?}");
    // A claimant that is also a watcher gets one line per comment too.
    worker.watch_task(&task).await.expect("watch");
    daemon.client.note_task(&task, "again").await.expect("note");
    assert_eq!(updates(&worker, "me").await.len(), 1);
}
