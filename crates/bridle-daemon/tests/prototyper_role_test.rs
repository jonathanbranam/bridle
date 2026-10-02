//! The base `prototyper` role: loads from the real workflow dir, spawns by name, and takes
//! a project's `.bridle/roles/prototyper.md` append.

mod support;

use std::path::Path;

use bridle_api::types::{AgentState, SpawnRequest, Workdir};
use bridle_daemon::config::{BranchesConfig, CommandsConfig, Config, stable_system_prompt};
use support::{start_daemon_with_config, wait_for_state};

fn real_workflow() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../workflow")
        .canonicalize()
        .expect("workflow dir")
        .display()
        .to_string()
}

fn load(repo: &Path) -> Config {
    std::fs::create_dir_all(repo.join(".bridle")).expect("mkdir");
    std::fs::write(
        repo.join(".bridle/config.toml"),
        format!("workflow = {:?}\n", real_workflow()),
    )
    .expect("write");
    Config::load_with_home(repo, Some(repo)).expect("config loads with the real workflow")
}

fn prompt(cfg: &Config, repo: &Path) -> String {
    stable_system_prompt(
        "prototyper",
        &cfg.roles["prototyper"],
        repo,
        &BranchesConfig::default(),
        &CommandsConfig::default(),
    )
}

#[test]
fn real_base_roles_dir_loads_and_prototyper_gets_its_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = load(dir.path());
    // Every role file in the real dir is readable text; none can break loading.
    for entry in std::fs::read_dir(Path::new(&real_workflow()).join("base/roles")).expect("dir") {
        let path = entry.expect("entry").path();
        assert!(!std::fs::read_to_string(&path).expect("reads").is_empty());
    }
    let role = &cfg.roles["prototyper"];
    assert_eq!(role.model, "sonnet");
    assert!(role.system_prompt.is_some());
    let text = prompt(&cfg, dir.path());
    assert!(text.contains("You are a prototyper"), "{text}");
    assert!(
        text.contains("The prototype prompt is the whole brief"),
        "{text}"
    );
    assert!(text.contains("genuinely different approach"), "{text}");
    assert!(text.contains("The project decides"), "{text}");
}

#[test]
fn project_append_is_added_and_absent_is_fine() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = load(dir.path());
    assert!(!prompt(&cfg, dir.path()).contains("Prototypes live in"));
    std::fs::create_dir_all(dir.path().join(".bridle/roles")).expect("mkdir");
    std::fs::write(
        dir.path().join(".bridle/roles/prototyper.md"),
        "Prototypes live in proto/<name>/.\n",
    )
    .expect("write");
    let text = prompt(&cfg, dir.path());
    assert!(text.contains("The project decides"), "{text}");
    assert!(text.contains("Prototypes live in proto/<name>/."), "{text}");
}

#[tokio::test]
async fn spawns_by_role_name_with_no_config() {
    let (daemon, _repo) =
        start_daemon_with_config(None, Some("[roles.manager]\nautostart = false\n")).await;
    let a = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "prototyper".to_string(),
            name: Some("proto".to_string()),
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
}
