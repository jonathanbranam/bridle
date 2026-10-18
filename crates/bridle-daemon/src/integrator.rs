//! `bridle land`: squash a task's branch into the integration branch in a dedicated worktree,
//! run the project's check there, and only then move the integration branch's ref
//! (docs/design/agent-host/roles-and-config.md, "The integrator"). Never pushes.

use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::worktree::{self, MergeProbe, WorktreeError, run_git};

/// Where in the branch's diff an `arch-revision` is required.
const ARCH_PREFIX: &str = "design/architecture/";

/// Lines of check output reported on failure.
const TAIL_LINES: usize = 40;

/// File in the workspace root holding the test count of the last passing full-suite land; workers
/// read it (`$BRIDLE_WORKSPACE/last-full-test-count`) to sanity-check their own run's count.
pub const COUNT_FILE: &str = "last-full-test-count";

/// A sniff check, not an exact match: a count under `1/BAND` or over `BAND` times the last
/// full-suite count is a failure to look into.
const BAND: u64 = 2;

#[derive(Debug, thiserror::Error)]
pub enum LandError {
    #[error(
        "check ran {count} tests, outside the sane band for the last full run of {last:?}; look into it"
    )]
    CountOutOfBand { count: u64, last: Option<u64> },
    #[error("{0}")]
    Refused(String),
    #[error("merge conflict in: {}", .0.join(", "))]
    Conflict(Vec<String>),
    #[error("check failed ({cmd}); last output:\n{tail}")]
    CheckFailed { cmd: String, tail: String },
    #[error("{0} moved, retry")]
    Moved(String),
    #[error(transparent)]
    Git(#[from] WorktreeError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct LandInput<'a> {
    pub repo: &'a Path,
    /// `<workspace>/integration`.
    pub dir: PathBuf,
    pub integration: &'a str,
    pub branch: &'a str,
    pub task: &'a str,
    pub title: &'a str,
    pub summary: Option<&'a str>,
    pub is_arch_revision: bool,
    /// The ticket's kind needs no CHANGELOG line (question, research, explore, incident).
    pub changelog_exempt: bool,
    pub check: Option<&'a str>,
    /// The commit the worker reported a green check on.
    pub checked_commit: Option<&'a str>,
    /// `[integration] check_skip_paths` globs.
    pub check_skip_paths: &'a [String],
    /// `[worktrees] setup` and timeout: run in the check worktree before the check, when one runs.
    pub setup: Option<(&'a str, std::time::Duration)>,
    /// Receives the check's process group while it runs (br-eggc), so the daemon can stop it.
    pub check_pgid: Option<std::sync::Arc<std::sync::atomic::AtomicU32>>,
}

pub struct Landed {
    /// The squash commit the integration branch now points at.
    pub commit: String,
    pub notes: Vec<String>,
}

/// The caller holds the daemon's one-landing-at-a-time lock.
pub async fn land(i: &LandInput<'_>) -> Result<Landed, LandError> {
    let mut notes = Vec::new();
    let old = run_git(
        i.repo,
        &["rev-parse", &format!("refs/heads/{}", i.integration)],
    )
    .await?
    .trim()
    .to_string();

    if !i.is_arch_revision {
        let touched = run_git(
            i.repo,
            &["diff", "--name-only", &format!("{old}...{}", i.branch)],
        )
        .await?;
        if let Some(p) = touched.lines().find(|p| p.starts_with(ARCH_PREFIX)) {
            return Err(LandError::Refused(format!(
                "{} touches {p}; only an arch-revision task may change design/architecture/**",
                i.branch
            )));
        }
    }

    match worktree::merge_probe(i.repo, &old, i.branch).await? {
        MergeProbe::Conflicts(paths) => return Err(LandError::Conflict(paths)),
        MergeProbe::Clean => {}
        MergeProbe::Unsupported => notes.push("git < 2.38: skipped the merge probe".to_string()),
    }

    let wt = prepare_worktree(i, &old).await?;
    // One single-parent commit per task; the `Branch:` trailer is how `is_merged` recognises it.
    let msg = commit_message(i);
    // `merge.ff=only` in a user's config would make a squash of a moved base fail.
    let squashed = match run_git(&wt, &["-c", "merge.ff=true", "merge", "--squash", i.branch]).await
    {
        Ok(_) => run_git(&wt, &["commit", "-q", "-m", &msg]).await.map(drop),
        Err(e) => Err(e),
    };
    if let Err(e) = squashed {
        let paths = run_git(&wt, &["diff", "--name-only", "--diff-filter=U"])
            .await
            .unwrap_or_default();
        // A squash leaves no MERGE_HEAD, so `merge --abort` can't undo it.
        let _ = run_git(&wt, &["reset", "-q", "--hard", &old]).await;
        let paths: Vec<String> = paths.lines().map(str::to_string).collect();
        return Err(if paths.is_empty() {
            LandError::Git(e)
        } else {
            LandError::Conflict(paths)
        });
    }

    if let Some(msg) = unticked_tasks(&wt, i.task) {
        let _ = run_git(&wt, &["reset", "-q", "--hard", &old]).await;
        return Err(LandError::Refused(msg));
    }

    if let Some(msg) = changelog_missing(&wt, i.changelog_exempt, &old).await {
        let _ = run_git(&wt, &["reset", "-q", "--hard", &old]).await;
        return Err(LandError::Refused(msg));
    }

    // When the integration branch is an ancestor of the task tip, the squash tree equals the tip
    // tree, so the worker's green check covers it if it was on this very tip. A moved base means
    // a real merge. A git error here reads as "not an ancestor", so the check runs.
    let mut count = None;
    let fast_forward = run_git(i.repo, &["merge-base", "--is-ancestor", &old, i.branch])
        .await
        .is_ok();
    let tip = run_git(i.repo, &["rev-parse", &format!("{}^{{commit}}", i.branch)])
        .await
        .map(|t| t.trim().to_string())
        .unwrap_or_default();
    let reported = i.checked_commit.map(str::trim).filter(|c| !c.is_empty());
    // A short sha from the worker's report is fine, but not an empty or unrelated prefix.
    let tip_checked = reported.is_some_and(|c| !tip.is_empty() && tip.starts_with(c));
    match i.check {
        Some(_) if fast_forward && tip_checked => {
            notes.push(
                "check skipped: fast-forward of an unchanged base, tip is the commit the worker \
                 reported a green check on"
                    .to_string(),
            );
        }
        Some(cmd) => {
            if let Some((setup, timeout)) = i.setup {
                worktree::run_setup(&wt, setup, timeout).await?;
            }
            count = run_check(&wt, cmd, i.check_pgid.as_deref()).await?;
            if let Some(n) = count {
                band_check(n, read_count(i))?;
            }
            notes.push(if !fast_forward {
                "check ran: the integration branch had moved past the task's base".to_string()
            } else if reported.is_none() {
                "check ran: no checked commit reported (--checked-commit)".to_string()
            } else {
                "check ran: the branch tip is not the commit the worker reported checked"
                    .to_string()
            });
        }
        None => notes.push("no [integration] check configured: skipped".to_string()),
    }

    let mut new = run_git(&wt, &["rev-parse", "HEAD"])
        .await?
        .trim()
        .to_string();
    let mut old = old;
    if let Some(moved_to) = skippable_move(i, &old).await? {
        // Only paths the check can't see changed under us: replay the squash on top of them.
        let replay = match run_git(&wt, &["reset", "-q", "--hard", &moved_to]).await {
            Ok(_) => run_git(&wt, &["cherry-pick", &new]).await.map(drop),
            Err(e) => Err(e),
        };
        if replay.is_err() {
            let _ = run_git(&wt, &["cherry-pick", "--abort"]).await;
            return Err(LandError::Moved(i.integration.to_string()));
        }
        new = run_git(&wt, &["rev-parse", "HEAD"])
            .await?
            .trim()
            .to_string();
        old = moved_to;
        notes.push(
            "integration branch moved during the check by commits touching only \
             [integration] check_skip_paths: merged them in without re-running it"
                .to_string(),
        );
    }
    advance(i, &old, &new).await?;
    if let Some(n) = count {
        // Best effort: a missed write only leaves the band looser.
        let _ = std::fs::write(count_path(i), format!("{n}\n"));
    }
    Ok(Landed { commit: new, notes })
}

/// Every landed task adds a line under `## Unreleased` in CHANGELOG.md, written on its branch
/// (br-ubjd). `Some(reason)` refuses the landing. Exempt: `exempt` (the ticket's kind), and a
/// squash that touches nothing outside `docs/tickets/`.
async fn changelog_missing(wt: &Path, exempt: bool, old: &str) -> Option<String> {
    if exempt {
        return None;
    }
    let names = run_git(wt, &["diff", "--name-only", old, "HEAD"])
        .await
        .ok()?;
    if names.lines().all(|p| p.starts_with("docs/tickets/")) {
        return None;
    }
    let diff = run_git(wt, &["diff", "-U0", old, "HEAD", "--", "CHANGELOG.md"])
        .await
        .unwrap_or_default();
    let text = std::fs::read_to_string(wt.join("CHANGELOG.md")).unwrap_or_default();
    if added_under_unreleased(&diff, &text) {
        return None;
    }
    Some(
        "the branch adds no line under \"## Unreleased\" in CHANGELOG.md (under ### Added, Changed, Fixed or Internal); every landed task gets \
         one, written on its branch (only question/research/explore/incident tickets and \
         changes confined to docs/tickets/ are exempt)"
            .to_string(),
    )
}

const CHANGELOG_HEADINGS: [&str; 4] = ["### Added", "### Changed", "### Fixed", "### Internal"];

/// Whether a `-U0` diff of CHANGELOG.md adds a non-blank, non-heading line inside the
/// `## Unreleased` section of the new text, under one of the four `###` headings.
fn added_under_unreleased(diff: &str, new_text: &str) -> bool {
    let lines: Vec<&str> = new_text.lines().collect();
    let Some(start) = lines.iter().position(|l| l.trim_end() == "## Unreleased") else {
        return false;
    };
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.starts_with("## "))
        .map_or(lines.len(), |n| start + 1 + n);
    let mut at = 0usize; // 1-based number of the next new-file line
    for l in diff.lines() {
        if let Some(h) = l.strip_prefix("@@ ") {
            at = h
                .split('+')
                .nth(1)
                .and_then(|r| r.split([',', ' ']).next())
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        } else if l.starts_with('+') && !l.starts_with("+++") {
            let idx = at.saturating_sub(1);
            let under_heading = idx > start
                && idx < end
                && lines[start + 1..idx]
                    .iter()
                    .rev()
                    .find(|x| x.starts_with("### "))
                    .is_some_and(|x| CHANGELOG_HEADINGS.contains(&x.trim_end()));
            if under_heading && !l[1..].trim().is_empty() && !l[1..].starts_with('#') {
                return true;
            }
            at += 1;
        }
    }
    false
}

