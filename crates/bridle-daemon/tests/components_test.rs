//! Components on tasks and spawns (docs/design/components.md, "Scope by the
//! task or spawn, not the cwd").

mod support;
use support::ClientExt as _;

use bridle_api::types::{EditTaskRequest, NewTaskRequest, SpawnRequest, TaskKind, Workdir};
use support::{
    default_overrides, fake_claude_env_dump_wrapper, start_daemon, start_daemon_with_config,
    wait_for_event,
};

const CONFIG: &str = r#"
[components.client]
[components.client-games]
parent = "client"
[components.client-play]
parent = "client"
[components.server]
"#;

fn req(title: &str, components: &[&str]) -> NewTaskRequest {
    NewTaskRequest {
        ticket: None,
        for_human: false,
        priority: None,
        title: title.to_string(),
        kind: TaskKind::Feature,
        body: String::new(),
        size: None,
        components: components.iter().map(|s| s.to_string()).collect(),
    }
}

fn ids(tasks: &[bridle_api::types::Task]) -> Vec<String> {
    tasks.iter().map(|t| t.title.clone()).collect()
}

#[tokio::test]
async fn task_components_round_trip_validate_and_filter_by_descendant() {
    let (daemon, _tmp) = start_daemon_with_config(None, Some(CONFIG)).await;
    let c = &daemon.client;

    let games = c
        .new_open_task(&req("games", &["client-games"]))
        .await
        .expect("games");
    assert_eq!(games.components, vec!["client-games"]);
    c.new_open_task(&req("both", &["client-play", "server", "server"]))
        .await
        .expect("both");
    c.new_open_task(&req("wide", &[])).await.expect("wide");

    // Unknown ids are rejected, on create and on edit.
    let err = c.new_open_task(&req("bad", &["nope"])).await.unwrap_err();
    assert!(err.to_string().contains("no component"), "{err}");
    let err = c
        .edit_task(
            &games.id,
            &EditTaskRequest {
                ticket: None,
                components: Some(vec!["nope".to_string()]),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no component"), "{err}");

    // A parent matches tasks naming its children; a child doesn't match a sibling.
    let client = c.list_tasks_component("client").await.expect("client");
    assert_eq!(ids(&client), vec!["games", "both"]);
    let games_only = c.list_tasks_component("client-games").await.expect("games");
    assert_eq!(ids(&games_only), vec!["games"]);
    let server = c.list_tasks_component("server").await.expect("server");
    assert_eq!(ids(&server), vec!["both"]);
    // Dedupe on the way in.
    let both = c.list_tasks().await.expect("list");
    let both = both
        .iter()
        .find(|t| t.title == "both")
        .expect("both listed");
    assert_eq!(both.components, vec!["client-play", "server"]);

    // Edit replaces the list; an empty list clears it.
    let edited = c
        .edit_task(
            &games.id,
            &EditTaskRequest {
                ticket: None,
                components: Some(vec!["server".to_string()]),
                ..Default::default()
            },
        )
        .await
        .expect("edit");
    assert_eq!(edited.components, vec!["server"]);
    let cleared = c
        .edit_task(
            &games.id,
            &EditTaskRequest {
                ticket: None,
                components: Some(vec![]),
                ..Default::default()
            },
        )
        .await
        .expect("clear");
    assert!(cleared.components.is_empty());
    assert!(cleared.updated_at >= games.updated_at);
}

#[tokio::test]
async fn a_project_without_components_behaves_as_before() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let t = c.new_open_task(&req("plain", &[])).await.expect("new");
    assert!(t.components.is_empty());
    // Naming one is still rejected: there are none to name.
    assert!(c.new_open_task(&req("x", &["a"])).await.is_err());
    assert!(c.list_tasks_component("a").await.expect("list").is_empty());
    assert_eq!(c.list_tasks().await.expect("list").len(), 1);
}

#[tokio::test]
async fn spawn_records_components_and_passes_them_in_the_env() {
    let env_dir = tempfile::tempdir().expect("env tempdir");
    let env_path = env_dir.path().join("env.json");
    let wrapper = fake_claude_env_dump_wrapper(env_dir.path(), &env_path);
    let overrides = bridle_daemon::Overrides {
        claude_program: wrapper.to_string_lossy().into_owned(),
        ..default_overrides()
    };
    let (daemon, _tmp) = start_daemon_with_config(Some(overrides), Some(CONFIG)).await;

    let spawn = |name: &str, components: &[&str]| SpawnRequest {
        role: "worker".to_string(),
        name: Some(name.to_string()),
        workdir: Some(Workdir::Repo),
        components: components.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    };

    let err = daemon
        .client
        .spawn(&spawn("bad", &["nope"]))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no component"), "{err}");

    let agent = daemon
        .client
        .spawn(&spawn("w1", &["client-games", "server"]))
        .await
        .expect("spawn");
    assert_eq!(agent.components, vec!["client-games", "server"]);
    wait_for_event(
        &daemon.client,
        bridle_api::types::event_kind::AGENT_SPAWNED,
        Some(&agent.id),
        |_| true,
    )
    .await;
    let env: std::collections::HashMap<String, String> = support::wait_for_dump(
        "fake-claude env file (w1)",
        &env_path,
        |env: &std::collections::HashMap<String, String>| {
            env.get("BRIDLE_AGENT_NAME").map(String::as_str) == Some("w1")
        },
    )
    .await;
    assert_eq!(
        env.get("BRIDLE_COMPONENTS").map(String::as_str),
        Some("client-games,server")
    );
    // k6b3: an agent's BRIDLE_HOME is its own, never the machine's live one.
    let home = env.get("BRIDLE_HOME").expect("BRIDLE_HOME in agent env");
    assert!(
        home.ends_with(&format!("agent-home/{}", agent.id)),
        "{home}"
    );
    let shown = daemon.client.get_agent("w1").await.expect("get");
    assert_eq!(shown.components, vec!["client-games", "server"]);
}
