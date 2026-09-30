//! `bridle systemd install`: a systemd user unit per project this machine owns, so daemons come
//! back after a reboot with no terminal (ticket 4r3k, step 2). Writes unit files only; never
//! runs `systemctl` or `loginctl`. The Linux sibling of `launchd.rs`.

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use bridle_api::discovery;
use bridle_api::machines::MachineMap;

use crate::cli::{Cli, SystemdAction, SystemdArgs, SystemdInstallArgs};
use crate::error::CliError;
use crate::render;

pub fn run(cli: &Cli, args: &SystemdArgs) -> Result<(), CliError> {
    check_os(std::env::consts::OS)?;
    let home = PathBuf::from(std::env::var_os("HOME").ok_or_else(|| anyhow!("$HOME is not set"))?);
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let units_dir = config_home.join("systemd/user");
    let SystemdAction::Install(a) = &args.action;
    let machines = MachineMap::load(&discovery::bridle_home()).map_err(anyhow::Error::new)?;
    let projects_dir = match &a.projects_dir {
        Some(d) => d.clone(),
        None => {
            let cwd = std::env::current_dir().context("current directory")?;
            cwd.parent().map(Path::to_path_buf).unwrap_or(cwd)
        }
    };
    let env = UnitEnv {
        exe: std::env::current_exe().context("locating the bridle binary")?,
        home: home.to_string_lossy().into_owned(),
        path: std::env::var("PATH").unwrap_or_default(),
        user: std::env::var("USER").unwrap_or_else(|_| "$USER".into()),
    };
    let out = install(
        cli.project.as_deref(),
        a,
        &machines,
        &projects_dir,
        &units_dir,
        &env,
    )?;
    if cli.json {
        render::print_json(&out)?;
    } else {
        for line in out["message"].as_str().unwrap_or_default().lines() {
            println!("{line}");
        }
    }
    Ok(())
}

fn check_os(os: &str) -> Result<(), CliError> {
    if os == "linux" {
        Ok(())
    } else {
        Err(CliError::Other(anyhow!("bridle systemd is Linux only")))
    }
}

struct UnitEnv {
    exe: PathBuf,
    home: String,
    path: String,
    user: String,
}

/// The projects to install: `--project` if the config puts it here, else every project it does.
fn owned_here(project: Option<&str>, machines: &MachineMap) -> Result<Vec<String>, CliError> {
    let Some(this) = machines.machine.name.as_deref() else {
        return Err(CliError::Other(anyhow!(
            "this machine has no name: set `[machine] name = \"...\"` in ~/.bridle/config.toml"
        )));
    };
    let here = |p: &&bridle_api::machines::ProjectPlace| p.machine == this;
    match project {
        Some(p) => match machines.projects.get(p).filter(here) {
            Some(_) => Ok(vec![p.to_string()]),
            None => Err(CliError::Other(anyhow!(
                "project '{p}' is not on this machine ('{this}') in [projects]"
            ))),
        },
        None => {
            let all: Vec<String> = machines
                .projects
                .iter()
                .filter(|(_, p)| here(p))
                .map(|(n, _)| n.clone())
                .collect();
            if all.is_empty() {
                return Err(CliError::Other(anyhow!(
                    "no project in [projects] is on this machine ('{this}')"
                )));
            }
            Ok(all)
        }
    }
}

/// One `ExecStart`/`Environment` word: quoted when it holds anything systemd would split or
/// expand (`%` is a specifier, `$` an env reference).
fn quote(s: &str) -> String {
    let plain = !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "/._-:=+,@".contains(c));
    if plain {
        return s.to_string();
    }
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%")
        .replace('$', "$$");
    format!("\"{escaped}\"")
}

fn render_unit(project: &str, repo: &Path, workspace: &Path, env: &UnitEnv) -> String {
    let exec = [
        env.exe.to_string_lossy().into_owned(),
        "--project".into(),
        project.into(),
        "serve".into(),
        "--repo".into(),
        repo.to_string_lossy().into_owned(),
        "--workspace".into(),
        workspace.to_string_lossy().into_owned(),
    ]
    .iter()
    .map(|a| quote(a))
    .collect::<Vec<_>>()
    .join(" ");
    let log = discovery::state_dir(workspace).join("daemon.log");
    // Restart only on a crash (non-zero exit), so a deliberate `stop-daemon` stays stopped.
    // The port comes from `[projects]`, which `serve` reads itself.
    format!(
        "[Unit]\nDescription=bridle daemon for {project}\nAfter=network-online.target\n\n\
         [Service]\nExecStart={exec}\nWorkingDirectory={wd}\nRestart=on-failure\nRestartSec=5\n\
         Environment={path}\nEnvironment={home}\n\
         StandardOutput=append:{log}\nStandardError=append:{log}\n\n\
         [Install]\nWantedBy=default.target\n",
        wd = quote(&repo.to_string_lossy()),
        path = quote(&format!("PATH={}", env.path)),
        home = quote(&format!("HOME={}", env.home)),
        log = log.display(),
    )
}

