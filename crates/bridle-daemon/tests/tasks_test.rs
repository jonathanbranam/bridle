//! The task record's HTTP surface (P0-1): create, show, edit, list, drop,
//! reopen. See docs/design/storage.md.

mod support;

use bridle_api::ClientError;
use bridle_api::types::{DropTaskRequest, EditTaskRequest, NewTaskRequest, TaskKind, TaskState};
use support::start_daemon;

fn new_req(title: &str, kind: TaskKind) -> NewTaskRequest {
    NewTaskRequest {
        title: title.to_string(),
        kind,
        body: String::new(),
    }
}

#[tokio::test]
async fn create_show_list_and_edit_a_task() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    let task = c
        .new_task(&NewTaskRequest {
            title: "Add foo".to_string(),
            kind: TaskKind::Feature,
            body: "a description".to_string(),
        })
        .await
        .expect("new task");
    // The test harness's repo directory is named "repo", so the default
    // prefix (config::default_task_prefix) is its first two letters.
    assert!(task.id.starts_with("re-"), "got id {}", task.id);
    assert_eq!(task.state, TaskState::Open);
    assert_eq!(task.title, "Add foo");
    assert_eq!(task.body, "a description");

    let fetched = c.get_task(&task.id).await.expect("get task");
    assert_eq!(fetched.id, task.id);
    assert_eq!(fetched.body, "a description");

    let list = c.list_tasks().await.expect("list tasks");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, task.id);

    let edited = c
        .edit_task(
            &task.id,
            &EditTaskRequest {
                title: Some("Add foo, better".to_string()),
                body: None,
            },
        )
        .await
        .expect("edit task");
    assert_eq!(edited.title, "Add foo, better");
    assert_eq!(edited.body, "a description");
    assert_eq!(edited.state, TaskState::Open);
}

#[tokio::test]
async fn get_of_an_unknown_id_is_404() {
    let (daemon, _tmp) = start_daemon(None).await;
    let err = daemon
        .client
        .get_task("br-nope")
        .await
        .expect_err("no such task");
    assert!(matches!(err, ClientError::Api { status: 404, .. }));
}

#[tokio::test]
async fn new_task_rejects_a_blank_title() {
    let (daemon, _tmp) = start_daemon(None).await;
    let err = daemon
        .client
        .new_task(&new_req("   ", TaskKind::Chore))
        .await
        .expect_err("blank title");
    assert!(matches!(err, ClientError::Api { status: 400, .. }));
}

#[tokio::test]
async fn drop_requires_a_reason() {
    let (daemon, _tmp) = start_daemon(None).await;
    let task = daemon
        .client
        .new_task(&new_req("Add foo", TaskKind::Bug))
        .await
        .expect("new task");

    let err = daemon
        .client
        .drop_task(
            &task.id,
            &DropTaskRequest {
                reason: String::new(),
            },
        )
        .await
        .expect_err("empty reason");
    assert!(matches!(err, ClientError::Api { status: 400, .. }));

    let dropped = daemon
        .client
        .drop_task(
            &task.id,
            &DropTaskRequest {
                reason: "budget cut".to_string(),
            },
        )
        .await
        .expect("drop with a reason");
    assert_eq!(dropped.state, TaskState::Dropped);
    assert_eq!(dropped.thread.len(), 1);
    assert!(dropped.thread[0].body.contains("budget cut"));
}

#[tokio::test]
async fn reopen_only_applies_to_a_dropped_task() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let task = c
        .new_task(&new_req("Add foo", TaskKind::Research))
        .await
        .expect("new task");

    let err = c.reopen_task(&task.id).await.expect_err("not dropped yet");
    assert!(matches!(err, ClientError::Api { status: 409, .. }));

    c.drop_task(
        &task.id,
        &DropTaskRequest {
            reason: "later".to_string(),
        },
    )
    .await
    .expect("drop");

    let reopened = c.reopen_task(&task.id).await.expect("reopen");
    assert_eq!(reopened.state, TaskState::Reopened);

    let err = c
        .reopen_task(&task.id)
        .await
        .expect_err("not dropped any more");
    assert!(matches!(err, ClientError::Api { status: 409, .. }));
}

#[tokio::test]
async fn a_task_created_before_restart_is_still_there_after() {
    let (daemon, tmp) = start_daemon(None).await;
    let task = daemon
        .client
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");
    // A graceful shutdown flushes the state branch (lib.rs::start), so the
    // restarted daemon's TaskManager can hydrate this task's body from its
    // file rather than falling back to an empty one.
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");

    let opts = bridle_daemon::ServeOptions {
        repo: daemon.repo.clone(),
        workspace: Some(daemon.workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let overrides = bridle_daemon::Overrides {
        claude_program: support::fake_claude_path().to_string_lossy().into_owned(),
        write_registry: false,
        stall_check_interval: std::time::Duration::from_secs(3600),
        tracker_interval: std::time::Duration::from_millis(200),
        governor_interval: std::time::Duration::from_secs(3600),
        governor_poll_interval_normal: std::time::Duration::from_secs(3600),
        governor_poll_interval_above_hold: std::time::Duration::from_secs(3600),
        task_flush_interval: std::time::Duration::from_secs(3600),
    };
    let running = bridle_daemon::start(opts, overrides)
        .await
        .expect("restart daemon");
    let token = std::fs::read_to_string(daemon.workspace.join(".bridle/tokens/human"))
        .expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));

    let fetched = client
        .get_task(&task.id)
        .await
        .expect("get task after restart");
    assert_eq!(fetched.title, "Add foo");
    assert_eq!(fetched.state, TaskState::Open);

    running.shutdown();
    running.join().await.expect("join");
    drop(tmp);
}
