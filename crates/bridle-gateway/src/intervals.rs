//! The human-time interval math, pure so tests use fixed fixtures (ticket u6w9, docs/design/
//! human-web-ui.md "Human time"). A prompt is a point in time; time is inferred from the prompts
//! and the replies that follow them, per session.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};

use crate::config::InteractionsConfig;
use crate::interactions::SessionInterval;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The human sent something: a prompt in a session, or a message through bridle.
    Prompt,
    /// The agent finished answering the last prompt.
    Reply,
}

/// One collected point: a prompt, a reply, or a message the human sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub machine: String,
    pub project: String,
    /// The prompt's role, or the recipient of a message.
    pub agent: String,
    pub session: String,
    pub at: DateTime<Utc>,
    pub kind: Kind,
}

/// A stretch of one session's time the human spent on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    pub machine: String,
    pub project: String,
    pub agent: String,
    pub session: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl From<&Interval> for SessionInterval {
    fn from(i: &Interval) -> Self {
        Self {
            machine: i.machine.clone(),
            project: i.project.clone(),
            agent: i.agent.clone(),
            session: i.session.clone(),
            start: i.start.to_rfc3339(),
            end: i.end.to_rfc3339(),
        }
    }
}

fn secs(d: std::time::Duration) -> Duration {
    Duration::seconds(d.as_secs() as i64)
}

/// Every session's intervals, ordered by start. Within a session they never overlap.
pub fn intervals(events: &[Event], cfg: &InteractionsConfig) -> Vec<Interval> {
    let (gap, tail, lead) = (secs(cfg.gap), secs(cfg.tail), secs(cfg.lead));
    let mut sessions: BTreeMap<(&str, &str, &str), Vec<&Event>> = BTreeMap::new();
    for e in events {
        sessions
            .entry((&e.machine, &e.project, &e.session))
            .or_default()
            .push(e);
    }
    let mut out = Vec::new();
    for mut evs in sessions.into_values() {
        evs.sort_by_key(|e| (e.at, e.kind == Kind::Reply));
        let prompts: Vec<&Event> = evs
            .iter()
            .copied()
            .filter(|e| e.kind == Kind::Prompt)
            .collect();
        // Set while a run continues: where the next prompt's interval starts.
        let mut cursor: Option<DateTime<Utc>> = None;
        let mut prev_end: Option<DateTime<Utc>> = None;
        for (i, p) in prompts.iter().enumerate() {
            let next = prompts.get(i + 1).map(|q| q.at);
            // The last reply between this prompt and the next: the agent's turn ending.
            let reply = evs
                .iter()
                .filter(|e| e.kind == Kind::Reply && e.at > p.at && next.is_none_or(|n| e.at < n))
                .map(|e| e.at)
                .next_back();
            let (end, continues) = match (reply, next) {
                (Some(r), Some(n)) if n - r <= gap => (n, true),
                (Some(r), _) => (r + tail, false),
                (None, Some(n)) if n - p.at <= gap => (n, true),
                (None, _) => (p.at + tail, false),
            };
            // A run's first prompt gets its lead, but never into the session's previous interval.
            let start = cursor.unwrap_or_else(|| {
                let lead_start = p.at - lead;
                prev_end.map_or(lead_start, |e| lead_start.max(e))
            });
            out.push(Interval {
                machine: p.machine.clone(),
                project: p.project.clone(),
                agent: p.agent.clone(),
                session: p.session.clone(),
                start,
                end,
            });
            cursor = continues.then_some(end);
            prev_end = Some(end);
        }
    }
    out.sort_by(|a, b| (a.start, &a.session).cmp(&(b.start, &b.session)));
    out
}

/// Merges spans into disjoint ones, ordered; touching spans join.
pub fn union(
    mut spans: Vec<(DateTime<Utc>, DateTime<Utc>)>,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    spans.sort();
    let mut out: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
    for (s, e) in spans {
        match out.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => out.push((s, e)),
        }
    }
    out
}

pub fn minutes(spans: &[(DateTime<Utc>, DateTime<Utc>)]) -> f64 {
    spans
        .iter()
        .map(|(s, e)| (*e - *s).as_seconds_f64() / 60.0)
        .sum()
}

/// The human's time: the union of every session's intervals, so two conversations at once count
/// once.
pub fn human_time(intervals: &[Interval]) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    union(intervals.iter().map(|i| (i.start, i.end)).collect())
}