fn install(
    project: Option<&str>,
    a: &SystemdInstallArgs,
    machines: &MachineMap,
    projects_dir: &Path,
    units_dir: &Path,
    env: &UnitEnv,
) -> Result<serde_json::Value, CliError> {
    let projects = owned_here(project, machines)?;
    let mut units: Vec<(String, PathBuf, String)> = Vec::new();
    for project in &projects {
        let repo = projects_dir.join(project);
        if !repo.is_dir() {
            return Err(CliError::Other(anyhow!(
                "{} is not a directory: no clone of '{project}' there (see --projects-dir)",
                repo.display()
            )));
        }
        let path = units_dir.join(format!("bridle-{project}.service"));
        if path.exists() && !a.force {
            return Err(CliError::Other(anyhow!(
                "{} already exists; pass --force to overwrite",
                path.display()
            )));
        }
        units.push((
            project.clone(),
            path,
            render_unit(project, &repo, projects_dir, env),
        ));
    }
    std::fs::create_dir_all(units_dir)
        .with_context(|| format!("creating {}", units_dir.display()))?;
    for (_, path, text) in &units {
        let state_dir = discovery::state_dir(projects_dir);
        std::fs::create_dir_all(&state_dir)
            .with_context(|| format!("creating {}", state_dir.display()))?;
        std::fs::write(path, text).with_context(|| format!("writing {}", path.display()))?;
    }

    let names: Vec<String> = units
        .iter()
        .map(|(p, _, _)| format!("bridle-{p}.service"))
        .collect();
    let enable = format!("systemctl --user enable --now {}", names.join(" "));
    let reload = "systemctl --user daemon-reload".to_string();
    let linger = format!("sudo loginctl enable-linger {}", env.user);
    let mut message = String::new();
    for (_, path, _) in &units {
        message.push_str(&format!("wrote {}\n", path.display()));
    }
    message.push_str(&format!(
        "nothing was started; stop any daemon already running for these projects first, then run:\n  {reload}\n  {enable}\nso they start at boot without a login (once):\n  {linger}"
    ));
    Ok(serde_json::json!({
        "units": units.iter().map(|(_, p, _)| p).collect::<Vec<_>>(),
        "daemon_reload": reload, "enable": enable, "linger": linger, "message": message,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridle_api::machines::{MachineSelf, ProjectPlace};
    use std::collections::BTreeMap;

    fn machines() -> MachineMap {
        let place = |machine: &str, port| ProjectPlace {
            machine: machine.into(),
            port,
        };
        MachineMap {
            machine: MachineSelf {
                name: Some("nuc".into()),
            },
            machines: BTreeMap::new(),
            projects: BTreeMap::from([
                ("bridle".into(), place("mbp", 7401)),
                ("meta-notes".into(), place("nuc", 7402)),
                ("dotfiles".into(), place("nuc", 7403)),
            ]),
        }
    }

    fn env() -> UnitEnv {
        UnitEnv {
            exe: "/opt/bin/bridle".into(),
            home: "/home/jo".into(),
            path: "/usr/bin:/home/jo/.cargo/bin".into(),
            user: "jo".into(),
        }
    }

    fn args(force: bool) -> SystemdInstallArgs {
        SystemdInstallArgs {
            projects_dir: None,
            force,
        }
    }

    #[test]
    fn selects_the_projects_the_config_puts_here() {
        let m = machines();
        assert_eq!(owned_here(None, &m).unwrap(), ["dotfiles", "meta-notes"]);
        assert_eq!(owned_here(Some("dotfiles"), &m).unwrap(), ["dotfiles"]);
        assert!(owned_here(Some("bridle"), &m).is_err());
        assert!(owned_here(Some("nope"), &m).is_err());
        let unnamed = MachineMap::default();
        assert!(owned_here(None, &unnamed).is_err());
    }

    #[test]
    fn install_writes_units_and_prints_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path().join("ws");
        std::fs::create_dir_all(ws.join("meta-notes")).unwrap();
        std::fs::create_dir_all(ws.join("dotfiles")).unwrap();
        let units = tmp.path().join("systemd/user");

        let out = install(None, &args(false), &machines(), &ws, &units, &env()).unwrap();
        let text = std::fs::read_to_string(units.join("bridle-meta-notes.service")).unwrap();
        assert!(text.contains(&format!(
            "ExecStart=/opt/bin/bridle --project meta-notes serve --repo {0}/meta-notes --workspace {0}\n",
            ws.display()
        )));
        assert!(text.contains("Restart=on-failure\n"));
        assert!(text.contains("WantedBy=default.target\n"));
        assert!(text.contains("Environment=PATH=/usr/bin:/home/jo/.cargo/bin\n"));
        assert!(units.join("bridle-dotfiles.service").is_file());
        assert!(!units.join("bridle-bridle.service").exists());
        assert!(ws.join(".bridle").is_dir());
        assert_eq!(
            out["enable"],
            "systemctl --user enable --now bridle-dotfiles.service bridle-meta-notes.service"
        );
        assert_eq!(out["linger"], "sudo loginctl enable-linger jo");
        assert_eq!(out["daemon_reload"], "systemctl --user daemon-reload");

        let err = install(None, &args(false), &machines(), &ws, &units, &env()).unwrap_err();
        assert!(err.to_string().contains("--force"));
        install(None, &args(true), &machines(), &ws, &units, &env()).unwrap();
    }

    #[test]
    fn a_missing_clone_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path().join("ws");
        std::fs::create_dir_all(ws.join("dotfiles")).unwrap();
        let units = tmp.path().join("units");
        let err = install(None, &args(false), &machines(), &ws, &units, &env()).unwrap_err();
        assert!(err.to_string().contains("meta-notes"));
        assert!(!units.exists());
    }

    #[test]
    fn quoting_protects_spaces_and_specifiers() {
        assert_eq!(quote("/a/b-c"), "/a/b-c");
        assert_eq!(quote("/a b/100%"), "\"/a b/100%%\"");
    }

    #[test]
    fn refuses_off_linux() {
        assert!(
            check_os("macos")
                .unwrap_err()
                .to_string()
                .contains("Linux only")
        );
        assert!(check_os("linux").is_ok());
    }
}
