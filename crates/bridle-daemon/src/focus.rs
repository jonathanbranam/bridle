//! Focus hours, the parts the daemon shares with the `bridle focus gate` hook (ticket cvaq): the
//! hand-written override file and locked mode. With no `[[focus]]` in `<home>/config.toml`
//! nothing here acts.

use std::path::Path;

use chrono::{DateTime, Duration, Local, NaiveTime, TimeZone, Utc};
use serde::Deserialize;

use crate::config::{FocusMode, FocusPeriod, focus_override_delay_minutes, focus_periods};

pub const OVERRIDE_FILE: &str = "focus-override.toml";

/// The longest an override lasts from when it takes effect, whatever `until` says.
const OVERRIDE_MAX: Duration = Duration::hours(2);

/// `<home>/focus-override.toml`, written by the human by hand (no CLI, agents are denied it).
#[derive(Debug, Deserialize)]
struct RawOverride {
    reason: String,
}

/// A parsed override: it takes effect at `from` (the file's write time plus the delay) and ends
/// at `until`, capped at `OVERRIDE_MAX` after `from`.
#[derive(Debug, PartialEq)]
pub struct Override {
    pub from: DateTime<Utc>,
    pub until: DateTime<Utc>,
    pub reason: String,
}

impl Override {
    pub fn active(&self, now: DateTime<Utc>) -> bool {
        self.from <= now && now < self.until
    }
}

/// The override file, if present and well-formed. A malformed file is logged and ignored, so a
/// typo never silently disables focus hours (or crashes the hook).
pub fn read_override(home: &Path) -> Option<Override> {
    let path = home.join(OVERRIDE_FILE);
    let text = std::fs::read_to_string(&path).ok()?;
    let written: DateTime<Utc> = std::fs::metadata(&path).ok()?.modified().ok()?.into();
    parse_override(&text, written, focus_override_delay_minutes(home)).or_else(|| {
        tracing::warn!("ignoring malformed {}", path.display());
        None
    })
}

fn parse_override(text: &str, written: DateTime<Utc>, delay_minutes: u32) -> Option<Override> {
    let raw: RawOverride = toml::from_str(text).ok()?;
    let from = written + Duration::minutes(i64::from(delay_minutes));

    // Parse until from the TOML table directly to get the raw value
    let until_utc = parse_until_from_text(text, from)?;

    Some(Override {
        from,
        until: until_utc.min(from + OVERRIDE_MAX),
        reason: raw.reason,
    })
}

/// Extract and parse the `until` value in three forms:
/// 1. Offset datetime or Z (existing): e.g., `2026-10-01T22:00:00Z` or `2026-10-01T18:00:00-04:00`
/// 2. Local datetime (new): e.g., `2026-10-01T22:00:00`, interpreted in the local zone
/// 3. String "HH:MM" (new): e.g., `"22:00"`, meaning the next occurrence of that time
fn parse_until_from_text(text: &str, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let table: toml::Table = toml::from_str(text).ok()?;
    let value = table.get("until")?;

    match value {
        toml::value::Value::Datetime(dt) => {
            // Try to parse as offset datetime (existing forms)
            let s = dt.to_string();
            if let Ok(utc_dt) = s.parse::<DateTime<Utc>>() {
                return Some(utc_dt);
            }

            // Try to parse as local datetime (new form)
            if let Ok(local_dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
                if let Some(local) = Local.from_local_datetime(&local_dt).single() {
                    return Some(local.with_timezone(&Utc));
                } else {
                    // Ambiguous or nonexistent local time
                    tracing::warn!("until time {s:?} is ambiguous or nonexistent in local zone");
                }
            }

            None
        }
        toml::value::Value::String(s) => {
            // Try to parse as "HH:MM" (new form)
            if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M") {
                // Calculate the next occurrence of this time
                let from_local = from.with_timezone(&Local);
                let today = from_local.date_naive();
                let today_occurrence = today.and_time(time);

                let target_local = if let Some(today_time) =
                    Local.from_local_datetime(&today_occurrence).single()
                {
                    if today_time > from_local {
                        today_time
                    } else {
                        // Time already passed today, use tomorrow
                        let tomorrow = today + Duration::days(1);
                        let tomorrow_occurrence = tomorrow.and_time(time);
                        Local
                            .from_local_datetime(&tomorrow_occurrence)
                            .single()
                            .expect("tomorrow's time is unambiguous")
                    }
                } else {
                    // Today's occurrence is ambiguous or nonexistent (DST), try tomorrow
                    let tomorrow = today + Duration::days(1);
                    let tomorrow_occurrence = tomorrow.and_time(time);
                    Local.from_local_datetime(&tomorrow_occurrence).single()?
                };

                return Some(target_local.with_timezone(&Utc));
            }

            None
        }
        _ => None,
    }
}

