//! `bridle project stop` (53gy B): the first half of moving a project. Stops the project's
//! agents, daemon and boot/restart units, pushes the state branch and the integration branch
//! confirming origin has both, and last pushes a `released` mark in `owner.toml`, which
//! `project take-over` requires. Any failure says so, exits non-zero and leaves the units as
//! they were. `--machine` runs it on another machine over ssh, the way `token pair` does.

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use bridle_api::Client;
use bridle_api::discovery::{bridle_home, list_registry};
use bridle_api::machines::{MachineMap, owner_names};
use bridle_daemon::config::Config;
use bridle_daemon::paths::Workspace;
use bridle_daemon::state_branch::{local_owner, release_project};
use bridle_daemon::store::{LeftBehind, active_schedules, left_behind};

use crate::cli::{Cli, ProjectStopArgs};
use crate::commands::{client_for, daemon::stop_daemon_with};
use crate::error::CliError;
use crate::token_pair::{Exec, Fleet, Runner, Where};

/// Runs `launchctl` or `systemctl`; the seam for the tests, which run neither.
pub trait Ctl {
    fn run(&self, prog: &str, args: &[String]) -> Result<(), String>;
}

pub struct RealCtl;

impl Ctl for RealCtl {
    fn run(&self, prog: &str, args: &[String]) -> Result<(), String> {
        let out = Command::new(prog)
            .args(args)
            .output()
            .map_err(|e| format!("cannot run {prog}: {e}"))?;
        if out.status.success() {
            return Ok(());
        }
        let err = String::from_utf8_lossy(&out.stderr);
        Err(err.lines().next().unwrap_or("failed").trim().to_string())
    }
}

/// One installed service unit of the project: the daemon's, or the mail bridge's.
#[derive(Debug, PartialEq, Eq)]
pub struct Unit {
    pub what: &'static str,
    /// The launchd label or systemd unit name.
    pub name: String,
    pub file: PathBuf,
}

/// The units this OS would have for `project` whose file is installed under `home`.
pub fn installed_units(os: &str, home: &Path, config_home: &Path, project: &str) -> Vec<Unit> {
    let all = match os {
        "macos" => ["", "mail."]
            .map(|infix| {
                let name = format!("dev.bridle.{infix}{project}");
                let file = home.join(format!("Library/LaunchAgents/{name}.plist"));
                (infix, name, file)
            })
            .to_vec(),
        "linux" => ["", "mail-"]
            .map(|infix| {
                let name = format!("bridle-{infix}{project}.service");
                let file = config_home.join("systemd/user").join(&name);
                (infix, name, file)
            })
            .to_vec(),
        _ => Vec::new(),
    };
    all.into_iter()
        .filter(|(_, _, file)| file.exists())
        .map(|(infix, name, file)| Unit {
            what: if infix.is_empty() {
                "daemon"
            } else {
                "mail bridge"
            },
            name,
            file,
        })
        .collect()
}

/// Unloads and disables each unit, leaving its file installed. The first failure is returned.
pub fn unload(os: &str, uid: &str, units: &[Unit], ctl: &dyn Ctl) -> Result<Vec<String>, String> {
    let strings = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut done = Vec::new();
    for u in units {
        if os == "macos" {
            let target = format!("gui/{uid}/{}", u.name);
            // bootout fails for a job that is not loaded; that is already the state wanted.
            if ctl.run("launchctl", &strings(&["print", &target])).is_ok() {
                ctl.run("launchctl", &strings(&["bootout", &target]))
                    .map_err(|e| format!("{}: launchctl bootout: {e}", u.name))?;
            }
            // Without this the plist loads again at the next login.
            ctl.run("launchctl", &strings(&["disable", &target]))
                .map_err(|e| format!("{}: launchctl disable: {e}", u.name))?;
        } else {
            ctl.run(
                "systemctl",
                &strings(&["--user", "disable", "--now", &u.name]),
            )
            .map_err(|e| format!("{}: systemctl disable: {e}", u.name))?;
        }
        done.push(format!(
            "{} unit {} unloaded and disabled (file kept: {})",
            u.what,
            u.name,
            u.file.display()
        ));
    }
    Ok(done)
}

