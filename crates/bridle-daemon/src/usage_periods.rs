//! Weeks, days and five-hour windows from the rate-limit history (br-pemk). Pure functions
//! over the readings: a period is a cluster of near-equal `resets_at` values, so the real
//! boundaries show, not a fixed grid.

use bridle_api::types::{RateLimitPoint, UsageDay, UsagePeriods, UsageWeek, UsageWindow};
use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::Tz;

/// The history carries no status, so "hit the limit" is utilization at the cap.
const HIT: f64 = 1.0;

fn peak(points: impl Iterator<Item = f64>) -> Option<f64> {
    points.fold(None, |m, u| Some(m.map_or(u, |m: f64| m.max(u))))
}

/// Claude Code reports `resets_at` with millisecond jitter, and it drifts over a minute edge.
/// Readings whose `resets_at` lie within this of the previous one share a period. Well under
/// the five-hour window (and the week), well over any jitter seen.
const RESET_TOLERANCE: Duration = Duration::minutes(10);

/// Points grouped into periods by `resets_at`: (latest resets_at, peak). Readings with no
/// `resets_at` or utilization belong to no period.
fn by_reset(points: &[RateLimitPoint]) -> Vec<(DateTime<Utc>, f64)> {
    let mut rs: Vec<(DateTime<Utc>, f64)> = points
        .iter()
        .filter_map(|p| Some((p.resets_at?, p.utilization?)))
        .collect();
    rs.sort_by_key(|(r, _)| *r);
    let mut out: Vec<(DateTime<Utc>, f64)> = Vec::new();
    for (r, u) in rs {
        match out.last_mut() {
            Some((last, m)) if r - *last <= RESET_TOLERANCE => {
                *last = r;
                *m = m.max(u);
            }
            _ => out.push((r, u)),
        }
    }
    out
}

pub fn windows(five_hour: &[RateLimitPoint]) -> Vec<UsageWindow> {
    by_reset(five_hour)
        .into_iter()
        .map(|(r, peak)| UsageWindow {
            starts_at: r - Duration::hours(5),
            resets_at: r,
            peak,
            hit_limit: peak >= HIT,
        })
        .collect()
}

fn days(
    starts: DateTime<Utc>,
    ends: DateTime<Utc>,
    five: &[RateLimitPoint],
    seven: &[RateLimitPoint],
    tz: Tz,
) -> Vec<UsageDay> {
    let mut out = Vec::new();
    let mut date = starts.with_timezone(&tz).date_naive();
    loop {
        let day_start = tz
            .from_local_datetime(&date.and_hms_opt(0, 0, 0).expect("midnight exists"))
            .earliest()
            .map_or(starts, |t| t.with_timezone(&Utc));
        let next = date.succ_opt().expect("date in range");
        let day_end = tz
            .from_local_datetime(&next.and_hms_opt(0, 0, 0).expect("midnight exists"))
            .earliest()
            .map_or(ends, |t| t.with_timezone(&Utc));
        let (s, e) = (day_start.max(starts), day_end.min(ends));
        if s >= ends {
            break;
        }
        let within = |ps: &[RateLimitPoint]| -> Option<f64> {
            peak(
                ps.iter()
                    .filter(|p| p.observed_at >= s && p.observed_at < e)
                    .filter_map(|p| p.utilization),
            )
        };
        let pf = within(five);
        out.push(UsageDay {
            date: date.format("%Y-%m-%d").to_string(),
            starts_at: s,
            ends_at: e,
            peak_seven_day: within(seven),
            peak_five_hour: pf,
            hit_five_hour: pf.is_some_and(|p| p >= HIT),
        });
        date = next;
    }
    out
}

pub fn weeks(five: &[RateLimitPoint], seven: &[RateLimitPoint], tz: Tz) -> Vec<UsageWeek> {
    by_reset(seven)
        .into_iter()
        .map(|(r, peak)| {
            let starts = r - Duration::days(7);
            UsageWeek {
                starts_at: starts,
                resets_at: r,
                peak,
                hit_limit: peak >= HIT,
                days: days(starts, r, five, seven, tz),
            }
        })
        .collect()
}

