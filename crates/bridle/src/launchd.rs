//! `bridle launchd install|uninstall`: a per-project LaunchAgent so the daemon (and so every
//! agent and build under it) has no GUI responsible app. Writes and removes the plist only;
//! never runs `launchctl`. See docs/design/cli.md and ticket qr8z.

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use bridle_api::discovery;

use crate::cli::{Cli, LaunchdAction, LaunchdArgs, LaunchdInstallArgs};
use crate::error::CliError;
use crate::render;

pub fn run(cli: &Cli, args: &LaunchdArgs) -> Result<(), CliError> {
    if !cfg!(target_os = "macos") {
        return Err(CliError::Other(anyhow!("bridle launchd is macOS only")));
    }
    let home = PathBuf::from(std::env::var_os("HOME").ok_or_else(|| anyhow!("$HOME is not set"))?);
    let agents_dir = home.join("Library/LaunchAgents");
    let out = match &args.action {
        LaunchdAction::Install(a) => install(cli, a, &agents_dir)?,
        LaunchdAction::Uninstall => uninstall(cli, &agents_dir)?,
    };
    if cli.json {
        render::print_json(&out)?;
    } else {
        for line in out["message"].as_str().unwrap_or_default().lines() {
            println!("{line}");
        }
    }
    Ok(())
}

pub(crate) fn project_name(cli: &Cli, repo: &Path) -> String {
    // Same default as the daemon's own project name.
    cli.project.clone().unwrap_or_else(|| {
        repo.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "bridle".to_string())
    })
}

fn label(project: &str) -> String {
    format!("dev.bridle.{project}")
}

pub(crate) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

struct Plist<'a> {
    label: &'a str,
    program_args: Vec<String>,
    working_dir: &'a Path,
    log: &'a Path,
    path_env: &'a str,
    home: &'a str,
}

impl Plist<'_> {
    fn render(&self) -> String {
        let args: String = self
            .program_args
            .iter()
            .map(|a| format!("    <string>{}</string>\n", xml_escape(a)))
            .collect();
        let s = |p: &Path| xml_escape(&p.to_string_lossy());
        // KeepAlive only on a crash (non-zero exit), so a deliberate `stop-daemon` stays stopped.
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
{args}  </array>
  <key>WorkingDirectory</key>
  <string>{wd}</string>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <dict>
    <key>SuccessfulExit</key>
    <false/>
  </dict>
  <key>StandardOutPath</key>
  <string>{log}</string>
  <key>StandardErrorPath</key>
  <string>{log}</string>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>{path}</string>
    <key>HOME</key>
    <string>{home}</string>
  </dict>
</dict>
</plist>
"#,
            label = xml_escape(self.label),
            wd = s(self.working_dir),
            log = s(self.log),
            path = xml_escape(self.path_env),
            home = xml_escape(self.home),
        )
    }
}

fn install(
    cli: &Cli,
    a: &LaunchdInstallArgs,
    agents_dir: &Path,
) -> Result<serde_json::Value, CliError> {
    let repo = match &a.repo {
        Some(r) => r.clone(),
        None => std::env::current_dir().context("current directory")?,
    };
    let repo = repo
        .canonicalize()
        .with_context(|| format!("repo path {}", repo.display()))?;
    let workspace = match &a.workspace {
        Some(w) => w.clone(),
        None => repo
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| repo.clone()),
    };
    let project = project_name(cli, &repo);
    let label = label(&project);
    let plist_path = agents_dir.join(format!("{label}.plist"));
    if plist_path.exists() && !a.force {
        return Err(CliError::Other(anyhow!(
            "{} already exists; pass --force to overwrite",
            plist_path.display()
        )));
    }
    let state_dir = discovery::state_dir(&workspace);
    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("creating {}", state_dir.display()))?;
    std::fs::create_dir_all(agents_dir)
        .with_context(|| format!("creating {}", agents_dir.display()))?;

    let exe = std::env::current_exe().context("locating the bridle binary")?;
    let home = std::env::var("HOME").unwrap_or_default();
    let path_env = std::env::var("PATH").unwrap_or_default();
    let plist = Plist {
        label: &label,
        program_args: vec![
            exe.to_string_lossy().into_owned(),
            "--project".into(),
            project.clone(),
            "serve".into(),
            "--repo".into(),
            repo.to_string_lossy().into_owned(),
            "--workspace".into(),
            workspace.to_string_lossy().into_owned(),
        ],
        working_dir: &repo,
        log: &state_dir.join("daemon.log"),
        path_env: &path_env,
        home: &home,
    };
    std::fs::write(&plist_path, plist.render())
        .with_context(|| format!("writing {}", plist_path.display()))?;

    let (bootstrap, bootout) = commands(&label, &plist_path);
    let message = format!(
        "wrote {}\nnothing was started; to load it (stop any daemon already running for this project first):\n  {bootstrap}\nto unload:\n  {bootout}",
        plist_path.display()
    );
    Ok(serde_json::json!({
        "plist": plist_path, "label": label, "bootstrap": bootstrap, "bootout": bootout, "message": message,
    }))
}

