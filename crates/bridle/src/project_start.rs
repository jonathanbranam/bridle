//! `bridle project start` (53gy C): the last step of moving a project. Installs the launchd or
//! systemd unit if it is missing, enables it (boot, restart on failure) and starts it, then
//! finishes what `take-over` could not do without a daemon: re-creates the project's tokens
//! through `token pair`'s code and carries over the scheduled messages `project stop` left on
//! `bridle/state`. `--machine` runs it on another machine over ssh, the way `token pair` does.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::Context;
use bridle_api::Client;
use bridle_api::discovery::{bridle_home, list_registry};
use bridle_api::machines::{MachineMap, owner_names};
use bridle_api::types::{Schedule, ScheduleAddRequest};
use bridle_daemon::paths::Workspace;
use bridle_daemon::state_branch::{SCHEDULES_FILE, local_owner};
use chrono::{DateTime, Utc};

use crate::cli::{Cli, LaunchdInstallArgs, ProjectStartArgs, SystemdInstallArgs};
use crate::commands::client_for;
use crate::error::CliError;
use crate::project_stop::{Ctl, RealCtl, Unit, installed_units};
use crate::token_pair::{Exec, Fleet, Options, Runner, Where, pair};

/// Written by `take-over` in the workspace's `.bridle/`, removed once `start` has carried it in.
const MOVED_IN: &str = "moved-in.json";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct MovedIn {
    schedules: Vec<Schedule>,
}

/// Records, after a take-over, what `project start` must do once the daemon is up: re-create the
/// tokens (the file existing says so) and carry the schedules the old owner's stop committed.
pub fn note_moved_in(ws: &Workspace) -> anyhow::Result<()> {
    let schedules = match std::fs::read_to_string(ws.state_branch_dir().join(SCHEDULES_FILE)) {
        Ok(text) => serde_json::from_str(&text).context("reading the carried schedules")?,
        // A stop from before schedules were carried, or a forced take-over of a dead machine.
        Err(_) => Vec::new(),
    };
    std::fs::create_dir_all(ws.state_dir())?;
    std::fs::write(
        ws.state_dir().join(MOVED_IN),
        serde_json::to_string_pretty(&MovedIn { schedules })?,
    )?;
    Ok(())
}

/// Enables and starts each unit; running it on units already running changes nothing.
pub fn start_units(
    os: &str,
    uid: &str,
    units: &[Unit],
    ctl: &dyn Ctl,
) -> Result<Vec<String>, String> {
    let strings = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut done = Vec::new();
    if os != "macos" {
        ctl.run("systemctl", &strings(&["--user", "daemon-reload"]))
            .map_err(|e| format!("systemctl daemon-reload: {e}"))?;
    }
    for u in units {
        if os == "macos" {
            let target = format!("gui/{uid}/{}", u.name);
            // Undo the `disable` a stop left, or the plist would not load at the next login.
            ctl.run("launchctl", &strings(&["enable", &target]))
                .map_err(|e| format!("{}: launchctl enable: {e}", u.name))?;
            if ctl.run("launchctl", &strings(&["print", &target])).is_err() {
                let domain = format!("gui/{uid}");
                ctl.run(
                    "launchctl",
                    &strings(&["bootstrap", &domain, &u.file.to_string_lossy()]),
                )
                .map_err(|e| format!("{}: launchctl bootstrap: {e}", u.name))?;
            }
        } else {
            ctl.run(
                "systemctl",
                &strings(&["--user", "enable", "--now", &u.name]),
            )
            .map_err(|e| format!("{}: systemctl enable: {e}", u.name))?;
        }
        done.push(format!("{} unit {} enabled and started", u.what, u.name));
    }
    Ok(done)
}

/// The add request that recreates `s` on the new daemon, or why it can't be: a one-shot whose
/// time has passed would be refused (or fire at once), so it is reported instead.
pub fn to_request(s: &Schedule, now: DateTime<Utc>) -> Result<ScheduleAddRequest, String> {
    let mut req = ScheduleAddRequest {
        to: Some(s.target.clone()),
        tz: Some(s.tz.clone()),
        body: s.body.clone(),
        ..Default::default()
    };
    match (s.kind.as_str(), &s.cron, s.next_fire_at) {
        ("cron", Some(c), _) => req.cron = Some(c.clone()),
        ("once", _, Some(at)) if at > now => req.at = Some(at.to_rfc3339()),
        _ => return Err(format!("{} -> {}: its time has passed", s.id, s.target)),
    }
    Ok(req)
}