/// For a version 2 ticket (stages, br-tj7q) the landed tree's `tasks.md` must be fully ticked:
/// the worker works through it item by item. Version 1 tickets and tasks without a ticket file
/// pass. `Some(reason)` refuses the landing.
fn unticked_tasks(wt: &Path, task: &str) -> Option<String> {
    let bare = task.rsplit('-').next().unwrap_or(task);
    let open = wt.join("docs/tickets/open");
    for entry in std::fs::read_dir(&open).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".md") else {
            continue;
        };
        if !stem.ends_with(&format!("-{bare}")) {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).ok()?;
        let version_2 = text
            .lines()
            .skip(1)
            .take_while(|l| l.trim_end() != "---")
            .any(|l| l.trim_end() == "version: 2");
        if !version_2 {
            return None;
        }
        let tasks = std::fs::read_to_string(open.join(stem).join("tasks.md")).ok()?;
        let open_items: Vec<&str> = tasks
            .lines()
            .map(str::trim_start)
            .filter(|l| l.starts_with("- [ ]"))
            .collect();
        if open_items.is_empty() {
            return None;
        }
        return Some(format!(
            "{stem}/tasks.md has {} unticked item(s) (first: {}); a version 2 ticket lands with every task done",
            open_items.len(),
            open_items[0]
        ));
    }
    None
}

