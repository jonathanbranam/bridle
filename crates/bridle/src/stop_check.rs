//! `bridle stop-check`: Claude Code's Stop hook for the worker role
//! (docs/design/coordination.md, docs/spikes/05-stop-hook-findings.md).
//! Refuses to let a worker stop with a claimed task that has no thread entry
//! (note, question or answer) from itself at or after the claim — an
//! unreleased claim with no handoff.
//!
//! Kept as pure functions over already-fetched data so the decision itself
//! is unit-testable without a daemon; `commands/hook.rs` does the stdin parsing
//! and the API calls, and never lets an error here block a stop (the
//! protocol: any bug in bridle's own tooling must allow, not block).

use bridle_api::types::Ticket;

/// The task among `claimed` (already filtered to the calling principal's own
/// claims), if any, with no thread entry from its own claimant at or after
/// `claimed_at`. Tasks with no `claimed_by`/`claimed_at` are skipped rather
/// than treated as violations — the wire type allows it, but `?claimed_by=me`
/// never returns one.
pub fn first_unacknowledged_claim(claimed: &[Ticket]) -> Option<&Ticket> {
    claimed.iter().find(|task| {
        let (Some(claimed_by), Some(claimed_at)) = (task.claimed_by.as_deref(), task.claimed_at)
        else {
            return false;
        };
        !task
            .thread
            .iter()
            .any(|entry| entry.from == claimed_by && entry.at >= claimed_at)
    })
}

/// The task among `claimed` that looks finished but never reported: the
/// tree is clean with commits ahead of the integration branch (`finished`),
/// and the task has no summary or no thread entry from its claimant
/// starting `done:`. Overnight, workers printed their `bridle send done`
/// instead of running it and sat idle.
pub fn first_unreported_finish(claimed: &[Ticket], finished: bool) -> Option<&Ticket> {
    if !finished {
        return None;
    }
    claimed.iter().find(|task| {
        let Some(claimed_by) = task.claimed_by.as_deref() else {
            return false;
        };
        let reported = task
            .thread
            .iter()
            .any(|e| e.from == claimed_by && e.body.trim_start().starts_with("done:"));
        task.summary.as_deref().is_none_or(|s| s.trim().is_empty()) || !reported
    })
}

/// Direct instruction for a finished worker with no summary or report.
pub fn unreported_reason_for(task: &Ticket) -> String {
    format!(
        "Your work on {id} ({title}) looks finished (clean tree, commits ahead) but you have not \
         reported it. RUN these commands now with the Bash tool; printing them does nothing: \
         `bridle task summary {id} --text \"<what changed>\"`, then \
         `bridle send <manager> --task {id} \"done: <one-line summary>; <commit sha>\"`, \
         where <manager> is your spawner or the manager agent (`bridle agent list --json`), never `human`.",
        id = task.id,
        title = task.title
    )
}

/// Whether the worktree at `dir` looks finished: clean, with commits ahead
/// of the local integration branch (`main`, else `master`). Any git failure
/// is "not finished", so the hook falls back to its older behaviour.
pub fn looks_finished(dir: &std::path::Path) -> bool {
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    if git(&["status", "--porcelain"]).is_none_or(|s| !s.is_empty()) {
        return false;
    }
    ["main", "master"].iter().any(|base| {
        git(&["rev-list", "--count", &format!("{base}..HEAD")])
            .and_then(|n| n.parse::<u64>().ok())
            .is_some_and(|n| n > 0)
    })
}

/// Lines of failing check output quoted in the block reason.
const OUTPUT_TAIL_LINES: usize = 40;

