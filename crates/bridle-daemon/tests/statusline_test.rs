//! `POST /v1/statusline`: docs/design/agent-host/api.md, docs/design/usage-and-budget.md.

mod support;

use bridle_api::types::{StatusLineRateLimitReading, StatusLineReport};
use chrono::Utc;

#[tokio::test]
async fn statusline_report_feeds_rate_limits_and_usage() {
    let (daemon, _tmp) = support::start_daemon(None).await;

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
    let (daemon, _tmp) = support::start_daemon(None).await;

    daemon
        .client
        .report_statusline(&StatusLineReport::default())
        .await
        .expect("report statusline");

    let usage = daemon.client.usage().await.expect("usage");
    assert_eq!(usage.interactive_today.len(), 1);
    assert!(usage.rate_limits.is_empty());

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}