/// The integration branch's new tip, when it moved past `old` and every path changed since
/// matches `check_skip_paths`. `None` when it didn't move, the setting is empty, or any other
/// path changed (the guard in `advance` then reports "moved").
async fn skippable_move(i: &LandInput<'_>, old: &str) -> Result<Option<String>, WorktreeError> {
    if i.check_skip_paths.is_empty() {
        return Ok(None);
    }
    let cur = run_git(
        i.repo,
        &["rev-parse", &format!("refs/heads/{}", i.integration)],
    )
    .await?
    .trim()
    .to_string();
    if cur == old
        || run_git(i.repo, &["merge-base", "--is-ancestor", old, &cur])
            .await
            .is_err()
    {
        return Ok(None);
    }
    let changed = run_git(i.repo, &["diff", "--name-only", old, &cur]).await?;
    let all_skippable = changed
        .lines()
        .all(|p| i.check_skip_paths.iter().any(|g| glob_match(g, p)));
    Ok(all_skippable.then_some(cur))
}

/// `*` and `?` match within a path segment, `**` across segments.
fn glob_match(glob: &str, path: &str) -> bool {
    fn go(g: &[u8], p: &[u8]) -> bool {
        match g.split_first() {
            None => p.is_empty(),
            Some((b'*', rest)) if rest.first() == Some(&b'*') => {
                let rest = &rest[1..];
                let rest = rest.strip_prefix(b"/").unwrap_or(rest);
                (0..=p.len()).any(|n| go(rest, &p[n..]))
            }
            Some((b'*', rest)) => (0..=p.len())
                .take_while(|&n| n == 0 || p[n - 1] != b'/')
                .any(|n| go(rest, &p[n..])),
            Some((b'?', rest)) => p.first().is_some_and(|&c| c != b'/') && go(rest, &p[1..]),
            Some((c, rest)) => p.first() == Some(c) && go(rest, &p[1..]),
        }
    }
    go(glob.as_bytes(), path.as_bytes())
}

