//! Ticket files the daemon moves itself: resolving one at integration (br-ggnp). The CLI's
//! `ticket resolve` uses the same code.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, anyhow, bail};

/// Move the open ticket `id` to `resolved/`, stamping `resolved:` first. Returns the new path.
pub fn resolve(root: &Path, id: &str, resolved: &str) -> anyhow::Result<PathBuf> {
    let found = find_by_id(&root.join("open"), id)?;
    let dest = root
        .join("resolved")
        .join(found.file_name().expect("a file"));
    if find_by_id(&root.join("resolved"), id).is_ok() {
        bail!("ticket {id} is already in resolved/");
    }
    let text =
        std::fs::read_to_string(&found).with_context(|| format!("reading {}", found.display()))?;
    let stamped = set_resolved(&text, resolved)
        .ok_or_else(|| anyhow!("{} has no frontmatter", found.display()))?;
    std::fs::create_dir_all(root.join("resolved")).context("creating resolved/")?;
    std::fs::write(&dest, stamped).with_context(|| format!("writing {}", dest.display()))?;
    std::fs::remove_file(&found).with_context(|| format!("removing {}", found.display()))?;
    // The stage folder (design.md, attachments) travels with the file.
    let folder = crate::stages::folder_of(&found);
    if folder.is_dir() {
        let to = crate::stages::folder_of(&dest);
        std::fs::rename(&folder, &to)
            .with_context(|| format!("moving {} to {}", folder.display(), to.display()))?;
    }
    Ok(dest)
}

