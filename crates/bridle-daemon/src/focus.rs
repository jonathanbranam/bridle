//! Focus hours, the parts the daemon shares with the `bridle focus gate` hook (ticket cvaq): the
//! hand-written override file and locked mode. With no `[[focus]]` in `<home>/config.toml`
//! nothing here acts.

use std::path::Path;

use chrono::{DateTime, Duration, Local, Utc};
use serde::Deserialize;

use crate::config::{FocusMode, FocusPeriod, focus_override_delay_minutes, focus_periods};

pub const OVERRIDE_FILE: &str = "focus-override.toml";

/// The longest an override lasts from when it takes effect, whatever `until` says.
const OVERRIDE_MAX: Duration = Duration::hours(2);

/// `<home>/focus-override.toml`, written by the human by hand (no CLI, agents are denied it).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOverride {
    until: toml::value::Datetime,
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
    Some(Override {
        from,
        until: raw
            .until
            .to_string()
            .parse::<DateTime<Utc>>()
            .ok()?
            .min(from + OVERRIDE_MAX),
        reason: raw.reason,
    })
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
    use chrono::TimeZone;

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
