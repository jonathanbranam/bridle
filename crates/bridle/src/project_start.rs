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
use bridle_daemon::state_branch::{OUTBOX_SEQ_FILE, SCHEDULES_FILE, local_owner};
use bridle_daemon::store::Store;
use chrono::{DateTime, Utc};

use crate::cli::{Cli, LaunchdInstallArgs, ProjectStartArgs, SystemdInstallArgs};
use crate::commands::client_for;
use crate::error::CliError;
use crate::project_mail::{Old, OldMachine};
use crate::project_stop::{Ctl, RealCtl, Unit, installed_units};
use crate::token_pair::{Exec, Fleet, Options, Runner, Where, pair};

/// Moves the mail bridge once the tokens exist: given the old machine's name and the output.
type MailStep<'a> = dyn Fn(Option<&str>, &mut dyn Write) -> Result<(), String> + 'a;

/// Runs the mail step. A failure is printed with the command to run, not returned: the project
/// is up and mail can be fixed afterwards (br-btpy). Returns whether it succeeded.
fn mail_step(mail: &MailStep<'_>, from: Option<&str>, project: &str, out: &mut dyn Write) -> bool {
    match mail(from, out) {
        Ok(()) => true,
        Err(e) => {
            let _ = writeln!(
                out,
                "{project}: mail bridge not installed ({e}); fix it, then run `bridle mail install --project {project}`"
            );
            false
        }
    }
}

/// Removes the moved-in note once the mail step succeeded. A failed mail step keeps it, so the
/// next `project start` retries and the old machine's `[mail]` and `mail.key` are not lost
/// (br-whce).
fn finish_moved_in(note: &Path, mail_ok: bool) -> anyhow::Result<()> {
    if mail_ok {
        std::fs::remove_file(note).context("removing moved-in.json")?;
    }
    Ok(())
}

/// Written by `take-over` in the workspace's `.bridle/`, removed once `start` has carried it in.
const MOVED_IN: &str = "moved-in.json";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct MovedIn {
    schedules: Vec<Schedule>,
    /// The machine that owned the project before, where its `[mail]` section and `mail.key` are
    /// fetched from. Absent after a take-over with no previous owner.
    #[serde(default)]
    from: Option<String>,
}

/// Records, after a take-over, what `project start` must do once the daemon is up: re-create the
/// tokens (the file existing says so) and carry the schedules the old owner's stop committed.
pub fn note_moved_in(ws: &Workspace, from: Option<&str>) -> anyhow::Result<()> {
    let schedules = match std::fs::read_to_string(ws.state_branch_dir().join(SCHEDULES_FILE)) {
        Ok(text) => serde_json::from_str(&text).context("reading the carried schedules")?,
        // A stop from before schedules were carried, or a forced take-over of a dead machine.
        Err(_) => Vec::new(),
    };
    std::fs::create_dir_all(ws.state_dir())?;
    std::fs::write(
        ws.state_dir().join(MOVED_IN),
        serde_json::to_string_pretty(&MovedIn {
            schedules,
            from: from.map(str::to_string),
        })?,
    )?;
    Ok(())
}