/// Carries `schedules` over with `add`; returns how many went and a line for each that did not.
pub async fn carry_schedules<F, Fut>(
    schedules: &[Schedule],
    now: DateTime<Utc>,
    mut add: F,
) -> (usize, Vec<String>)
where
    F: FnMut(ScheduleAddRequest) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let (mut sent, mut problems) = (0, Vec::new());
    for s in schedules {
        match to_request(s, now) {
            Ok(req) => match add(req).await {
                Ok(()) => sent += 1,
                Err(e) => problems.push(format!("{} -> {}: {e}", s.id, s.target)),
            },
            Err(e) => problems.push(e),
        }
    }
    (sent, problems)
}

pub async fn run(cli: &Cli, args: &ProjectStartArgs) -> Result<(), CliError> {
    let home = bridle_home();
    if let Some(m) = &args.machine {
        let fleet = Fleet::from_config(&MachineMap::load(&home).map_err(|e| anyhow::anyhow!(e))?);
        if *m != fleet.local {
            let host = fleet
                .hosts
                .get(m)
                .ok_or_else(|| anyhow::anyhow!("no machine `{m}` in [machines]"))?;
            let project = cli
                .project
                .clone()
                .ok_or_else(|| anyhow::anyhow!("name the project with --project"))?;
            return remote(
                &Exec,
                &Where::Ssh(host.clone()),
                &project,
                args,
                &mut std::io::stdout(),
            );
        }
    }
    local(cli, args, &home, &RealCtl).await
}

/// Runs the start on another machine; its output is what is shown here.
pub fn remote(
    runner: &dyn Runner,
    at: &Where,
    project: &str,
    args: &ProjectStartArgs,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let mut a = vec![
        "projects".to_string(),
        "start".into(),
        "--project".into(),
        project.into(),
    ];
    if let Some(r) = &args.repo {
        a.extend(["--repo".into(), r.display().to_string()]);
    }
    let text = runner
        .run(at, &a, None)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let _ = write!(out, "{text}");
    Ok(())
}

/// Installs the daemon's unit for `project`; the existing `launchd`/`systemd` install code, which
/// only writes the file.
fn install_unit(
    cli: &Cli,
    os: &str,
    home_dir: &Path,
    config_home: &Path,
    project: &str,
    repo: &Path,
    ws: &Workspace,
) -> Result<(), CliError> {
    if os == "macos" {
        let a = LaunchdInstallArgs {
            repo: Some(repo.to_path_buf()),
            workspace: Some(ws.workspace.clone()),
            force: false,
        };
        crate::launchd::install(cli, &a, &home_dir.join("Library/LaunchAgents"))?;
    } else {
        let machines = MachineMap::load(&bridle_home()).map_err(anyhow::Error::new)?;
        let a = SystemdInstallArgs {
            projects_dir: Some(ws.workspace.clone()),
            force: false,
        };
        crate::systemd::install(
            Some(project),
            &a,
            &machines,
            &ws.workspace,
            &config_home.join("systemd/user"),
            &crate::systemd::UnitEnv::current(home_dir)?,
        )?;
    }
    Ok(())
}

