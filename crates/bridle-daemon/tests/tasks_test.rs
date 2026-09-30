//! The task record's HTTP surface (P0-1): create, show, edit, list, drop,
//! reopen. See docs/design/storage.md. Plus questions (P0-3): `ask`/`answer`
//! and their effect on readiness (docs/design/coordination.md, "Questions do
//! not stop work"). Plus `plan` and the queue (j479): `open` -> `planned`,
//! and the PM-owned queue record (roles-and-lifecycle.md, "the queue").

mod support;

use bridle_api::ClientError;
use bridle_api::types::{
    AgentState, DoneTaskRequest, DropTaskRequest, EditTaskRequest, NewTaskRequest,
    SetSummaryRequest, SpawnRequest, TaskKind, TaskSize, TaskState, ThreadEntryKind, Workdir,
};
use support::{start_daemon, wait_for_state};

fn new_req(title: &str, kind: TaskKind) -> NewTaskRequest {
    NewTaskRequest {
        for_human: false,
        components: Vec::new(),
        title: title.to_string(),
        kind,
        body: String::new(),
        size: None,
    }
}

#[tokio::test]
async fn create_show_list_and_edit_a_task() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    let task = c
        .new_task(&NewTaskRequest {
            for_human: false,
            components: Vec::new(),
            title: "Add foo".to_string(),
            kind: TaskKind::Feature,
            body: "a description".to_string(),
            size: None,
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
                components: None,
                size: None,
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

/// End to end through the HTTP surface `bridle ask`/`bridle answer`/`bridle
/// inbox` are thin clients of: asking blocks the task (an open question in
/// `GET /v1/questions`, which is exactly what `TaskManager::is_ready` checks
/// — see `tasks.rs`'s own
/// `asking_a_question_blocks_ready_and_answering_unblocks_it` for readiness
/// itself), then answering clears it. The full ready-tasks round trip
/// through a `plan` needs a `planned` task to bite on; that's covered by
/// `plan_moves_open_to_planned_and_ask_still_blocks_ready` below, so this
/// one just checks the open-questions index itself.
#[tokio::test]
async fn ask_blocks_a_task_and_answer_frees_it_again() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");
    assert!(c.list_open_questions().await.expect("list").is_empty());

    let asked = c
        .ask_question(&task.id, "which endpoint?", None)
        .await
        .expect("ask question");
    assert_eq!(asked.thread.len(), 1);
    assert_eq!(asked.thread[0].kind, ThreadEntryKind::Question);
    assert_eq!(asked.thread[0].body, "which endpoint?");

    let open = c.list_open_questions().await.expect("list open questions");
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].task_id, task.id);
    assert_eq!(open[0].body, "which endpoint?");

    // A second question while one is already open is a conflict.
    let err = c
        .ask_question(&task.id, "another one?", None)
        .await
        .expect_err("already has an open question");
    assert!(matches!(err, ClientError::Api { status: 409, .. }));

    let answered = c
        .answer_question(&task.id, "the v1 endpoint")
        .await
        .expect("answer question");
    assert_eq!(answered.thread.len(), 2);
    assert_eq!(answered.thread[1].kind, ThreadEntryKind::Answer);
    assert_eq!(answered.thread[1].body, "the v1 endpoint");

    assert!(
        c.list_open_questions()
            .await
            .expect("list open questions")
            .is_empty(),
        "answering clears the task from the open-questions index"
    );

    // Answering again with nothing open is a conflict.
    let err = c
        .answer_question(&task.id, "still there?")
        .await
        .expect_err("no open question left");
    assert!(matches!(err, ClientError::Api { status: 409, .. }));
}

/// End to end through the HTTP surface `bridle task note` is a thin client
/// of: a note lands in the task's thread and is visible on `get_task` (what
/// `bridle task show` renders), from both a human and an external
/// principal. Unlike `ask_question`, `note_task` has no readiness or
/// open-question bookkeeping, so there's nothing else to assert here.
#[tokio::test]
async fn note_appears_in_the_task_thread_from_human_and_agent() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");

    let noted = c
        .note_task(&task.id, "starting on this now")
        .await
        .expect("note task");
    assert_eq!(noted.thread.len(), 1);
    assert_eq!(noted.thread[0].kind, ThreadEntryKind::Note);
    assert_eq!(noted.thread[0].body, "starting on this now");

    let external = daemon.external_client("w1").await;
    let noted = external
        .note_task(&task.id, "picking this up")
        .await
        .expect("note task as external principal");
    assert_eq!(noted.thread.len(), 2);
    assert_eq!(noted.thread[1].kind, ThreadEntryKind::Note);
    assert_eq!(noted.thread[1].body, "picking this up");

    let fetched = c.get_task(&task.id).await.expect("get task");
    assert_eq!(fetched.thread.len(), 2);
}

