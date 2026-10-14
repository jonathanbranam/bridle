//! A period's usage grouped role -> name -> session with share-of-week estimates (br-zn6s,
//! br-ktgd decision 4). Pure over the store's session rows and the seven_day history.

use std::collections::BTreeMap;

use bridle_api::types::{RateLimitPoint, TokenTotals, UsageGroups, UsageShareGroup};
use chrono::{DateTime, Utc};

use crate::store::SessionUsage;
use crate::usage_periods::RESET_TOLERANCE;

const SHARE_NOTE: &str = "estimate: each group's share of the period's cost x the seven_day \
points the period gained; Anthropic does not publish the limit's weights";

/// The seven_day percentage points gained in `[since, until)`: per week (readings clustered by
/// `resets_at`), the highest reading in range minus the last one before `since` in the same
/// week (0 when the week starts in range), summed.
pub fn seven_day_points(
    history: &[RateLimitPoint],
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> f64 {
    let mut rs: Vec<(DateTime<Utc>, DateTime<Utc>, f64)> = history
        .iter()
        .filter_map(|p| Some((p.resets_at?, p.observed_at, p.utilization?)))
        .collect();
    rs.sort_by_key(|(r, at, _)| (*r, *at));
    let mut weeks: Vec<Vec<(DateTime<Utc>, f64)>> = Vec::new();
    let mut last_reset: Option<DateTime<Utc>> = None;
    for (r, at, u) in rs {
        if last_reset.is_none_or(|l| r - l > RESET_TOLERANCE) {
            weeks.push(Vec::new());
        }
        last_reset = Some(r);
        if let Some(w) = weeks.last_mut() {
            w.push((at, u));
        }
    }
    let mut total = 0.0;
    for mut w in weeks {
        w.sort_by_key(|(at, _)| *at);
        let in_range =
            |at: DateTime<Utc>| since.is_none_or(|s| at >= s) && until.is_none_or(|u| at < u);
        let Some(peak) = w
            .iter()
            .filter(|(at, _)| in_range(*at))
            .map(|(_, u)| *u)
            .reduce(f64::max)
        else {
            continue;
        };
        let before = since
            .and_then(|s| w.iter().rev().find(|(at, _)| *at < s).map(|(_, u)| *u))
            .unwrap_or(0.0);
        total += (peak - before).max(0.0) * 100.0;
    }
    total
}

#[derive(Default)]
struct Sums {
    turns: Option<u32>,
    tokens: TokenTotals,
    cost: f64,
    growth: i64,
}

impl Sums {
    fn add(&mut self, o: &Sums) {
        if let Some(t) = o.turns {
            *self.turns.get_or_insert(0) += t;
        }
        self.tokens.input += o.tokens.input;
        self.tokens.output += o.tokens.output;
        self.tokens.cache_read += o.tokens.cache_read;
        self.tokens.cache_write += o.tokens.cache_write;
        self.cost += o.cost;
        self.growth += o.growth;
    }
}

pub fn build(
    sessions: Vec<SessionUsage>,
    points: f64,
    project: &str,
    machine: &str,
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> UsageGroups {
    let total_cost: f64 = sessions.iter().map(|s| s.cost_usd).sum();
    let group = |key: &str,
                 sums: &Sums,
                 agent_id: Option<String>,
                 children: Vec<UsageShareGroup>| UsageShareGroup {
        key: key.to_string(),
        project: project.to_string(),
        machine: machine.to_string(),
        turns: sums.turns,
        tokens: sums.tokens.clone(),
        cost_usd: sums.cost,
        context_growth: sums.growth,
        share_points: if total_cost > 0.0 {
            sums.cost / total_cost * points
        } else {
            0.0
        },
        external: agent_id.is_none(),
        agent_id,
        children,
    };
    let mut tree: BTreeMap<String, BTreeMap<String, Vec<SessionUsage>>> = BTreeMap::new();
    for s in sessions {
        tree.entry(s.role.clone())
            .or_default()
            .entry(s.name.clone())
            .or_default()
            .push(s);
    }
    let mut roles = Vec::new();
    for (role, names) in tree {
        let (mut role_sums, mut name_groups) = (Sums::default(), Vec::new());
        for (name, sessions) in names {
            let (mut name_sums, mut session_groups) = (Sums::default(), Vec::new());
            for s in sessions {
                let sums = Sums {
                    turns: s.turns,
                    tokens: s.tokens.clone(),
                    cost: s.cost_usd,
                    growth: match (s.first_context, s.last_context) {
                        (Some(a), Some(b)) => b - a,
                        _ => 0,
                    },
                };
                name_sums.add(&sums);
                session_groups.push(group(&s.session_id, &sums, s.agent_id, Vec::new()));
            }
            role_sums.add(&name_sums);
            name_groups.push(group(&name, &name_sums, None, session_groups));
        }
        roles.push(group(&role, &role_sums, None, name_groups));
    }
    UsageGroups {
        project: project.to_string(),
        machine: machine.to_string(),
        since,
        until,
        total_cost_usd: total_cost,
        seven_day_points: points,
        points_per_dollar: (total_cost > 0.0 && points > 0.0).then(|| points / total_cost),
        share_note: SHARE_NOTE.to_string(),
        roles,
    }
}
