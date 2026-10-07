//! Conflict threads (docs/design/impact-and-conflicts.md, "The conflict protocol"):
//! `impact check` opens a conflict once, tells the manager (both tasks are unclaimed),
//! and `conflict resolve` records each of the three outcomes.

mod support;
use support::ClientExt as _;

use std::collections::BTreeMap;

use bridle_api::types::{
    AgentState, EdgeKind, Impact, ImpactCheckRequest, MessageQuery, NewTaskRequest,
    ResolveConflictRequest, SetImpactRequest, SpawnRequest, TaskKind, Workdir,
};
use support::{TestDaemon, start_daemon, wait_for_state};

async fn planned_task(d: &TestDaemon, title: &str, scenario: &str) -> String {
    let c = &d.client;
    let t = c
        .new_open_task(&NewTaskRequest {
            ticket: None,
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
    c.plan_task(&t.id).await.expect("plan");
    let impact = Impact {
        modify: vec![scenario.to_string()],
        ..Impact::default()
    };
    c.set_task_impact(&t.id, &SetImpactRequest { impact })
        .await
        .expect("impact");
    t.id
}

async fn check(d: &TestDaemon) -> Vec<String> {
    d.client
        .impact_check(&ImpactCheckRequest {
            spec_map: BTreeMap::new(),
        })
        .await
        .expect("check")
        .opened
}

#[tokio::test]
async fn conflict_opens_once_notifies_the_manager_and_resolves_each_way() {
    let (d, _tmp) = start_daemon(None).await;
    let c = &d.client;
    let mgr = c
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
    wait_for_state(c, &mgr.id, AgentState::Idle).await;

    let a = planned_task(&d, "A", "s-aaaa").await;
    let b = planned_task(&d, "B", "s-aaaa").await;
    let x = planned_task(&d, "X", "s-bbbb").await;
    let y = planned_task(&d, "Y", "s-bbbb").await;
    let p = planned_task(&d, "P", "s-cccc").await;
    let q = planned_task(&d, "Q", "s-cccc").await;

    let opened = check(&d).await;
    assert_eq!(opened.len(), 3, "one conflict per shared scenario");
    assert!(
        check(&d).await.is_empty(),
        "the same overlap doesn't open a second"
    );
    assert_eq!(c.list_conflicts().await.expect("list").len(), 3);

    // Unclaimed tasks: the manager gets one message per task, naming the other.
    let msgs = c
        .list_messages(&MessageQuery {
            to: Some(mgr.id.clone()),
            ..MessageQuery::default()
        })
        .await
        .expect("messages");
    let conflict_msgs: Vec<_> = msgs
        .iter()
        .filter(|m| m.body.contains("conflict C"))
        .collect();
    assert_eq!(conflict_msgs.len(), 6);
    assert!(
        conflict_msgs
            .iter()
            .any(|m| m.body.contains(&a) && m.body.contains(&b))
    );
    let thread = c.get_task(&a).await.expect("task").thread;
    assert!(thread.iter().any(|e| e.body.contains("conflict C")));

    let by_pair = |c: &bridle_api::types::Conflict| (c.tasks[0].clone(), c.tasks[1].clone());
    let list = c.list_conflicts().await.expect("list");
    let id_of = |t: &str| {
        list.iter()
            .find(|c| by_pair(c).0 == t || by_pair(c).1 == t)
            .expect("conflict")
            .id
            .clone()
    };

    // compatible
    let r = c
        .resolve_conflict(
            &id_of(&a),
            &ResolveConflictRequest {
                compatible: Some("different branches of the flow".into()),
                ..Default::default()
            },
        )
        .await
        .expect("compatible");
    assert_eq!(r.state, "resolved");
    assert_eq!(
        r.resolution.as_deref(),
        Some("compatible: different branches of the flow")
    );
    assert!(
        c.resolve_conflict(
            &id_of(&a),
            &ResolveConflictRequest {
                compatible: Some("again".into()),
                ..Default::default()
            },
        )
        .await
        .is_err(),
        "a resolved conflict stays resolved"
    );

    // order adds a blocks edge
    c.resolve_conflict(
        &id_of(&x),
        &ResolveConflictRequest {
            order: Some([x.clone(), y.clone()]),
            ..Default::default()
        },
    )
    .await
    .expect("order");
    let edges = c.list_edges().await.expect("edges");
    assert!(
        edges
            .iter()
            .any(|e| e.from == x && e.to == y && e.kind == EdgeKind::Blocks)
    );

    // merge-into records the survivor, and only one of the pair is accepted
    assert!(
        c.resolve_conflict(
            &id_of(&p),
            &ResolveConflictRequest {
                merge_into: Some(a.clone()),
                ..Default::default()
            },
        )
        .await
        .is_err()
    );
    let r = c
        .resolve_conflict(
            &id_of(&p),
            &ResolveConflictRequest {
                merge_into: Some(q.clone()),
                ..Default::default()
            },
        )
        .await
        .expect("merge-into");
    assert_eq!(r.resolution, Some(format!("merge-into: {q}")));
}