/// The file (in the worktree's own git dir, so never committed) recording
/// the tree id the worker's check last passed on. A tree, not a HEAD sha: the
/// worker's own green run happens before its commit, and the commit changes
/// HEAD but not the tree. The justfile's `record-pass` writes it from the
/// working tree; this hook writes it after its own passing run.
fn pass_record(dir: &std::path::Path) -> Option<(std::path::PathBuf, String)> {
    let git = |arg: &str| {
        std::process::Command::new("git")
            .args(["rev-parse", arg])
            .current_dir(dir)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let git_dir = std::path::PathBuf::from(git("--absolute-git-dir")?);
    Some((git_dir.join("bridle-check-passed"), git("HEAD^{tree}")?))
}

/// Marker (in the git dir) present while this hook runs the check, so the
/// daemon's stall detector doesn't count the silent wait as a stall.
pub const RUNNING_MARKER: &str = "bridle-stop-check-running";

/// The block reason if the project's `check_worker` `command` has not passed
/// on the tree at the worktree's current HEAD; `None` (allow) when it has, when none is
/// bound, or when git can't say what HEAD is. A missing pass record runs the
/// check here and records HEAD on success, so a later stop at the same HEAD
/// skips the rerun. Blocks with the tail of the output on failure.
pub fn unchecked_head_reason(dir: &std::path::Path, command: Option<&str>) -> Option<String> {
    let command = command?;
    let (record, tree) = pass_record(dir)?;
    if std::fs::read_to_string(&record).is_ok_and(|s| s.trim() == tree) {
        return None;
    }
    let marker = record.with_file_name(RUNNING_MARKER);
    let _ = std::fs::write(&marker, "");
    let output = std::process::Command::new("sh")
        .args(["-c", command])
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .output();
    let _ = std::fs::remove_file(&marker);
    let output = output.ok()?;
    if output.status.success() {
        let _ = std::fs::write(&record, &tree);
        return None;
    }
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    let lines: Vec<&str> = text.lines().collect();
    let tail = lines[lines.len().saturating_sub(OUTPUT_TAIL_LINES)..].join("\n");
    Some(format!(
        "`{command}` failed on the tree at HEAD. Fix it, commit, and run it green before \
         stopping. Last output:\n{tail}"
    ))
}

/// The block reason, phrased as a direct instruction since Claude Code
/// delivers it as an ordinary user-turn message, not a system directive
/// (spike 05, surprise 2).
pub fn reason_for(task: &Ticket) -> String {
    format!(
        "You have an unreleased claim on {} ({}) with no note since claiming it. \
         Release it with `bridle release {}` or leave a handoff note with \
         `bridle task comment {} <text>` before stopping.",
        task.id, task.title, task.id, task.id
    )
}

/// The flat `{"decision":"block","reason":"..."}` shape spike 05 confirmed
/// works for the Stop event; the `hookSpecificOutput` wrapper is silently
/// ignored for it.
pub fn block_json(reason: &str) -> String {
    serde_json::json!({ "decision": "block", "reason": reason }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridle_api::types::{ThreadEntry, ThreadEntryKind, TicketKind, TicketState};
    use chrono::{Duration, Utc};

    fn task_with(
        claimed_by: Option<&str>,
        claimed_at: Option<chrono::DateTime<Utc>>,
        thread: Vec<ThreadEntry>,
    ) -> Ticket {
        let now = Utc::now();
        Ticket {
            components: Vec::new(),
            size: None,
            priority: Default::default(),
            priority_at: None,
            branch: None,
            commit: None,
            summary: None,
            ticket: None,
            parent: None,
            impact: Default::default(),
            settle_until: None,
            held_by: None,
            held_reason: None,
            held_until: None,
            held_at: None,
            id: "tw-0001".to_string(),
            title: "do the thing".to_string(),
            kind: TicketKind::Chore,
            state: TicketState::Claimed,
            body: String::new(),
            thread,
            created_at: now,
            updated_at: now,
            created_by: "human".to_string(),
            watchers: vec![],
            claimed_by: claimed_by.map(String::from),
            claimed_at,
        }
    }

    fn entry(from: &str, at: chrono::DateTime<Utc>) -> ThreadEntry {
        ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: from.to_string(),
            body: "fyi".to_string(),
            at,
            for_human: false,
        }
    }

    #[test]
    fn no_claims_is_fine() {
        assert!(first_unacknowledged_claim(&[]).is_none());
    }

    #[test]
    fn claim_with_a_note_after_claiming_is_fine() {
        let claimed_at = Utc::now() - Duration::minutes(5);
        let task = task_with(
            Some("agent:w1"),
            Some(claimed_at),
            vec![entry("agent:w1", claimed_at + Duration::minutes(1))],
        );
        assert!(first_unacknowledged_claim(&[task]).is_none());
    }

    #[test]
    fn claim_with_no_thread_entry_is_blocked() {
        let claimed_at = Utc::now();
        let task = task_with(Some("agent:w1"), Some(claimed_at), vec![]);
        let blocked = first_unacknowledged_claim(std::slice::from_ref(&task));
        assert_eq!(blocked.map(|t| t.id.as_str()), Some("tw-0001"));
    }

    #[test]
    fn claim_with_only_a_note_before_claiming_is_blocked() {
        let claimed_at = Utc::now();
        let task = task_with(
            Some("agent:w1"),
            Some(claimed_at),
            vec![entry("agent:w1", claimed_at - Duration::minutes(1))],
        );
        assert!(first_unacknowledged_claim(std::slice::from_ref(&task)).is_some());
    }

    #[test]
    fn note_from_someone_else_does_not_count() {
        let claimed_at = Utc::now();
        let task = task_with(
            Some("agent:w1"),
            Some(claimed_at),
            vec![entry("human", claimed_at + Duration::minutes(1))],
        );
        assert!(first_unacknowledged_claim(std::slice::from_ref(&task)).is_some());
    }

    #[test]
    fn task_missing_claim_metadata_is_skipped_not_blocked() {
        let task = task_with(None, None, vec![]);
        assert!(first_unacknowledged_claim(&[task]).is_none());
    }

    #[test]
    fn reports_the_first_violation_by_position() {
        let claimed_at = Utc::now();
        let ok = task_with(
            Some("agent:w1"),
            Some(claimed_at),
            vec![entry("agent:w1", claimed_at)],
        );
        let mut bad = task_with(Some("agent:w1"), Some(claimed_at), vec![]);
        bad.id = "tw-0002".to_string();
        let tasks = [ok, bad];
        let blocked = first_unacknowledged_claim(&tasks);
        assert_eq!(blocked.map(|t| t.id.as_str()), Some("tw-0002"));
    }

    fn done_entry(from: &str) -> ThreadEntry {
        ThreadEntry {
            body: "done: all good; abc123".to_string(),
            ..entry(from, Utc::now())
        }
    }

    #[test]
    fn finished_without_report_is_blocked() {
        let mut task = task_with(Some("agent:w1"), Some(Utc::now()), vec![]);
        assert!(first_unreported_finish(std::slice::from_ref(&task), true).is_some());
        task.summary = Some("did it".into());
        assert!(first_unreported_finish(std::slice::from_ref(&task), true).is_some());
        assert!(unreported_reason_for(&task).contains("printing them does nothing"));
        let reason = unreported_reason_for(&task);
        assert!(reason.contains("bridle send <manager>"));
        assert!(!reason.contains("<sender>"));
        assert!(reason.contains("never `human`"));
    }

    #[test]
    fn finished_and_reported_is_allowed() {
        let mut task = task_with(
            Some("agent:w1"),
            Some(Utc::now()),
            vec![done_entry("agent:w1")],
        );
        task.summary = Some("did it".into());
        assert!(first_unreported_finish(&[task], true).is_none());
    }

    #[test]
    fn unfinished_tree_is_left_to_the_claim_check() {
        let task = task_with(Some("agent:w1"), Some(Utc::now()), vec![]);
        assert!(first_unreported_finish(&[task], false).is_none());
    }

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for args in [
            &["init", "-q"][..],
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "x",
            ],
        ] {
            let ok = std::process::Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .status()
                .expect("git")
                .success();
            assert!(ok);
        }
        dir
    }

    #[test]
    fn no_binding_allows() {
        let dir = repo();
        assert!(unchecked_head_reason(dir.path(), None).is_none());
    }

    #[test]
    fn no_record_runs_the_check_and_records_a_pass() {
        let dir = repo();
        assert!(unchecked_head_reason(dir.path(), Some("touch ran")).is_none());
        assert!(dir.path().join("ran").exists());
        assert!(dir.path().join(".git/bridle-check-passed").exists());
    }

    #[test]
    fn recorded_pass_at_head_skips_the_check() {
        let dir = repo();
        assert!(unchecked_head_reason(dir.path(), Some("true")).is_none());
        // Would fail if it ran again.
        assert!(unchecked_head_reason(dir.path(), Some("false")).is_none());
    }

    fn commit_file(dir: &std::path::Path, name: &str) {
        std::fs::write(dir.join(name), name).expect("write");
        for args in [
            &["add", "-A"][..],
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-q",
                "-m",
                "y",
            ],
        ] {
            let ok = std::process::Command::new("git")
                .args(args)
                .current_dir(dir)
                .status()
                .expect("git")
                .success();
            assert!(ok);
        }
    }

    /// What the justfile's `record-pass` writes: the working tree's id.
    fn record_working_tree(dir: &std::path::Path) {
        let tmp = dir.join(".git/tmp-index");
        std::fs::copy(dir.join(".git/index"), &tmp).expect("copy index");
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .env("GIT_INDEX_FILE", &tmp)
                .current_dir(dir)
                .output()
                .expect("git")
        };
        run(&["add", "-A"]);
        let out = run(&["write-tree"]);
        std::fs::write(dir.join(".git/bridle-check-passed"), out.stdout).expect("record");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn recorded_tree_skips_the_check_after_committing_it() {
        let dir = repo();
        std::fs::write(dir.path().join("f"), "f").expect("write");
        record_working_tree(dir.path());
        commit_file(dir.path(), "f");
        assert!(unchecked_head_reason(dir.path(), Some("false")).is_none());
    }

    #[test]
    fn tree_changed_after_the_green_run_runs_the_check() {
        let dir = repo();
        std::fs::write(dir.path().join("f"), "f").expect("write");
        record_working_tree(dir.path());
        commit_file(dir.path(), "g");
        assert!(unchecked_head_reason(dir.path(), Some("false")).is_some());
    }

    #[test]
    fn marker_is_gone_after_the_check_runs() {
        let dir = repo();
        let _ = unchecked_head_reason(dir.path(), Some("true"));
        assert!(!dir.path().join(".git").join(RUNNING_MARKER).exists());
    }

    #[test]
    fn failing_check_blocks_with_the_output_tail_and_records_nothing() {
        let dir = repo();
        let reason =
            unchecked_head_reason(dir.path(), Some("echo fmt is red; exit 1")).expect("blocked");
        assert!(reason.contains("fmt is red"));
        assert!(!dir.path().join(".git/bridle-check-passed").exists());
    }

    #[test]
    fn block_json_uses_the_flat_shape() {
        let json = block_json("hello");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["decision"], "block");
        assert_eq!(value["reason"], "hello");
        assert!(value.get("hookSpecificOutput").is_none());
    }

    #[test]
    fn reason_names_the_task_id_and_title() {
        let task = task_with(Some("agent:w1"), Some(Utc::now()), vec![]);
        let reason = reason_for(&task);
        assert!(reason.contains("tw-0001"));
        assert!(reason.contains("do the thing"));
    }
}
