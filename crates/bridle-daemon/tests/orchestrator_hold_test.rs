//! `bridle orchestrator hold` / `release` (br-8fsx): human only, shown in status, persisted.

mod support;

use bridle_api::ClientError;

#[tokio::test]
async fn hold_is_human_only_shows_in_status_survives_a_restart_and_releases() {
    let (daemon, tmp) = support::start_daemon(None).await;
    let workspace = daemon.workspace.clone();
    let repo = daemon.repo.clone();

    let other = daemon.external_client("orchestrator").await;
    let err = other
        .orchestrator_hold(Some("nope".into()))
        .await
        .expect_err("forbidden");
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );
    let err = other.orchestrator_release().await.expect_err("forbidden");
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );
    assert!(
        daemon
            .client
            .status()
            .await
            .unwrap()
            .orchestrator_hold
            .is_none()
    );

    daemon
        .client
        .orchestrator_hold(Some("fixing the box".into()))
        .await
        .expect("hold");
    let held = daemon.client.status().await.unwrap().orchestrator_hold;
    assert_eq!(held.unwrap().reason.as_deref(), Some("fixing the box"));

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");

    let opts = bridle_daemon::ServeOptions {
        repo: repo.clone(),
        workspace: Some(workspace.clone()),
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
        usage_http: bridle_daemon::usage_http::UsageHttp {
            url: "http://127.0.0.1:9/".to_string(),
            token: Some("test".to_string()),
        },
        task_flush_interval: std::time::Duration::from_secs(3600),
        claim_lease_check_interval: std::time::Duration::from_secs(3600),
        port_check_interval: std::time::Duration::from_secs(3600),
        load_watch: false,
        upgrade: Default::default(),
        ci_tick_interval: std::time::Duration::from_secs(3600),
        drain_wake_after: std::time::Duration::from_secs(600),
        settle_wake_interval: std::time::Duration::from_secs(3600),
        doc_watch_interval: std::time::Duration::from_secs(3600),
        queue_nudge_debounce: std::time::Duration::from_secs(3600),
        open_watch_debounce: std::time::Duration::from_secs(3600),
        host: None,
    };
    let running = bridle_daemon::start(opts, overrides)
        .await
        .expect("start second daemon");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));

    let held = client.status().await.unwrap().orchestrator_hold;
    assert_eq!(held.unwrap().reason.as_deref(), Some("fixing the box"));
    assert_eq!(client.orchestrator_release().await.unwrap(), (true, false));
    assert!(client.status().await.unwrap().orchestrator_hold.is_none());
    assert_eq!(client.orchestrator_release().await.unwrap(), (false, false));

    running.shutdown();
    running.join().await.expect("join");
    drop(tmp);
}

#[test]
fn autostart_is_read_from_the_machine_config_only_when_asked() {
    let home = tempfile::tempdir().unwrap();
    assert!(bridle_daemon::config::Config::machine_orchestrator_autostart(home.path()));
    std::fs::write(
        home.path().join("config.toml"),
        "[orchestrator]\nautostart = false\n",
    )
    .unwrap();
    assert!(!bridle_daemon::config::Config::machine_orchestrator_autostart(home.path()));
    std::fs::write(home.path().join("config.toml"), "[orchestrator\n").unwrap();
    assert!(
        bridle_daemon::config::Config::machine_orchestrator_autostart(home.path()),
        "a typo must not keep it down"
    );
}
