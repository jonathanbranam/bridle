//! `bridle focus gate`: the UserPromptSubmit hook of focus hours (ticket cvaq, slice A).
//! With no `[[focus]]` in `~/.bridle/config.toml`, or outside every period, or in a project
//! that set `focus_hours = false`, it prints nothing and does nothing.

use std::path::Path;

use bridle_daemon::config::{FocusMode, FocusPeriod, focus_opted_out, focus_periods};
use bridle_daemon::focus::{locked_period, read_override};
use chrono::{DateTime, Local, Utc};

/// Repeat the nudge once this long has passed since the last one (the human's call: 30 min is
/// too long).
const NUDGE_EVERY_SECS: i64 = 5 * 60;

const STATE_FILE: &str = "focus-nudge";

/// The `bridle status` line for an override, if a file exists and focus hours are configured.
pub fn status_line(home: &Path, now: DateTime<Utc>) -> Option<String> {
    if focus_periods(home).ok()?.is_empty() {
        return None;
    }
    let o = read_override(home)?;
    let et = |t: DateTime<Utc>| t.with_timezone(&Local).format("%-I:%M %p").to_string();
    if o.active(now) {
        Some(format!(
            "override active until {}: {}",
            et(o.until),
            o.reason
        ))
    } else if now < o.from {
        Some(format!(
            "override pending, takes effect {}: {}",
            et(o.from),
            o.reason
        ))
    } else {
        None
    }
}

/// The hook's stdout for `now`: the JSON that adds the nudge to the prompt's context, or
/// `None` for silence. Records the nudge in `<home>/focus-nudge` as `<unix secs> <period>`.
pub fn gate(home: &Path, repo: &Path, now: DateTime<Local>) -> Option<String> {
    let periods = focus_periods(home).ok()?;
    if periods.is_empty() || focus_opted_out(repo) {
        return None;
    }
    // Locked blocks every prompt (an active override lifts it, inside `locked_period`).
    if let Some(p) = locked_period(home, now) {
        return Some(block_output(&p));
    }
    let period = periods
        .iter()
        .find(|p| p.mode == FocusMode::Quiet && p.matches(now))?;
    if read_override(home).is_some_and(|o| o.active(now.with_timezone(&Utc))) {
        return None;
    }
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
        "QUIET HOURS ({name}) until {end} ET. Hard limits for this reply: at most 3 sentences or \
         60 words. The first sentence nudges the human back to their real work. No tool calls \
         except the one the human asked for; no research, ticket filing, planning or new threads. \
         Defer anything extra with one line, \"saved for {end} ET\", and write it down only as a \
         single bridle message to yourself if truly needed. Ask no follow-up questions unless the \
         human asked for something that cannot proceed without one.",
        name = period.name,
        end = period.end.format("%-I:%M %p"),
    )
}

