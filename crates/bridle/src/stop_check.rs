//! `bridle stop-check`: Claude Code's Stop hook for the worker role
//! (docs/design/coordination.md, docs/spikes/05-stop-hook-findings.md).
//! Refuses to let a worker stop with a claimed task that has no thread entry
//! (note, question or answer) from itself at or after the claim — an
//! unreleased claim with no handoff.
//!
//! Kept as pure functions over already-fetched data so the decision itself
//! is unit-testable without a daemon; `commands.rs` does the stdin parsing
//! and the API calls, and never lets an error here block a stop (the
//! protocol: any bug in bridle's own tooling must allow, not block).

use bridle_api::types::Task;

/// The task among `claimed` (already filtered to the calling principal's own
/// claims), if any, with no thread entry from its own claimant at or after
/// `claimed_at`. Tasks with no `claimed_by`/`claimed_at` are skipped rather
/// than treated as violations — the wire type allows it, but `?claimed_by=me`
/// never returns one.
pub fn first_unacknowledged_claim(claimed: &[Task]) -> Option<&Task> {
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
pub fn first_unreported_finish(claimed: &[Task], finished: bool) -> Option<&Task> {
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
pub fn unreported_reason_for(task: &Task) -> String {
    format!(
        "Your work on {id} ({title}) looks finished (clean tree, commits ahead) but you have not \
         reported it. RUN these commands now with the Bash tool; printing them does nothing: \
         `bridle task summary {id} --text \"<what changed>\"`, then \
         `bridle send <sender> --task {id} \"done: <one-line summary>; <commit sha>\"`.",
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

/// The block reason, phrased as a direct instruction since Claude Code
/// delivers it as an ordinary user-turn message, not a system directive
/// (spike 05, surprise 2).
pub fn reason_for(task: &Task) -> String {
    format!(
        "You have an unreleased claim on {} ({}) with no note since claiming it. \
         Release it with `bridle release {}` or leave a handoff note with \
         `bridle task note {} <text>` before stopping.",
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
    use bridle_api::types::{TaskKind, TaskState, ThreadEntry, ThreadEntryKind};
    use chrono::{Duration, Utc};

    fn task_with(
        claimed_by: Option<&str>,
        claimed_at: Option<chrono::DateTime<Utc>>,
        thread: Vec<ThreadEntry>,
    ) -> Task {
        let now = Utc::now();
        Task {
            components: Vec::new(),
            size: None,
            branch: None,
            commit: None,
            summary: None,
            impact: Default::default(),
            id: "tw-0001".to_string(),
            title: "do the thing".to_string(),
            kind: TaskKind::Chore,
            state: TaskState::Claimed,
            body: String::new(),
            thread,
            created_at: now,
            updated_at: now,
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
