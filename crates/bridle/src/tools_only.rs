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
    if !is_tools_only(repo, &bridle_home())? {
        bail!(
            "{} is not listed in [machine] tools_only in {}",
            repo.display(),
            bridle_home().join("config.toml").display()
        );
    }
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
    // Check every hook before writing any, so a refusal leaves nothing half-installed.
    for name in HOOKS {
        if let Ok(existing) = std::fs::read_to_string(dir.join(name))
            && !existing.contains(MARKER)
        {
            bail!(
                "{} already has a {name} hook that isn't bridle's; not overwriting it. Move it \
                 aside (or chain to it) and re-run.",
                dir.display()
            );
        }
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let body = format!(
        "#!/bin/sh\n{MARKER}\n\
         echo 'bridle: {}' >&2\n\
         echo 'bridle: override deliberately with --no-verify.' >&2\n\
         exit 1\n",
        reason(repo).replace('\'', "")
    );
    for name in HOOKS {
        let path = dir.join(name);
        std::fs::write(&path, &body).with_context(|| format!("writing {}", path.display()))?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        println!("installed {}", path.display());
    }
    Ok(())
}