fn git_out(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Work stop does not push: a dirty checkout or worktree, a branch origin lacks.
fn left_unpushed(repo: &Path, skip: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    if git_out(repo, &["status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|s| !s.trim().is_empty())
    {
        out.push("the main checkout has uncommitted changes".to_string());
    }
    let refs = git_out(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short)\t%(upstream)\t%(upstream:track)",
            "refs/heads",
        ],
    )
    .unwrap_or_default();
    let mut branches = Vec::new();
    for line in refs.lines() {
        let mut f = line.split('\t');
        let (name, upstream, track) = (
            f.next().unwrap_or(""),
            f.next().unwrap_or(""),
            f.next().unwrap_or(""),
        );
        if skip.contains(&name) {
            continue;
        }
        if upstream.is_empty() {
            branches.push(format!("{name} (not on origin)"));
        } else if track.contains("ahead") {
            branches.push(format!("{name} {track}"));
        }
    }
    let shown = branches.len().min(8);
    for b in &branches[..shown] {
        out.push(format!("branch {b}"));
    }
    if branches.len() > shown {
        out.push(format!(
            "... and {} more unpushed branches",
            branches.len() - shown
        ));
    }
    let trees = git_out(repo, &["worktree", "list", "--porcelain"]).unwrap_or_default();
    for path in trees
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .skip(1)
    {
        let path = Path::new(path);
        if path.ends_with(".bridle/state") {
            continue;
        }
        if git_out(path, &["status", "--porcelain", "--untracked-files=no"])
            .is_some_and(|s| !s.trim().is_empty())
        {
            out.push(format!(
                "worktree {} has uncommitted changes",
                path.display()
            ));
        }
    }
    out
}

/// The listing a stop shows before it asks.
pub fn listing(
    project: &str,
    integration: &str,
    agents: Option<u32>,
    units: &[Unit],
    left: Option<&LeftBehind>,
    unpushed: &[String],
) -> String {
    let mut s = format!("Stopping {project} will:\n");
    match agents {
        Some(n) => {
            s += &format!(
                "  stop its daemon and {n} running agent(s); their Claude sessions do not resume on another machine\n"
            )
        }
        None => s += "  find no running daemon here (nothing to stop but its units)\n",
    }
    for u in units {
        s += &format!(
            "  unload and disable the {} unit {} (the file stays)\n",
            u.what, u.name
        );
    }
    s += &format!(
        "  push bridle/state and {integration}, confirm origin has both, then push a `released` mark\n"
    );
    if let Some(l) = left {
        s += "Left on this machine (bridle.db does not move; tokens are re-created with `bridle token pair`):\n";
        if l.undelivered_mail > 0 {
            s += &format!(
                "  {} UNDELIVERED message(s) in the outbox: they are not delivered once the daemon is gone. \
                 Answer no to stop here and let them drain first.\n",
                l.undelivered_mail
            );
        }
        s += &format!(
            "  {} unread message(s), {} active scheduled message(s), {} agent record(s), {} event(s)\n",
            l.unread_messages, l.active_schedules, l.agents, l.events
        );
    }
    if !unpushed.is_empty() {
        s += "Not pushed by stop:\n";
        for u in unpushed {
            s += &format!("  {u}\n");
        }
    }
    s
}

fn confirm(out: &mut dyn Write, yes: bool) -> Result<(), CliError> {
    if yes {
        return Ok(());
    }
    if !std::io::stdin().is_terminal() {
        return Err(
            anyhow::anyhow!("nothing was stopped: not a terminal; pass --yes to go ahead").into(),
        );
    }
    let _ = write!(out, "Continue? [y/N] ");
    let _ = out.flush();
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("reading the answer")?;
    if matches!(line.trim(), "y" | "Y" | "yes") {
        Ok(())
    } else {
        Err(anyhow::anyhow!("nothing was stopped").into())
    }
}

pub async fn run(cli: &Cli, args: &ProjectStopArgs) -> Result<(), CliError> {
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

/// Lists, confirms and stops on another machine: its `--list` output is shown here, the
/// confirmation is asked here, and the stop itself runs there with `--yes`.
pub fn remote(
    runner: &dyn Runner,
    at: &Where,
    project: &str,
    args: &ProjectStopArgs,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let mut base = vec![
        "projects".to_string(),
        "stop".into(),
        "--project".into(),
        project.into(),
    ];
    if let Some(r) = &args.repo {
        base.extend(["--repo".into(), r.display().to_string()]);
    }
    for l in &args.leave {
        base.extend(["--leave".into(), l.clone()]);
    }
    let mut ask = base.clone();
    ask.push("--list".into());
    let listing = runner
        .run(at, &ask, None)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let _ = write!(out, "{listing}");
    if args.list {
        return Ok(());
    }
    confirm(out, args.yes)?;
    base.push("--yes".into());
    let done = runner
        .run(at, &base, None)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let _ = write!(out, "{done}");
    Ok(())
}

async fn local(
    cli: &Cli,
    args: &ProjectStopArgs,
    home: &Path,
    ctl: &dyn Ctl,
) -> Result<(), CliError> {
    let running = cli
        .project
        .as_deref()
        .and_then(|p| list_registry().into_iter().find(|d| d.project == p));
    let repo = match (&args.repo, &running) {
        (Some(r), _) => r.clone(),
        (None, Some(d)) => PathBuf::from(&d.repo),
        (None, None) => std::env::current_dir().context("current directory")?,
    }
    .canonicalize()
    .context("repo path")?;
    let project = cli.project.clone().unwrap_or_else(|| {
        repo.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    let running = running.or_else(|| list_registry().into_iter().find(|d| d.project == project));
    let names = owner_names(home);
    match local_owner(&repo).await {
        Some(o) if names.contains(&o.host) => {}
        Some(o) => {
            return Err(anyhow::anyhow!(
                "{project} is owned by {}, not this machine ({}); nothing to stop here",
                o.host,
                names[0]
            )
            .into());
        }
        None => {
            return Err(anyhow::anyhow!(
                "{project} has no owner recorded on bridle/state in {}; nothing to stop",
                repo.display()
            )
            .into());
        }
    }
    let integration = Config::load(&repo)
        .context("loading .bridle/config.toml")?
        .branches
        .integration;
    let ws = Workspace::new(repo.clone(), args.workspace.clone());
    let os = std::env::consts::OS;
    let home_dir = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(".config"));
    let units = installed_units(os, &home_dir, &config_home, &project);

    // The drain stops idle agents; so does this, so a move that skipped or outran it still works.
    if running.is_some()
        && !args.list
        && let Ok(client) = client_for(cli).await
    {
        for name in crate::project_drain::stop_idle_agents(&client).await {
            println!("stopped idle agent {name}");
        }
    }
    let agents = match &running {
        Some(d) => Some(
            Client::new_with_timeout(d.url.clone(), None, std::time::Duration::from_secs(2))
                .health()
                .await
                .map(|h| h.agent_count)
                .unwrap_or(0),
        ),
        None => None,
    };
    let (unstable, leave_listed) =
        crate::stable::unstable(&repo, &ws.db(), &integration, &args.leave, true);
    let left = left_behind(&ws.db());
    let unpushed = left_unpushed(&repo, &["bridle/state", &integration]);
    let mut out = std::io::stdout();
    let _ = write!(
        out,
        "{}",
        listing(
            &project,
            &integration,
            agents,
            &units,
            left.as_ref(),
            &unpushed
        )
    );
    if !leave_listed.is_empty() {
        let _ = writeln!(out, "LEFT in the checkout: {}", leave_listed.join(", "));
    }
    if !unstable.is_empty() {
        let list: String = unstable.iter().map(|u| format!("\n  {u}")).collect();
        return Err(anyhow::anyhow!(
            "{project} is not stable; nothing was stopped (land or drop the work first):{list}"
        )
        .into());
    }
    if args.list {
        return Ok(());
    }
    confirm(&mut out, args.yes)?;

    if running.is_some() {
        let client = client_for(cli).await?;
        stop_daemon_with(
            &client,
            &mut out,
            std::time::Duration::from_secs(60),
            std::time::Duration::from_millis(500),
        )
        .await?;
    }
    let schedules = serde_json::to_string_pretty(&active_schedules(&ws.db()))
        .context("encoding the scheduled messages")?;
    let outbox_seq = match ws.db().exists() {
        true => bridle_daemon::store::Store::open(ws.db())
            .await
            .context("opening the database to read the outbox counter")?
            .outbox_high_water()
            .await
            .context("reading the outbox counter")?,
        false => 0,
    };
    release_project(
        &repo,
        &ws.state_branch_dir(),
        &integration,
        &names,
        &schedules,
        outbox_seq,
    )
    .await
    .map_err(|e| {
        CliError::Other(e.context(format!(
            "{project} is stopped but NOT released: origin may lack its latest work and its \
                 units were left as they were; fix the cause and run `bridle project stop` again"
        )))
    })?;
    let _ = writeln!(
        out,
        "origin has bridle/state and {integration}; released mark pushed"
    );
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let done = unload(os, &uid, &units, ctl).map_err(|e| {
        anyhow::anyhow!(
            "{project} is released and pushed, but a unit could not be unloaded ({e}); \
             unload it by hand, or run `bridle project stop` again"
        )
    })?;
    for line in done {
        let _ = writeln!(out, "{line}");
    }
    for line in retire(&repo, &ws, &project, home).await? {
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(
        out,
        "{project} is stopped and released; take it over on the new machine"
    );
    Ok(())
}

/// After a successful stop (kx5u K5): this machine can never serve the project again. Lists the
/// clone in `[machine] tools_only`, drops the registry entry and removes the upgrade worktree and
/// `upgrade-*` tree. `bridle.db` stays: events and usage can still be read there. Running it
/// again changes nothing.
async fn retire(
    repo: &Path,
    ws: &Workspace,
    project: &str,
    home: &Path,
) -> anyhow::Result<Vec<String>> {
    let mut lines = Vec::new();
    if crate::tools_only::add_entry(repo, home)? {
        lines.push(format!(
            "{} added to [machine] tools_only in {}: `bridle serve` refuses it here now",
            repo.display(),
            home.join("config.toml").display()
        ));
    }
    bridle_api::discovery::remove_registry(project).context("removing the registry entry")?;
    let state = ws.state_dir();
    if let Ok(rd) = std::fs::read_dir(&state) {
        let mut found = false;
        for e in rd.flatten() {
            if !e.file_name().to_string_lossy().starts_with("upgrade-") {
                continue;
            }
            found = true;
            let path = e.path();
            let _ = bridle_daemon::worktree::remove(repo, &path, true).await;
            if path.is_dir() {
                std::fs::remove_dir_all(&path)
                    .with_context(|| format!("removing {}", path.display()))?;
            } else {
                std::fs::remove_file(&path)
                    .with_context(|| format!("removing {}", path.display()))?;
            }
        }
        if found {
            let _ = bridle_daemon::worktree::prune(repo).await;
            lines.push("upgrade worktree and build tree removed".to_string());
        }
    }
    if ws.db().exists() {
        lines.push(format!(
            "{} stays here: events and usage can still be read there",
            ws.db().display()
        ));
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Rec(RefCell<Vec<String>>, Option<&'static str>);

    impl Ctl for Rec {
        fn run(&self, prog: &str, args: &[String]) -> Result<(), String> {
            let line = format!("{prog} {}", args.join(" "));
            self.0.borrow_mut().push(line.clone());
            match self.1 {
                Some(fail) if line.contains(fail) => Err("refused".into()),
                _ => Ok(()),
            }
        }
    }

    fn files(os: &str) -> (tempfile::TempDir, Vec<Unit>) {
        let dir = tempfile::tempdir().unwrap();
        let (sub, names): (&str, [&str; 2]) = if os == "macos" {
            (
                "Library/LaunchAgents",
                ["dev.bridle.p.plist", "dev.bridle.mail.p.plist"],
            )
        } else {
            (
                ".config/systemd/user",
                ["bridle-p.service", "bridle-mail-p.service"],
            )
        };
        std::fs::create_dir_all(dir.path().join(sub)).unwrap();
        for n in names {
            std::fs::write(dir.path().join(sub).join(n), "x").unwrap();
        }
        let units = installed_units(os, dir.path(), &dir.path().join(".config"), "p");
        (dir, units)
    }

    #[tokio::test]
    async fn retire_closes_the_old_machine_twice_the_same_and_keeps_the_database() {
        let h = tempfile::tempdir().unwrap();
        let repo = tempfile::tempdir().unwrap();
        let wsdir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(repo.path().to_path_buf(), Some(wsdir.path().to_path_buf()));
        std::fs::create_dir_all(ws.state_dir().join("upgrade-target/x")).unwrap();
        std::fs::write(ws.db(), "db").unwrap();
        let first = retire(repo.path(), &ws, "p-none", h.path()).await.unwrap();
        assert!(first.iter().any(|l| l.contains("tools_only")));
        assert!(!ws.state_dir().join("upgrade-target").exists());
        assert!(ws.db().exists());
        assert!(crate::tools_only::refuse_serve_in(repo.path(), h.path()).is_err());
        let second = retire(repo.path(), &ws, "p-none", h.path()).await.unwrap();
        assert!(!second.iter().any(|l| l.contains("tools_only")));
        assert!(ws.db().exists());
    }

    #[test]
    fn only_installed_units_are_found_and_the_files_stay() {
        let (dir, units) = files("linux");
        assert_eq!(units.len(), 2);
        assert!(installed_units("linux", dir.path(), &dir.path().join("none"), "p").is_empty());
        let ctl = Rec(RefCell::default(), None);
        unload("linux", "501", &units, &ctl).unwrap();
        assert_eq!(
            *ctl.0.borrow(),
            [
                "systemctl --user disable --now bridle-p.service",
                "systemctl --user disable --now bridle-mail-p.service"
            ]
        );
        assert!(units.iter().all(|u| u.file.exists()));
    }

    #[test]
    fn launchd_boots_out_a_loaded_job_and_disables_it() {
        let (_dir, units) = files("macos");
        let ctl = Rec(RefCell::default(), None);
        unload("macos", "501", &units[..1], &ctl).unwrap();
        assert_eq!(
            *ctl.0.borrow(),
            [
                "launchctl print gui/501/dev.bridle.p",
                "launchctl bootout gui/501/dev.bridle.p",
                "launchctl disable gui/501/dev.bridle.p"
            ]
        );
        // Not loaded: no bootout, still disabled.
        let ctl = Rec(RefCell::default(), Some("print"));
        unload("macos", "501", &units[..1], &ctl).unwrap();
        assert_eq!(ctl.0.borrow().len(), 2);
    }

    #[test]
    fn an_unload_failure_is_returned() {
        let (_dir, units) = files("linux");
        let err = unload(
            "linux",
            "1",
            &units,
            &Rec(RefCell::default(), Some("disable")),
        )
        .unwrap_err();
        assert!(err.contains("bridle-p.service"), "{err}");
    }

    #[test]
    fn the_listing_names_undelivered_mail_and_what_is_not_pushed() {
        let left = LeftBehind {
            undelivered_mail: 3,
            ..Default::default()
        };
        let text = listing(
            "p",
            "main",
            Some(2),
            &[],
            Some(&left),
            &["branch x (not on origin)".to_string()],
        );
        assert!(text.contains("2 running agent(s)"), "{text}");
        assert!(text.contains("3 UNDELIVERED"), "{text}");
        assert!(text.contains("branch x (not on origin)"), "{text}");
    }

    struct Fake(RefCell<Vec<Vec<String>>>);

    impl Runner for Fake {
        fn run(&self, _: &Where, args: &[String], _: Option<&str>) -> Result<String, String> {
            self.0.borrow_mut().push(args.to_vec());
            Ok("ok\n".into())
        }
    }

    #[test]
    fn a_remote_stop_lists_first_and_stops_with_yes() {
        let fake = Fake(RefCell::default());
        let args = ProjectStopArgs {
            repo: None,
            workspace: None,
            yes: true,
            list: false,
            machine: Some("m".into()),
            leave: vec![],
        };
        let mut out = Vec::new();
        remote(&fake, &Where::Ssh("h".into()), "p", &args, &mut out).unwrap();
        let calls = fake.0.borrow();
        assert!(calls[0].contains(&"--list".to_string()));
        assert!(
            calls[1].contains(&"--yes".to_string()) && !calls[1].contains(&"--list".to_string())
        );
    }
}
