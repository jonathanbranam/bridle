//! One agent per branch: spawning into a directory that is another agent's
//! worktree is refused until that agent is removed; renew still reuses it.

mod support;

use bridle_api::types::{
    AgentState, RemoveQuery, RenewRequest, SpawnRequest, StopRequest, Workdir,
};
use support::{start_daemon, wait_for_state};

fn req(name: &str, workdir: Workdir) -> SpawnRequest {
    SpawnRequest {
        components: Vec::new(),
        role: "worker".to_string(),
        name: Some(name.to_string()),
        prompt: None,
        workdir: Some(workdir),
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
    }
}

#[tokio::test]
async fn spawn_into_another_agents_worktree_is_refused_until_it_is_removed() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let first = c
        .spawn(&req("w1", Workdir::Worktree { base: None }))
        .await
        .expect("spawn");
    let wt = first.worktree.clone().expect("worktree");
    wait_for_state(c, &first.id, AgentState::Idle).await;

    // Refused while it is running, and while it is stopped.
    let err = c
        .spawn(&req("w1-2", Workdir::Path { path: wt.clone() }))
        .await
        .expect_err("running holder");
    let msg = err.to_string();
    assert!(
        msg.contains("bridle renew w1") && msg.contains("bridle remove w1"),
        "{msg}"
    );
    c.stop(&first.id, &StopRequest { now: true })
        .await
        .expect("stop");
    wait_for_state(c, &first.id, AgentState::Stopped).await;
    c.spawn(&req("w1-2", Workdir::Path { path: wt.clone() }))
        .await
        .expect_err("stopped holder");

    // Renew keeps working: the same agent reuses its branch.
    let renewed = c
        .renew(
            &first.id,
            &RenewRequest {
                ignore_budget: false,
            },
        )
        .await
        .expect("renew");
    assert_eq!(renewed.branch, first.branch);
    c.stop(&first.id, &StopRequest { now: true })
        .await
        .expect("stop");
    wait_for_state(c, &first.id, AgentState::Stopped).await;

    // Once the holder is removed the directory is free (branch kept).
    c.remove(&first.id, &RemoveQuery::default())
        .await
        .expect("remove");
    // Remove deletes the worktree; recreate the directory.
    std::fs::create_dir_all(&wt).expect("mkdir");
    c.spawn(&req("w1-2", Workdir::Path { path: wt }))
        .await
        .expect("spawn after removal");
}
