//! `bridle report`: the "what happened" report for one project over a window, built
//! mechanically from tasks, edges and git (docs/design/report.md). No LLM, so it costs nothing
//! and is the same every time.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command as Proc;

use bridle_api::{Edge, EdgeKind, Task, TaskKind, TaskState};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};

use super::client_for_read;
use crate::cli::{Cli, ReportArgs};
use crate::error::CliError;

const COMMIT_SUBJECTS_SHOWN: usize = 20;

pub(super) async fn report(cli: &Cli, args: &ReportArgs) -> Result<(), CliError> {
    let since = parse_window(&args.since).map_err(|e| CliError::Other(anyhow::anyhow!(e)))?;
    let client = client_for_read(cli).await?;
    let tasks = client.list_tasks().await?;
    let edges = client.list_edges().await?;
    let now = Utc::now();
    let commits = git_subjects(now - since);
    let text = render(&tasks, &edges, &commits, now - since, now);
    if args.write {
        let dir = PathBuf::from("docs/reports");
        std::fs::create_dir_all(&dir)
            .map_err(|e| CliError::Other(anyhow::anyhow!("{}: {e}", dir.display())))?;
        let path = dir.join(format!("{}.md", eastern_date(now)));
        std::fs::write(&path, &text)
            .map_err(|e| CliError::Other(anyhow::anyhow!("{}: {e}", path.display())))?;
        println!("{}", path.display());
    } else {
        print!("{text}");
    }
    Ok(())
}

/// `24h`, `48h`, `90m`, `2d`.
pub(crate) fn parse_window(s: &str) -> Result<Duration, String> {
    let bad = || format!("bad --since {s:?}: use a number and h, m or d, like 24h");
    let (num, unit) = s.split_at(s.len().saturating_sub(1));
    let n: i64 = num.parse().map_err(|_| bad())?;
    if n <= 0 {
        return Err(bad());
    }
    match unit {
        "m" => Ok(Duration::minutes(n)),
        "h" => Ok(Duration::hours(n)),
        "d" => Ok(Duration::days(n)),
        _ => Err(bad()),
    }
}

/// Commit subjects on main (else HEAD) since `since`, newest first.
fn git_subjects(since: DateTime<Utc>) -> Vec<String> {
    let run = |rev: &str| {
        Proc::new("git")
            .args(["log", rev, "--format=%s", "--since"])
            .arg(since.to_rfc3339())
            .output()
            .ok()
            .filter(|o| o.status.success())
    };
    run("main")
        .or_else(|| run("HEAD"))
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

// US Eastern with the US rule (since 2007), so no timezone database is needed; the same
// arithmetic as the gateway's report.
fn eastern_offset(at: DateTime<Utc>) -> Duration {
    let year = at.year();
    let transition = |month, nth, hour| {
        NaiveDate::from_weekday_of_month_opt(year, month, Weekday::Sun, nth)
            .and_then(|d| d.and_hms_opt(hour, 0, 0))
            .map(|d| d.and_utc())
    };
    match (transition(3, 2, 7), transition(11, 1, 6)) {
        (Some(start), Some(end)) if at >= start && at < end => Duration::hours(-4),
        _ => Duration::hours(-5),
    }
}

fn eastern_date(at: DateTime<Utc>) -> NaiveDate {
    (at + eastern_offset(at)).date_naive()
}

/// "Oct 8, 7:00 AM".
fn eastern(at: DateTime<Utc>) -> String {
    (at + eastern_offset(at))
        .format("%b %-d, %-I:%M %p")
        .to_string()
}

/// When the task was integrated: the daemon's `integrated: <sha>` thread note, else its last
/// update (the task has no field for it).
fn integrated_at(t: &Task) -> DateTime<Utc> {
    t.thread
        .iter()
        .find(|e| e.body.starts_with("integrated:"))
        .map_or(t.updated_at, |e| e.at)
}

fn is_integrated_in(t: &Task, from: DateTime<Utc>, to: DateTime<Utc>) -> bool {
    t.state == TaskState::Integrated && (from..=to).contains(&integrated_at(t))
}

fn created_in(t: &Task, from: DateTime<Utc>, to: DateTime<Utc>) -> bool {
    (from..=to).contains(&t.created_at)
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(9)]
}

fn delivered_line(t: &Task) -> String {
    match &t.commit {
        Some(c) => format!("- {}: {} ({})", t.id, t.title, short(c)),
        None => format!("- {}: {}", t.id, t.title),
    }
}