/// `<task id>: <title>`, the task summary as the body, then the `Ticket:` and `Branch:` trailers.
fn commit_message(i: &LandInput<'_>) -> String {
    let mut msg = format!("{}: {}\n\n", i.task, i.title);
    if let Some(s) = i.summary.map(str::trim).filter(|s| !s.is_empty()) {
        msg.push_str(s);
        msg.push_str("\n\n");
    }
    msg.push_str(&format!("Task: {}\nBranch: {}\n", i.task, i.branch));
    msg
}

/// Move the integration branch from `old` to `new`. A worktree with the branch checked out is
/// advanced by a fast-forward merge there, since `update-ref` would leave its index and files at
/// the old tip (the new tip showing as staged changes). The old value is the guard either way:
/// a moved integration branch is refused, not overwritten.
async fn advance(i: &LandInput<'_>, old: &str, new: &str) -> Result<(), LandError> {
    let moved = || LandError::Moved(i.integration.to_string());
    let Some(dir) = checked_out_at(i.repo, i.integration).await? else {
        let refname = format!("refs/heads/{}", i.integration);
        return run_git(i.repo, &["update-ref", &refname, new, old])
            .await
            .map(drop)
            .map_err(|_| moved());
    };
    let tip = run_git(&dir, &["rev-parse", "HEAD"]).await?;
    if tip.trim() != old {
        return Err(moved());
    }
    let dirty = run_git(&dir, &["status", "--porcelain", "--untracked-files=no"]).await?;
    // Git fast-forwards past dirty files the merge doesn't touch, so only an overlap refuses.
    let touched = run_git(&dir, &["diff", "--name-only", old, new]).await?;
    let touched: Vec<&str> = touched.lines().collect();
    let overlap: Vec<&str> = dirty
        .lines()
        .filter_map(|l| l.get(3..))
        // A rename shows as `old -> new`; either side counts.
        .flat_map(|p| p.split(" -> "))
        .map(|p| p.trim_matches('"'))
        .filter(|p| touched.contains(p))
        .collect();
    if !overlap.is_empty() {
        return Err(LandError::Refused(format!(
            "{} is checked out at {} with uncommitted changes to files the landing touches ({}); \
             commit or stash them, then retry",
            i.integration,
            dir.display(),
            overlap.join(", ")
        )));
    }
    run_git(&dir, &["merge", "--ff-only", new])
        .await
        .map(drop)
        .map_err(|_| moved())
}