/// The one `*-<id>.md` file in `dir`; errors on none or several.
pub fn find_by_id(dir: &Path, id: &str) -> anyhow::Result<PathBuf> {
    // File names are bare, so a qualified `br-k7tm` finds `*-k7tm.md`.
    let id = id.rsplit('-').next().unwrap_or(id);
    let suffix = format!("-{id}.md");
    let mut hits: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with(&suffix))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    hits.sort();
    match hits.len() {
        0 => bail!("no ticket with id {id} in {}", dir.display()),
        1 => Ok(hits.remove(0)),
        _ => bail!(
            "ticket id {id} is ambiguous in {}: {}",
            dir.display(),
            hits.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// `text` with `resolved: <ts>` as the last frontmatter line (replacing any existing one, in
/// either name);
/// `None` when there is no frontmatter block.
pub fn set_resolved(text: &str, resolved: &str) -> Option<String> {
    let mut lines: Vec<&str> = text.lines().collect();
    if lines.first() != Some(&"---") {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    lines.retain({
        let mut i = 0;
        move |l| {
            i += 1;
            !(i > 1 && i <= end && (l.starts_with("resolved:") || l.starts_with("closed:")))
        }
    });
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    let stamp = format!("resolved: {resolved}");
    lines.insert(end, &stamp);
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

/// Commits `paths` (relative to `repo`) with `msg` when `repo` is checked out on the project's
/// integration branch, so a ticket file an agent writes is never left uncommitted (br-gakc) and
/// `ticket ready` works right after `ticket new`. Commit only, never a push (rule
/// `one-pusher-for-the-integration-branch`); elsewhere (a worker's branch) the worker's own
/// commit carries the file. `false` when git failed.
pub fn commit_ticket_files(repo: &Path, paths: &[&Path], msg: &str) -> bool {
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(repo)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
    };
    let Ok(config) = crate::config::Config::load(repo) else {
        return true;
    };
    let on_integration = git(&["symbolic-ref", "--short", "HEAD"])
        .is_some_and(|o| String::from_utf8_lossy(&o.stdout).trim() == config.branches.integration);
    if !on_integration {
        return true;
    }
    let names: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
    let mut add = vec!["add", "-A", "--"];
    add.extend(names.iter().map(String::as_str));
    let mut commit = vec!["commit", "-q", "--no-verify", "-m", msg, "--"];
    commit.extend(names.iter().map(String::as_str));
    git(&add).is_some() && git(&commit).is_some()
}

/// Resolves ticket `id`'s file when the ticket is integrated or dropped (br-ggnp): stamps
/// `resolved:`, moves it to `resolved/` and commits that on the integration branch, locally only.
/// Idempotent: no open file (already resolved, or a file-less row) is `Ok(None)`. Returns the
/// thread note to record.
pub fn resolve_and_commit(repo: &Path, id: &str) -> anyhow::Result<Option<String>> {
    let root = repo.join("docs/tickets");
    let tail = id.rsplit('-').next().unwrap_or(id);
    if find_by_id(&root.join("open"), tail).is_err() {
        return Ok(None);
    }
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let from = find_by_id(&root.join("open"), tail)?;
    let dest = resolve(&root, tail, &now)?;
    let rel = |p: &Path| p.strip_prefix(repo).unwrap_or(p).to_path_buf();
    let (from, to) = (rel(&from), rel(&dest));
    let mut paths = vec![from.clone(), to.clone()];
    if crate::stages::folder_of(&dest).is_dir() {
        paths.push(crate::stages::folder_of(&from));
        paths.push(crate::stages::folder_of(&to));
    }
    let refs: Vec<&Path> = paths.iter().map(PathBuf::as_path).collect();
    let committed = commit_ticket_files(repo, &refs, &format!("Ticket {id}: resolved"));
    Ok(Some(if committed {
        format!("ticket file moved to {}", to.display())
    } else {
        format!("ticket file moved to {} but not committed", to.display())
    }))
}

/// Ticket ids that are integrated (per `integrated`) whose file is still under `open/`.
pub fn stale_open_files(repo: &Path, integrated: &[String]) -> Vec<String> {
    let open = repo.join("docs/tickets/open");
    integrated
        .iter()
        .filter(|id| find_by_id(&open, id).is_ok())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .current_dir(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{args:?}");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn repo_with_open_ticket() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        git(d, &["init", "-q", "-b", "main"]);
        // resolve_and_commit runs git without the helper's `-c` identity; CI runners have no
        // global identity, so the repo itself must carry one (br-hnef).
        git(d, &["config", "user.name", "t"]);
        git(d, &["config", "user.email", "t@t"]);
        std::fs::create_dir_all(d.join("docs/tickets/open")).unwrap();
        std::fs::write(
            d.join("docs/tickets/open/thing-ab12.md"),
            "---\nid: br-ab12\ntitle: t\n---\nbody\n",
        )
        .unwrap();
        git(d, &["add", "."]);
        git(d, &["commit", "-q", "-m", "init"]);
        dir
    }

    #[test]
    fn integrating_moves_stamps_and_commits_once_and_is_idempotent() {
        let dir = repo_with_open_ticket();
        let d = dir.path();
        let note = resolve_and_commit(d, "br-ab12").unwrap().unwrap();
        assert!(note.contains("resolved/thing-ab12.md") && !note.contains("not committed"));
        assert!(!d.join("docs/tickets/open/thing-ab12.md").exists());
        let text = std::fs::read_to_string(d.join("docs/tickets/resolved/thing-ab12.md")).unwrap();
        assert!(text.contains("\nresolved: "));
        assert_eq!(git(d, &["status", "--short"]), "");
        assert_eq!(git(d, &["rev-list", "--count", "HEAD"]).trim(), "2");
        assert!(git(d, &["log", "-1", "--format=%s"]).contains("Ticket br-ab12: resolved"));
        // Again, and for a row with no file: nothing to do, no error, no new commit.
        assert!(resolve_and_commit(d, "br-ab12").unwrap().is_none());
        assert!(resolve_and_commit(d, "br-zz99").unwrap().is_none());
        assert_eq!(git(d, &["rev-list", "--count", "HEAD"]).trim(), "2");
    }

    #[test]
    fn stale_open_files_flags_integrated_tickets_still_in_open() {
        let dir = repo_with_open_ticket();
        let ids = vec!["br-ab12".to_string(), "br-zz99".to_string()];
        assert_eq!(stale_open_files(dir.path(), &ids), vec!["br-ab12"]);
        resolve_and_commit(dir.path(), "br-ab12").unwrap();
        assert!(stale_open_files(dir.path(), &ids).is_empty());
    }
}