fn section(out: &mut String, title: &str, lines: &[String]) {
    let _ = writeln!(out, "\n## {title}\n");
    if lines.is_empty() {
        out.push_str("none\n");
    }
    for l in lines {
        let _ = writeln!(out, "{l}");
    }
}

fn sub(out: &mut String, title: &str, lines: &[String]) {
    let _ = writeln!(out, "{title}:");
    if lines.is_empty() {
        out.push_str("- none\n");
    }
    for l in lines {
        let _ = writeln!(out, "{l}");
    }
}

/// The report for the window `[from, to]`. Pure: everything it needs is passed in.
pub(crate) fn render(
    tasks: &[Task],
    edges: &[Edge],
    commits: &[String],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> String {
    let mut out = format!("# What happened, {} to {}\n", eastern(from), eastern(to));

    let mut sorted: Vec<&Task> = tasks.iter().collect();
    sorted.sort_by_key(|t| t.created_at);

    let features: Vec<String> = sorted
        .iter()
        .filter(|t| t.kind == TaskKind::Feature && is_integrated_in(t, from, to))
        .map(|t| delivered_line(t))
        .collect();
    section(&mut out, "Features built and delivered", &features);

    let found: Vec<String> = sorted
        .iter()
        .filter(|t| t.kind == TaskKind::Bug && created_in(t, from, to))
        .map(|t| format!("- {}: {} ({})", t.id, t.title, t.state))
        .collect();
    let fixed: Vec<String> = sorted
        .iter()
        .filter(|t| t.kind == TaskKind::Bug && is_integrated_in(t, from, to))
        .map(|t| delivered_line(t))
        .collect();
    let _ = write!(out, "\n## Bugs\n\n");
    sub(&mut out, "Identified", &found);
    out.push('\n');
    sub(&mut out, "Fixed and delivered", &fixed);

    let pending: Vec<String> = sorted
        .iter()
        .filter(|t| {
            t.kind != TaskKind::Incident
                && matches!(
                    t.state,
                    TaskState::Pending | TaskState::Open | TaskState::Planned | TaskState::Claimed
                )
        })
        .map(|t| {
            let blockers: Vec<&str> = edges
                .iter()
                .filter(|e| e.kind == EdgeKind::Blocks && e.to == t.id)
                .filter(|e| {
                    tasks
                        .iter()
                        .find(|b| b.id == e.from)
                        .is_some_and(|b| b.state != TaskState::Integrated)
                })
                .map(|e| e.from.as_str())
                .collect();
            let why = if !blockers.is_empty() {
                format!("blocked by {}", blockers.join(", "))
            } else if let (TaskState::Claimed, Some(who)) = (t.state, &t.claimed_by) {
                format!("claimed by {who}")
            } else {
                format!("{}, waiting", t.state)
            };
            format!("- {}: {} ({why})", t.id, t.title)
        })
        .collect();
    section(&mut out, "Pending or blocked", &pending);

    let incidents: Vec<String> = sorted
        .iter()
        .filter(|t| {
            t.kind == TaskKind::Incident
                && (created_in(t, from, to) || is_integrated_in(t, from, to))
        })
        .map(|t| {
            let note = t.thread.last().map_or("no notes".to_string(), |e| {
                e.body.lines().next().unwrap_or("").to_string()
            });
            format!("- {}: {} [{}] {note}", t.id, t.title, t.state)
        })
        .collect();
    section(&mut out, "Incidents", &incidents);

    let created = tasks.iter().filter(|t| created_in(t, from, to)).count();
    let other: Vec<&Task> = sorted
        .iter()
        .filter(|t| {
            !matches!(
                t.kind,
                TaskKind::Feature | TaskKind::Bug | TaskKind::Incident
            ) && is_integrated_in(t, from, to)
        })
        .copied()
        .collect();
    let mut lines = vec![
        format!("- tasks created: {created}"),
        format!("- other tasks integrated: {}", other.len()),
    ];
    lines.extend(
        other
            .iter()
            .map(|t| format!("  - {}: {} ({})", t.id, t.title, t.kind)),
    );
    lines.push(format!("- commits on main: {}", commits.len()));
    lines.extend(
        commits
            .iter()
            .take(COMMIT_SUBJECTS_SHOWN)
            .map(|s| format!("  - {s}")),
    );
    if commits.len() > COMMIT_SUBJECTS_SHOWN {
        lines.push(format!(
            "  - ... and {} more",
            commits.len() - COMMIT_SUBJECTS_SHOWN
        ));
    }
    section(&mut out, "Anything else", &lines);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridle_api::{ThreadEntry, ThreadEntryKind};
    use chrono::TimeZone;

    fn at(h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 8, h, 0, 0).unwrap()
    }

    fn task(id: &str, kind: TaskKind, state: TaskState, created: DateTime<Utc>) -> Task {
        serde_json::from_value(serde_json::json!({
            "id": id, "title": format!("title {id}"), "kind": kind.as_str(),
            "state": state.as_str(), "body": "", "thread": [],
            "created_at": created, "updated_at": created,
        }))
        .unwrap()
    }

    fn note(t: &mut Task, body: &str, when: DateTime<Utc>) {
        t.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: "x".into(),
            body: body.into(),
            at: when,
        });
    }

    #[test]
    fn empty_sections_say_none() {
        let r = render(&[], &[], &[], at(0), at(12));
        assert_eq!(r.matches("none\n").count(), 5, "{r}");
        assert!(r.contains("tasks created: 0"));
        assert!(r.is_ascii());
    }

    #[test]
    fn window_edges_and_sections() {
        let (from, to) = (at(0), at(12));
        let mut f_in = task("f1", TaskKind::Feature, TaskState::Integrated, at(1));
        f_in.commit = Some("abcdef1234567".into());
        note(&mut f_in, "integrated: abcdef1234567", from); // exactly on the edge: in
        let mut f_old = task("f2", TaskKind::Feature, TaskState::Integrated, at(1));
        note(&mut f_old, "integrated: x", from - Duration::seconds(1)); // just outside
        let bug_new = task("b1", TaskKind::Bug, TaskState::Open, at(2));
        let mut bug_fixed = task(
            "b2",
            TaskKind::Bug,
            TaskState::Integrated,
            from - Duration::days(3),
        );
        note(&mut bug_fixed, "integrated: y", at(5));
        let waiting = task("p1", TaskKind::Chore, TaskState::Planned, at(3));
        let blocker = task("p0", TaskKind::Chore, TaskState::Claimed, at(3));
        let mut inc = task("i1", TaskKind::Incident, TaskState::Integrated, at(4));
        note(&mut inc, "integrated: z", at(6));
        note(&mut inc, "fixed by restart\nmore", at(7));
        let chore = {
            let mut c = task("c1", TaskKind::Chore, TaskState::Integrated, at(1));
            note(&mut c, "integrated: w", at(8));
            c
        };
        let edges = vec![Edge {
            from: "p0".into(),
            to: "p1".into(),
            kind: EdgeKind::Blocks,
            created_at: at(3),
        }];
        let commits: Vec<String> = (0..25).map(|i| format!("commit {i}")).collect();
        let r = render(
            &[
                f_in, f_old, bug_new, bug_fixed, waiting, blocker, inc, chore,
            ],
            &edges,
            &commits,
            from,
            to,
        );
        assert!(r.contains("- f1: title f1 (abcdef123)"), "{r}");
        assert!(!r.contains("f2:"), "{r}");
        assert!(r.contains("- b1: title b1 (open)"));
        assert!(r.contains("Fixed and delivered:\n- b2: title b2"));
        assert!(r.contains("- p1: title p1 (blocked by p0)"));
        assert!(r.contains("- p0: title p0 (claimed, waiting)") || r.contains("p0: title p0"));
        assert!(r.contains("- i1: title i1 [integrated] fixed by restart"));
        assert!(r.contains("other tasks integrated: 1"));
        assert!(r.contains("commits on main: 25"));
        assert!(r.contains("commit 19") && !r.contains("commit 20\n"));
        assert!(r.contains("... and 5 more"));
        assert!(r.is_ascii());
    }

    #[test]
    fn eastern_times_are_bare() {
        // 11:00 UTC in October is 7:00 AM EDT; in January 6:00 AM EST.
        assert_eq!(eastern(at(11)), "Oct 8, 7:00 AM");
        let jan = Utc.with_ymd_and_hms(2026, 1, 8, 11, 0, 0).unwrap();
        assert_eq!(eastern(jan), "Jan 8, 6:00 AM");
    }

    #[test]
    fn window_parses() {
        assert_eq!(parse_window("48h").unwrap(), Duration::hours(48));
        assert_eq!(parse_window("2d").unwrap(), Duration::days(2));
        assert!(parse_window("x").is_err());
        assert!(parse_window("0h").is_err());
    }
}
