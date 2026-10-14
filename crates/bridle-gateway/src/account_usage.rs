//! Account usage summed over every project the gateway reaches (br-v6cg, ktgd step 3):
//! `GET /api/v1/usage/account?since=&until=`. Each reachable daemon is asked for its usage
//! groups (role -> name -> session) and its periods. The rate limits are account-wide, so the
//! periods come once, from the first daemon that answers, and are never summed. A daemon that
//! can't answer is listed as unreachable, never counted as zero. Shares are re-estimated here
//! over the account's cost, since each daemon's own share is of its own project's cost.

use std::time::Duration;

use axum::Json;
use axum::extract::Query;
use bridle_api::client::Client;
use bridle_api::types::{
    RateLimitPoint, TokenTotals, UsageGroups, UsageGroupsQuery, UsagePeriods, UsagePeriodsQuery,
    UsageShareGroup,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::discovery::{PROBE_TIMEOUT, ProjectStatus, current_targets};
use crate::items::{Source, source};

const SHARE_NOTE: &str = "share_points is an estimate: the group's share of the account's cost \
x the seven_day points gained. Anthropic doesn't publish the limit's weights; points_per_dollar \
is the measured ratio.";

#[derive(Debug, Deserialize)]
pub struct AccountUsageParams {
    /// RFC 3339; only turns, reports and readings at or after it.
    pub since: Option<DateTime<Utc>>,
    /// RFC 3339; only turns and reports before it.
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountTokens {
    #[ts(type = "number")]
    pub input: u64,
    #[ts(type = "number")]
    pub output: u64,
    #[ts(type = "number")]
    pub cache_read: u64,
    #[ts(type = "number")]
    pub cache_write: u64,
}

/// One row at any level: a role, a name in it, or a session of that name.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountGroup {
    /// The role, the name, or the session id.
    pub key: String,
    /// The project and machine the row came from; `None` when a rolled-up row spans several.
    pub project: Option<String>,
    pub machine: Option<String>,
    /// `None` when no row of the group counts turns (external sessions).
    pub turns: Option<u32>,
    pub tokens: AccountTokens,
    pub cost_usd: f64,
    #[ts(type = "number")]
    pub context_growth: i64,
    /// Estimate: this group's share of the account's cost x the seven_day points.
    pub share_points: f64,
    /// A session only: the hosted agent id; `None` for an external session.
    pub agent_id: Option<String>,
    pub external: bool,
    /// A role's names, then a name's sessions.
    pub children: Vec<AccountGroup>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountDay {
    /// `YYYY-MM-DD` in the response's `timezone`.
    pub date: String,
    pub starts_at: String,
    pub ends_at: String,
    pub peak_seven_day: Option<f64>,
    pub peak_five_hour: Option<f64>,
    pub hit_five_hour: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountWeek {
    pub starts_at: String,
    pub resets_at: String,
    pub peak: f64,
    pub hit_limit: bool,
    pub days: Vec<AccountDay>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountWindow {
    pub starts_at: String,
    pub resets_at: String,
    pub peak: f64,
    pub hit_limit: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountReading {
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
    pub observed_at: String,
}

/// The account's weeks, days and real five-hour windows, with the raw readings. Taken once.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountPeriods {
    pub timezone: String,
    pub weeks: Vec<AccountWeek>,
    pub windows: Vec<AccountWindow>,
    pub five_hour: Vec<AccountReading>,
    pub seven_day: Vec<AccountReading>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct AccountUsage {
    /// Every project asked, with `reachable` and the reason it wasn't. Totals cover the
    /// reachable ones only.
    pub projects: Vec<ProjectStatus>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub total_cost_usd: f64,
    /// Seven_day points gained in the period (0-100): account-wide, counted once.
    pub seven_day_points: f64,
    /// Measured: `seven_day_points / total_cost_usd`; `None` with no cost or no points.
    pub points_per_dollar: Option<f64>,
    pub share_note: String,
    /// Summed across projects by role, then name, then session.
    pub roles: Vec<AccountGroup>,
    /// `None` when no reachable daemon could give its periods.
    pub periods: Option<AccountPeriods>,
}

fn rfc(t: DateTime<Utc>) -> String {
    t.to_rfc3339()
}

fn tokens(t: &TokenTotals) -> AccountTokens {
    AccountTokens {
        input: t.input,
        output: t.output,
        cache_read: t.cache_read,
        cache_write: t.cache_write,
    }
}

fn group(g: &UsageShareGroup) -> AccountGroup {
    AccountGroup {
        key: g.key.clone(),
        project: Some(g.project.clone()),
        machine: Some(g.machine.clone()),
        turns: g.turns,
        tokens: tokens(&g.tokens),
        cost_usd: g.cost_usd,
        context_growth: g.context_growth,
        share_points: 0.0,
        agent_id: g.agent_id.clone(),
        external: g.external,
        children: g.children.iter().map(group).collect(),
    }
}

/// Folds `from` into `into` (same key): sums, and merges children by key. Project and machine
/// become `None` once the rows differ.
fn merge(into: &mut AccountGroup, from: AccountGroup) {
    if into.project != from.project {
        into.project = None;
    }
    if into.machine != from.machine {
        into.machine = None;
    }
    into.turns = match (into.turns, from.turns) {
        (None, None) => None,
        (a, b) => Some(a.unwrap_or(0) + b.unwrap_or(0)),
    };
    into.tokens.input += from.tokens.input;
    into.tokens.output += from.tokens.output;
    into.tokens.cache_read += from.tokens.cache_read;
    into.tokens.cache_write += from.tokens.cache_write;
    into.cost_usd += from.cost_usd;
    into.context_growth += from.context_growth;
    into.external &= from.external;
    merge_all(&mut into.children, from.children);
}

fn merge_all(into: &mut Vec<AccountGroup>, from: Vec<AccountGroup>) {
    for g in from {
        // Sessions are unique per agent or session id, so equal keys there are the same
        // session seen twice (or the empty "before sessions" key): sum them too.
        match into.iter_mut().find(|x| x.key == g.key) {
            Some(x) => merge(x, g),
            None => into.push(g),
        }
    }
}

fn set_shares(groups: &mut [AccountGroup], total_cost: f64, points: f64) {
    for g in groups {
        g.share_points = if total_cost > 0.0 {
            g.cost_usd / total_cost * points
        } else {
            0.0
        };
        set_shares(&mut g.children, total_cost, points);
    }
}

fn reading(p: &RateLimitPoint) -> AccountReading {
    AccountReading {
        utilization: p.utilization,
        resets_at: p.resets_at.map(rfc),
        observed_at: rfc(p.observed_at),
    }
}

fn periods(p: UsagePeriods) -> AccountPeriods {
    AccountPeriods {
        timezone: p.timezone,
        weeks: p
            .weeks
            .into_iter()
            .map(|w| AccountWeek {
                starts_at: rfc(w.starts_at),
                resets_at: rfc(w.resets_at),
                peak: w.peak,
                hit_limit: w.hit_limit,
                days: w
                    .days
                    .into_iter()
                    .map(|d| AccountDay {
                        date: d.date,
                        starts_at: rfc(d.starts_at),
                        ends_at: rfc(d.ends_at),
                        peak_seven_day: d.peak_seven_day,
                        peak_five_hour: d.peak_five_hour,
                        hit_five_hour: d.hit_five_hour,
                    })
                    .collect(),
            })
            .collect(),
        windows: p
            .windows
            .into_iter()
            .map(|w| AccountWindow {
                starts_at: rfc(w.starts_at),
                resets_at: rfc(w.resets_at),
                peak: w.peak,
                hit_limit: w.hit_limit,
            })
            .collect(),
        five_hour: p.five_hour.iter().map(reading).collect(),
        seven_day: p.seven_day.iter().map(reading).collect(),
    }
}

struct Read {
    status: ProjectStatus,
    groups: Option<UsageGroups>,
    periods: Option<UsagePeriods>,
}

async fn read_project(s: Source, params: &AccountUsageParams, timeout: Duration) -> Read {
    let mut status = ProjectStatus {
        project: s.project.clone(),
        machine: s.machine.clone(),
        url: s.url.clone(),
        reachable: false,
        reason: None,
    };
    let down = |mut status: ProjectStatus, reason: String| {
        status.reason = Some(reason);
        Read {
            status,
            groups: None,
            periods: None,
        }
    };
    if let Some(problem) = &s.problem {
        return down(status, problem.clone());
    }
    let Some(url) = &s.url else {
        return down(status, "no address".to_string());
    };
    let client = Client::new_with_timeout(url.clone(), s.token.clone(), timeout);
    let groups = client
        .usage_groups(&UsageGroupsQuery {
            since: params.since,
            until: params.until,
        })
        .await;
    let groups = match groups {
        Ok(g) => g,
        Err(e) => return down(status, e.to_string()),
    };
    // The groups are what the sums need; periods are best effort (an older daemon lacks them).
    let periods = client
        .usage_periods(&UsagePeriodsQuery {
            since: params.since,
        })
        .await
        .ok();
    status.reachable = true;
    Read {
        status,
        groups: Some(groups),
        periods,
    }
}

pub(crate) async fn account_usage_from(
    sources: Vec<Source>,
    params: AccountUsageParams,
    timeout: Duration,
) -> AccountUsage {
    let reads = sources
        .into_iter()
        .map(|s| read_project(s, &params, timeout));
    let mut reads = futures::future::join_all(reads).await;
    reads.sort_by(|a, b| a.status.project.cmp(&b.status.project));

    let mut roles: Vec<AccountGroup> = Vec::new();
    let mut total_cost = 0.0;
    let mut points: f64 = 0.0;
    let mut account_periods = None;
    let mut statuses = Vec::new();
    for r in reads {
        statuses.push(r.status);
        if let Some(g) = r.groups {
            total_cost += g.total_cost_usd;
            // Every daemon reads the same account limits; a gap in one's history only lowers
            // its figure, so the highest is the best reading.
            points = points.max(g.seven_day_points);
            merge_all(&mut roles, g.roles.iter().map(group).collect());
        }
        if account_periods.is_none() {
            account_periods = r.periods.map(periods);
        }
    }
    roles.sort_by(|a, b| b.cost_usd.total_cmp(&a.cost_usd).then(a.key.cmp(&b.key)));
    set_shares(&mut roles, total_cost, points);
    AccountUsage {
        projects: statuses,
        since: params.since.map(rfc),
        until: params.until.map(rfc),
        total_cost_usd: total_cost,
        seven_day_points: points,
        points_per_dollar: (total_cost > 0.0 && points > 0.0).then(|| points / total_cost),
        share_note: SHARE_NOTE.to_string(),
        roles,
        periods: account_periods,
    }
}

/// `GET /api/v1/usage/account?since=&until=`: see the module doc. Never an error for a down
/// daemon; those are in `projects` with `reachable: false`.
pub async fn account_usage_route(Query(params): Query<AccountUsageParams>) -> Json<AccountUsage> {
    let targets = current_targets().await;
    // Token lookup reads files: blocking work.
    let sources = tokio::task::spawn_blocking(move || targets.into_iter().map(source).collect())
        .await
        .unwrap_or_default();
    Json(account_usage_from(sources, params, PROBE_TIMEOUT * 5).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use serde_json::{Value, json};

    fn row(key: &str, project: &str, cost: f64, children: Vec<Value>) -> Value {
        json!({
            "key": key, "project": project, "machine": "m", "turns": 2,
            "tokens": {"input": 10, "output": 5, "cache_read": 0, "cache_write": 0},
            "cost_usd": cost, "context_growth": 100, "share_points": 99.0,
            "children": children,
        })
    }

    async fn daemon(project: &str, cost: f64) -> String {
        let session = row("s-1", project, cost, vec![]);
        let name = row("pm", project, cost, vec![session]);
        let role = row("advisor", project, cost, vec![name]);
        let groups = json!({
            "project": project, "machine": "m", "total_cost_usd": cost,
            "seven_day_points": 30.0, "share_note": "est", "roles": [role],
        });
        let periods = json!({
            "timezone": "America/New_York",
            "weeks": [{"starts_at": "2026-01-01T00:00:00Z", "resets_at": "2026-01-08T00:00:00Z",
                "peak": 0.3, "hit_limit": false, "days": []}],
            "windows": [], "five_hour": [], "seven_day": [],
        });
        let app = Router::new()
            .route("/v1/usage/groups", get(move || async move { Json(groups) }))
            .route(
                "/v1/usage/periods",
                get(move || async move { Json(periods) }),
            );
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        url
    }

    fn src(project: &str, url: &str) -> Source {
        Source {
            project: project.into(),
            machine: Some("m".into()),
            url: Some(url.into()),
            token: None,
            problem: None,
        }
    }

    #[tokio::test]
    async fn sums_reachable_projects_and_lists_the_down_one() {
        let a = daemon("alpha", 1.0).await;
        let b = daemon("beta", 3.0).await;
        let sources = vec![
            src("beta", &b),
            src("gamma", "http://127.0.0.1:1"),
            src("alpha", &a),
        ];
        let params = AccountUsageParams {
            since: None,
            until: None,
        };
        let u = account_usage_from(sources, params, Duration::from_secs(2)).await;
        let reach: Vec<_> = u
            .projects
            .iter()
            .map(|p| (p.project.as_str(), p.reachable))
            .collect();
        assert_eq!(reach, [("alpha", true), ("beta", true), ("gamma", false)]);
        assert!(u.projects[2].reason.is_some());
        assert_eq!(u.total_cost_usd, 4.0);
        // Account-wide points are counted once, not per project.
        assert_eq!(u.seven_day_points, 30.0);
        assert_eq!(u.points_per_dollar, Some(7.5));
        assert_eq!(u.roles.len(), 1);
        let role = &u.roles[0];
        assert_eq!(
            (role.key.as_str(), role.cost_usd, role.turns),
            ("advisor", 4.0, Some(4))
        );
        assert_eq!(role.tokens.input, 20);
        assert_eq!(role.context_growth, 200);
        // Spans two projects, so the rolled-up row names neither.
        assert_eq!(
            (role.project.clone(), role.machine.clone()),
            (None, Some("m".into()))
        );
        // The shares are re-estimated over the account and add up to the points.
        assert!((role.share_points - 30.0).abs() < 1e-9);
        assert!((role.children[0].children[0].share_points - 30.0).abs() < 1e-9);
        let p = u.periods.expect("periods once");
        assert_eq!(p.weeks.len(), 1);
    }

    #[tokio::test]
    async fn nothing_reachable_is_empty_not_zero_usage() {
        let params = AccountUsageParams {
            since: None,
            until: None,
        };
        let u = account_usage_from(
            vec![src("gamma", "http://127.0.0.1:1")],
            params,
            Duration::from_secs(1),
        )
        .await;
        assert!(!u.projects[0].reachable);
        assert!(u.roles.is_empty() && u.periods.is_none());
    }
}