/// The worktree (the clone included) that has `branch` checked out, if any.
async fn checked_out_at(repo: &Path, branch: &str) -> Result<Option<PathBuf>, WorktreeError> {
    let list = run_git(repo, &["worktree", "list", "--porcelain"]).await?;
    let want = format!("branch refs/heads/{branch}");
    let mut dir = None;
    for line in list.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            dir = Some(PathBuf::from(p));
        } else if line == want {
            return Ok(dir);
        }
    }
    Ok(None)
}

/// The integration worktree, created on first use, reset onto a scratch branch at `tip`.
async fn prepare_worktree(i: &LandInput<'_>, tip: &str) -> Result<PathBuf, WorktreeError> {
    let scratch = format!("integrate/{}", i.task);
    if i.dir.join(".git").exists() {
        // A previous landing may have died mid-merge.
        let _ = run_git(&i.dir, &["merge", "--abort"]).await;
        run_git(&i.dir, &["checkout", "-f", "-B", &scratch, tip]).await?;
        run_git(&i.dir, &["clean", "-fdq"]).await?;
    } else {
        worktree::prune(i.repo).await?;
        let dir = i.dir.to_string_lossy();
        run_git(
            i.repo,
            &["worktree", "add", "-f", "-B", &scratch, &dir, tip],
        )
        .await?;
    }
    Ok(i.dir.clone())
}

fn count_path(i: &LandInput<'_>) -> PathBuf {
    i.dir.with_file_name(COUNT_FILE)
}

fn read_count(i: &LandInput<'_>) -> Option<u64> {
    std::fs::read_to_string(count_path(i))
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Zero always fails; with a last count, so does under half or over double of it.
fn band_check(n: u64, last: Option<u64>) -> Result<(), LandError> {
    let out = n == 0 || last.is_some_and(|l| n * BAND < l || n > l * BAND);
    if out {
        return Err(LandError::CountOutOfBand { count: n, last });
    }
    Ok(())
}

/// nextest's `Summary [..] N tests run: ...` line.
fn parse_test_count(text: &str) -> Option<u64> {
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("Summary"))?;
    let (before, _) = line.split_once(" tests run")?;
    before.rsplit(' ').next()?.parse().ok()
}

/// The check's test count, when its output has a nextest summary line.
async fn run_check(
    wt: &Path,
    cmd: &str,
    pgid: Option<&std::sync::atomic::AtomicU32>,
) -> Result<Option<u64>, LandError> {
    let child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(wt)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        // Its own group, so a forced restart can stop the whole check, not just `sh`.
        .process_group(0)
        .spawn()?;
    if let (Some(p), Some(id)) = (pgid, child.id()) {
        p.store(id, std::sync::atomic::Ordering::SeqCst);
    }
    let out = child.wait_with_output().await;
    if let Some(p) = pgid {
        p.store(0, std::sync::atomic::Ordering::SeqCst);
    }
    let out = out?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    if out.status.success() {
        return Ok(parse_test_count(&text));
    }
    let lines: Vec<&str> = text.lines().collect();
    let tail = lines[lines.len().saturating_sub(TAIL_LINES)..].join("\n");
    Err(LandError::CheckFailed {
        cmd: cmd.to_string(),
        tail,
    })
}