/// `bridle claim`/`bridle release`'s HTTP surface, in its "not ready"
/// conflict shape (a fresh task is `open`, not `planned`). The full
/// claim -> ready-exclusion -> release -> ready-again round trip (plus a
/// second claim being rejected) is covered where the state-machine details
/// live: `TaskManager`'s own `claim_blocks_ready_and_release_unblocks_it` in
/// `tasks.rs`; `plan_then_claim_round_trips_through_the_http_surface` below
/// covers plan -> claim end to end at this layer.
#[tokio::test]
async fn claim_of_an_unknown_task_is_404() {
    let (daemon, _tmp) = start_daemon(None).await;
    let err = daemon
        .client
        .claim_task("br-nope")
        .await
        .expect_err("no such task");
    assert!(matches!(err, ClientError::Api { status: 404, .. }));
}

#[tokio::test]
async fn claim_of_a_task_that_is_not_ready_is_a_conflict() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");
    assert_eq!(task.state, TaskState::Open);

    let err = c
        .claim_task(&task.id)
        .await
        .expect_err("open, not planned, isn't ready to claim");
    assert!(matches!(err, ClientError::Api { status: 409, .. }));
}

#[tokio::test]
async fn release_of_an_unclaimed_task_is_a_conflict() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");

    let err = c.release_task(&task.id).await.expect_err("never claimed");
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
        bridle_home: Some(support::machine_home_dir(tmp.path())),
        stall_check_interval: std::time::Duration::from_secs(3600),
        tracker_interval: std::time::Duration::from_millis(200),
        governor_interval: std::time::Duration::from_secs(3600),
        governor_poll_interval_normal: std::time::Duration::from_secs(3600),
        governor_poll_interval_above_hold: std::time::Duration::from_secs(3600),
        task_flush_interval: std::time::Duration::from_secs(3600),
        claim_lease_check_interval: std::time::Duration::from_secs(3600),
        port_check_interval: std::time::Duration::from_secs(3600),
        upgrade: Default::default(),
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

#[tokio::test]
async fn plan_moves_open_to_planned_and_rejects_a_second_plan() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");
    assert_eq!(task.state, TaskState::Open);

    let planned = c.plan_task(&task.id).await.expect("plan");
    assert_eq!(planned.state, TaskState::Planned);
    assert!(
        c.ready_tasks()
            .await
            .expect("ready tasks")
            .iter()
            .any(|t| t.id == task.id),
        "a planned task with no blockers is ready"
    );

    let err = c.plan_task(&task.id).await.expect_err("already planned");
    assert!(matches!(err, ClientError::Api { status: 409, .. }));
}

#[tokio::test]
async fn plan_of_an_unknown_task_is_404() {
    let (daemon, _tmp) = start_daemon(None).await;
    let err = daemon
        .client
        .plan_task("br-nope")
        .await
        .expect_err("no such task");
    assert!(matches!(err, ClientError::Api { status: 404, .. }));
}

/// `plan` -> `claim` -> `release`, the same round trip
/// `claim_of_a_task_that_is_not_ready_is_a_conflict`/`release_of_an_unclaimed_task_is_a_conflict`
/// above only reach the "not ready yet" half of.
#[tokio::test]
async fn plan_then_claim_round_trips_through_the_http_surface() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");
    c.plan_task(&task.id).await.expect("plan");

    let claimed = c.claim_task(&task.id).await.expect("claim");
    assert_eq!(claimed.state, TaskState::Claimed);
    assert_eq!(claimed.claimed_by.as_deref(), Some("human"));

    let released = c.release_task(&task.id).await.expect("release");
    assert_eq!(released.state, TaskState::Planned);
    assert_eq!(released.claimed_by, None);
}

