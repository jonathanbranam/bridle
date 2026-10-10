//! Tools-only clones (hw6c, docs/design/cli.md): a clone listed in `[machine] tools_only` of
//! `~/.bridle/config.toml` is kept for its tools. `bridle serve` refuses there, the launch scripts
//! ask `bridle machine tools-only-check`, and git hooks installed by `tools-only-install` refuse
//! commits and pushes (`--no-verify` stays the deliberate override).
//!
//! The same file owns the owner-only `pre-push` hook (8z7j): git has one `pre-push`, so it is one
//! marked script, and `bridle machine push-check` is what it runs for the integration branch.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, bail};
use bridle_api::discovery::bridle_home;
use bridle_daemon::config::{Config, is_tools_only};
use bridle_daemon::state_branch::Owner;

use crate::cli::{MachineAction, ToolsOnlyArgs};
use crate::error::CliError;

/// Written into every hook we install; a hook without it is the user's and is left alone.
const MARKER: &str = "# bridle-tools-only-hook";
/// Marks the owner-only pre-push script (8z7j). Either marker makes a hook ours.
const OWNER_MARKER: &str = "# bridle-owner-hook";
const HOOKS: [&str; 2] = ["pre-commit", "pre-push"];
const STATE_BRANCH: &str = "bridle/state";
const OWNER_FILE: &str = "owner.toml";
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

fn reason(repo: &Path) -> String {
    format!(
        "{} is a tools-only clone on this machine ([machine] tools_only in {}): it is not the \
         project's home. Run in the served project's workspace instead.",
        repo.display(),
        bridle_home().join("config.toml").display()
    )
}

/// `bridle serve`'s guard.
pub fn refuse_serve(repo: &Path) -> anyhow::Result<()> {
    if is_tools_only(repo, &bridle_home())? {
        bail!("refusing to serve: {}", reason(repo));
    }
    Ok(())
}

pub fn run(action: &MachineAction) -> Result<(), CliError> {
    match action {
        MachineAction::ToolsOnlyCheck(a) => {
            let repo = repo_of(a)?;
            if is_tools_only(&repo, &bridle_home()).map_err(anyhow::Error::from)? {
                return Err(anyhow::anyhow!("{}", reason(&repo)).into());
            }
            Ok(())
        }
        MachineAction::ToolsOnlyInstall(a) => Ok(install(&repo_of(a)?)?),
        MachineAction::PushCheck(a) => push_check(&a.rest),
    }
}

/// The pre-push hook's helper (8z7j): refuses a push of the integration branch from any clone
/// but the owner's. Reads git's ref lines on stdin; everything else passes untouched.
fn push_check(_hook_args: &[String]) -> Result<(), CliError> {
    use std::io::Read;
    let mut lines = String::new();
    std::io::stdin()
        .read_to_string(&mut lines)
        .context("reading the ref lines")?;
    let repo = repo_of(&ToolsOnlyArgs { repo: None })?;
    let main_repo = main_checkout(&repo)?;
    let integration = Config::load(&main_repo)
        .context("loading .bridle/config.toml")?
        .branches
        .integration;
    let target = format!("refs/heads/{integration}");
    // The remote ref is the third field of `<local ref> <local sha> <remote ref> <remote sha>`.
    if !lines
        .lines()
        .any(|l| l.split_whitespace().nth(2) == Some(target.as_str()))
    {
        return Ok(());
    }
    let project = main_repo
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let host = hostname().context("reading this machine's name")?;
    match read_owner(&main_repo) {
        Some(o) if o.host == host => Ok(()),
        Some(o) => Err(anyhow::anyhow!(
            "refusing to push {integration}. Project {project} is owned by {} (since {}).\n        \
             To move it here: bridle serve --take-over",
            o.host,
            o.since
        )
        .into()),
        None => Err(anyhow::anyhow!(
            "refusing to push {integration}. Project {project} has no readable owner ({STATE_BRANCH}:{OWNER_FILE}); \
             failing closed.\n        To claim it here: bridle serve --take-over"
        )
        .into()),
    }
}

fn hostname() -> Option<String> {
    let out = Command::new("hostname").output().ok()?;
    let h = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!h.is_empty()).then_some(h)
}

/// The clone's main checkout, also when run from one of its worktrees.
fn main_checkout(repo: &Path) -> anyhow::Result<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .context("running git")?;
    let common = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    Ok(common
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo.to_path_buf()))
}

/// `owner.toml` from a fresh fetch of origin's state branch; the local branch when origin can't
/// be reached (the push could not succeed either). None on any failure: the caller refuses.
fn read_owner(repo: &Path) -> Option<Owner> {
    let show = |rev: &str| {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["show", &format!("{rev}:{OWNER_FILE}")])
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8(out.stdout).ok())?
    };
    let fetched = fetch_state(repo).then(|| show("FETCH_HEAD")).flatten();
    let text = fetched.or_else(|| show(&format!("refs/heads/{STATE_BRANCH}")))?;
    toml::from_str(&text).ok()
}

