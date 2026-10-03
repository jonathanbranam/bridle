//! The handlers of `/api/v1/interactions/*` (ticket u6w9, docs/design/human-web-ui.md "Human
//! time"): the stored events through [`crate::intervals`], bucketed by US Eastern day.

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Datelike, Duration, DurationRound, NaiveDate, Timelike, Utc, Weekday};
use serde::Deserialize;
use serde_json::json;

use crate::collect::Store;
use crate::config::InteractionsConfig;
use crate::interactions::*;
use crate::intervals::{self, Interval};

/// What the handlers read: the collected events and the interval rules.
#[derive(Clone)]
pub struct Interactions {
    pub store: Store,
    pub config: InteractionsConfig,
}

/// Longest range a request may cover, so a typo can't ask for decades of buckets.
const MAX_DAYS: i64 = 400;

type Span = (DateTime<Utc>, DateTime<Utc>);

#[derive(Debug)]
pub struct BadRequest(String);

impl IntoResponse for BadRequest {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(json!({ "error": self.0 }))).into_response()
    }
}

// --- US Eastern ---------------------------------------------------------------------------
// Whole-hour offsets with the US rule (since 2007), so no timezone database is needed.

/// UTC offset of US Eastern at `at`: -4h in daylight time, else -5h.
fn eastern_offset(at: DateTime<Utc>) -> Duration {
    let year = at.year();
    let transition = |month, nth, hour| {
        NaiveDate::from_weekday_of_month_opt(year, month, Weekday::Sun, nth)
            .and_then(|d| d.and_hms_opt(hour, 0, 0))
            .map(|d| d.and_utc())
    };
    // 2:00 EST = 07:00 UTC in March; 2:00 EDT = 06:00 UTC in November.
    match (transition(3, 2, 7), transition(11, 1, 6)) {
        (Some(start), Some(end)) if at >= start && at < end => Duration::hours(-4),
        _ => Duration::hours(-5),
    }
}

/// The UTC instant of Eastern midnight starting `date`.
fn day_start(date: NaiveDate) -> DateTime<Utc> {
    let midnight = date.and_hms_opt(0, 0, 0).expect("midnight").and_utc();
    // Transitions are at 2:00, so the offset at 05:00 UTC is midnight's.
    midnight - eastern_offset(midnight + Duration::hours(5))
}

fn monday_of(date: NaiveDate) -> NaiveDate {
    date - Duration::days(date.weekday().num_days_from_monday() as i64)
}

// --- parameters ---------------------------------------------------------------------------

fn date(name: &str, v: &Option<String>) -> Result<NaiveDate, BadRequest> {
    let v = v
        .as_deref()
        .ok_or_else(|| BadRequest(format!("{name} is required (YYYY-MM-DD)")))?;
    NaiveDate::parse_from_str(v, "%Y-%m-%d")
        .map_err(|_| BadRequest(format!("{name} '{v}' is not a date; use YYYY-MM-DD")))
}

/// The days `from..=to` as a range of UTC instants, and the two dates.
fn range(from: &Option<String>, to: &Option<String>) -> Result<(NaiveDate, NaiveDate), BadRequest> {
    let (from, to) = (date("from", from)?, date("to", to)?);
    if to < from {
        return Err(BadRequest("to is before from".into()));
    }
    if (to - from).num_days() >= MAX_DAYS {
        return Err(BadRequest(format!("a range is at most {MAX_DAYS} days")));
    }
    Ok((from, to))
}

fn next_day(d: NaiveDate) -> NaiveDate {
    d + Duration::days(1)
}

fn weekdays(v: &Option<String>) -> Result<Vec<Weekday>, BadRequest> {
    use Weekday::*;
    let Some(v) = v.as_deref() else {
        return Ok(vec![Mon, Tue, Wed, Thu, Fri, Sat, Sun]);
    };
    match v {
        "weekday" => return Ok(vec![Mon, Tue, Wed, Thu, Fri]),
        "weekend" => return Ok(vec![Sat, Sun]),
        _ => {}
    }
    v.split(',')
        .map(|d| {
            d.trim().parse::<Weekday>().map_err(|_| {
                BadRequest(format!(
                    "days '{v}': use weekday, weekend or a list like mon,tue"
                ))
            })
        })
        .collect()
}