/// `for_human` creates the task planned and claimed by the human; the human
/// finishes it with `done` and no commit.
#[tokio::test]
async fn for_human_task_is_claimed_by_the_human_and_done_without_a_commit() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let mut req = new_req("[at restart] move tokens", TaskKind::Feature);
    req.for_human = true;
    let task = c.new_task(&req).await.expect("new task");
    assert_eq!(task.state, TaskState::Claimed);
    assert_eq!(task.claimed_by.as_deref(), Some("human"));

    let listed = c.list_tasks_claimed_by("human").await.expect("list");
    assert!(listed.iter().any(|t| t.id == task.id));

    let done = c
        .done_task(
            &task.id,
            &DoneTaskRequest {
                commit: String::new(),
                branch: None,
                ..Default::default()
            },
        )
        .await
        .expect("done");
    assert_eq!(done.state, TaskState::Integrated);
}

/// `bridle queue`'s HTTP surface: empty by default, set by the PM/human
/// (here, human), read back verbatim, and `?top_tier=true` picks the
/// highest tier with a startable task — skipping one stuck on a dependency
/// rather than returning nothing (roles-and-lifecycle.md, "the queue").
#[tokio::test]
async fn queue_set_get_and_top_tier_pick_the_right_tier_when_blocked() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    assert!(
        c.get_queue()
            .await
            .expect("get empty queue")
            .tiers
            .is_empty()
    );

    let blocker = c
        .new_task(&new_req("Blocker", TaskKind::Chore))
        .await
        .expect("new blocker");
    let blocked = c
        .new_task(&new_req("Blocked", TaskKind::Feature))
        .await
        .expect("new blocked");
    let next = c
        .new_task(&new_req("Next", TaskKind::Feature))
        .await
        .expect("new next");
    c.add_edge(&bridle_api::types::NewEdgeRequest {
        from: blocker.id.clone(),
        to: blocked.id.clone(),
        kind: bridle_api::types::EdgeKind::Blocks,
    })
    .await
    .expect("add edge");
    c.plan_task(&blocker.id).await.expect("plan blocker");
    c.plan_task(&blocked.id).await.expect("plan blocked");
    c.plan_task(&next.id).await.expect("plan next");

    let q = c
        .set_queue(vec![vec![blocked.id.clone()], vec![next.id.clone()]])
        .await
        .expect("set queue");
    assert_eq!(
        q.tiers,
        vec![vec![blocked.id.clone()], vec![next.id.clone()]]
    );
    assert_eq!(c.get_queue().await.expect("get queue").tiers, q.tiers);

    // Tier 1's only task is blocked (its blocker isn't itself queued, so
    // it's backlog): the top startable tier is tier 2, not tier 1.
    let top = c.top_tier_ready_tasks().await.expect("top tier");
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].id, next.id);

    // Resolving the blocker frees tier 1, which now outranks tier 2 again.
    c.drop_task(
        &blocker.id,
        &DropTaskRequest {
            reason: "done another way".to_string(),
        },
    )
    .await
    .expect("drop blocker");
    let top = c.top_tier_ready_tasks().await.expect("top tier");
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].id, blocked.id);

    let q = c
        .add_queue_tier(vec![blocker.id.clone()])
        .await
        .expect("add tier");
    assert_eq!(q.tiers.len(), 3);
}

/// Only the PM (or the human) may write the queue; every other principal,
/// the manager included, is read-only (roles-and-lifecycle.md, "the
/// queue").
#[tokio::test]
async fn only_pm_or_human_may_write_the_queue() {
    let (daemon, _tmp) = start_daemon(None).await;
    let task = daemon
        .client
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");

    let manager = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "manager".to_string(),
            name: Some("mgr".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn manager");
    wait_for_state(&daemon.client, &manager.id, AgentState::Idle).await;
    let manager_client = daemon.agent_client(&manager.id);

    let err = manager_client
        .set_queue(vec![vec![task.id.clone()]])
        .await
        .expect_err("manager can't write the queue");
    assert!(matches!(err, ClientError::Api { status: 403, .. }));

    let err = manager_client
        .add_queue_tier(vec![task.id.clone()])
        .await
        .expect_err("manager can't write the queue");
    assert!(matches!(err, ClientError::Api { status: 403, .. }));

    // The human still can.
    daemon
        .client
        .set_queue(vec![vec![task.id.clone()]])
        .await
        .expect("human can write the queue");
}

/// `bridle rebuild`'s HTTP surface: a no-op against an empty database
/// succeeds, but refuses once the database has anything to lose
/// (docs/design/storage.md, "Rebuild"). The reconstruction itself is
/// covered at the `TaskManager` level (`tasks.rs`'s own tests); this is just
/// the route/auth wiring on top.
#[tokio::test]
async fn rebuild_is_a_no_op_when_empty_and_refuses_once_populated() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    c.rebuild(false).await.expect("rebuild against an empty db");
    assert!(c.list_tasks().await.expect("list tasks").is_empty());

    c.new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");

    let err = c.rebuild(false).await.unwrap_err();
    assert!(matches!(err, ClientError::Api { status: 409, .. }));
}

