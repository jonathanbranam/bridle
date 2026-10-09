//! Scheduled messages (hrcn, br-9xze): a schedule is a stored message that the daemon sends
//! to a principal at a time, once or on a cron expression. See "Scheduled messages" in
//! docs/design/agent-host/daemon.md.

use std::collections::BTreeSet;
use std::future::Future;
use std::time::Duration;

use bridle_api::types::Schedule;
use chrono::{
    DateTime, Datelike, Duration as ChronoDuration, LocalResult, NaiveDate, TimeZone, Utc,
};
use chrono_tz::Tz;

use crate::store::Store;

/// How often the loop looks for due schedules.
pub const TICK: Duration = Duration::from_secs(15);

/// A message later than this says so.
const LATE_AFTER: i64 = 120;

/// How far ahead a cron is searched before it counts as never firing (Feb 29 needs 8 years
/// at the century boundaries).
const HORIZON_DAYS: i64 = 366 * 8;

/// A parsed five-field cron expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cron {
    minutes: BTreeSet<u32>,
    hours: BTreeSet<u32>,
    days: BTreeSet<u32>,
    months: BTreeSet<u32>,
    weekdays: BTreeSet<u32>,
    day_any: bool,
    weekday_any: bool,
}

impl Cron {
    /// Standard 5 fields: minute hour day-of-month month day-of-week. Each is a list of `*`,
    /// `n`, `a-b`, with an optional `/step`. Day-of-week is 0-7 (0 and 7 are Sunday).
    pub fn parse(expr: &str) -> Result<Self, String> {
        let fields: Vec<&str> = expr.split_whitespace().collect();
        let [min, hour, day, month, weekday] = fields[..] else {
            return Err(format!(
                "a cron expression has 5 fields (minute hour day-of-month month day-of-week), got {}",
                fields.len()
            ));
        };
        let mut weekdays = field(weekday, 0, 7, "day-of-week")?;
        if weekdays.remove(&7) {
            weekdays.insert(0);
        }
        Ok(Self {
            minutes: field(min, 0, 59, "minute")?,
            hours: field(hour, 0, 23, "hour")?,
            days: field(day, 1, 31, "day-of-month")?,
            months: field(month, 1, 12, "month")?,
            weekdays,
            day_any: day.starts_with('*'),
            weekday_any: weekday.starts_with('*'),
        })
    }

    fn day_matches(&self, date: NaiveDate) -> bool {
        if !self.months.contains(&date.month()) {
            return false;
        }
        let dom = self.days.contains(&date.day());
        let dow = self
            .weekdays
            .contains(&date.weekday().num_days_from_sunday());
        // Standard cron: when both day fields are restricted, either one matches.
        match (self.day_any, self.weekday_any) {
            (false, false) => dom || dow,
            _ => dom && dow,
        }
    }

    /// The first occurrence strictly after `after`, in `tz`. A local time that does not exist
    /// (spring forward) is skipped that day; an ambiguous one (fall back) counts once, at its
    /// first occurrence. None when nothing fires within the horizon.
    pub fn next_after(&self, tz: Tz, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let first = after.with_timezone(&tz).date_naive();
        for offset in 0..HORIZON_DAYS {
            let date = first + ChronoDuration::days(offset);
            if !self.day_matches(date) {
                continue;
            }
            for &h in &self.hours {
                for &m in &self.minutes {
                    let local = date.and_hms_opt(h, m, 0)?;
                    let at = match tz.from_local_datetime(&local) {
                        LocalResult::Single(t) => t,
                        LocalResult::Ambiguous(first, _) => first,
                        LocalResult::None => continue,
                    };
                    let at = at.with_timezone(&Utc);
                    if at > after {
                        return Some(at);
                    }
                }
            }
        }
        None
    }

    /// The last occurrence at or before `now` that is at or after `from` (an occurrence).
    fn latest_through(&self, tz: Tz, from: DateTime<Utc>, now: DateTime<Utc>) -> DateTime<Utc> {
        let mut latest = from;
        while let Some(next) = self.next_after(tz, latest) {
            if next > now {
                break;
            }
            latest = next;
        }
        latest
    }
}

