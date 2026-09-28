//! g3ck: with `[branches] integration` unset the daemon needs `main`; a repo on another
//! branch must fail at startup naming the setting, not on every spawn.

mod support;

fn overrides(tmp: &std::path::Path) -> bridle_daemon::Overrides {
    bridle_daemon::Overrides {
        claude_program: support::fake_claude_path().to_string_lossy().into_owned(),
        write_registry: false,
        bridle_home: Some(support::machine_home_dir(tmp)),
        ..Default::default()
    }
}

#[tokio::test]
async fn start_fails_naming_the_setting_when_default_main_is_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    support::init_repo(&repo).await;
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["branch", "-m", "main", "master"])
        .output()
        .expect("git branch -m");
    assert!(out.status.success());

    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: Some(tmp.path().join("ws")),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let err = bridle_daemon::start(opts, overrides(tmp.path()))
        .await
        .err()
        .expect("start must fail");
    assert!(
        err.to_string().contains("branches.integration"),
        "unhelpful error: {err}"
    );
}