#[tokio::test]
async fn size_is_set_on_new_shown_edited_and_kept_by_other_edits() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    let mut req = new_req("Small", TaskKind::Chore);
    req.size = Some(TaskSize::S);
    let task = c.new_task(&req).await.expect("new task");
    assert_eq!(task.size, Some(TaskSize::S));
    let unsized_task = c
        .new_task(&new_req("Unsized", TaskKind::Chore))
        .await
        .expect("new task");
    assert_eq!(unsized_task.size, None);

    let edited = c
        .edit_task(
            &unsized_task.id,
            &EditTaskRequest {
                size: Some(TaskSize::L),
                ..Default::default()
            },
        )
        .await
        .expect("edit size");
    assert_eq!(edited.size, Some(TaskSize::L));
    let retitled = c
        .edit_task(
            &edited.id,
            &EditTaskRequest {
                title: Some("Bigger".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("edit title");
    assert_eq!(retitled.size, Some(TaskSize::L));

    let list = c.list_tasks().await.expect("list");
    // list order isn't guaranteed for tasks created in the same instant
    let size_of = |id: &str| list.iter().find(|t| t.id == id).expect("listed").size;
    assert_eq!(size_of(&task.id), Some(TaskSize::S));
    assert_eq!(size_of(&unsized_task.id), Some(TaskSize::L));
}

#[tokio::test]
async fn done_records_branch_commit_and_a_replaceable_summary() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    let task = c
        .new_task(&new_req("Landed", TaskKind::Feature))
        .await
        .expect("new task");
    assert!(task.summary.is_none());
    c.set_task_summary(&task.id, &SetSummaryRequest { text: "v1".into() })
        .await
        .expect("summary");
    let s = c
        .set_task_summary(&task.id, &SetSummaryRequest { text: "v2".into() })
        .await
        .expect("replace");
    assert_eq!(s.summary.as_deref(), Some("v2"));
    let err = c
        .set_task_summary(&task.id, &SetSummaryRequest { text: " ".into() })
        .await
        .unwrap_err();
    assert!(matches!(err, ClientError::Api { status: 400, .. }));

    let head = git_out(&daemon.repo, &["rev-parse", "HEAD"]);
    let done = c
        .done_task(
            &task.id,
            &DoneTaskRequest {
                commit: head.clone(),
                branch: Some("bridle/landed".into()),
                ..Default::default()
            },
        )
        .await
        .expect("done");
    let shown = c.get_task(&done.id).await.expect("show");
    assert_eq!(shown.branch.as_deref(), Some("bridle/landed"));
    assert_eq!(shown.commit.as_deref(), Some(head.as_str()));
    assert_eq!(shown.summary.as_deref(), Some("v2"));
}

#[tokio::test]
async fn search_matches_words_in_title_body_and_summary() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    let t1 = c
        .new_task(&NewTaskRequest {
            for_human: false,
            components: Vec::new(),
            title: "Fix database connection pool".to_string(),
            kind: TaskKind::Bug,
            body: "The connection pool is leaking memory in production".to_string(),
            size: None,
        })
        .await
        .expect("new task");

    let t2 = c
        .new_task(&NewTaskRequest {
            for_human: false,
            components: Vec::new(),
            title: "Refactor API endpoint".to_string(),
            kind: TaskKind::Feature,
            body: "The endpoint needs optimization for better performance".to_string(),
            size: None,
        })
        .await
        .expect("new task");

    let t3 = c
        .new_task(&NewTaskRequest {
            for_human: false,
            components: Vec::new(),
            title: "Add logging".to_string(),
            kind: TaskKind::Feature,
            body: "Add debug logging to trace requests".to_string(),
            size: None,
        })
        .await
        .expect("new task");

    let _summary = c
        .set_task_summary(
            &t2.id,
            &SetSummaryRequest {
                text: "Optimized the API endpoint and reduced latency".into(),
            },
        )
        .await
        .expect("set summary");

    let list = c.list_tasks().await.expect("list all");
    assert_eq!(list.len(), 3);

    // Search for "connection" - matches t1's title and body
    let results = c.search_tasks(&["connection"]).await.expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, t1.id);

    // Search for "pool" - matches t1's title and body
    let results = c.search_tasks(&["pool"]).await.expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, t1.id);

    // Search for "endpoint" - matches t2's title and summary
    let results = c.search_tasks(&["endpoint"]).await.expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, t2.id);

    // Search for "logging" - matches t3's title and body
    let results = c.search_tasks(&["logging"]).await.expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, t3.id);

    // Case-insensitive search
    let results = c.search_tasks(&["CONNECTION"]).await.expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, t1.id);

    // Multiple words - AND logic, matches t1
    let results = c
        .search_tasks(&["connection", "pool"])
        .await
        .expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, t1.id);

    // Multiple words - no match (both words needed but not in same task)
    let results = c.search_tasks(&["endpoint", "pool"]).await.expect("search");
    assert_eq!(results.len(), 0);

    // No match
    let results = c.search_tasks(&["nonexistent"]).await.expect("search");
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn search_includes_done_and_dropped_tasks() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;

    let open_task = c
        .new_task(&NewTaskRequest {
            for_human: false,
            components: Vec::new(),
            title: "Open task".to_string(),
            kind: TaskKind::Feature,
            body: "Search in open".to_string(),
            size: None,
        })
        .await
        .expect("new task");

    let done_task = c
        .new_task(&NewTaskRequest {
            for_human: false,
            components: Vec::new(),
            title: "Done task".to_string(),
            kind: TaskKind::Feature,
            body: "Search in done".to_string(),
            size: None,
        })
        .await
        .expect("new task");

    let dropped_task = c
        .new_task(&NewTaskRequest {
            for_human: false,
            components: Vec::new(),
            title: "Dropped task".to_string(),
            kind: TaskKind::Feature,
            body: "Search in dropped".to_string(),
            size: None,
        })
        .await
        .expect("new task");

    // Mark one as done
    c.done_task(
        &done_task.id,
        &DoneTaskRequest {
            commit: "abc123".into(),
            branch: None,
            ..Default::default()
        },
    )
    .await
    .expect("done task");

    // Mark one as dropped
    c.drop_task(
        &dropped_task.id,
        &DropTaskRequest {
            reason: "not needed".into(),
        },
    )
    .await
    .expect("drop task");

    // Search for "search" - should find all three
    let results = c.search_tasks(&["search"]).await.expect("search");
    assert_eq!(results.len(), 3);
    let ids: Vec<_> = results.iter().map(|t| t.id.clone()).collect();
    assert!(ids.contains(&open_task.id));
    assert!(ids.contains(&done_task.id));
    assert!(ids.contains(&dropped_task.id));
}