/// Disjoint stretches and how many sessions cover each: the simultaneous conversations. Ordered;
/// moments nobody covers are left out.
pub fn concurrency(intervals: &[Interval]) -> Vec<(DateTime<Utc>, DateTime<Utc>, u32)> {
    let mut edges: BTreeMap<DateTime<Utc>, i32> = BTreeMap::new();
    for i in intervals.iter().filter(|i| i.end > i.start) {
        *edges.entry(i.start).or_default() += 1;
        *edges.entry(i.end).or_default() -= 1;
    }
    let mut out = Vec::new();
    let mut count = 0i32;
    let mut prev: Option<DateTime<Utc>> = None;
    for (at, delta) in edges {
        if let (Some(p), true) = (prev, count > 0) {
            out.push((p, at, count as u32));
        }
        count += delta;
        prev = Some(at);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(min: i64) -> DateTime<Utc> {
        "2026-10-03T12:00:00Z"
            .parse::<DateTime<Utc>>()
            .expect("time")
            + Duration::minutes(min)
    }

    fn ev(session: &str, at: DateTime<Utc>, kind: Kind) -> Event {
        Event {
            machine: "m".into(),
            project: "p".into(),
            agent: "orchestrator".into(),
            session: session.into(),
            at,
            kind,
        }
    }

    fn prompt(session: &str, min: i64) -> Event {
        ev(session, t(min), Kind::Prompt)
    }

    fn reply(session: &str, min: i64) -> Event {
        ev(session, t(min), Kind::Reply)
    }

    fn spans(evs: &[Event]) -> Vec<(i64, i64)> {
        let base = t(0);
        intervals(evs, &InteractionsConfig::default())
            .iter()
            .map(|i| ((i.start - base).num_minutes(), (i.end - base).num_minutes()))
            .collect()
    }

    #[test]
    fn a_lone_prompt_counts_lead_before_and_tail_after() {
        assert_eq!(spans(&[prompt("s", 30)]), [(29, 32)]);
    }

    #[test]
    fn the_gap_runs_from_the_reply_end_and_waiting_counts() {
        // Prompt at 0, a 30 minute turn, the human's next prompt 8 minutes after the reply:
        // within the gap of the reply, though 38 minutes after the prompt.
        let evs = [
            prompt("s", 0),
            reply("s", 30),
            prompt("s", 38),
            reply("s", 39),
        ];
        assert_eq!(spans(&evs), [(-1, 38), (38, 41)]);
    }

    #[test]
    fn a_next_prompt_past_the_gap_ends_the_run_at_reply_plus_tail() {
        let evs = [prompt("s", 0), reply("s", 5), prompt("s", 30)];
        assert_eq!(spans(&evs), [(-1, 7), (29, 32)]);
    }

    #[test]
    fn without_a_reply_a_prompt_counts_to_the_next_within_the_gap_else_tail() {
        assert_eq!(spans(&[prompt("s", 0), prompt("s", 4)]), [(-1, 4), (4, 6)]);
        assert_eq!(
            spans(&[prompt("s", 0), prompt("s", 15)]),
            [(-1, 2), (14, 17)]
        );
    }

    #[test]
    fn a_reply_after_the_next_prompt_is_not_this_prompts_reply() {
        let evs = [prompt("s", 0), prompt("s", 3), reply("s", 20)];
        assert_eq!(spans(&evs), [(-1, 3), (3, 22)]);
    }

    #[test]
    fn a_stray_reply_before_any_prompt_counts_nothing() {
        assert_eq!(spans(&[reply("s", 0)]), []);
    }

    #[test]
    fn the_lead_never_reaches_into_the_previous_interval() {
        // Reply at 5 + tail 2 = 7; the next prompt at 7.5 would lead from 6.5.
        let mut late = prompt("s", 8);
        late.at += Duration::seconds(-30);
        let evs = [prompt("s", 0), reply("s", 5), late];
        let got = spans(&evs);
        assert_eq!(got[1].0, 7);
    }

    #[test]
    fn sessions_are_independent_and_union_counts_overlap_once() {
        let evs = [prompt("a", 0), reply("a", 4), prompt("b", 2), reply("b", 8)];
        let ivs = intervals(&evs, &InteractionsConfig::default());
        // a: -1..6, b: 1..10
        assert_eq!(ivs.len(), 2);
        let h = human_time(&ivs);
        assert_eq!(h, [(t(-1), t(10))]);
        assert!((minutes(&h) - 11.0).abs() < 1e-9);
    }

    #[test]
    fn concurrency_counts_sessions_covering_a_moment() {
        let evs = [prompt("a", 0), reply("a", 4), prompt("b", 2), reply("b", 8)];
        let ivs = intervals(&evs, &InteractionsConfig::default());
        let c = concurrency(&ivs);
        assert_eq!(c, [(t(-1), t(1), 1), (t(1), t(6), 2), (t(6), t(10), 1)]);
        let peak = c.iter().map(|x| x.2).max();
        assert_eq!(peak, Some(2));
    }
}