pub fn periods(five: Vec<RateLimitPoint>, seven: Vec<RateLimitPoint>, tz: Tz) -> UsagePeriods {
    UsagePeriods {
        timezone: tz.name().to_string(),
        weeks: weeks(&five, &seven, tz),
        windows: windows(&five),
        five_hour: five,
        seven_day: seven,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().expect("test time")
    }

    fn pt(window: &str, u: f64, resets: &str, obs: &str) -> RateLimitPoint {
        RateLimitPoint {
            window: window.to_string(),
            utilization: Some(u),
            resets_at: Some(at(resets)),
            observed_at: at(obs),
        }
    }

    #[test]
    fn two_windows_and_a_week_reset() {
        let w1 = "2026-10-07T12:00:00Z";
        let w2 = "2026-10-07T17:00:00Z";
        let five = vec![
            pt("five_hour", 0.4, w1, "2026-10-07T08:00:00Z"),
            pt("five_hour", 1.0, w1, "2026-10-07T10:00:00Z"),
            pt("five_hour", 0.1, w2, "2026-10-07T13:00:00Z"),
            pt("five_hour", 0.6, w2, "2026-10-07T15:00:00Z"),
        ];
        let wk1 = "2026-10-08T07:00:00Z";
        let wk2 = "2026-10-15T07:00:00Z";
        let seven = vec![
            pt("seven_day", 0.9, wk1, "2026-10-07T10:00:00Z"),
            pt("seven_day", 0.95, wk1, "2026-10-07T15:00:00Z"),
            pt("seven_day", 0.02, wk2, "2026-10-08T08:00:00Z"),
        ];
        let p = periods(five, seven, chrono_tz::America::New_York);

        assert_eq!(p.windows.len(), 2);
        assert_eq!(p.windows[0].starts_at, at("2026-10-07T07:00:00Z"));
        assert_eq!((p.windows[0].peak, p.windows[0].hit_limit), (1.0, true));
        assert_eq!((p.windows[1].peak, p.windows[1].hit_limit), (0.6, false));

        assert_eq!(p.weeks.len(), 2);
        assert_eq!(p.weeks[0].resets_at, at(wk1));
        assert_eq!(p.weeks[0].starts_at, at("2026-10-01T07:00:00Z"));
        assert_eq!((p.weeks[0].peak, p.weeks[0].hit_limit), (0.95, false));
        assert_eq!(p.weeks[1].peak, 0.02);

        // Week 1 is Oct 1 3 AM to Oct 8 3 AM Eastern: eight calendar days, ends clipped.
        let days = &p.weeks[0].days;
        assert_eq!(days.len(), 8);
        assert_eq!(days[0].date, "2026-10-01");
        assert_eq!(days[0].starts_at, at(wk1) - Duration::days(7));
        assert_eq!(days[7].date, "2026-10-08");
        assert_eq!(days[7].ends_at, at(wk1));
        let oct7 = &days[6];
        assert_eq!(oct7.date, "2026-10-07");
        assert_eq!(oct7.peak_seven_day, Some(0.95));
        assert_eq!(oct7.peak_five_hour, Some(1.0));
        assert!(oct7.hit_five_hour);
        assert_eq!(days[0].peak_seven_day, None);
        assert!(!days[0].hit_five_hour);
    }

    #[test]
    fn jittered_resets_make_one_period() {
        let five = vec![
            pt(
                "five_hour",
                0.2,
                "2026-10-13T22:49:59.900Z",
                "2026-10-13T20:00:00Z",
            ),
            pt(
                "five_hour",
                0.5,
                "2026-10-13T22:50:00.100Z",
                "2026-10-13T21:00:00Z",
            ),
            pt(
                "five_hour",
                0.3,
                "2026-10-13T22:49:59.950Z",
                "2026-10-13T21:30:00Z",
            ),
            pt(
                "five_hour",
                0.1,
                "2026-10-14T03:50:00.200Z",
                "2026-10-14T00:00:00Z",
            ),
        ];
        let seven = vec![
            pt(
                "seven_day",
                0.4,
                "2026-10-15T07:59:59.990Z",
                "2026-10-10T00:00:00Z",
            ),
            pt(
                "seven_day",
                0.6,
                "2026-10-15T08:00:00.010Z",
                "2026-10-11T00:00:00Z",
            ),
            pt(
                "seven_day",
                0.0,
                "2026-10-22T08:00:00.005Z",
                "2026-10-16T00:00:00Z",
            ),
        ];
        let p = periods(five, seven, chrono_tz::UTC);
        assert_eq!(p.windows.len(), 2);
        assert_eq!(p.windows[0].resets_at, at("2026-10-13T22:50:00.100Z"));
        assert_eq!(p.windows[0].peak, 0.5);
        assert_eq!(p.weeks.len(), 2);
        assert_eq!(p.weeks[0].resets_at, at("2026-10-15T08:00:00.010Z"));
        assert_eq!(p.weeks[0].peak, 0.6);
    }

    #[test]
    fn no_readings_no_periods() {
        let p = periods(vec![], vec![], chrono_tz::UTC);
        assert!(p.weeks.is_empty() && p.windows.is_empty());
        assert_eq!(p.timezone, "UTC");
    }
}