/// `send --task`: the full text is a note on the thread and the recipient
/// gets a short message naming the task; an unknown task sends nothing.
#[tokio::test]
async fn send_with_task_notes_the_thread_and_notifies_briefly() {
    use bridle_api::types::{MessageQuery, SendRequest};
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");
    let external = daemon.external_client("w1").await;

    let long = "first line\nsecond line with the detail";
    let msgs = external
        .send(&SendRequest {
            to: Some("human".to_string()),
            body: long.to_string(),
            task: Some(task.id.clone()),
            ..Default::default()
        })
        .await
        .expect("send with task");
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].body, format!("{}: note added\nfirst line", task.id));

    let fetched = c.get_task(&task.id).await.expect("get task");
    assert_eq!(fetched.thread.len(), 1);
    assert_eq!(fetched.thread[0].body, long);

    let err = external
        .send(&SendRequest {
            to: Some("human".to_string()),
            body: "x".to_string(),
            task: Some("re-nope".to_string()),
            ..Default::default()
        })
        .await
        .expect_err("unknown task");
    assert!(matches!(err, ClientError::Api { status: 404, .. }));
    let inbox = c
        .list_messages(&MessageQuery {
            to: Some("human".to_string()),
            ..Default::default()
        })
        .await
        .expect("list");
    assert_eq!(inbox.len(), 1, "the failed send must not notify");
}