// --- math ---------------------------------------------------------------------------------

/// The intervals cut to `[lo, hi)`; those left empty are dropped.
fn clip(all: &[Interval], lo: DateTime<Utc>, hi: DateTime<Utc>) -> Vec<Interval> {
    all.iter()
        .filter_map(|i| {
            let (start, end) = (i.start.max(lo), i.end.min(hi));
            (start < end).then(|| Interval {
                start,
                end,
                ..i.clone()
            })
        })
        .collect()
}

fn group_minutes(ivs: &[Interval], group: InteractionGroup) -> Vec<GroupMinutes> {
    let mut by: BTreeMap<&str, Vec<Span>> = BTreeMap::new();
    for i in ivs {
        let key = match group {
            InteractionGroup::Project => &i.project,
            InteractionGroup::Agent => &i.agent,
            InteractionGroup::Machine => &i.machine,
        };
        by.entry(key).or_default().push((i.start, i.end));
    }
    let mut out: Vec<GroupMinutes> = by
        .into_iter()
        .map(|(k, spans)| GroupMinutes {
            key: k.to_string(),
            minutes: intervals::minutes(&intervals::union(spans)),
        })
        .collect();
    out.sort_by(|a, b| b.minutes.total_cmp(&a.minutes).then(a.key.cmp(&b.key)));
    out
}

fn human_minutes(ivs: &[Interval]) -> f64 {
    intervals::minutes(&intervals::human_time(ivs))
}

fn span(s: DateTime<Utc>, e: DateTime<Utc>) -> TimeSpan {
    TimeSpan {
        start: s.to_rfc3339(),
        end: e.to_rfc3339(),
    }
}

impl Interactions {
    fn all(&self) -> Vec<Interval> {
        intervals::intervals(&self.store.events(), &self.config)
    }
}

// --- handlers -----------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ReportQuery {
    from: Option<String>,
    to: Option<String>,
    group: Option<String>,
    bucket: Option<String>,
}

pub async fn report(
    State(ix): State<Interactions>,
    Query(q): Query<ReportQuery>,
) -> Result<Json<InteractionReport>, BadRequest> {
    let (from, to) = range(&q.from, &q.to)?;
    let group = match q.group.as_deref() {
        None | Some("project") => InteractionGroup::Project,
        Some("agent") => InteractionGroup::Agent,
        Some("machine") => InteractionGroup::Machine,
        Some(g) => {
            return Err(BadRequest(format!(
                "group '{g}': use project, agent or machine"
            )));
        }
    };
    let bucket = match q.bucket.as_deref() {
        None | Some("day") => InteractionBucket::Day,
        Some("week") => InteractionBucket::Week,
        Some(b) => return Err(BadRequest(format!("bucket '{b}': use day or week"))),
    };
    let all = ix.all();
    let (lo, hi) = (day_start(from), day_start(next_day(to)));
    let mut buckets = Vec::new();
    let mut first = match bucket {
        InteractionBucket::Day => from,
        InteractionBucket::Week => monday_of(from),
    };
    while first <= to {
        let after = first
            + Duration::days(match bucket {
                InteractionBucket::Day => 1,
                InteractionBucket::Week => 7,
            });
        let ivs = clip(&all, day_start(first).max(lo), day_start(after).min(hi));
        buckets.push(ReportBucket {
            start: first.to_string(),
            human_minutes: human_minutes(&ivs),
            groups: group_minutes(&ivs, group),
        });
        first = after;
    }
    let ivs = clip(&all, lo, hi);
    Ok(Json(InteractionReport {
        group,
        bucket,
        buckets,
        human_minutes: human_minutes(&ivs),
        groups: group_minutes(&ivs, group),
        unreachable: ix.store.unreachable(),
    }))
}

#[derive(Deserialize)]
pub struct DayQuery {
    date: Option<String>,
}

