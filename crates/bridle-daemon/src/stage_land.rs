//! Lands a stage writer's branch on the integration branch (br-upxq, epic br-tj7q): the daemon
//! checks that the branch changes only the ticket's file and its stage folder, then merges it into
//! the project checkout. Only after that does the stage count as written, so the human is never
//! asked to review a stage that is not on main. Docs: docs/design/stages.md.

use std::path::Path;

use crate::integrator::checked_out_at;
use crate::worktree::run_git;

const OPEN: &str = "docs/tickets/open";

/// Merges `branch` into `integration`, returning the integration branch's tip. The error text goes
/// back to the writer as the reason `stage-done` failed.
pub async fn land(
    repo: &Path,
    integration: &str,
    branch: &str,
    file: &str,
    stage: &str,
) -> Result<String, String> {
    let git = |e: crate::worktree::WorktreeError| e.to_string();
    if file.contains('/') || !file.ends_with(".md") {
        return Err(format!("{file} is not a ticket file name"));
    }
    if branch == integration {
        return Err(format!(
            "{branch} is the integration branch; nothing to land"
        ));
    }
    let ticket = format!("{OPEN}/{file}");
    let folder = format!("{OPEN}/{}", file.trim_end_matches(".md"));
    let old = run_git(repo, &["rev-parse", &format!("refs/heads/{integration}")])
        .await
        .map_err(git)?
        .trim()
        .to_string();
    let touched = run_git(repo, &["diff", "--name-only", &format!("{old}...{branch}")])
        .await
        .map_err(git)?;
    if let Some(p) = touched
        .lines()
        .find(|p| *p != ticket && !p.starts_with(&format!("{folder}/")))
    {
        return Err(format!(
            "{branch} touches {p}; a stage may change only {ticket} and {folder}/"
        ));
    }
    let Some(dir) = checked_out_at(repo, integration).await.map_err(git)? else {
        return Err(format!("{integration} is not checked out in the project"));
    };
    let dirty = run_git(&dir, &["status", "--porcelain", "--", &ticket, &folder])
        .await
        .map_err(git)?;
    if let Some(l) = dirty.lines().next() {
        return Err(format!(
            "{integration} has uncommitted changes to {}; commit or discard them, then retry",
            l.get(3..).unwrap_or(l)
        ));
    }
    let msg = format!(
        "{}: stage {stage} written ({branch})",
        file.trim_end_matches(".md")
    );
    // `merge.ff=only` in a user's config would refuse a clean merge of a branch that is behind main.
    let merged = run_git(
        &dir,
        &[
            "-c",
            "merge.ff=true",
            "merge",
            "--no-edit",
            "-m",
            &msg,
            branch,
        ],
    )
    .await;
    if let Err(e) = merged {
        let paths = run_git(&dir, &["diff", "--name-only", "--diff-filter=U"])
            .await
            .unwrap_or_default();
        let _ = run_git(&dir, &["merge", "--abort"]).await;
        let paths: Vec<&str> = paths.lines().collect();
        return Err(if paths.is_empty() {
            format!("merging {branch} into {integration} failed: {e}")
        } else {
            format!(
                "{branch} conflicts with {integration} in: {}",
                paths.join(", ")
            )
        });
    }
    let tip = run_git(&dir, &["rev-parse", "--short", "HEAD"])
        .await
        .map_err(git)?;
    Ok(tip.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn g(dir: &Path, args: &[&str]) {
        run_git(dir, args).await.expect("git");
    }

    fn put(dir: &Path, rel: &str, text: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    /// A repo on `main` holding ticket `t-aaaa.md`, and a branch `bridle/d` off it.
    async fn repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path();
        g(d, &["init", "-q", "-b", "main"]).await;
        g(d, &["config", "user.email", "t@t"]).await;
        g(d, &["config", "user.name", "t"]).await;
        put(
            d,
            "docs/tickets/open/t-aaaa.md",
            "---\nid: aaaa\n---\nempty\n",
        );
        put(d, "code.rs", "a\n");
        g(d, &["add", "."]).await;
        g(d, &["commit", "-q", "-m", "init"]).await;
        g(d, &["checkout", "-q", "-b", "bridle/d"]).await;
        tmp
    }

    async fn commit_on_branch(d: &Path, rel: &str, text: &str) {
        put(d, rel, text);
        g(d, &["add", "."]).await;
        g(d, &["commit", "-q", "-m", "work"]).await;
        g(d, &["checkout", "-q", "main"]).await;
    }

    #[tokio::test]
    async fn a_stage_branch_lands_and_is_readable_on_main() {
        let tmp = repo().await;
        let d = tmp.path();
        commit_on_branch(d, "docs/tickets/open/t-aaaa/design.md", "the design\n").await;
        let tip = land(d, "main", "bridle/d", "t-aaaa.md", "design")
            .await
            .unwrap();
        let on_main = run_git(d, &["rev-parse", "--short", "main"]).await.unwrap();
        assert_eq!(tip, on_main.trim());
        assert!(d.join("docs/tickets/open/t-aaaa/design.md").exists());
    }

    /// Makes main move on past the branch, so landing needs a real merge, not a fast-forward.
    async fn move_main(d: &Path) {
        put(d, "other.rs", "c\n");
        g(d, &["add", "."]).await;
        g(d, &["commit", "-q", "-m", "main moves"]).await;
    }

    #[tokio::test]
    async fn a_clean_merge_of_a_branch_behind_main_lands_under_merge_ff_only() {
        let tmp = repo().await;
        let d = tmp.path();
        commit_on_branch(d, "docs/tickets/open/t-aaaa/design.md", "the design\n").await;
        move_main(d).await;
        g(d, &["config", "merge.ff", "only"]).await;
        let tip = land(d, "main", "bridle/d", "t-aaaa.md", "design")
            .await
            .unwrap();
        let on_main = run_git(d, &["rev-parse", "--short", "main"]).await.unwrap();
        assert_eq!(tip, on_main.trim());
        assert!(d.join("docs/tickets/open/t-aaaa/design.md").exists());
        assert!(d.join("other.rs").exists());
    }

    #[tokio::test]
    async fn a_conflict_under_merge_ff_only_still_fails_back_to_the_writer() {
        let tmp = repo().await;
        let d = tmp.path();
        commit_on_branch(
            d,
            "docs/tickets/open/t-aaaa.md",
            "---\nid: aaaa\n---\nbranch\n",
        )
        .await;
        put(
            d,
            "docs/tickets/open/t-aaaa.md",
            "---\nid: aaaa\n---\nmain\n",
        );
        g(d, &["commit", "-q", "-am", "main edit"]).await;
        g(d, &["config", "merge.ff", "only"]).await;
        let e = land(d, "main", "bridle/d", "t-aaaa.md", "proposal")
            .await
            .unwrap_err();
        assert!(e.contains("conflicts") && e.contains("t-aaaa.md"), "{e}");
        let st = run_git(d, &["status", "--porcelain"]).await.unwrap();
        assert!(st.trim().is_empty(), "{st}");
    }

    #[tokio::test]
    async fn a_file_outside_the_ticket_is_refused_naming_the_path() {
        let tmp = repo().await;
        let d = tmp.path();
        commit_on_branch(d, "code.rs", "b\n").await;
        let e = land(d, "main", "bridle/d", "t-aaaa.md", "design")
            .await
            .unwrap_err();
        assert!(e.contains("code.rs"), "{e}");
        assert_eq!(std::fs::read_to_string(d.join("code.rs")).unwrap(), "a\n");
    }

    #[tokio::test]
    async fn a_conflict_is_refused_and_leaves_main_clean() {
        let tmp = repo().await;
        let d = tmp.path();
        commit_on_branch(
            d,
            "docs/tickets/open/t-aaaa.md",
            "---\nid: aaaa\n---\nbranch\n",
        )
        .await;
        put(
            d,
            "docs/tickets/open/t-aaaa.md",
            "---\nid: aaaa\n---\nmain\n",
        );
        g(d, &["commit", "-q", "-am", "main edit"]).await;
        let e = land(d, "main", "bridle/d", "t-aaaa.md", "proposal")
            .await
            .unwrap_err();
        assert!(e.contains("conflicts") && e.contains("t-aaaa.md"), "{e}");
        let st = run_git(d, &["status", "--porcelain"]).await.unwrap();
        assert!(st.trim().is_empty(), "{st}");
    }

    #[tokio::test]
    async fn uncommitted_edits_to_the_ticket_on_main_are_refused() {
        let tmp = repo().await;
        let d = tmp.path();
        commit_on_branch(d, "docs/tickets/open/t-aaaa/design.md", "x\n").await;
        put(
            d,
            "docs/tickets/open/t-aaaa.md",
            "---\nid: aaaa\n---\nedited\n",
        );
        let e = land(d, "main", "bridle/d", "t-aaaa.md", "design")
            .await
            .unwrap_err();
        assert!(e.contains("uncommitted") && e.contains("t-aaaa.md"), "{e}");
    }
}