/// The return-trip half of a take-over (d6h6): a database left from an earlier time on this
/// machine is moved, with its `-wal` and `-shm`, to `db-backups/<date>-<sha>/` in the workspace
/// (kept, not deleted, not in git), so the daemon builds a fresh one from the state branch. The
/// fresh database's outbox counter is seeded above both the old database's and the one the last
/// owner's stop committed, so receivers (who dedup on machine, daemon and id) never see an id
/// reused. Returns where the old database went, if there was one.
pub async fn fresh_database(ws: &Workspace) -> anyhow::Result<Option<PathBuf>> {
    let db = ws.db();
    // A running daemon keeps the renamed file open and goes on serving (and minting tokens) from
    // it until its next restart, which then opens the empty one (br-nbgp).
    if let Ok(d) = bridle_api::discovery::read_daemon_json(&ws.workspace)
        && bridle_api::discovery::pid_alive(d.pid)
    {
        anyhow::bail!(
            "the daemon for {} is running (pid {}) and has the database open; \
             stop it first (`bridle project stop`), then run `bridle project take-over` again",
            d.project,
            d.pid
        );
    }
    let mut seed = std::fs::read_to_string(ws.state_branch_dir().join(OUTBOX_SEQ_FILE))
        .ok()
        .and_then(|t| t.trim().parse::<i64>().ok())
        .unwrap_or(0);
    let mut backup = None;
    if db.exists() {
        let old = Store::open(&db).await.context("opening the old database")?;
        seed = seed.max(
            old.outbox_high_water()
                .await
                .context("reading its outbox counter")?,
        );
        drop(old);
        let sha = Command::new("git")
            .arg("-C")
            .arg(&ws.repo)
            .args(["rev-parse", "--short", "refs/heads/bridle/state"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "nosha".to_string());
        let mut dir = ws
            .state_dir()
            .join("db-backups")
            .join(format!("{}-{sha}", Utc::now().format("%Y-%m-%d")));
        // Two returns on one day at one state commit are possible; never overwrite a backup.
        let mut n = 1;
        while dir.exists() {
            n += 1;
            dir.set_file_name(format!("{}-{sha}-{n}", Utc::now().format("%Y-%m-%d")));
        }
        std::fs::create_dir_all(&dir).context("creating the backup folder")?;
        for suffix in ["", "-wal", "-shm"] {
            let from = PathBuf::from(format!("{}{suffix}", db.display()));
            if from.exists() {
                let to = dir.join(from.file_name().expect("db file name"));
                std::fs::rename(&from, &to)
                    .with_context(|| format!("moving {}", from.display()))?;
            }
        }
        backup = Some(dir);
    }
    if seed > 0 {
        let fresh = Store::open(&db)
            .await
            .context("creating the new database")?;
        fresh
            .outbox_seed_counter(seed)
            .await
            .context("seeding the outbox counter")?;
    }
    Ok(backup)
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
        // A brand-new project: `serve` makes `bridle/state` and claims the owner on its first run.
        None if bridle_daemon::state_branch::is_new_project(&repo).await == Ok(true) => {}
        None => {
            return Err(anyhow::anyhow!(
                "{project} has no owner recorded on bridle/state in {}: run `bridle project take-over` here first",
                repo.display()
            )
            .into());
        }
    }
    let ws = Workspace::new(repo.clone(), args.workspace.clone());
    let sys = Sys::current();
    let mut out = std::io::stdout();
    start_service(cli, &sys, &project, &repo, &ws, ctl, &mut out).await?;
    let mail = |from: Option<&str>, out: &mut dyn Write| -> Result<(), String> {
        bring_mail(cli, &sys, ctl, home, &repo, &project, from, out)
    };
    carry_in(cli, home, &ws, &project, &mail, &mut out).await
}

/// This machine's facts the unit and mail steps need.
pub struct Sys {
    pub os: &'static str,
    pub uid: String,
    pub home_dir: PathBuf,
    pub config_home: PathBuf,
}

impl Sys {
    pub fn current() -> Self {
        let home_dir = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let config_home = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".config"));
        let uid = Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        Sys {
            os: std::env::consts::OS,
            uid,
            home_dir,
            config_home,
        }
    }

    pub fn units(&self, project: &str) -> Vec<Unit> {
        installed_units(self.os, &self.home_dir, &self.config_home, project)
    }
}

/// Installs the daemon's unit if it is missing, starts it and waits until it answers.
pub async fn start_service(
    cli: &Cli,
    sys: &Sys,
    project: &str,
    repo: &Path,
    ws: &Workspace,
    ctl: &dyn Ctl,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let mut units = sys.units(project);
    if !units.iter().any(|u| u.what == "daemon") {
        install_unit(
            cli,
            sys.os,
            &sys.home_dir,
            &sys.config_home,
            project,
            repo,
            ws,
        )?;
        let _ = writeln!(out, "installed the unit for {project}");
        units = sys.units(project);
    }
    for line in start_units(sys.os, &sys.uid, &units, ctl).map_err(|e| {
        anyhow::anyhow!(
            "a unit could not be started ({e}); fix it, then run `bridle project start` again"
        )
    })? {
        let _ = writeln!(out, "{line}");
    }
    wait_for_daemon(project, Duration::from_secs(60)).await?;
    let _ = writeln!(out, "{project}: daemon is up");
    Ok(())
}