fn field(spec: &str, lo: u32, hi: u32, name: &str) -> Result<BTreeSet<u32>, String> {
    let bad = |why: &str| format!("cron {name} field {spec:?}: {why}");
    let mut out = BTreeSet::new();
    for part in spec.split(',') {
        let (range, step) = match part.split_once('/') {
            Some((r, s)) => (r, s.parse::<u32>().map_err(|_| bad("bad step"))?),
            None => (part, 1),
        };
        if step == 0 {
            return Err(bad("step must be at least 1"));
        }
        let (from, to) = if range == "*" {
            (lo, hi)
        } else if let Some((a, b)) = range.split_once('-') {
            (
                a.parse::<u32>().map_err(|_| bad("not a number"))?,
                b.parse::<u32>().map_err(|_| bad("not a number"))?,
            )
        } else {
            let n = range.parse::<u32>().map_err(|_| bad("not a number"))?;
            // `n/step` runs from n to the end, like Vixie cron.
            (n, if part.contains('/') { hi } else { n })
        };
        if from < lo || to > hi || from > to {
            return Err(bad(&format!("must be within {lo}-{hi}")));
        }
        out.extend((from..=to).step_by(step as usize));
    }
    Ok(out)
}

pub fn parse_tz(name: &str) -> Result<Tz, String> {
    name.parse::<Tz>().map_err(|_| {
        format!("unknown time zone {name:?}: use an IANA name such as America/New_York")
    })
}

/// `--at`: RFC 3339, or `YYYY-MM-DD HH:MM` read in `tz`.
pub fn parse_at(text: &str, tz: Tz) -> Result<DateTime<Utc>, String> {
    if let Ok(t) = DateTime::parse_from_rfc3339(text) {
        return Ok(t.with_timezone(&Utc));
    }
    let local = chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M").map_err(|_| {
        format!("time {text:?}: use RFC 3339 or \"YYYY-MM-DD HH:MM\" (read in the schedule's zone)")
    })?;
    match tz.from_local_datetime(&local) {
        LocalResult::Single(t) | LocalResult::Ambiguous(t, _) => Ok(t.with_timezone(&Utc)),
        LocalResult::None => Err(format!("{text} does not exist in {tz} (clocks skip it)")),
    }
}

/// What firing one due schedule does.
#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    /// The message body, with the lateness note when it is due.
    pub body: String,
    pub missed: bool,
    /// The next firing; none ends a `once`.
    pub next: Option<DateTime<Utc>>,
}

/// Plans the firing of `s` (whose `next_fire_at` has passed) at `now`. A cron that missed
/// several occurrences fires once, for the latest, then moves to the next future one.
pub fn plan(s: &Schedule, now: DateTime<Utc>) -> Plan {
    let tz = s.tz.parse::<Tz>().unwrap_or(chrono_tz::America::New_York);
    let due = s.next_fire_at.unwrap_or(now);
    let (due, next) = match s.cron.as_deref().map(Cron::parse) {
        Some(Ok(cron)) => {
            let latest = cron.latest_through(tz, due, now);
            (latest, cron.next_after(tz, now.max(latest)))
        }
        _ => (due, None),
    };
    let missed = (now - due).num_seconds() > LATE_AFTER;
    let mut body = format!("Scheduled {} (set by {}): {}", s.id, s.created_by, s.body);
    if missed {
        body.push_str(&format!(
            " (due {}, sent late)",
            due.with_timezone(&tz).format("%Y-%m-%d %H:%M %Z")
        ));
    }
    Plan { body, missed, next }
}

/// One schedule that was sent.
#[derive(Debug)]
pub struct Fired {
    pub id: String,
    pub target: String,
    pub missed: bool,
}