fn git_out(dir: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=t@example.com", "-c", "user.name=T"])
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Spawns an idle worker `name` on its own worktree/branch, with one commit on it.
async fn spawn_worker_with_commit(
    daemon: &support::TestDaemon,
    name: &str,
) -> bridle_api::types::Agent {
    let agent = daemon
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
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let wt = std::path::PathBuf::from(agent.worktree.clone().expect("worktree"));
    std::fs::write(wt.join(format!("{name}.txt")), name).expect("write file");
    git_out(&wt, &["add", "."]);
    git_out(&wt, &["commit", "-q", "-m", name]);
    agent
}

#[tokio::test]
async fn done_with_a_landed_branch_removes_its_agents_worktree_and_branch() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let agent = spawn_worker_with_commit(&daemon, "w1").await;
    let wt = agent.worktree.clone().expect("worktree");
    // Squash-landed: the branch isn't an ancestor of main afterwards.
    git_out(&daemon.repo, &["merge", "--squash", "bridle/w1"]);
    git_out(
        &daemon.repo,
        &["commit", "-q", "-m", "land w1\n\nBranch: bridle/w1"],
    );
    let head = git_out(&daemon.repo, &["rev-parse", "HEAD"]);

    let task = c
        .new_task(&new_req("Landed", TaskKind::Feature))
        .await
        .expect("new task");
    let done = c
        .done_task(
            &task.id,
            &DoneTaskRequest {
                commit: head,
                branch: Some("bridle/w1".into()),
                ..Default::default()
            },
        )
        .await
        .expect("done");

    assert_eq!(done.state, TaskState::Integrated);
    assert!(c.get_agent("w1").await.is_err(), "agent should be removed");
    assert!(
        !std::path::Path::new(&wt).exists(),
        "worktree should be gone"
    );
    assert!(git_out(&daemon.repo, &["branch", "--list", "bridle/w1"]).is_empty());
    let note = done.thread.last().expect("cleanup note");
    assert!(note.body.contains("agent w1"), "note: {}", note.body);
    assert!(
        note.body.contains("branch bridle/w1"),
        "note: {}",
        note.body
    );
}

#[tokio::test]
async fn done_refuses_a_commit_not_on_the_integration_branch() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let agent = spawn_worker_with_commit(&daemon, "w1").await;
    let wt = std::path::PathBuf::from(agent.worktree.clone().expect("worktree"));
    let unmerged = git_out(&wt, &["rev-parse", "HEAD"]);

    let task = c
        .new_task(&new_req("Unlanded", TaskKind::Feature))
        .await
        .expect("new task");
    let err = c
        .done_task(
            &task.id,
            &DoneTaskRequest {
                commit: unmerged,
                branch: Some("bridle/w1".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 409, .. }),
        "{err:?}"
    );
    assert_eq!(
        c.get_task(&task.id).await.expect("show").state,
        TaskState::Open
    );
    assert!(c.get_agent("w1").await.is_ok());
    assert!(wt.exists());
    assert!(!git_out(&daemon.repo, &["branch", "--list", "bridle/w1"]).is_empty());
}

#[tokio::test]
async fn status_lists_stopped_agents_whose_branch_has_merged() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let landed = spawn_worker_with_commit(&daemon, "landed").await;
    let open = spawn_worker_with_commit(&daemon, "open").await;
    git_out(&daemon.repo, &["merge", "-q", "bridle/landed"]);
    assert!(
        c.status()
            .await
            .expect("status")
            .merged_leftovers
            .is_empty()
    );

    for a in [&landed, &open] {
        c.stop(&a.id, &bridle_api::types::StopRequest { now: false })
            .await
            .expect("stop");
    }
    assert_eq!(
        c.status().await.expect("status").merged_leftovers,
        vec!["landed".to_string()]
    );
}