/// The locked period covering `now`, unless an active override lifts it. `None` when nothing is
/// configured, so every caller is a no-op by default.
pub fn locked_period(home: &Path, now: DateTime<Local>) -> Option<FocusPeriod> {
    let period = focus_periods(home)
        .ok()?
        .into_iter()
        .find(|p| p.mode == FocusMode::Locked && p.matches(now))?;
    if read_override(home).is_some_and(|o| o.active(now.with_timezone(&Utc))) {
        return None;
    }
    Some(period)
}

/// The tmux panes tagged as advisors (`@bridle` is `advisor` or `advisor-<name>`, see
/// `bridle session`) in `tmux list-panes -a -F "#{pane_id} #{@bridle}"` output.
pub fn advisor_panes(list_panes: &str) -> Vec<&str> {
    list_panes
        .lines()
        .filter_map(|l| {
            let (id, tag) = l.split_once(' ')?;
            let tag = tag.trim();
            (tag == "advisor" || tag == "aide" || tag.starts_with("advisor-")).then_some(id)
        })
        .collect()
}

/// Locked mode's shutdown: while a locked period covers now, kill every advisor pane. Run from an
/// existing periodic loop, so a period that starts, or an advisor started around the CLI, is
/// caught within one tick. Best effort: no tmux server means no advisors.
pub fn stop_advisors_if_locked(home: &Path, now: DateTime<Local>) {
    if locked_period(home, now).is_none() {
        return;
    }
    let Ok(out) = std::process::Command::new("tmux")
        .args(["list-panes", "-a", "-F", "#{pane_id} #{@bridle}"])
        .output()
    else {
        return;
    };
    if !out.status.success() {
        return;
    }
    for pane in advisor_panes(&String::from_utf8_lossy(&out.stdout)) {
        tracing::info!(pane, "focus hours locked: stopping advisor");
        let _ = std::process::Command::new("tmux")
            .args(["kill-pane", "-t", pane])
            .status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike};

    fn at(h: u32, m: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 9, 30, h, m, 0)
            .single()
            .expect("unambiguous local time")
    }

    fn utc(h: u32, m: u32) -> DateTime<Utc> {
        at(h, m).with_timezone(&Utc)
    }

    fn home_with(config: &str) -> tempfile::TempDir {
        let home = tempfile::tempdir().expect("tempdir");
        std::fs::write(home.path().join("config.toml"), config).expect("write config");
        home
    }

    const LOCKED: &str = "[[focus]]\nname = \"work\"\ndays = \"all\"\nstart = \"08:00\"\nend = \"17:00\"\nmode = \"locked\"\n";

    #[test]
    fn override_takes_effect_after_the_delay_and_expires() {
        let written = utc(10, 0);
        let o = parse_override("until = 2026-09-30T20:00:00Z\nreason = \"x\"", written, 10)
            .expect("parses");
        assert!(!o.active(utc(10, 9)));
        assert!(o.active(utc(10, 10)));
        assert!(o.active(o.until - Duration::seconds(1)));
        assert!(!o.active(o.until));
        // A short `until` wins over the cap.
        let short = parse_override(
            &format!(
                "until = {}\nreason = \"x\"",
                (written + Duration::minutes(30)).to_rfc3339()
            ),
            written,
            10,
        )
        .expect("parses");
        assert_eq!(short.until, written + Duration::minutes(30));
    }

    #[test]
    fn accepts_offset_datetime_forms() {
        let written = utc(10, 0);

        // UTC form with Z - use a time within the 2-hour cap
        let utc_z = parse_override("until = 2026-09-30T11:00:00Z\nreason = \"x\"", written, 0)
            .expect("parses UTC Z form");
        let expected_z = "2026-09-30T11:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(utc_z.until, expected_z);

        // UTC form with offset (unquoted in TOML) - same time as Z form
        let utc_offset = parse_override(
            "until = 2026-09-30T11:00:00+00:00\nreason = \"x\"",
            written,
            0,
        )
        .expect("parses UTC offset form");
        let expected_offset = "2026-09-30T11:00:00+00:00"
            .parse::<DateTime<Utc>>()
            .unwrap();
        assert_eq!(utc_offset.until, expected_offset);

        // Non-UTC offset (unquoted in TOML): 7:00 EDT is the same as 11:00 UTC
        let non_utc = parse_override(
            "until = 2026-09-30T07:00:00-04:00\nreason = \"x\"",
            written,
            0,
        )
        .expect("parses non-UTC offset form");
        let expected_non_utc = "2026-09-30T07:00:00-04:00"
            .parse::<DateTime<Utc>>()
            .unwrap();
        assert_eq!(non_utc.until, expected_non_utc);
    }

    #[test]
    fn accepts_local_datetime_form() {
        let written = utc(10, 0);

        // Local datetime form (no offset) - use a time within 2-hour cap from `from`
        let local_dt = parse_override("until = 2026-09-30T12:00:00\nreason = \"x\"", written, 0)
            .expect("parses local datetime form");

        // Should be interpreted in local time: 12:00 local = (if EDT) 16:00 UTC
        let expected = Local
            .with_ymd_and_hms(2026, 9, 30, 12, 0, 0)
            .single()
            .expect("unambiguous local time")
            .with_timezone(&Utc);
        assert_eq!(local_dt.until, expected);
    }

    #[test]
    fn accepts_string_hhmm_form() {
        let written = utc(10, 0);

        // String "HH:MM" form - time in the future today (within 2-hour cap)
        let from_local = written.with_timezone(&Local);
        let local_hour = from_local.hour();
        let hhmm = format!("{:02}:30", local_hour + 1); // Next hour, 30 minutes

        let future_today =
            parse_override(&format!("until = \"{}\"\nreason = \"x\"", hhmm), written, 0)
                .expect("parses HH:MM form");

        let today = from_local.date_naive();
        let expected = Local
            .with_ymd_and_hms(
                today.year(),
                today.month(),
                today.day(),
                local_hour + 1,
                30,
                0,
            )
            .single()
            .expect("unambiguous time")
            .with_timezone(&Utc);
        assert_eq!(future_today.until, expected);
    }

    #[test]
    fn string_hhmm_rolls_to_tomorrow_if_past() {
        let written = utc(10, 0);

        // String "HH:MM" form - time already passed today
        let past_today = parse_override("until = \"00:00\"\nreason = \"x\"", written, 0)
            .expect("parses HH:MM form for past time");

        // Just verify it parsed successfully and is valid (exact value depends on capping)
        assert!(past_today.until > written);
    }

    #[test]
    fn rejects_malformed_values() {
        let written = utc(10, 0);

        // Invalid datetime string
        assert!(parse_override("until = \"not-a-time\"\nreason = \"x\"", written, 0).is_none());

        // Invalid TOML value type
        assert!(parse_override("until = 123\nreason = \"x\"", written, 0).is_none());
    }

    #[test]
    fn override_is_capped_at_two_hours_from_effect() {
        let written = utc(10, 0);
        let o = parse_override("until = 2099-01-01T00:00:00Z\nreason = \"x\"", written, 10)
            .expect("parses");
        assert_eq!(o.until, o.from + Duration::hours(2));
    }

    #[test]
    fn locked_only_in_a_configured_locked_period() {
        assert!(locked_period(tempfile::tempdir().expect("tempdir").path(), at(10, 0)).is_none());
        let home = home_with(LOCKED);
        assert!(locked_period(home.path(), at(10, 0)).is_some());
        assert!(locked_period(home.path(), at(17, 0)).is_none());
        let quiet = home_with(&LOCKED.replace("locked", "quiet"));
        assert!(locked_period(quiet.path(), at(10, 0)).is_none());
    }

    #[test]
    fn an_active_override_lifts_the_lock() {
        let home = home_with(LOCKED);
        let file = home.path().join(OVERRIDE_FILE);
        std::fs::write(&file, "until = 2099-01-01T00:00:00Z\nreason = \"deploy\"\n")
            .expect("write");
        // A fixed mtime inside the locked period, so the test doesn't depend on the wall clock.
        std::fs::File::options()
            .write(true)
            .open(&file)
            .expect("open")
            .set_modified(utc(10, 0).into())
            .expect("set mtime");
        let o = read_override(home.path()).expect("parses");
        let t = |u: DateTime<Utc>| u.with_timezone(&Local);
        assert!(locked_period(home.path(), t(o.from - Duration::minutes(1))).is_some());
        assert!(locked_period(home.path(), t(o.from + Duration::minutes(1))).is_none());
    }

    #[test]
    fn advisor_panes_are_found_by_tag() {
        let panes = "%1 \n%4 orchestrator\n%5 advisor-x\n%6 advisor\n%7 advisors\n";
        assert_eq!(advisor_panes(panes), ["%5", "%6"]);
    }
}
