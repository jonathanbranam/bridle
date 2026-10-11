//! The text of a handover request (ft3b): what the session is asked to put in its note.
//! Layers, the way role docs do: `<workflow>/base/handover/standard.md`, then the role's part
//! `<workflow>/base/handover/<role>.md` appended; a project's `.bridle/handover/<role>.md`
//! replaces both. With no workflow root the built-in standard text stands in.

use std::path::Path;

const STANDARD: &str = include_str!("../../../workflow/base/handover/standard.md");

/// The role a session identity plays: `advisor/notes` is an advisor.
pub fn role_of(identity: &str) -> &str {
    identity.split('/').next().unwrap_or(identity)
}

/// The handover instructions for `role` in the project at `repo`. `workflow_root` is the
/// resolved workflow directory (see `Config::workflow_root`), if the project has one.
pub fn instructions(repo: &Path, workflow_root: Option<&Path>, role: &str) -> String {
    let read = |p: std::path::PathBuf| std::fs::read_to_string(p).ok();
    if let Some(text) = read(repo.join(".bridle/handover").join(format!("{role}.md"))) {
        return text.trim().to_string();
    }
    let base = workflow_root.map(|r| r.join("base/handover"));
    let standard = base
        .as_ref()
        .and_then(|b| read(b.join("standard.md")))
        .unwrap_or_else(|| STANDARD.to_string());
    let mut out = standard.trim().to_string();
    if let Some(part) = base.and_then(|b| read(b.join(format!("{role}.md")))) {
        out.push_str("\n\n");
        out.push_str(part.trim());
    }
    out
}

/// [`instructions`] for the project in `repo`, finding its workflow root from its config; the
/// built-in standard text when the config can't be loaded (a handover never fails over this).
pub fn instructions_for_repo(repo: &Path, role: &str) -> String {
    let root = crate::config::Config::load(repo)
        .ok()
        .and_then(|c| c.workflow_root(repo).ok().flatten());
    instructions(repo, root.as_deref(), role)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tempdir");
        let h = d.path().join("wf/base/handover");
        std::fs::create_dir_all(&h).expect("dir");
        std::fs::write(h.join("standard.md"), "STANDARD\n").expect("w");
        std::fs::write(h.join("advisor.md"), "ADVISOR PART\n").expect("w");
        d
    }

    #[test]
    fn role_without_part_gets_the_standard_text() {
        let d = repo();
        let root = d.path().join("wf");
        assert_eq!(instructions(d.path(), Some(&root), "worker"), "STANDARD");
    }

    #[test]
    fn role_part_extends_the_standard_text() {
        let d = repo();
        let root = d.path().join("wf");
        assert_eq!(
            instructions(d.path(), Some(&root), "advisor"),
            "STANDARD\n\nADVISOR PART"
        );
    }

    #[test]
    fn project_file_replaces_it() {
        let d = repo();
        let root = d.path().join("wf");
        std::fs::create_dir_all(d.path().join(".bridle/handover")).expect("dir");
        std::fs::write(d.path().join(".bridle/handover/advisor.md"), "MINE\n").expect("w");
        assert_eq!(instructions(d.path(), Some(&root), "advisor"), "MINE");
        assert_eq!(instructions(d.path(), Some(&root), "worker"), "STANDARD");
    }

    #[test]
    fn no_workflow_uses_the_built_in_text() {
        let d = tempfile::tempdir().expect("tempdir");
        assert!(instructions(d.path(), None, "aide").contains("cannot be summarised"));
        assert_eq!(role_of("advisor/notes"), "advisor");
    }
}
