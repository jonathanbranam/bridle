//! `bridle focus gate`: the UserPromptSubmit hook of focus hours (ticket cvaq, slice A).
//! With no `[[focus]]` in `~/.bridle/config.toml`, or outside every period, or in a project
//! that set `focus_hours = false`, it prints nothing and does nothing.
//! Also records one JSON line per prompt, and (`bridle focus reply`, the Stop hook) per reply, to
//! `<bridle_home>/prompts.jsonl` (ticket u6w9).

use std::io::Write as _;
use std::path::Path;

use bridle_daemon::config::{FocusMode, FocusPeriod, focus_end, focus_opted_out, focus_periods};
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
        return Some(block_output(&end_text(&periods, &p, now)));
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
    Some(hook_output(&nudge_text(
        period,
        &end_text(&periods, period, now),
    )))
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

fn nudge_text(period: &FocusPeriod, end: &str) -> String {
    format!(
        "QUIET HOURS ({name}) until {end} ET. Hard limits for this reply: at most 3 sentences or \
         60 words. The first sentence nudges the human back to their real work. No tool calls \
         except the one the human asked for; no research, ticket filing, planning or new threads. \
         Defer anything extra with one line, \"saved for {end} ET\", and write it down only as a \
         single bridle message to yourself if truly needed. Ask no follow-up questions unless the \
         human asked for something that cannot proceed without one. Restarting watchers (wake \
         loops, background tasks) is always allowed in quiet hours; it is not the human's request \
         and the limits above do not apply to it.",
        name = period.name,
        end = end,
    )
}

/// The time quiet actually ends, following periods that touch or overlap; says so when the
/// follow hit its 7-day cap.
fn end_text(periods: &[FocusPeriod], period: &FocusPeriod, now: DateTime<Local>) -> String {
    let (end, capped) = focus_end(periods, period, now);
    let t = end.format("%-I:%M %p");
    if capped {
        format!("{t} (capped at 7 days: periods cover the whole week)")
    } else {
        t.to_string()
    }
}

fn block_output(end: &str) -> String {
    serde_json::json!({
        "decision": "block",
        "reason": format!(
            "Locked until {end}. Email bridle@dev.branam.us if it matters.",
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
            end_text(&focus_periods(home).unwrap_or_default(), &p, now),
        )),
        None => Ok(()),
    }
}

/// Record one `prompt` line to <bridle_home>/prompts.jsonl with the prompt timestamp and metadata.
/// Fails open: any error is swallowed. Never blocks on stdin if it's absent or a tty.
/// Called before the gate's early returns so it records on every prompt.
fn record_prompt(home: &Path, session_id: Option<String>) {
    record_event(home, "prompt", session_id);
}

/// `event` is `prompt` (the human sent one) or `reply` (the agent finished answering: the Stop
/// hook). Lines from before the field existed are prompts.
fn record_event(home: &Path, event: &str, session_id: Option<String>) {
    let now = Utc::now();
    let role = role_from_env();
    let machine = get_machine_hostname();
    let project = std::env::var("BRIDLE_PROJECT").ok();

    let record = serde_json::json!({
        "at": now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "session": session_id,
        "role": role,
        "machine": if machine.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(machine) },
        "project": project,
        "event": event,
    });

    let _ = std::fs::create_dir_all(home);
    let path = home.join("prompts.jsonl");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let line = format!("{}\n", record);
        let _ = file.write_all(line.as_bytes());
    }
}

/// Get the machine hostname from HOSTNAME env var or by running the hostname command.
fn get_machine_hostname() -> String {
    if let Ok(name) = std::env::var("HOSTNAME") {
        return name;
    }
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// The hook's stdin JSON. Returns None if stdin is absent, is a tty, or JSON parsing fails.
/// Bounded to 200 ms to avoid blocking the human's prompt.
fn read_stdin_json() -> Option<serde_json::Value> {
    use std::io::Read;
    use std::sync::mpsc;
    use std::time::Duration;

    let stdin = std::io::stdin();
    if nix::unistd::isatty(&stdin).unwrap_or(false) {
        return None;
    }

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = String::new();
        match stdin.lock().read_to_string(&mut buf) {
            Ok(0) | Err(_) => {
                let _ = tx.send(None);
            }
            Ok(_) => {
                let _ = tx.send(serde_json::from_str::<serde_json::Value>(&buf).ok());
            }
        }
    });

    rx.recv_timeout(Duration::from_millis(200)).ok().flatten()
}

/// Get the role from BRIDLE_AS environment variable, with optional advisor name.
fn role_from_env() -> Option<String> {
    std::env::var("BRIDLE_AS").ok().and_then(|role| {
        if role == "advisor" {
            if let Ok(name) = std::env::var("BRIDLE_ADVISOR_NAME") {
                Some(format!("advisor-{}", name))
            } else {
                Some("advisor".to_string())
            }
        } else if role == "orchestrator" || role == "aide" {
            Some(role)
        } else {
            None
        }
    })
}