fn block_output(period: &FocusPeriod) -> String {
    serde_json::json!({
        "decision": "block",
        "reason": format!(
            "Locked until {}. Email bridle@dev.branam.us if it matters.",
            period.end.format("%-I:%M %p"),
        ),
    })
    .to_string()
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

/// Locked mode refuses to start an advisor (`bridle session advisor`, `bridle advisor start`).
pub fn refuse_advisor_if_locked(home: &Path, now: DateTime<Local>) -> Result<(), anyhow::Error> {
    match locked_period(home, now) {
        Some(p) => Err(anyhow::anyhow!(
            "focus hours ({}) are locked until {}: no advisors. Email bridle@dev.branam.us if it \
             matters.",
            p.name,
            p.end.format("%-I:%M %p"),
        )),
        None => Ok(()),
    }
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
    use bridle_daemon::focus::OVERRIDE_FILE;
    use chrono::{Duration, TimeZone};

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
            first.contains("QUIET HOURS (work) until 6:00 PM ET"),
            "{first}"
        );
        for limit in [
            "3 sentences or 60 words",
            "first sentence nudges",
            "No tool calls",
            "saved for 6:00 PM ET",
            "Ask no follow-up",
        ] {
            assert!(first.contains(limit), "{limit}: {first}");
        }
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
    fn locked_blocks_every_prompt() {
        let home = home_with(Some(&WORK.replace("quiet", "locked")));
        for m in [0, 1, 30] {
            let out = gate(home.path(), home.path(), at(2026, 9, 30, 10, m)).expect("blocks");
            let v: serde_json::Value = serde_json::from_str(&out).expect("json");
            assert_eq!(v["decision"], "block");
            assert_eq!(
                v["reason"],
                "Locked until 6:00 PM. Email bridle@dev.branam.us if it matters."
            );
        }
        assert_eq!(gate(home.path(), home.path(), at(2026, 9, 30, 18, 0)), None);
    }

    #[test]
    fn advisors_are_refused_only_while_locked() {
        let home = home_with(Some(&WORK.replace("quiet", "locked")));
        let err = refuse_advisor_if_locked(home.path(), at(2026, 9, 30, 10, 0)).unwrap_err();
        assert!(err.to_string().contains("locked until 6:00 PM"), "{err}");
        assert!(refuse_advisor_if_locked(home.path(), at(2026, 9, 30, 19, 0)).is_ok());
        let quiet = home_with(Some(WORK));
        assert!(refuse_advisor_if_locked(quiet.path(), at(2026, 9, 30, 10, 0)).is_ok());
        assert!(refuse_advisor_if_locked(home_with(None).path(), at(2026, 9, 30, 10, 0)).is_ok());
    }

    #[test]
    fn a_project_opt_out_and_an_active_override_lift_the_lock() {
        let home = home_with(Some(&WORK.replace("quiet", "locked")));
        let repo = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(repo.path().join(".bridle")).expect("mkdir");
        std::fs::write(
            repo.path().join(".bridle/config.toml"),
            "focus_hours = false\n",
        )
        .expect("write");
        assert_eq!(gate(home.path(), repo.path(), at(2026, 9, 30, 10, 0)), None);
        std::fs::write(
            home.path().join(OVERRIDE_FILE),
            "until = 2099-01-01T00:00:00Z\nreason = \"deploy\"\n",
        )
        .expect("write");
        let o = read_override(home.path()).expect("parses");
        let t = (o.from + chrono::Duration::minutes(1)).with_timezone(&Local);
        assert_eq!(gate(home.path(), home.path(), t), None);
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
    fn malformed_override_is_ignored() {
        let home = home_with(Some(WORK));
        std::fs::write(home.path().join(OVERRIDE_FILE), "garbage [").expect("write");
        assert!(read_override(home.path()).is_none());
        assert!(gate(home.path(), home.path(), at(2026, 9, 30, 10, 0)).is_some());
    }

    #[test]
    fn gate_honors_an_active_override_only() {
        let home = home_with(Some(WORK));
        std::fs::write(
            home.path().join(OVERRIDE_FILE),
            "until = 2099-01-01T00:00:00Z\nreason = \"deploy\"\n",
        )
        .expect("write");
        // A period that always matches: pending nudges, active is silent, expired nudges.
        let always = home_with(Some(
            "[[focus]]\nname = \"all\"\ndays = \"all\"\nstart = \"00:00\"\nend = \"23:59\"\n",
        ));
        std::fs::copy(
            home.path().join(OVERRIDE_FILE),
            always.path().join(OVERRIDE_FILE),
        )
        .expect("copy");
        let o = read_override(always.path()).expect("parses");
        let g = |t: DateTime<Utc>| gate(always.path(), always.path(), t.with_timezone(&Local));
        assert!(g(o.from - Duration::minutes(1)).is_some(), "pending nudges");
        assert_eq!(g(o.from + Duration::minutes(1)), None, "active is silent");
        assert!(
            g(o.until + Duration::minutes(1)).is_some(),
            "expired nudges"
        );
    }

    #[test]
    fn a_bad_mode_is_silent_not_an_error() {
        let home = home_with(Some(&WORK.replace("quiet", "loud")));
        assert_eq!(gate(home.path(), home.path(), at(2026, 9, 30, 10, 0)), None);
    }
}