fn spawn_req(name: &str) -> SpawnRequest {
    SpawnRequest {
        components: Vec::new(),
        role: "worker".to_string(),
        name: Some(name.to_string()),
        prompt: None,
        workdir: Some(Workdir::Repo),
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
    }
}

fn agent_client(daemon: &support::TestDaemon, agent_id: &str) -> bridle_api::Client {
    let path = daemon
        .workspace
        .join(".bridle/agents")
        .join(agent_id)
        .join("token");
    let token = std::fs::read_to_string(path).expect("agent token file");
    bridle_api::Client::new(daemon.running.url.clone(), Some(token.trim().to_string()))
}

async fn inbox(c: &bridle_api::Client, to: &str) -> Vec<bridle_api::types::Message> {
    c.list_messages(&bridle_api::types::MessageQuery {
        to: Some(to.to_string()),
        ..Default::default()
    })
    .await
    .expect("list messages")
}

/// br-b966: `ask` sends a pointer (kind question) to `--to`, `answer` sends
/// one back to the asker, and a blank body is refused with nothing sent.
#[tokio::test]
async fn ask_with_to_notifies_that_recipient_and_answer_notifies_the_asker() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let boss = c.spawn(&spawn_req("boss")).await.expect("spawn boss");
    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");

    let err = c
        .ask_question(&task.id, "  ", Some("boss"))
        .await
        .expect_err("blank question");
    assert!(
        matches!(err, ClientError::Api { status: 400, .. }),
        "{err:?}"
    );
    assert!(inbox(c, &boss.id).await.is_empty());

    // An unknown recipient leaves no open question behind.
    let err = c
        .ask_question(&task.id, "which endpoint?", Some("nobody"))
        .await
        .expect_err("unknown recipient");
    assert!(
        matches!(err, ClientError::Api { status: 404, .. }),
        "{err:?}"
    );
    assert!(c.list_open_questions().await.expect("list").is_empty());

    c.ask_question(&task.id, "which endpoint?", Some("boss"))
        .await
        .expect("ask");
    let msgs = inbox(c, &boss.id).await;
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].kind, bridle_api::types::MessageKind::Question);
    assert!(msgs[0].body.contains(&task.id) && msgs[0].body.contains("which endpoint?"));

    // The human answers; the asker (the human here) is not messaged about
    // their own answer, but a boss-asked question gets its pointer back.
    let boss_c = agent_client(&daemon, &boss.id);
    c.answer_question(&task.id, "v1").await.expect("answer");
    assert!(inbox(c, "human").await.is_empty());
    boss_c
        .ask_question(&task.id, "and the port?", Some("human"))
        .await
        .expect("ask as boss");
    assert_eq!(inbox(c, "human").await.len(), 1);
    c.answer_question(&task.id, "8080").await.expect("answer");
    let back = inbox(c, &boss.id).await;
    assert_eq!(back.len(), 2);
    assert_eq!(back[1].kind, bridle_api::types::MessageKind::Answer);
}

/// br-b966: with no `--to`, an agent's question goes to its spawner and a
/// human caller's to the human.
#[tokio::test]
async fn ask_defaults_to_the_spawner_or_the_human() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let mut req = spawn_req("boss");
    req.role = "manager".to_string();
    let boss = c.spawn(&req).await.expect("spawn boss");
    let w1 = agent_client(&daemon, &boss.id)
        .spawn(&spawn_req("w1"))
        .await
        .expect("boss spawns w1");
    let task = c
        .new_task(&new_req("Add foo", TaskKind::Feature))
        .await
        .expect("new task");

    agent_client(&daemon, &w1.id)
        .ask_question(&task.id, "which endpoint?", None)
        .await
        .expect("ask as w1");
    let questions = |msgs: Vec<bridle_api::types::Message>| {
        msgs.into_iter()
            .filter(|m| m.kind == bridle_api::types::MessageKind::Question)
            .count()
    };
    assert_eq!(questions(inbox(c, &boss.id).await), 1);
    assert_eq!(questions(inbox(c, "human").await), 0);

    c.answer_question(&task.id, "v1").await.expect("answer");
    let back = inbox(c, &w1.id).await;
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].kind, bridle_api::types::MessageKind::Answer);

    c.ask_question(&task.id, "and you?", None)
        .await
        .expect("ask as human");
    assert_eq!(inbox(c, "human").await.len(), 1);
}