/// The hook entry point. Never fails: a hook that errors would show in the human's session.
pub fn run_gate() {
    let home = bridle_api::discovery::bridle_home();
    let input = read_stdin_json();
    // Background-task notifications (a watcher finishing) arrive as prompts too; they are not
    // the human, so they are neither logged nor gated (cc45).
    if !input.as_ref().is_none_or(is_human_prompt) {
        return;
    }
    record_prompt(&home, session_id_of(input.as_ref()));

    let repo = std::env::var_os("CLAUDE_PROJECT_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if let Some(out) = gate(&home, &repo, Local::now()) {
        println!("{out}");
    }
}

fn session_id_of(input: Option<&serde_json::Value>) -> Option<String> {
    input?.get("session_id")?.as_str().map(str::to_string)
}

/// False for the harness's own prompts (`<task-notification>`), which carry no human input.
/// Anything else, including stdin without a `prompt` field, counts as the human (fail open).
fn is_human_prompt(input: &serde_json::Value) -> bool {
    !input
        .get("prompt")
        .and_then(|p| p.as_str())
        .is_some_and(|p| p.trim_start().starts_with("<task-notification>"))
}

/// The Stop hook's entry point: records that the agent finished replying, so the human's reading
/// time can start there. Prints nothing and never blocks the stop.
pub fn run_reply() {
    record_event(
        &bridle_api::discovery::bridle_home(),
        "reply",
        session_id_of(read_stdin_json().as_ref()),
    );
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
    fn task_notifications_are_not_human_prompts() {
        let j = |p: &str| serde_json::json!({"session_id": "s", "prompt": p});
        assert!(!is_human_prompt(&j(
            "<task-notification>\n<task-id>x</task-id>"
        )));
        assert!(is_human_prompt(&j("what is the status?")));
        assert!(is_human_prompt(&serde_json::json!({"session_id": "s"})));
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
            "Restarting watchers",
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
        // A period that always matches: pending nudges, active is silent, expired nudges. The
        // times come from the real clock (the override is capped at two hours after the file's
        // write time), so a second period covers 23:59..24:00 that the first leaves out.
        let always = home_with(Some(concat!(
            "[[focus]]\nname = \"all\"\ndays = \"all\"\nstart = \"00:00\"\nend = \"23:59\"\n",
            "[[focus]]\nname = \"late\"\ndays = \"all\"\nstart = \"23:00\"\nend = \"00:00\"\n",
        )));
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

    #[test]
    fn records_line_to_prompts_jsonl() {
        let home = tempfile::tempdir().expect("tempdir");
        record_prompt(home.path(), None);

        let path = home.path().join("prompts.jsonl");
        assert!(path.exists(), "prompts.jsonl should be created");

        let content = std::fs::read_to_string(&path).expect("read prompts.jsonl");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        let record: serde_json::Value = serde_json::from_str(lines[0]).expect("valid JSON");
        assert!(
            record["at"].is_string(),
            "at field should be present and a string"
        );
        assert!(
            record.get("session").is_some(),
            "session field should be present"
        );
        assert!(record.get("role").is_some(), "role field should be present");
        assert!(
            record.get("machine").is_some(),
            "machine field should be present"
        );
        assert!(
            record.get("project").is_some(),
            "project field should be present"
        );
    }

    #[test]
    fn prompt_and_reply_lines_carry_their_event() {
        let home = tempfile::tempdir().expect("tempdir");
        record_prompt(home.path(), None);
        record_event(home.path(), "reply", None);

        let content =
            std::fs::read_to_string(home.path().join("prompts.jsonl")).expect("read prompts.jsonl");
        let events: Vec<String> = content
            .lines()
            .map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).expect("valid JSON");
                v["event"].as_str().expect("event").to_string()
            })
            .collect();
        assert_eq!(events, ["prompt", "reply"]);
    }

    #[test]
    fn records_two_lines_on_two_calls() {
        let home = tempfile::tempdir().expect("tempdir");
        record_prompt(home.path(), None);
        record_prompt(home.path(), None);

        let content =
            std::fs::read_to_string(home.path().join("prompts.jsonl")).expect("read prompts.jsonl");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "should have exactly two lines");

        let record1: serde_json::Value = serde_json::from_str(lines[0]).expect("valid JSON");
        let record2: serde_json::Value = serde_json::from_str(lines[1]).expect("valid JSON");
        assert!(record1["at"].is_string());
        assert!(record2["at"].is_string());
    }

    #[test]
    fn unwritable_home_does_not_panic() {
        let home = tempfile::tempdir().expect("tempdir");
        let file_path = home.path().join("regular_file");
        std::fs::write(&file_path, "regular file").expect("write file");
        let unwritable_home = file_path.join("subdir");

        record_prompt(&unwritable_home, None);
    }

    #[test]
    fn missing_home_does_not_panic() {
        let missing_home = std::path::Path::new("/nonexistent/path/that/does/not/exist");
        record_prompt(missing_home, None);
    }

    #[test]
    fn gate_output_unchanged_despite_record_prompt_errors() {
        let home = tempfile::tempdir().expect("tempdir");
        let file_path = home.path().join("regular_file");
        std::fs::write(&file_path, "regular file").expect("write file");
        let unwritable_home = file_path.join("subdir");

        record_prompt(&unwritable_home, None);

        let gate_before = gate(home.path(), home.path(), at(2026, 9, 30, 10, 0));
        assert_eq!(gate_before, None, "gate output should be unchanged");
    }
}
