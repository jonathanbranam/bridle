//! Tools-only clones (hw6c, docs/design/cli.md): a clone listed in `[machine] tools_only` of
//! `~/.bridle/config.toml` is kept for its tools. `bridle serve` refuses there, the launch scripts
//! ask `bridle machine tools-only-check`, and git hooks installed by `tools-only-install` refuse
//! commits and pushes (`--no-verify` stays the deliberate override).

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};
use bridle_api::discovery::bridle_home;
use bridle_daemon::config::is_tools_only;

use crate::cli::{MachineAction, ToolsOnlyArgs};
use crate::error::CliError;

/// Written into every hook we install; a hook without it is the user's and is left alone.
const MARKER: &str = "# bridle-tools-only-hook";
const HOOKS: [&str; 2] = ["pre-commit", "pre-push"];

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

fn install(repo: &Path) -> anyhow::Result<()> {
    let listed = is_tools_only(repo, &bridle_home())?;
    // `--git-path` follows core.hooksPath and worktrees.
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
    let dir = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    if listed {
        return install_hooks(&dir, repo);
    }
    // No longer listed: undo a previous install rather than refuse.
    if !restore_hooks(&dir)? {
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
    std::fs::read_to_string(path).is_ok_and(|t| t.contains(MARKER))
}

fn install_hooks(dir: &Path, repo: &Path) -> anyhow::Result<()> {
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
    let body = format!(
        "#!/bin/sh\n{MARKER}\n\
         echo 'bridle: {}' >&2\n\
         echo 'bridle: override deliberately with --no-verify.' >&2\n\
         exit 1\n",
        reason(repo).replace('\'', "")
    );
    for name in HOOKS {
        let path = dir.join(name);
        if path.exists() && !is_ours(&path) {
            let moved = aside(dir, name);
            std::fs::rename(&path, &moved)
                .with_context(|| format!("moving {} aside", path.display()))?;
            println!("moved {} to {}", path.display(), moved.display());
        }
        std::fs::write(&path, &body).with_context(|| format!("writing {}", path.display()))?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        println!("installed {}", path.display());
    }
    Ok(())
}

/// Removes bridle's hooks and puts moved-aside ones back; true if there was anything to do. A
/// different hook that appeared since is never clobbered: both stay, and we say so.
fn restore_hooks(dir: &Path) -> anyhow::Result<bool> {
    let mut acted = false;
    for name in HOOKS {
        let path = dir.join(name);
        if is_ours(&path) {
            std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
            println!("removed {}", path.display());
            acted = true;
        }
        let moved = aside(dir, name);
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

        assert!(restore_hooks(dir).unwrap());
        assert!(
            std::fs::read_to_string(dir.join("pre-commit"))
                .unwrap()
                .contains("mine")
        );
        assert!(!dir.join("pre-push").exists());
        assert!(!aside(dir, "pre-commit").exists());
        assert!(!restore_hooks(dir).unwrap());
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
        restore_hooks(dir).unwrap();
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