#[cfg(test)]
pub async fn run_check_for_test(
    wt: &Path,
    cmd: &str,
    pgid: &std::sync::atomic::AtomicU32,
) -> Result<Option<u64>, LandError> {
    run_check(wt, cmd, Some(pgid)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ticket_tree(version: &str, tasks: Option<&str>) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        let open = d.path().join("docs/tickets/open");
        std::fs::create_dir_all(open.join("x-6c8x")).unwrap();
        std::fs::write(
            open.join("x-6c8x.md"),
            format!("---\nid: br-6c8x\n{version}---\nbody\n"),
        )
        .unwrap();
        if let Some(t) = tasks {
            std::fs::write(open.join("x-6c8x/tasks.md"), t).unwrap();
        }
        d
    }

    #[test]
    fn version_2_with_unticked_tasks_is_refused() {
        let d = ticket_tree("version: 2\n", Some("- [x] 1.1 a\n- [ ] 1.2 b\n"));
        let msg = unticked_tasks(d.path(), "br-6c8x").expect("refused");
        assert!(msg.contains("1 unticked") && msg.contains("1.2 b"), "{msg}");
    }

    #[test]
    fn version_2_with_all_ticked_or_version_1_lands() {
        let done = ticket_tree("version: 2\n", Some("- [x] 1.1 a\n"));
        assert!(unticked_tasks(done.path(), "br-6c8x").is_none());
        let v1 = ticket_tree("", Some("- [ ] 1.1 a\n"));
        assert!(unticked_tasks(v1.path(), "br-6c8x").is_none());
        let none = ticket_tree("version: 2\n", None);
        assert!(unticked_tasks(none.path(), "br-6c8x").is_none());
    }

    #[test]
    fn changelog_line_under_each_heading_is_found() {
        for h in ["Added", "Changed", "Fixed", "Internal"] {
            let text =
                format!("# C\n\n## Unreleased\n\n### {h}\n\n- new\n- old\n\n## 1.0\n\n- v1\n");
            let added = "@@ -6,0 +7 @@\n+- new\n";
            assert!(added_under_unreleased(added, &text), "{h}");
            // A line added to a released section does not count.
            let released = "@@ -10,0 +11 @@\n+- v1\n";
            assert!(!added_under_unreleased(released, &text));
            // Nor does a bare heading.
            assert!(!added_under_unreleased("@@ -4,0 +5 @@\n+### x\n", &text));
        }
    }

    #[test]
    fn changelog_line_without_a_known_heading_is_not_found() {
        let none = "# C\n\n## Unreleased\n\n- new\n\n## 1.0\n";
        assert!(!added_under_unreleased("@@ -4,0 +5 @@\n+- new\n", none));
        let other = "# C\n\n## Unreleased\n\n### Docs\n\n- new\n\n## 1.0\n";
        assert!(!added_under_unreleased("@@ -6,0 +7 @@\n+- new\n", other));
        assert!(!added_under_unreleased("", none));
    }

    #[test]
    fn globs_match_paths() {
        assert!(glob_match("docs/**", "docs/a.md"));
        assert!(glob_match("docs/**", "docs/tickets/open/a.md"));
        assert!(!glob_match("docs/**", "crates/docs/a.md"));
        assert!(!glob_match("docs/**", "docs"));
        assert!(glob_match("*.md", "README.md"));
        assert!(!glob_match("*.md", "docs/README.md"));
        assert!(glob_match("**/*.md", "docs/a/README.md"));
    }

    #[test]
    fn parses_the_count_from_a_nextest_summary() {
        let out = "noise\n     Summary [  12.345s] 1234 tests run: 1234 passed, 3 skipped\n";
        assert_eq!(parse_test_count(out), Some(1234));
        assert_eq!(
            parse_test_count("     Summary [   0.1s] 1 test run: 1 passed"),
            None
        );
        assert_eq!(parse_test_count("cargo test: ok"), None);
    }

    #[test]
    fn the_band_is_half_to_double_and_zero_always_fails() {
        assert!(band_check(100, None).is_ok());
        assert!(band_check(0, None).is_err());
        assert!(band_check(0, Some(0)).is_err());
        assert!(band_check(50, Some(100)).is_ok());
        assert!(band_check(200, Some(100)).is_ok());
        assert!(band_check(49, Some(100)).is_err());
        assert!(band_check(201, Some(100)).is_err());
    }
}