async fn local(
    cli: &Cli,
    args: &ProjectStartArgs,
    home: &Path,
    ctl: &dyn Ctl,
) -> Result<(), CliError> {
    let repo = match &args.repo {
        Some(r) => r.clone(),
        None => std::env::current_dir().context("current directory")?,
    }
    .canonicalize()
    .context("repo path")?;
    let project = cli.project.clone().unwrap_or_else(|| {
        repo.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    let names = owner_names(home);
    match local_owner(&repo).await {
        Some(o) if names.contains(&o.host) && o.released.is_none() => {}
        Some(o) if names.contains(&o.host) => {
            return Err(anyhow::anyhow!(
                "{project} was released by this machine ({}): run `bridle project take-over` to claim it again",
                o.host
            )
            .into());
        }
        Some(o) => {
            return Err(anyhow::anyhow!(
                "{project} is owned by {}, not this machine ({}): run `bridle project take-over` here first",
                o.host,
                names[0]
            )
            .into());
        }
        None => {
            return Err(anyhow::anyhow!(
                "{project} has no owner recorded on bridle/state in {}: run `bridle project take-over` here first",
                repo.display()
            )
            .into());
        }
    }
    let ws = Workspace::new(repo.clone(), args.workspace.clone());
    let os = std::env::consts::OS;
    let home_dir = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(".config"));
    let mut out = std::io::stdout();

    let mut units = installed_units(os, &home_dir, &config_home, &project);
    if !units.iter().any(|u| u.what == "daemon") {
        install_unit(cli, os, &home_dir, &config_home, &project, &repo, &ws)?;
        let _ = writeln!(out, "installed the unit for {project}");
        units = installed_units(os, &home_dir, &config_home, &project);
    }
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    for line in start_units(os, &uid, &units, ctl).map_err(|e| {
        anyhow::anyhow!(
            "a unit could not be started ({e}); fix it, then run `bridle project start` again"
        )
    })? {
        let _ = writeln!(out, "{line}");
    }

    wait_for_daemon(&project, Duration::from_secs(60)).await?;
    let _ = writeln!(out, "{project}: daemon is up");
    carry_in(cli, home, &ws, &project, &mut out).await
}

async fn wait_for_daemon(project: &str, limit: Duration) -> Result<(), CliError> {
    let start = Instant::now();
    loop {
        if let Some(d) = list_registry().into_iter().find(|d| d.project == project)
            && Client::new_with_timeout(d.url, None, Duration::from_secs(2))
                .health()
                .await
                .is_ok()
        {
            return Ok(());
        }
        if start.elapsed() > limit {
            return Err(anyhow::anyhow!(
                "the unit is started but no daemon for {project} answered in {}s: see daemon.log in the workspace's .bridle",
                limit.as_secs()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// What `take-over` left for the new daemon: tokens first (the schedules are added as the human,
/// with the token the daemon just made), then the schedules. The note is removed only when both
/// went through, so a failed start can simply be run again.
async fn carry_in(
    cli: &Cli,
    home: &Path,
    ws: &Workspace,
    project: &str,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let note = ws.state_dir().join(MOVED_IN);
    let Ok(text) = std::fs::read_to_string(&note) else {
        return Ok(());
    };
    let moved: MovedIn = serde_json::from_str(&text).context("reading moved-in.json")?;
    let fleet = Fleet::from_config(&MachineMap::load(home).map_err(|e| anyhow::anyhow!(e))?);
    let opts = Options {
        projects: vec![project.to_string()],
        ..Default::default()
    };
    // The same code as `bridle token pair`, narrowed to this project: no second minting path.
    let paired =
        tokio::task::spawn_blocking(move || pair(&opts, &fleet, &Exec, &mut std::io::stdout()))
            .await
            .map_err(|e| CliError::Other(e.into()))?
            .map_err(|e| anyhow::anyhow!(e))?;

    let client = client_for(cli).await?;
    let (sent, problems) = carry_schedules(&moved.schedules, Utc::now(), |req| {
        let client = &client;
        async move {
            client
                .add_schedule(&req)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
    })
    .await;
    let _ = writeln!(out, "{project}: carried {sent} scheduled message(s)");
    for p in &problems {
        let _ = writeln!(out, "  not carried: {p}");
    }
    if !paired || !problems.is_empty() {
        return Err(anyhow::anyhow!(
            "{project} is running, but {}: fix it and run `bridle project start` again",
            if paired {
                "some scheduled messages were not carried"
            } else {
                "some tokens were not re-created (see above; `bridle token pair` retries them)"
            }
        )
        .into());
    }
    std::fs::remove_file(&note).context("removing moved-in.json")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Rec(RefCell<Vec<String>>, bool);

    impl Ctl for Rec {
        fn run(&self, prog: &str, args: &[String]) -> Result<(), String> {
            let line = format!("{prog} {}", args.join(" "));
            self.0.borrow_mut().push(line.clone());
            // `print` fails for a job that is not loaded.
            if line.contains(" print ") && !self.1 {
                return Err("not loaded".into());
            }
            Ok(())
        }
    }

    fn unit(os: &str) -> Unit {
        let (name, file) = if os == "macos" {
            ("dev.bridle.p", "/h/Library/LaunchAgents/dev.bridle.p.plist")
        } else {
            (
                "bridle-p.service",
                "/h/.config/systemd/user/bridle-p.service",
            )
        };
        Unit {
            what: "daemon",
            name: name.into(),
            file: file.into(),
        }
    }

    #[test]
    fn systemd_reloads_then_enables_and_starts() {
        let ctl = Rec(RefCell::default(), true);
        start_units("linux", "1000", &[unit("linux")], &ctl).unwrap();
        assert_eq!(
            *ctl.0.borrow(),
            [
                "systemctl --user daemon-reload",
                "systemctl --user enable --now bridle-p.service"
            ]
        );
    }

    #[test]
    fn launchd_enables_and_loads_only_what_is_not_loaded() {
        let ctl = Rec(RefCell::default(), false);
        start_units("macos", "501", &[unit("macos")], &ctl).unwrap();
        let calls = ctl.0.borrow().clone();
        assert_eq!(calls[0], "launchctl enable gui/501/dev.bridle.p");
        assert!(
            calls[2].starts_with("launchctl bootstrap gui/501 "),
            "{calls:?}"
        );
        // Already loaded (a second start): nothing is bootstrapped again.
        let ctl = Rec(RefCell::default(), true);
        start_units("macos", "501", &[unit("macos")], &ctl).unwrap();
        assert!(!ctl.0.borrow().iter().any(|l| l.contains("bootstrap")));
    }

    #[test]
    fn a_failing_unit_is_reported() {
        struct Fail;
        impl Ctl for Fail {
            fn run(&self, _: &str, _: &[String]) -> Result<(), String> {
                Err("boom".into())
            }
        }
        let e = start_units("linux", "1", &[unit("linux")], &Fail).unwrap_err();
        assert!(e.contains("boom"), "{e}");
    }

    fn sched(kind: &str, cron: Option<&str>, next: Option<DateTime<Utc>>) -> Schedule {
        Schedule {
            id: "sc-0001".into(),
            created_by: "human".into(),
            target: "external:orchestrator".into(),
            body: "check in".into(),
            kind: kind.into(),
            cron: cron.map(str::to_string),
            tz: "America/New_York".into(),
            next_fire_at: next,
            last_fired_at: None,
            state: "active".into(),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn schedules_are_recreated_and_stale_ones_reported() {
        let now = Utc::now();
        let list = [
            sched("cron", Some("0 9 * * 1"), Some(now)),
            sched("once", None, Some(now + chrono::Duration::hours(2))),
            sched("once", None, Some(now - chrono::Duration::hours(2))),
        ];
        let seen = RefCell::new(Vec::new());
        let (sent, problems) = carry_schedules(&list, now, |req| {
            seen.borrow_mut().push(req);
            async { Ok(()) }
        })
        .await;
        assert_eq!(sent, 2);
        assert_eq!(problems.len(), 1, "{problems:?}");
        let seen = seen.borrow();
        assert_eq!(seen[0].cron.as_deref(), Some("0 9 * * 1"));
        assert_eq!(seen[0].to.as_deref(), Some("external:orchestrator"));
        assert!(seen[1].at.is_some() && seen[1].cron.is_none());
    }

    #[test]
    fn take_over_notes_the_schedules_the_stop_committed() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path().join("p"), Some(tmp.path().to_path_buf()));
        std::fs::create_dir_all(ws.state_branch_dir()).unwrap();
        let list = vec![sched("cron", Some("0 9 * * 1"), None)];
        std::fs::write(
            ws.state_branch_dir().join(SCHEDULES_FILE),
            serde_json::to_string(&list).unwrap(),
        )
        .unwrap();
        note_moved_in(&ws).unwrap();
        let moved: MovedIn =
            serde_json::from_str(&std::fs::read_to_string(ws.state_dir().join(MOVED_IN)).unwrap())
                .unwrap();
        assert_eq!(moved.schedules, list);
    }

    #[test]
    fn install_writes_a_systemd_unit_for_the_project() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("p")).unwrap();
        let machines: MachineMap = toml::from_str(
            "[machine]\nname = \"nuc\"\n[projects]\np = { machine = \"nuc\", port = 7400 }\n",
        )
        .unwrap();
        let units = tmp.path().join("units");
        crate::systemd::install(
            Some("p"),
            &SystemdInstallArgs {
                projects_dir: Some(tmp.path().to_path_buf()),
                force: false,
            },
            &machines,
            tmp.path(),
            &units,
            &crate::systemd::UnitEnv::current(tmp.path()).unwrap(),
        )
        .unwrap();
        assert!(units.join("bridle-p.service").exists());
    }
}