pub async fn day(
    State(ix): State<Interactions>,
    Query(q): Query<DayQuery>,
) -> Result<Json<DayReport>, BadRequest> {
    let d = date("date", &q.date)?;
    let ivs = clip(&ix.all(), day_start(d), day_start(next_day(d)));
    let human = intervals::human_time(&ivs);

    let mut by: BTreeMap<(&str, &str, &str, &str), Vec<TimeSpan>> = BTreeMap::new();
    for i in &ivs {
        by.entry((&i.machine, &i.project, &i.agent, &i.session))
            .or_default()
            .push(span(i.start, i.end));
    }
    let sessions = by
        .into_iter()
        .map(
            |((machine, project, agent, session), intervals)| SessionTimeline {
                machine: machine.into(),
                project: project.into(),
                agent: agent.into(),
                session: session.into(),
                intervals,
            },
        )
        .collect();

    let conc = intervals::concurrency(&ivs);
    let at = |pred: fn(u32) -> bool| {
        let spans = conc.iter().filter(|c| pred(c.2)).map(|c| (c.0, c.1));
        intervals::minutes(&spans.collect::<Vec<_>>())
    };
    let overlaps = intervals::union(
        conc.iter()
            .filter(|c| c.2 >= 2)
            .map(|c| (c.0, c.1))
            .collect(),
    )
    .into_iter()
    .map(|(s, e)| span(s, e))
    .collect();
    Ok(Json(DayReport {
        date: d.to_string(),
        first_start: human.first().map(|s| s.0.to_rfc3339()),
        last_end: human.last().map(|s| s.1.to_rfc3339()),
        human_minutes: intervals::minutes(&human),
        sessions,
        overlaps,
        peak_concurrency: conc.iter().map(|c| c.2).max().unwrap_or(0),
        minutes_at_1: at(|n| n == 1),
        minutes_at_2: at(|n| n == 2),
        minutes_at_3_plus: at(|n| n >= 3),
        unreachable: ix.store.unreachable(),
    }))
}

#[derive(Deserialize)]
pub struct HoursQuery {
    from: Option<String>,
    to: Option<String>,
    days: Option<String>,
}

pub async fn hours(
    State(ix): State<Interactions>,
    Query(q): Query<HoursQuery>,
) -> Result<Json<HoursReport>, BadRequest> {
    let (from, to) = range(&q.from, &q.to)?;
    let wanted = weekdays(&q.days)?;
    let all = ix.all();
    let mut totals = [0f64; 24];
    let mut day_count = 0u32;
    let mut d = from;
    while d <= to {
        if wanted.contains(&d.weekday()) {
            day_count += 1;
            let (lo, hi) = (day_start(d), day_start(next_day(d)));
            for (s, e) in intervals::human_time(&clip(&all, lo, hi)) {
                // Offsets are whole hours, so local hours start on UTC hour marks.
                let mut at = s;
                while at < e {
                    let next = (at.duration_trunc(Duration::hours(1)).expect("hour")
                        + Duration::hours(1))
                    .min(e);
                    let hour = (at + eastern_offset(at)).hour() as usize;
                    totals[hour] += intervals::minutes(&[(at, next)]);
                    at = next;
                }
            }
        }
        d = next_day(d);
    }
    let minutes_per_hour = totals
        .iter()
        .map(|t| {
            if day_count == 0 {
                0.0
            } else {
                t / day_count as f64
            }
        })
        .collect();
    Ok(Json(HoursReport {
        day_count,
        minutes_per_hour,
        unreachable: ix.store.unreachable(),
    }))
}

#[derive(Deserialize)]
pub struct RangeQuery {
    from: Option<String>,
    to: Option<String>,
}

