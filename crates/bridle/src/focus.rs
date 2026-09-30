//! `bridle focus gate`: the UserPromptSubmit hook of focus hours (ticket cvaq, slice A).
//! With no `[[focus]]` in `~/.bridle/config.toml`, or outside every period, or in a project
//! that set `focus_hours = false`, it prints nothing and does nothing.

use std::path::Path;

use bridle_daemon::config::{FocusMode, FocusPeriod, focus_opted_out, focus_periods};
use chrono::{DateTime, Local};

/// Repeat the nudge once this long has passed since the last one (the human's call: 30 min is
/// too long).
const NUDGE_EVERY_SECS: i64 = 5 * 60;

const STATE_FILE: &str = "focus-nudge";

/// The hook's stdout for `now`: the JSON that adds the nudge to the prompt's context, or
/// `None` for silence. Records the nudge in `<home>/focus-nudge` as `<unix secs> <period>`.
pub fn gate(home: &Path, repo: &Path, now: DateTime<Local>) -> Option<String> {
    let periods = focus_periods(home).ok()?;
    if periods.is_empty() || focus_opted_out(repo) {
        return None;
    }
    let period = periods
        .iter()
        .find(|p| p.mode == FocusMode::Quiet && p.matches(now))?;
    let state = home.join(STATE_FILE);
    let last = std::fs::read_to_string(&state).ok();
    if !nudge_due(last.as_deref(), period, now.timestamp()) {
        return None;
    }
    let _ = std::fs::create_dir_all(home);
    let _ = std::fs::write(&state, format!("{} {}\n", now.timestamp(), period.name));
    Some(hook_output(&nudge_text(period)))
}

/// Due on the first prompt of a period (no record, or the record is of another period) and
/// again once `NUDGE_EVERY_SECS` have passed. A clock that went backwards nudges too.
fn nudge_due(last: Option<&str>, period: &FocusPeriod, now: i64) -> bool {
    let Some((secs, name)) = last.and_then(|l| l.trim().split_once(' ')) else {
        return true;
    };
    let Ok(secs) = secs.parse::<i64>() else {
        return true;
    };
    name != period.name || !(0..NUDGE_EVERY_SECS).contains(&(now - secs))
}

fn nudge_text(period: &FocusPeriod) -> String {
    format!(
        "Quiet hours ({}) until {} ET. Lead your answer with a one-line nudge for the human to \
         go back to what they should be doing, then keep the answer minimal.",
        period.name,
        period.end.format("%-I:%M %p"),
    )
}

fn hook_output(context: &str) -> String {
    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "UserPromptSubmit",
            "additionalContext": context,
        }
    })
    .to_string()
}

/// The hook entry point. Never fails: a hook that errors would show in the human's session.
pub fn run_gate() {
    let repo = std::env::var_os("CLAUDE_PROJECT_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if let Some(out) = gate(&bridle_api::discovery::bridle_home(), &repo, Local::now()) {
        println!("{out}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(y, mo, d, h, mi, 0)
            .single()
            .expect("unambiguous local time")
    }

    const WORK: &str = r#"
        [[focus]]
        name = "work"
        days = ["mon", "tue", "wed", "thu", "fri"]
        start = "08:00"
        end = "18:00"
        mode = "quiet"
    "#;

    fn home_with(config: Option<&str>) -> tempfile::TempDir {
        let home = tempfile::tempdir().expect("tempdir");
        if let Some(c) = config {
            std::fs::write(home.path().join("config.toml"), c).expect("write config");
        }
        home
    }

    // 2026-09-30 is a Wednesday; 2026-10-03 a Saturday.
    #[test]
    fn unconfigured_is_silent_and_writes_nothing() {
        let home = home_with(None);
        assert_eq!(gate(home.path(), home.path(), at(2026, 9, 30, 10, 0)), None);
        assert!(!home.path().join(STATE_FILE).exists());
        let empty = home_with(Some("[budget]\nmax_workers = 3\n"));
        assert_eq!(
            gate(empty.path(), empty.path(), at(2026, 9, 30, 10, 0)),
            None
        );
    }

    #[test]
    fn nudges_first_prompt_then_every_five_minutes() {
        let home = home_with(Some(WORK));
        let g = |h, m| gate(home.path(), home.path(), at(2026, 9, 30, h, m));
        let first = g(10, 0).expect("first prompt nudges");
        assert!(
            first.contains("Quiet hours (work) until 6:00 PM ET"),
            "{first}"
        );
        assert_eq!(g(10, 4), None);
        assert!(g(10, 5).is_some());
        assert_eq!(g(10, 9), None);
        assert!(g(10, 30).is_some());
    }

    #[test]
    fn silent_outside_the_period() {
        let home = home_with(Some(WORK));
        assert_eq!(gate(home.path(), home.path(), at(2026, 9, 30, 7, 59)), None);
        assert_eq!(gate(home.path(), home.path(), at(2026, 9, 30, 18, 0)), None);
        assert_eq!(gate(home.path(), home.path(), at(2026, 10, 3, 10, 0)), None);
    }

    #[test]
    fn a_period_crossing_midnight_matches_on_both_sides() {
        let home = home_with(Some(
            r#"
            [[focus]]
            name = "sleep"
            days = "all"
            start = "23:00"
            end = "07:00"
            "#,
        ));
        assert!(gate(home.path(), home.path(), at(2026, 9, 30, 23, 30)).is_some());
        // A new period name or a gap over 5 minutes both re-nudge; here the gap.
        assert!(gate(home.path(), home.path(), at(2026, 10, 1, 6, 0)).is_some());
        assert_eq!(gate(home.path(), home.path(), at(2026, 10, 1, 12, 0)), None);
    }

    #[test]
    fn locked_parses_but_does_not_act_yet() {
        let home = home_with(Some(&WORK.replace("quiet", "locked")));
        assert_eq!(gate(home.path(), home.path(), at(2026, 9, 30, 10, 0)), None);
    }

    #[test]
    fn a_project_can_opt_out() {
        let home = home_with(Some(WORK));
        let repo = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(repo.path().join(".bridle")).expect("mkdir");
        std::fs::write(
            repo.path().join(".bridle/config.toml"),
            "focus_hours = false\n",
        )
        .expect("write");
        assert_eq!(gate(home.path(), repo.path(), at(2026, 9, 30, 10, 0)), None);
    }

    #[test]
    fn a_bad_mode_is_silent_not_an_error() {
        let home = home_with(Some(&WORK.replace("quiet", "loud")));
        assert_eq!(gate(home.path(), home.path(), at(2026, 9, 30, 10, 0)), None);
    }
}