/// Sends every schedule due at `now` through `send(target, body)` and moves each on. A send
/// that fails is logged and the schedule moves on anyway: a target that is gone would
/// otherwise fail every tick.
pub async fn fire_due<F, Fut>(store: &Store, now: DateTime<Utc>, send: F) -> Vec<Fired>
where
    F: Fn(String, String) -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let due = match store.schedules_due(now).await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("reading due schedules failed: {e}");
            return Vec::new();
        }
    };
    let mut fired = Vec::new();
    for s in due {
        let p = plan(&s, now);
        match send(s.target.clone(), p.body).await {
            Ok(()) => fired.push(Fired {
                id: s.id.clone(),
                target: s.target.clone(),
                missed: p.missed,
            }),
            Err(e) => {
                tracing::warn!(id = %s.id, target = %s.target, "scheduled message not sent: {e}")
            }
        }
        if let Err(e) = store.schedule_fired(&s.id, now, p.next).await {
            tracing::warn!(id = %s.id, "recording a fired schedule failed: {e}");
        }
    }
    fired
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    const NY: Tz = chrono_tz::America::New_York;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .expect("time")
            .with_timezone(&Utc)
    }

    fn cron(s: &str) -> Cron {
        Cron::parse(s).expect("cron")
    }

    #[test]
    fn cron_fields_and_errors() {
        // 09:30 EST on a weekday morning, from midnight UTC the same day.
        let next = cron("30 9 * * 1-5").next_after(NY, utc("2026-01-07T00:00:00Z"));
        assert_eq!(next, Some(utc("2026-01-07T14:30:00Z")));
        // Friday evening goes to Monday.
        let next = cron("30 9 * * 1-5").next_after(NY, utc("2026-01-10T00:00:00Z"));
        assert_eq!(next, Some(utc("2026-01-12T14:30:00Z")));
        let next = cron("*/20 8 * * *").next_after(NY, utc("2026-01-07T13:21:00Z"));
        assert_eq!(next, Some(utc("2026-01-07T13:40:00Z")));
        assert!(Cron::parse("* * * *").is_err());
        assert!(Cron::parse("61 * * * *").is_err());
        assert!(Cron::parse("*/0 * * * *").is_err());
        // Never fires: 31 February.
        assert_eq!(
            cron("0 0 31 2 *").next_after(NY, utc("2026-01-01T00:00:00Z")),
            None
        );
    }

    #[test]
    fn spring_forward_skips_the_missing_hour_that_day() {
        // 2026-03-08: 02:30 does not exist in New York. The next 02:30 is the 9th (EDT, 06:30Z).
        let next = cron("30 2 * * *").next_after(NY, utc("2026-03-08T00:00:00Z"));
        assert_eq!(next, Some(utc("2026-03-09T06:30:00Z")));
    }

    #[test]
    fn fall_back_fires_the_repeated_hour_once() {
        // 2026-11-01: 01:30 happens twice (05:30Z EDT, then 06:30Z EST). Only the first counts.
        let c = cron("30 1 * * *");
        let first = c.next_after(NY, utc("2026-11-01T00:00:00Z"));
        assert_eq!(first, Some(utc("2026-11-01T05:30:00Z")));
        let after = c.next_after(NY, first.expect("first"));
        assert_eq!(after, Some(utc("2026-11-02T06:30:00Z")));
    }

    #[test]
    fn at_reads_the_zone_and_refuses_a_skipped_time() {
        assert_eq!(
            parse_at("2026-01-07 09:30", NY),
            Ok(utc("2026-01-07T14:30:00Z"))
        );
        assert_eq!(
            parse_at("2026-01-07T09:30:00-05:00", chrono_tz::UTC),
            Ok(utc("2026-01-07T14:30:00Z"))
        );
        assert!(parse_at("2026-03-08 02:30", NY).is_err());
        assert!(parse_at("tomorrow", NY).is_err());
    }

    fn sched(kind: &str, cron: Option<&str>, due: &str) -> Schedule {
        Schedule {
            id: String::new(),
            created_by: "agent:w1".to_string(),
            target: "agent:w1".to_string(),
            body: "check the build".to_string(),
            kind: kind.to_string(),
            cron: cron.map(str::to_string),
            tz: "America/New_York".to_string(),
            next_fire_at: Some(utc(due)),
            last_fired_at: None,
            state: String::new(),
            created_at: Utc::now(),
        }
    }

    async fn store() -> (Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tmp");
        let store = Store::open(dir.path().join("bridle.db"))
            .await
            .expect("store");
        (store, dir)
    }

    type Sent = Arc<Mutex<Vec<(String, String)>>>;

    async fn tick(store: &Store, now: &str) -> (Vec<Fired>, Sent) {
        let sent: Sent = Arc::default();
        let fired = fire_due(store, utc(now), |t, b| {
            let sent = sent.clone();
            async move {
                sent.lock().expect("lock").push((t, b));
                Ok(())
            }
        })
        .await;
        (fired, sent)
    }

    #[tokio::test]
    async fn a_once_fires_on_time_then_is_done() {
        let (store, _d) = store().await;
        let s = store
            .schedule_add(sched("once", None, "2026-01-07T14:30:00Z"))
            .await
            .expect("add");
        let (fired, sent) = tick(&store, "2026-01-07T14:29:50Z").await;
        assert!(fired.is_empty(), "not yet");
        let (fired, sent2) = tick(&store, "2026-01-07T14:30:10Z").await;
        assert_eq!(fired.len(), 1);
        assert!(!fired[0].missed);
        let body = sent2.lock().expect("lock")[0].1.clone();
        assert_eq!(
            body,
            format!("Scheduled {} (set by agent:w1): check the build", s.id)
        );
        let none_before = sent.lock().expect("lock").is_empty();
        assert!(none_before);
        let after = store.schedule_get(&s.id).await.expect("get").expect("row");
        assert_eq!((after.state.as_str(), after.next_fire_at), ("done", None));
        let (fired, _) = tick(&store, "2026-01-07T15:00:00Z").await;
        assert!(fired.is_empty(), "only once");
        assert!(
            store
                .schedules_list(None, false)
                .await
                .expect("list")
                .is_empty()
        );
        assert_eq!(
            store.schedules_list(None, true).await.expect("list").len(),
            1
        );
    }

    #[tokio::test]
    async fn a_cron_fires_and_moves_to_the_next_occurrence() {
        let (store, _d) = store().await;
        let s = store
            .schedule_add(sched("cron", Some("30 9 * * *"), "2026-01-07T14:30:00Z"))
            .await
            .expect("add");
        let (fired, _) = tick(&store, "2026-01-07T14:30:05Z").await;
        assert_eq!(fired.len(), 1);
        let after = store.schedule_get(&s.id).await.expect("get").expect("row");
        assert_eq!(after.state, "active");
        assert_eq!(after.next_fire_at, Some(utc("2026-01-08T14:30:00Z")));
        assert_eq!(after.last_fired_at, Some(utc("2026-01-07T14:30:05Z")));
    }

    #[tokio::test]
    async fn a_missed_once_fires_late_with_the_note() {
        let (store, _d) = store().await;
        store
            .schedule_add(sched("once", None, "2026-01-07T14:30:00Z"))
            .await
            .expect("add");
        let (fired, sent) = tick(&store, "2026-01-07T18:00:00Z").await;
        assert!(fired[0].missed);
        let body = sent.lock().expect("lock")[0].1.clone();
        assert!(
            body.ends_with("(due 2026-01-07 09:30 EST, sent late)"),
            "{body}"
        );
    }

    #[tokio::test]
    async fn a_missed_cron_fires_once_for_the_latest_occurrence() {
        let (store, _d) = store().await;
        let s = store
            .schedule_add(sched("cron", Some("30 9 * * *"), "2026-01-03T14:30:00Z"))
            .await
            .expect("add");
        // Down for days: 4th..7th were also missed.
        let (fired, sent) = tick(&store, "2026-01-07T18:00:00Z").await;
        assert_eq!(fired.len(), 1, "one message, not a burst");
        let body = sent.lock().expect("lock")[0].1.clone();
        assert!(
            body.ends_with("(due 2026-01-07 09:30 EST, sent late)"),
            "{body}"
        );
        let after = store.schedule_get(&s.id).await.expect("get").expect("row");
        assert_eq!(after.next_fire_at, Some(utc("2026-01-08T14:30:00Z")));
        let (fired, _) = tick(&store, "2026-01-07T18:00:15Z").await;
        assert!(fired.is_empty());
    }

    #[tokio::test]
    async fn schedules_survive_a_reopen() {
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("bridle.db");
        let store = Store::open(&path).await.expect("store");
        let s = store
            .schedule_add(sched("cron", Some("0 * * * *"), "2026-01-07T15:00:00Z"))
            .await
            .expect("add");
        drop(store);
        let store = Store::open(&path).await.expect("reopen");
        let got = store.schedule_get(&s.id).await.expect("get").expect("kept");
        assert_eq!(got, s.clone());
        assert!(store.schedule_remove(&s.id).await.expect("rm"));
        assert!(!store.schedule_remove(&s.id).await.expect("rm again"));
    }
}
