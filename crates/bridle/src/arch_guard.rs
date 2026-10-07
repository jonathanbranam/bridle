//! `bridle arch-guard`: Claude Code's PreToolUse hook that keeps workers out
//! of `design/architecture/` unless their claimed task is an `arch-revision`
//! (docs/design/architecture-tier.md). Pure decision functions over
//! already-fetched data, like `stop_check`; `commands/hook.rs` does the stdin
//! parsing and API calls, and allows on any error of bridle's own.

use std::path::{Component, Path, PathBuf};

use bridle_api::types::{Task, TaskKind};

/// Tools that write a file; the hook's matcher only names the first three,
/// but `NotebookEdit` is handled here too in case a project widens it.
const EDIT_TOOLS: &[&str] = &["Edit", "Write", "MultiEdit", "NotebookEdit"];

/// The path a hook input's tool call would write, if it is an edit tool.
pub fn edited_path(input: &serde_json::Value) -> Option<String> {
    let tool = input.get("tool_name")?.as_str()?;
    if !EDIT_TOOLS.contains(&tool) {
        return None;
    }
    let tool_input = input.get("tool_input")?;
    ["file_path", "notebook_path"]
        .iter()
        .find_map(|k| tool_input.get(k)?.as_str())
        .map(String::from)
}

/// Resolves `.` and `..` lexically (the file may not exist yet, so no
/// canonicalize), against `cwd` when relative.
fn normalize(path: &str, cwd: Option<&str>) -> PathBuf {
    let path = Path::new(path);
    let joined = match (path.is_absolute(), cwd) {
        (false, Some(cwd)) => Path::new(cwd).join(path),
        _ => path.to_path_buf(),
    };
    let mut out = PathBuf::new();
    for c in joined.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// True when `path` (relative to `cwd`, if given) lies under a
/// `design/architecture` directory.
pub fn is_architecture_path(path: &str, cwd: Option<&str>) -> bool {
    let normalized = normalize(path, cwd);
    let comps: Vec<_> = normalized.components().collect();
    comps
        .windows(2)
        .any(|w| w[0].as_os_str() == "design" && w[1].as_os_str() == "architecture")
}

/// Whether the edit must be denied: it targets the architecture directory and
/// none of the caller's own claimed tasks is an `arch-revision`.
pub fn should_deny(path: &str, cwd: Option<&str>, claimed: &[Task]) -> bool {
    is_architecture_path(path, cwd) && !claimed.iter().any(|t| t.kind == TaskKind::ArchRevision)
}

pub fn deny_reason(path: &str) -> String {
    format!(
        "{path} is under design/architecture/, which only an arch-revision task may edit. \
         If you think the architecture is wrong, run `bridle arch propose --title <t> \
         --argument <text>`, then continue within the current architecture or block your \
         task on the revision."
    )
}

/// The PreToolUse deny shape (`hookSpecificOutput`, unlike the Stop hook's
/// flat `decision`/`reason`).
pub fn deny_json(reason: &str) -> String {
    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridle_api::types::TaskState;
    use chrono::Utc;

    fn task(kind: TaskKind) -> Task {
        let now = Utc::now();
        Task {
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
            id: "tw-0001".to_string(),
            title: "t".to_string(),
            kind,
            state: TaskState::Claimed,
            body: String::new(),
            thread: Vec::new(),
            created_at: now,
            updated_at: now,
            created_by: "human".to_string(),
            watchers: vec![],
            claimed_by: Some("agent:w1".to_string()),
            claimed_at: Some(now),
        }
    }

    const CWD: &str = "/work/wt/w1";

    #[test]
    fn paths_under_architecture_are_detected() {
        assert!(is_architecture_path(
            "/work/wt/w1/design/architecture/a.md",
            None
        ));
        assert!(is_architecture_path("design/architecture/a.md", Some(CWD)));
        assert!(is_architecture_path(
            "./design/./architecture/a.md",
            Some(CWD)
        ));
    }

    #[test]
    fn dotdot_tricks_are_resolved() {
        assert!(is_architecture_path(
            "design/specs/../architecture/a.md",
            Some(CWD)
        ));
        assert!(is_architecture_path(
            "../w1/design/architecture/a.md",
            Some(CWD)
        ));
        assert!(!is_architecture_path(
            "design/architecture/../specs/a.md",
            Some(CWD)
        ));
    }

    #[test]
    fn other_paths_are_not_architecture() {
        assert!(!is_architecture_path("design/specs/a.md", Some(CWD)));
        assert!(!is_architecture_path(
            "docs/design/architecture-tier.md",
            Some(CWD)
        ));
        assert!(!is_architecture_path(
            "src/architecture/design.rs",
            Some(CWD)
        ));
    }

    #[test]
    fn deny_unless_claimed_task_is_arch_revision() {
        let p = "design/architecture/a.md";
        assert!(should_deny(p, Some(CWD), &[]));
        assert!(should_deny(p, Some(CWD), &[task(TaskKind::Feature)]));
        assert!(!should_deny(
            p,
            Some(CWD),
            &[task(TaskKind::Feature), task(TaskKind::ArchRevision)]
        ));
        assert!(!should_deny("src/a.rs", Some(CWD), &[]));
    }

    #[test]
    fn edited_path_reads_edit_tools_only() {
        let edit = serde_json::json!({"tool_name":"Edit","tool_input":{"file_path":"a"}});
        assert_eq!(edited_path(&edit).as_deref(), Some("a"));
        let nb = serde_json::json!({"tool_name":"NotebookEdit","tool_input":{"notebook_path":"n"}});
        assert_eq!(edited_path(&nb).as_deref(), Some("n"));
        let bash = serde_json::json!({"tool_name":"Bash","tool_input":{"command":"ls"}});
        assert_eq!(edited_path(&bash), None);
    }

    #[test]
    fn deny_json_uses_the_pretooluse_shape() {
        let v: serde_json::Value = serde_json::from_str(&deny_json("no")).expect("json");
        assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
        assert_eq!(v["hookSpecificOutput"]["permissionDecisionReason"], "no");
    }

    #[test]
    fn reason_points_at_arch_propose() {
        assert!(deny_reason("x").contains("bridle arch propose"));
    }
}
