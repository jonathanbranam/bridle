//! `POST /v1/statusline`: docs/design/agent-host/api.md, docs/design/usage-and-budget.md.

mod support;

use bridle_api::types::{StatusLineRateLimitReading, StatusLineReport};
use chrono::Utc;
use std::time::Duration;

#[tokio::test]
async fn statusline_report_feeds_rate_limits_and_usage() {
    // The default test daemon polls the fake `get_usage` (five_hour 1%) on
    // every governor tick, which would overwrite the statusline reading;
    // an hour-long interval leaves only the report as a source. The first
    // poll still fires at startup, so wait for it before reporting.
    let (daemon, _tmp) = support::start_daemon(Some(bridle_daemon::Overrides {
        governor_poll_interval_normal: Duration::from_secs(3600),
        governor_poll_interval_above_hold: Duration::from_secs(3600),
        ..support::default_overrides()
    }))
    .await;
    support::wait_for("first get_usage poll", || async {
        let usage = daemon.client.usage().await.ok()?;
        usage
            .rate_limits
            .iter()
            .find(|rl| rl.window == "five_hour")?;
        Some(())
    })
    .await;

    daemon
        .client
        .report_statusline(&StatusLineReport {
            session_id: Some("sess-1".to_string()),
            model: Some("Opus".to_string()),
            cost_usd: Some(0.5),
            context_used_percentage: Some(0.005),
            context_used_tokens: Some(1_000),
            context_max_tokens: Some(200_000),
            rate_limits: vec![StatusLineRateLimitReading {
                window: "five_hour".to_string(),
                utilization: Some(0.42),
                resets_at: Some(Utc::now()),
            }],
        })
        .await
        .expect("report statusline");

    let usage = daemon.client.usage().await.expect("usage");
    assert_eq!(usage.interactive_today.len(), 1);
    assert_eq!(
        usage.interactive_today[0].session_id.as_deref(),
        Some("sess-1")
    );
    assert_eq!(usage.interactive_today[0].cost_usd, Some(0.5));

    let five_hour = usage
        .rate_limits
        .iter()
        .find(|rl| rl.window == "five_hour")
        .expect("five_hour recorded");
    assert_eq!(five_hour.utilization, Some(0.42));

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn statusline_report_with_no_rate_limits_still_records_usage() {
    // The default test daemon polls the fake `get_usage` (five_hour 1%) on
    // every governor tick, which would overwrite the statusline reading;
    // an hour-long interval leaves only the report as a source. The first
    // poll still fires at startup, so wait for it before reporting.
    let (daemon, _tmp) = support::start_daemon(Some(bridle_daemon::Overrides {
        governor_poll_interval_normal: Duration::from_secs(3600),
        governor_poll_interval_above_hold: Duration::from_secs(3600),
        ..support::default_overrides()
    }))
    .await;
    support::wait_for("first get_usage poll", || async {
        let usage = daemon.client.usage().await.ok()?;
        usage
            .rate_limits
            .iter()
            .find(|rl| rl.window == "five_hour")?;
        Some(())
    })
    .await;

    // Capture the rate limits from the poll before reporting.
    let usage_before = daemon.client.usage().await.expect("usage before report");
    let rate_limits_before = usage_before.rate_limits.clone();

    daemon
        .client
        .report_statusline(&StatusLineReport::default())
        .await
        .expect("report statusline");

    let usage = daemon.client.usage().await.expect("usage");
    assert_eq!(usage.interactive_today.len(), 1);
    // The report with no rate limits should not have added new rate limits.
    // Assert that rate_limits is unchanged from what the poll wrote.
    assert_eq!(usage.rate_limits, rate_limits_before);

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}