/// Fetches origin's state branch into FETCH_HEAD only (no ref moves), bounded in time.
fn fetch_state(repo: &Path) -> bool {
    let Ok(mut child) = Command::new("git")
        .arg("-C")
        .arg(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args([
            "fetch",
            "--quiet",
            "origin",
            &format!("refs/heads/{STATE_BRANCH}"),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    let deadline = Instant::now() + FETCH_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
}

/// The session launcher's guard: refuses in a tools-only clone, passes outside any repository
/// (a session may start from anywhere).
pub fn check_here() -> Result<(), CliError> {
    let args = ToolsOnlyArgs { repo: None };
    if repo_of(&args).is_err() {
        return Ok(());
    }
    run(&MachineAction::ToolsOnlyCheck(args))
}

fn repo_of(a: &ToolsOnlyArgs) -> anyhow::Result<PathBuf> {
    let dir = match &a.repo {
        Some(r) => r.clone(),
        None => std::env::current_dir().context("current directory")?,
    };
    let out = Command::new("git")
        .arg("-C")
        .arg(&dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!("{} is not in a git repository", dir.display());
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

/// `--git-path` follows core.hooksPath and worktrees.
fn hooks_dir(repo: &Path) -> anyhow::Result<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--path-format=absolute", "--git-path", "hooks"])
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!(
            "git could not locate the hooks directory of {}",
            repo.display()
        );
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

fn install(repo: &Path) -> anyhow::Result<()> {
    let listed = is_tools_only(repo, &bridle_home())?;
    let dir = hooks_dir(repo)?;
    let owner = owner_hook_wanted(repo);
    if listed {
        return install_hooks_with(&dir, repo, owner);
    }
    // No longer listed: undo a previous install rather than refuse.
    if !restore_hooks(&dir, owner)? {
        bail!(
            "{} is not listed in [machine] tools_only in {}",
            repo.display(),
            bridle_home().join("config.toml").display()
        );
    }
    Ok(())
}

fn aside(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.pre-bridle"))
}

fn is_ours(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|t| t.contains(MARKER) || t.contains(OWNER_MARKER))
}

/// Never write through a symlink: a shared-dotfiles hooks dir or file would change every repo
/// that links to it at once.
fn refuse_symlink(dir: &Path, name: &str) -> anyhow::Result<()> {
    let link = |p: &Path| std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink());
    if link(dir) || link(&dir.join(name)) {
        bail!(
            "{} (or its {name} hook) is a symlink; not writing through it. Make it a real \
             directory or file in this clone and re-run.",
            dir.display()
        );
    }
    Ok(())
}

/// The `pre-push` script: the tools-only refusal (when listed) then the owner check (when the
/// project pushes its state branch), each behind its marker.
fn pre_push_body(repo: &Path, tools_only: bool, owner: bool) -> String {
    let mut body = String::from("#!/bin/sh\n");
    if tools_only {
        body.push_str(&tools_only_body(repo, false));
    }
    if owner {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("bridle"));
        body.push_str(&format!(
            "{OWNER_MARKER}\nexec '{}' machine push-check \"$@\"\n",
            exe.display().to_string().replace('\'', "'\\''")
        ));
    }
    body
}

fn tools_only_body(repo: &Path, shebang: bool) -> String {
    format!(
        "{}{MARKER}\n\
         echo 'bridle: {}' >&2\n\
         echo 'bridle: override deliberately with --no-verify.' >&2\n\
         exit 1\n",
        if shebang { "#!/bin/sh\n" } else { "" },
        reason(repo).replace('\'', "")
    )
}

/// Whether this project's clones carry the owner check: only a project that pushes its state
/// branch has an `owner.toml` to check against.
fn owner_hook_wanted(repo: &Path) -> bool {
    Config::load(repo).is_ok_and(|c| c.state_push)
}

/// Installs the owner-only `pre-push` hook into `repo`'s clone (8z7j). Idempotent; rewrites a
/// hook of ours (a moved binary, a changed tools-only listing), never touches a foreign one,
/// and never writes through a symlink. Installs hooks only: no branch or setting is edited.
pub fn install_owner_hook(repo: &Path) -> anyhow::Result<()> {
    if !owner_hook_wanted(repo) {
        return Ok(());
    }
    let dir = hooks_dir(repo)?;
    refuse_symlink(&dir, "pre-push")?;
    let path = dir.join("pre-push");
    if path.exists() && !is_ours(&path) {
        bail!(
            "{} is not bridle's, so the owner-only push check is not installed; chain \
             `bridle machine push-check \"$@\"` into it by hand",
            path.display()
        );
    }
    let listed = is_tools_only(repo, &bridle_home()).unwrap_or(false);
    let body = pre_push_body(repo, listed, true);
    if std::fs::read_to_string(&path).is_ok_and(|t| t == body) {
        return Ok(());
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    write_executable(&path, &body)
}

fn write_executable(path: &Path, body: &str) -> anyhow::Result<()> {
    std::fs::write(path, body).with_context(|| format!("writing {}", path.display()))?;
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(test)]
fn install_hooks(dir: &Path, repo: &Path) -> anyhow::Result<()> {
    install_hooks_with(dir, repo, false)
}

fn install_hooks_with(dir: &Path, repo: &Path, owner: bool) -> anyhow::Result<()> {
    for name in HOOKS {
        refuse_symlink(dir, name)?;
    }
    // Check every hook before writing any, so a refusal leaves nothing half-installed.
    for name in HOOKS {
        let path = dir.join(name);
        if path.exists() && !is_ours(&path) && aside(dir, name).exists() {
            bail!(
                "{} has a {name} hook that isn't bridle's, and {} already exists; not \
                 overwriting either. Resolve them by hand and re-run.",
                dir.display(),
                aside(dir, name).display()
            );
        }
    }
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    for name in HOOKS {
        let body = if name == "pre-push" {
            pre_push_body(repo, true, owner)
        } else {
            tools_only_body(repo, true)
        };
        let path = dir.join(name);
        if path.exists() && !is_ours(&path) {
            let moved = aside(dir, name);
            std::fs::rename(&path, &moved)
                .with_context(|| format!("moving {} aside", path.display()))?;
            println!("moved {} to {}", path.display(), moved.display());
        }
        write_executable(&path, &body)?;
        println!("installed {}", path.display());
    }
    Ok(())
}

/// Removes bridle's hooks and puts moved-aside ones back; true if there was anything to do. A
/// different hook that appeared since is never clobbered: both stay, and we say so.
fn restore_hooks(dir: &Path, owner: bool) -> anyhow::Result<bool> {
    let mut acted = false;
    for name in HOOKS {
        let path = dir.join(name);
        let moved = aside(dir, name);
        // A pre-push that is only the owner check has nothing tools-only to undo.
        if std::fs::read_to_string(&path).is_ok_and(|t| t.contains(MARKER)) {
            if name == "pre-push" && owner && !moved.exists() {
                write_executable(&path, &pre_push_body(dir, false, true))?;
                println!("kept {} as the owner-only push check", path.display());
            } else {
                std::fs::remove_file(&path)
                    .with_context(|| format!("removing {}", path.display()))?;
                println!("removed {}", path.display());
            }
            acted = true;
        }
        if !moved.exists() {
            continue;
        }
        acted = true;
        if path.exists() {
            println!(
                "kept {} and {}: a different {name} hook is in place; merge them by hand",
                path.display(),
                moved.display()
            );
        } else {
            std::fs::rename(&moved, &path)
                .with_context(|| format!("restoring {}", path.display()))?;
            println!("restored {}", path.display());
        }
    }
    Ok(acted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, s: &str) {
        std::fs::write(p, s).unwrap();
    }

    #[test]
    fn moves_aside_installs_and_restores() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path();
        write(&dir.join("pre-commit"), "#!/bin/sh\necho mine\n");
        install_hooks(dir, Path::new("/r")).unwrap();
        assert!(is_ours(&dir.join("pre-commit")));
        assert!(is_ours(&dir.join("pre-push")));
        assert!(
            std::fs::read_to_string(aside(dir, "pre-commit"))
                .unwrap()
                .contains("mine")
        );
        assert!(!aside(dir, "pre-push").exists());

        // Idempotent: our own hook is not moved aside, the original stays put.
        install_hooks(dir, Path::new("/r")).unwrap();
        assert!(
            std::fs::read_to_string(aside(dir, "pre-commit"))
                .unwrap()
                .contains("mine")
        );

        assert!(restore_hooks(dir, false).unwrap());
        assert!(
            std::fs::read_to_string(dir.join("pre-commit"))
                .unwrap()
                .contains("mine")
        );
        assert!(!dir.join("pre-push").exists());
        assert!(!aside(dir, "pre-commit").exists());
        assert!(!restore_hooks(dir, false).unwrap());
    }

    #[test]
    fn refuses_when_aside_file_exists() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path();
        write(&dir.join("pre-push"), "user hook");
        write(&aside(dir, "pre-push"), "older");
        let err = install_hooks(dir, Path::new("/r")).unwrap_err().to_string();
        assert!(err.contains("pre-push"), "{err}");
        assert_eq!(
            std::fs::read_to_string(dir.join("pre-push")).unwrap(),
            "user hook"
        );
        assert!(!dir.join("pre-commit").exists(), "nothing half-installed");
    }

    #[test]
    fn restore_keeps_both_when_a_different_hook_exists() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path();
        write(&dir.join("pre-commit"), "original");
        install_hooks(dir, Path::new("/r")).unwrap();
        write(&dir.join("pre-commit"), "newer");
        restore_hooks(dir, false).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("pre-commit")).unwrap(),
            "newer"
        );
        assert_eq!(
            std::fs::read_to_string(aside(dir, "pre-commit")).unwrap(),
            "original"
        );
    }
}
