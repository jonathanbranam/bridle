//! The "stable project" check (br-mzy4, kx5u K3): a project moves only when everything is
//! landed or dropped, nothing is running, nothing is unpushed and no mail is undelivered. Run by
//! `bridle project preflight` and by `bridle project stop`, which refuses on any failure. It
//! reads only; it never commits or pushes (agents cannot push, br-t7ya).

use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// What is not stable, one line each. `leave` names checkout paths that may stay dirty; they
/// are returned in the second list so the caller can print them as LEFT. `stop_pushes` skips the
/// "integration branch is on origin" test: `project stop` pushes that branch itself and confirms
/// origin has it, so refusing first would leave nothing for it to do.
pub fn unstable(
    repo: &Path,
    db: &Path,
    integration: &str,
    leave: &[String],
    stop_pushes: bool,
) -> (Vec<String>, Vec<String>) {
    let mut bad = Vec::new();
    let mut left = Vec::new();
    bad.extend(bridle_daemon::store::unstable_in_db(db));
    let refs = git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads/bridle",
            "refs/heads/integrate",
        ],
    )
    .unwrap_or_default();
    for b in refs.lines().filter(|b| *b != "bridle/state") {
        let ahead = git(
            repo,
            &["rev-list", "--count", &format!("{integration}..{b}")],
        )
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);
        if ahead > 0 {
            bad.push(format!(
                "branch {b} is {ahead} commit(s) ahead of {integration}"
            ));
        }
    }
    let remote = format!("origin/{integration}");
    match git(
        repo,
        &["rev-list", "--count", &format!("{remote}..{integration}")],
    )
    .and_then(|s| s.trim().parse::<u32>().ok())
    {
        _ if stop_pushes => {}
        Some(0) => {}
        Some(n) => bad.push(format!(
            "{integration} is {n} commit(s) ahead of {remote}: not pushed"
        )),
        None => bad.push(format!(
            "{integration} is not on origin ({remote} not found)"
        )),
    }
    // `-z` and `-uall` so every untracked file is named, not just its directory.
    let status = git(repo, &["status", "--porcelain", "-z", "-uall"]).unwrap_or_default();
    for entry in status.split('\0').filter(|e| e.len() > 3) {
        let path = &entry[3..];
        if leave.iter().any(|l| l == path) {
            left.push(path.to_string());
        } else {
            bad.push(format!("main checkout is dirty: {path}"));
        }
    }
    (bad, left)
}

/// The preflight check's pass flag and message.
pub fn check(repo: &Path, leave: &[String]) -> (bool, String) {
    let integration = bridle_daemon::config::Config::load(repo)
        .map(|c| c.branches.integration)
        .unwrap_or_else(|_| "main".into());
    let db = bridle_daemon::paths::Workspace::new(repo.to_path_buf(), None).db();
    let (bad, left) = unstable(repo, &db, &integration, leave, false);
    let left = if left.is_empty() {
        String::new()
    } else {
        format!(" (LEFT: {})", left.join(", "))
    };
    if bad.is_empty() {
        (
            true,
            format!("all landed, nothing running, nothing unpushed, no undelivered mail{left}"),
        )
    } else {
        (false, format!("{}{left}", bad.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    /// A clone of a bare origin, both on `main`, in sync.
    fn project() -> (tempfile::TempDir, std::path::PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let origin = d.path().join("origin.git");
        let repo = d.path().join("p");
        std::fs::create_dir(&origin).unwrap();
        sh(&origin, &["init", "--bare", "-b", "main"]);
        std::fs::create_dir(&repo).unwrap();
        sh(&repo, &["init", "-b", "main"]);
        sh(&repo, &["commit", "--allow-empty", "-m", "i"]);
        sh(
            &repo,
            &["remote", "add", "origin", origin.to_str().unwrap()],
        );
        sh(&repo, &["push", "-u", "origin", "main"]);
        (d, repo)
    }

    fn db(dir: &Path, sql: &str) -> std::path::PathBuf {
        let p = dir.join("bridle.db");
        let c = rusqlite::Connection::open(&p).unwrap();
        c.execute_batch(&format!(
            "CREATE TABLE tasks (id TEXT, state TEXT); CREATE TABLE agents (name TEXT, state TEXT); \
             CREATE TABLE outbox (id TEXT, state TEXT); {sql}"
        ))
        .unwrap();
        p
    }

    #[test]
    fn a_clean_project_passes() {
        let (d, repo) = project();
        let db = db(d.path(), "INSERT INTO tasks VALUES ('br-1','integrated');");
        assert_eq!(unstable(&repo, &db, "main", &[], false), (vec![], vec![]));
    }

    #[test]
    fn each_kind_of_instability_is_named() {
        let (d, repo) = project();
        let db = db(
            d.path(),
            "INSERT INTO tasks VALUES ('br-1','claimed'),('br-2','built');
             INSERT INTO agents VALUES ('w','working');
             INSERT INTO outbox VALUES ('o1','queued');",
        );
        sh(&repo, &["checkout", "-b", "bridle/w"]);
        sh(&repo, &["commit", "--allow-empty", "-m", "w"]);
        sh(&repo, &["checkout", "main"]);
        sh(&repo, &["commit", "--allow-empty", "-m", "unpushed"]);
        std::fs::write(repo.join("review-documents.txt"), "x").unwrap();
        let (bad, left) = unstable(&repo, &db, "main", &[], false);
        let all = bad.join("\n");
        for want in [
            "claimed ticket(s) not landed or dropped: br-1",
            "built ticket(s) not landed or dropped: br-2",
            "agent(s) running: w",
            "1 undelivered message(s)",
            "branch bridle/w is 1 commit(s) ahead of main",
            "main is 1 commit(s) ahead of origin/main",
            "main checkout is dirty: review-documents.txt",
        ] {
            assert!(all.contains(want), "{want} not in {all}");
        }
        assert!(left.is_empty());
    }

    #[test]
    fn leave_lets_a_dirty_path_through_and_lists_it() {
        let (d, repo) = project();
        let db = db(d.path(), "");
        std::fs::write(repo.join("migrations.toml"), "x").unwrap();
        let (bad, left) = unstable(&repo, &db, "main", &["migrations.toml".into()], false);
        assert!(bad.is_empty(), "{bad:?}");
        assert_eq!(left, ["migrations.toml"]);
    }
}