/// Raw intervals touching the range, uncut, so the UI can roll them up its own way.
pub async fn raw_intervals(
    State(ix): State<Interactions>,
    Query(q): Query<RangeQuery>,
) -> Result<Json<IntervalsReport>, BadRequest> {
    let (from, to) = range(&q.from, &q.to)?;
    let (lo, hi) = (day_start(from), day_start(next_day(to)));
    let intervals = ix
        .all()
        .iter()
        .filter(|i| i.start < hi && i.end > lo)
        .map(Into::into)
        .collect();
    Ok(Json(IntervalsReport {
        intervals,
        unreachable: ix.store.unreachable(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intervals::{Event, Kind};

    fn utc(s: &str) -> DateTime<Utc> {
        s.parse().expect("time")
    }

    #[test]
    fn eastern_follows_the_us_daylight_rule() {
        assert_eq!(
            eastern_offset(utc("2026-03-08T06:59:00Z")),
            Duration::hours(-5)
        );
        assert_eq!(
            eastern_offset(utc("2026-03-08T07:00:00Z")),
            Duration::hours(-4)
        );
        assert_eq!(
            eastern_offset(utc("2026-11-01T05:59:00Z")),
            Duration::hours(-4)
        );
        assert_eq!(
            eastern_offset(utc("2026-11-01T06:00:00Z")),
            Duration::hours(-5)
        );
        let d = |s: &str| s.parse::<NaiveDate>().expect("date");
        assert_eq!(day_start(d("2026-10-03")), utc("2026-10-03T04:00:00Z"));
        assert_eq!(day_start(d("2026-12-01")), utc("2026-12-01T05:00:00Z"));
        assert_eq!(day_start(d("2026-03-08")), utc("2026-03-08T05:00:00Z"));
        assert_eq!(day_start(d("2026-11-01")), utc("2026-11-01T04:00:00Z"));
    }

    fn ev(project: &str, session: &str, at: &str, kind: Kind) -> Event {
        Event {
            machine: "mbp".into(),
            project: project.into(),
            agent: "orchestrator".into(),
            session: session.into(),
            at: utc(at),
            kind,
        }
    }

    /// Eastern 2026-10-03 is UTC-4. `a`, in project p: prompt 10:00 EDT, reply 10:04
    /// (interval 09:59-10:06). `b`, in project q: prompt 10:02, reply 10:08 (10:01-10:10).
    /// `c`, in p: a lone prompt at 23:59 EDT on the 3rd (23:58-00:01, over midnight).
    fn fixture() -> Interactions {
        let evs = [
            ev("p", "a", "2026-10-03T14:00:00Z", Kind::Prompt),
            ev("p", "a", "2026-10-03T14:04:00Z", Kind::Reply),
            ev("q", "b", "2026-10-03T14:02:00Z", Kind::Prompt),
            ev("q", "b", "2026-10-03T14:08:00Z", Kind::Reply),
            ev("p", "c", "2026-10-04T03:59:00Z", Kind::Prompt),
        ];
        Interactions {
            store: Store::from_events(&evs, &["laptop"]),
            config: InteractionsConfig::default(),
        }
    }

    fn s(v: &str) -> Option<String> {
        Some(v.to_string())
    }

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[tokio::test]
    async fn report_buckets_by_eastern_day_and_groups() {
        let q = ReportQuery {
            from: s("2026-10-03"),
            to: s("2026-10-04"),
            group: s("project"),
            bucket: None,
        };
        let Json(r) = report(State(fixture()), Query(q)).await.expect("ok");
        assert_eq!(r.unreachable, ["laptop"]);
        assert_eq!(r.buckets.len(), 2);
        // 3rd: union of 09:59-10:10 (11) and c's 23:58-24:00 (2); 4th: c's last minute.
        near(r.buckets[0].human_minutes, 13.0);
        near(r.buckets[1].human_minutes, 1.0);
        near(r.human_minutes, 14.0);
        // p: a (7) + c (3) = 10; q: 9; they add to more than the union (14 + overlap).
        assert_eq!(r.groups.len(), 2);
        assert_eq!(r.groups[0].key, "p");
        near(r.groups[0].minutes, 10.0);
        near(r.groups[1].minutes, 9.0);
    }

    #[tokio::test]
    async fn report_by_week_starts_on_monday() {
        let q = ReportQuery {
            from: s("2026-10-03"),
            to: s("2026-10-04"),
            group: s("machine"),
            bucket: s("week"),
        };
        let Json(r) = report(State(fixture()), Query(q)).await.expect("ok");
        assert_eq!(r.buckets.len(), 1);
        assert_eq!(r.buckets[0].start, "2026-09-28");
        near(r.buckets[0].human_minutes, 14.0);
        assert_eq!(r.groups[0].key, "mbp");
    }

    #[tokio::test]
    async fn day_has_timelines_overlap_and_concurrency() {
        let q = DayQuery {
            date: s("2026-10-03"),
        };
        let Json(d) = day(State(fixture()), Query(q)).await.expect("ok");
        assert_eq!(d.sessions.len(), 3);
        assert_eq!(d.first_start.as_deref(), Some("2026-10-03T13:59:00+00:00"));
        assert_eq!(d.last_end.as_deref(), Some("2026-10-04T04:00:00+00:00"));
        near(d.human_minutes, 13.0);
        assert_eq!(d.peak_concurrency, 2);
        // a 09:59-10:06 and b 10:01-10:10 overlap 10:01-10:06.
        assert_eq!(d.overlaps.len(), 1);
        near(d.minutes_at_2, 5.0);
        near(d.minutes_at_1, 8.0);
        near(d.minutes_at_3_plus, 0.0);
    }

    #[tokio::test]
    async fn an_empty_day_is_empty() {
        let q = DayQuery {
            date: s("2026-09-01"),
        };
        let Json(d) = day(State(fixture()), Query(q)).await.expect("ok");
        assert!(d.sessions.is_empty() && d.first_start.is_none());
        assert_eq!(d.peak_concurrency, 0);
    }

    #[tokio::test]
    async fn hours_average_over_matching_days_only() {
        // 2026-10-03 is a Saturday; the 2nd is a Friday with nothing.
        let q = |days: &str| HoursQuery {
            from: s("2026-10-02"),
            to: s("2026-10-03"),
            days: s(days),
        };
        let Json(all) = hours(State(fixture()), Query(q("fri,sat")))
            .await
            .expect("ok");
        assert_eq!(all.day_count, 2);
        assert_eq!(all.minutes_per_hour.len(), 24);
        // Hour 9 EDT: 09:59-10:00 = 1; hour 10: 10 minutes; hour 23: 2. Averaged over two days.
        near(all.minutes_per_hour[9], 0.5);
        near(all.minutes_per_hour[10], 5.0);
        near(all.minutes_per_hour[23], 1.0);
        let Json(end) = hours(State(fixture()), Query(q("weekend")))
            .await
            .expect("ok");
        assert_eq!(end.day_count, 1);
        near(end.minutes_per_hour[10], 10.0);
        let Json(wk) = hours(State(fixture()), Query(q("weekday")))
            .await
            .expect("ok");
        assert_eq!(wk.day_count, 1);
        near(wk.minutes_per_hour.iter().sum::<f64>(), 0.0);
    }

    #[tokio::test]
    async fn raw_intervals_are_uncut_and_overlapping_the_range() {
        let q = |d: &str| RangeQuery {
            from: s(d),
            to: s(d),
        };
        let Json(r) = raw_intervals(State(fixture()), Query(q("2026-10-04")))
            .await
            .expect("ok");
        // Only c reaches the 4th, whole.
        assert_eq!(r.intervals.len(), 1);
        assert_eq!(r.intervals[0].session, "c");
        assert_eq!(r.intervals[0].start, "2026-10-04T03:58:00+00:00");
        assert_eq!(r.unreachable, ["laptop"]);
    }

    #[test]
    fn bad_parameters_are_refused() {
        let bad = |from: &str, to: &str| range(&s(from), &s(to)).is_err();
        assert!(bad("2026-10-05", "2026-10-04"));
        assert!(bad("yesterday", "2026-10-04"));
        assert!(bad("2024-01-01", "2026-10-04"));
        assert!(range(&None, &s("2026-10-04")).is_err());
        assert!(weekdays(&s("funday")).is_err());
        assert_eq!(weekdays(&s("mon, tue")).expect("days").len(), 2);
    }
}