fn uninstall(cli: &Cli, agents_dir: &Path) -> Result<serde_json::Value, CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let label = label(&project_name(cli, &repo));
    let plist_path = agents_dir.join(format!("{label}.plist"));
    if !plist_path.exists() {
        return Err(CliError::Other(anyhow!(
            "{} does not exist",
            plist_path.display()
        )));
    }
    std::fs::remove_file(&plist_path)
        .with_context(|| format!("removing {}", plist_path.display()))?;
    let (_, bootout) = commands(&label, &plist_path);
    let message = format!(
        "removed {}\nif it is loaded, unload it (this stops the daemon):\n  {bootout}",
        plist_path.display()
    );
    Ok(
        serde_json::json!({"plist": plist_path, "label": label, "bootout": bootout, "message": message}),
    )
}

fn commands(label: &str, plist: &Path) -> (String, String) {
    (
        format!("launchctl bootstrap gui/$(id -u) {}", plist.display()),
        format!("launchctl bootout gui/$(id -u)/{label}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn cli(project: &str) -> Cli {
        Cli::parse_from(["bridle", "--project", project, "status"])
    }

    fn install_args(repo: &Path, force: bool) -> LaunchdInstallArgs {
        LaunchdInstallArgs {
            repo: Some(repo.to_path_buf()),
            workspace: None,
            force,
        }
    }

    #[test]
    fn install_writes_a_valid_plist_and_refuses_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("ws/proj & co");
        std::fs::create_dir_all(&repo).unwrap();
        let dir = tmp.path().join("LaunchAgents");
        let c = cli("proj");

        let out = install(&c, &install_args(&repo, false), &dir).unwrap();
        let path = dir.join("dev.bridle.proj.plist");
        assert_eq!(out["label"], "dev.bridle.proj");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("<string>dev.bridle.proj</string>"));
        assert!(text.contains("<string>serve</string>"));
        assert!(text.contains("proj &amp; co"));
        assert!(text.contains("<key>SuccessfulExit</key>"));
        assert!(text.contains("<key>PATH</key>"));
        assert!(tmp.path().join("ws/.bridle").is_dir());
        assert!(out["bootstrap"].as_str().unwrap().contains("bootstrap"));

        if cfg!(target_os = "macos") {
            let ok = std::process::Command::new("plutil")
                .arg("-lint")
                .arg(&path)
                .status()
                .map(|s| s.success())
                .unwrap_or(true);
            assert!(ok, "plutil -lint failed");
        }

        let err = install(&c, &install_args(&repo, false), &dir).unwrap_err();
        assert!(err.to_string().contains("--force"));
        install(&c, &install_args(&repo, true), &dir).unwrap();
    }

    #[test]
    fn uninstall_removes_the_plist() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("LaunchAgents");
        std::fs::create_dir_all(&dir).unwrap();
        // Uninstall names the project from --project, so the cwd doesn't matter.
        let c = cli("gone");
        assert!(uninstall(&c, &dir).is_err());
        let path = dir.join("dev.bridle.gone.plist");
        std::fs::write(&path, "x").unwrap();
        let out = uninstall(&c, &dir).unwrap();
        assert!(!path.exists());
        assert!(out["bootout"].as_str().unwrap().contains("bootout"));
    }
}