/// Installs and starts the project's mail bridge when this machine does mail and the project is
/// in `[mail] projects`; says why and returns `Ok` when there is nothing to do.
#[allow(clippy::too_many_arguments)]
pub fn bring_mail(
    cli: &Cli,
    sys: &Sys,
    ctl: &dyn Ctl,
    home: &Path,
    repo: &Path,
    project: &str,
    from: Option<&str>,
    out: &mut dyn Write,
) -> Result<(), String> {
    let fleet = Fleet::from_config(&MachineMap::load(home).map_err(|e| e.to_string())?);
    match crate::mail_install::mail_state(home, &sys.home_dir, false) {
        crate::mail_install::MailState::Off => {
            let _ = writeln!(out, "{project}: {}", crate::mail_install::MAIL_OFF);
            return Ok(());
        }
        crate::mail_install::MailState::NotSetUp(why) => {
            let _ = writeln!(
                out,
                "{project}: mail bridge skipped ({why}): {}",
                crate::mail_install::SET_UP_MAIL
            );
            return Ok(());
        }
        crate::mail_install::MailState::NotConfigured => {
            let _ = writeln!(
                out,
                "{project}: mail bridge skipped (no [mail] section in config.toml)"
            );
            return Ok(());
        }
        crate::mail_install::MailState::Ready => {}
    }
    let old = from
        .and_then(|m| fleet.hosts.get(m))
        .map(|h| OldMachine { host: h.clone() });
    let old: Option<&dyn Old> = old.as_ref().map(|o| o as &dyn Old);
    if !crate::project_mail::bring(old, home, project, out)? {
        let _ = writeln!(
            out,
            "{project}: mail bridge skipped (project is not in [mail] projects)"
        );
        return Ok(());
    }
    let has_unit = |units: &[Unit]| units.iter().any(|u| u.what == "mail bridge");
    if has_unit(&sys.units(project)) {
        let _ = writeln!(out, "{project}: mail bridge already installed");
    } else {
        crate::mail_install::install_unit(
            cli,
            &crate::cli::MailInstallArgs { force: false },
            repo.to_path_buf(),
            project.to_string(),
            false,
        )
        .map_err(|e| e.to_string())?;
    }
    let bridge: Vec<Unit> = sys
        .units(project)
        .into_iter()
        .filter(|u| u.what == "mail bridge")
        .collect();
    for line in start_units(sys.os, &sys.uid, &bridge, ctl)? {
        let _ = writeln!(out, "{line}");
    }
    Ok(())
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

/// The same code as `bridle token pair`, narrowed to this project: no second minting path.
/// It prints minted/kept per machine, and "unreachable" naming the machine.
pub async fn make_tokens(home: &Path, project: &str, out: &mut dyn Write) -> Result<(), CliError> {
    let fleet = Fleet::from_config(&MachineMap::load(home).map_err(|e| anyhow::anyhow!(e))?);
    let opts = Options {
        projects: vec![project.to_string()],
        ..Default::default()
    };
    let paired = tokio::task::spawn_blocking(move || {
        let mut buf = Vec::new();
        let r = pair(&opts, &fleet, &Exec, &mut buf);
        (r, buf)
    })
    .await
    .map_err(|e| CliError::Other(e.into()))?;
    let (paired, report) = paired;
    let _ = out.write_all(&report);
    let paired = paired.map_err(|e| anyhow::anyhow!(e))?;
    if !paired {
        return Err(anyhow::anyhow!(
            "{project} is running, but its tokens were not all re-created (see above; \
             a machine marked unreachable was skipped): fix it and run \
             `bridle token pair --projects {project}`"
        )
        .into());
    }
    Ok(())
}

/// Tokens, then what `take-over` left for the new daemon. The tokens are re-created on every
/// start (`token pair` keeps the working ones and says so), not only after a take-over: a fresh
/// database has none, whatever the note says. The schedules are added as the human, with the
/// token the daemon just made. The note is removed only when everything went through (the mail step included), so a
/// failed start can simply be run again.
async fn carry_in(
    cli: &Cli,
    home: &Path,
    ws: &Workspace,
    project: &str,
    mail: &MailStep<'_>,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let note = ws.state_dir().join(MOVED_IN);
    let moved: Option<MovedIn> = match std::fs::read_to_string(&note) {
        Ok(text) => Some(serde_json::from_str(&text).context("reading moved-in.json")?),
        Err(_) => None,
    };
    make_tokens(home, project, out).await?;
    let Some(moved) = moved else {
        // A first or plain start still gets its bridge; a mail problem never fails the start.
        let _ = mail_step(mail, None, project, out);
        return Ok(());
    };

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
    if !problems.is_empty() {
        return Err(anyhow::anyhow!(
            "{project} is running, but some scheduled messages were not carried: fix it and run `bridle project start` again"
        )
        .into());
    }
    // After the tokens: the bridge needs its `mail` token to start.
    let mail_ok = mail_step(mail, moved.from.as_deref(), project, out);
    finish_moved_in(&note, mail_ok)?;
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
    fn a_mail_failure_is_printed_with_the_command_not_returned() {
        let mut out = Vec::new();
        let failing = |_: Option<&str>, _: &mut dyn Write| Err("no mail token".to_string());
        mail_step(&failing, None, "acme", &mut out);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("no mail token"), "{text}");
        assert!(
            text.contains("bridle mail install --project acme"),
            "{text}"
        );
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
            relation: None,
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
    fn a_failed_mail_step_keeps_moved_in_and_a_success_removes_it() {
        let tmp = tempfile::tempdir().unwrap();
        let note = tmp.path().join(MOVED_IN);
        std::fs::write(&note, "{}").unwrap();
        let failing = |_: Option<&str>, _: &mut dyn Write| Err("no key".to_string());
        let mut out = Vec::new();
        let ok = mail_step(&failing, Some("old"), "acme", &mut out);
        assert!(!ok);
        finish_moved_in(&note, ok).unwrap();
        assert!(note.exists());
        let working = |_: Option<&str>, _: &mut dyn Write| Ok(());
        let ok = mail_step(&working, Some("old"), "acme", &mut out);
        finish_moved_in(&note, ok).unwrap();
        assert!(!note.exists());
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
        note_moved_in(&ws, None).unwrap();
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

    async fn send(store: &Store) -> String {
        store
            .outbox_enqueue(bridle_daemon::store::OutboxRow {
                id: String::new(),
                project: "other".into(),
                from: "agent:a".parse().expect("principal"),
                to: "human".into(),
                kind: bridle_api::types::MessageKind::System,
                body: "hi".into(),
                reply_to: None,
                when: bridle_api::types::When::Now,
                attempts: 0,
                last_error: None,
            })
            .await
            .expect("enqueue")
    }

    #[tokio::test]
    async fn a_return_trip_backs_up_the_old_database_and_seeds_above_it() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ws = Workspace::new(tmp.path().join("repo"), Some(tmp.path().to_path_buf()));
        std::fs::create_dir_all(ws.state_branch_dir()).expect("state dir");
        // OLD sent three messages before it stopped; NEW, a different machine, sent one.
        let old = Store::open(ws.db()).await.expect("old db");
        for _ in 0..3 {
            send(&old).await;
        }
        drop(old);
        std::fs::write(ws.state_branch_dir().join(OUTBOX_SEQ_FILE), "1").expect("seq");

        let backup = fresh_database(&ws).await.expect("fresh").expect("backup");

        assert!(backup.starts_with(ws.state_dir().join("db-backups")));
        assert!(backup.join("bridle.db").exists(), "old database kept");
        let fresh = Store::open(ws.db()).await.expect("fresh db");
        assert_eq!(send(&fresh).await, "o-0004", "above OLD's last id");
    }

    #[tokio::test]
    async fn a_running_daemon_blocks_setting_the_database_aside() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ws = Workspace::new(tmp.path().join("repo"), Some(tmp.path().to_path_buf()));
        std::fs::create_dir_all(ws.state_branch_dir()).expect("state dir");
        drop(Store::open(ws.db()).await.expect("old db"));
        let info = bridle_api::types::DaemonInfo {
            project: "p".into(),
            workspace: ws.workspace.display().to_string(),
            repo: ws.repo.display().to_string(),
            url: "http://127.0.0.1:1".into(),
            pid: std::process::id() as i32,
            started_at: Utc::now(),
            version: "t".into(),
            task_prefix: None,
        };
        bridle_api::discovery::write_daemon_json(&ws.workspace, &info).expect("daemon.json");

        let err = fresh_database(&ws).await.expect_err("refused");
        assert!(err.to_string().contains("stop it first"), "{err}");
        assert!(
            ws.db().exists(),
            "the database stays where the daemon has it"
        );
    }

    #[tokio::test]
    async fn a_first_visit_has_nothing_to_back_up_and_seeds_from_the_stop() {
        let tmp = tempfile::tempdir().expect("tmp");
        let ws = Workspace::new(tmp.path().join("repo"), Some(tmp.path().to_path_buf()));
        std::fs::create_dir_all(ws.state_branch_dir()).expect("state dir");
        assert!(fresh_database(&ws).await.expect("fresh").is_none());
        assert!(!ws.db().exists(), "no seed, no database made here");

        std::fs::write(ws.state_branch_dir().join(OUTBOX_SEQ_FILE), "7").expect("seq");
        assert!(fresh_database(&ws).await.expect("fresh").is_none());
        let fresh = Store::open(ws.db()).await.expect("db");
        assert_eq!(send(&fresh).await, "o-0008");
    }
}
