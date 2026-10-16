//! `bridle project setup` (br-xtgf, epic br-dffz option C): one command that brings a project
//! with a clone to fully set up on this machine. A fixed list of steps, each with a `check` that
//! reads the real state and an `apply` that calls the existing command's code (`projects add`,
//! `project start`, `token pair`, `mail install`). The checklist is printed first, the unmet
//! steps are applied in order, and the first failure stops the run with the fix. Nothing is
//! stored: a second run reads the same state and does only what is missing.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bridle_api::Client;
use bridle_api::discovery::{bridle_home, list_registry};
use bridle_api::machines::{MachineMap, owner_names};
use bridle_daemon::paths::Workspace;
use bridle_daemon::state_branch::{Owner, is_new_project, local_owner};

use crate::cli::{Cli, ProjectAddArgs, ProjectSetupArgs};
use crate::error::CliError;
use crate::mail_install::{MAIL_OFF, MailState, mail_state};
use crate::project_stop::{Ctl, RealCtl};
use crate::project_sync::{Real, add, sync, sync_counting};
use crate::token_pair::{Exec, Fleet, Options, pair};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Done(String),
    /// Nothing to do on this machine, and why (mail off, not set up).
    Skipped(String),
    /// Missing; `apply` can make it.
    Unmet(String),
    /// Missing, and only the human can fix it: the run stops here with this text.
    Blocked(String),
    /// The state could not be read (a machine or origin unreachable): not done, not known missing.
    Unknown(String),
}

impl Status {
    fn tag(&self) -> &'static str {
        match self {
            Status::Done(_) => "ok",
            Status::Skipped(_) => "skip",
            Status::Unmet(_) => "todo",
            Status::Blocked(_) => "blocked",
            Status::Unknown(_) => "unknown",
        }
    }

    fn text(&self) -> &str {
        match self {
            Status::Done(t)
            | Status::Skipped(t)
            | Status::Unmet(t)
            | Status::Blocked(t)
            | Status::Unknown(t) => t,
        }
    }

    fn complete(&self) -> bool {
        matches!(self, Status::Done(_) | Status::Skipped(_))
    }
}

/// The steps, in the fixed order, as the driver sees them (a fake in the tests).
trait Steps {
    fn names(&self) -> Vec<&'static str>;
    async fn check(&mut self, i: usize) -> Status;
    /// Makes step `i`; the `Ok` text is its one-line result, the `Err` text the exact fix.
    async fn apply(&mut self, i: usize, out: &mut dyn Write) -> Result<String, String>;
}

/// Runs the checklist and, unless `check_only`, applies what is unmet. `Err` carries the exit 1.
async fn drive<S: Steps>(
    steps: &mut S,
    project: &str,
    check_only: bool,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let names = steps.names();
    let mut statuses = Vec::new();
    for i in 0..names.len() {
        statuses.push(steps.check(i).await);
    }
    let _ = writeln!(out, "{project}:");
    for (n, s) in names.iter().zip(&statuses) {
        let _ = writeln!(out, "  {n} ... {}: {}", s.tag(), s.text());
    }
    let incomplete = statuses.iter().filter(|s| !s.complete()).count();
    if check_only {
        return if incomplete == 0 {
            let _ = writeln!(out, "{project} is fully set up");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "{project} is not fully set up ({incomplete} of {} steps); run `bridle project setup` to do the rest",
                names.len()
            )
            .into())
        };
    }
    if incomplete == 0 {
        let _ = writeln!(out, "nothing to do: {project} is fully set up");
        return Ok(());
    }
    for (i, name) in names.iter().enumerate() {
        match &statuses[i] {
            Status::Unmet(_) => {}
            Status::Blocked(why) => return Err(stop(name, why)),
            _ => continue,
        }
        let _ = writeln!(out, "{name}:");
        match steps.apply(i, out).await {
            Ok(done) => {
                let _ = writeln!(out, "  {name} ... ok: {done}");
            }
            Err(fix) => return Err(stop(name, &fix)),
        }
    }
    let unknown: Vec<String> = names
        .iter()
        .zip(&statuses)
        .filter(|(_, s)| matches!(s, Status::Unknown(_)))
        .map(|(n, s)| format!("{n} ({})", s.text()))
        .collect();
    if unknown.is_empty() {
        let _ = writeln!(out, "Done: {project} is fully set up");
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "applied what was missing, but could not tell: {}; fix that, then run the same command again",
            unknown.join(", ")
        )
        .into())
    }
}

fn stop(step: &str, fix: &str) -> CliError {
    anyhow::anyhow!("stopped at {step}: {fix}\nthen run the same command again").into()
}

// The step checks, as pure functions of what was read, so each can be tested.

/// `placed` is the project's `[projects]` entry here; `sync` is a read-only `projects sync`.
fn register_status(placed: Option<(String, u16)>, sync: (bool, usize)) -> Status {
    let Some((machine, port)) = placed else {
        return Status::Unmet("not in [projects]; will add it with the next free port".into());
    };
    match sync {
        (false, _) => Status::Unknown(format!(
            "{machine}:{port} is listed here, but some machines could not be read (see `bridle projects sync`)"
        )),
        (true, 0) => Status::Done(format!("{machine}:{port} on every machine")),
        (true, n) => Status::Unmet(format!(
            "{machine}:{port}; {n} entries differ on other machines"
        )),
    }
}

/// `new` is `is_new_project`: neither a local nor a reachable origin `bridle/state`.
fn state_status(owner: Option<Owner>, names: &[String], new: Result<bool, String>) -> Status {
    match owner {
        Some(o) if names.contains(&o.host) && o.released.is_none() => {
            Status::Done(format!("owner {}", o.host))
        }
        Some(o) if names.contains(&o.host) => Status::Blocked(format!(
            "released by this machine ({}): run `bridle project take-over` to claim it again",
            o.host
        )),
        Some(o) => Status::Blocked(format!(
            "owned by {}, not this machine ({}): run `bridle project take-over` here first",
            o.host, names[0]
        )),
        None => match new {
            Ok(true) => Status::Unmet("new project: the daemon's first start creates it".into()),
            Ok(false) => Status::Blocked(
                "no owner recorded on bridle/state here: run `bridle project take-over` first"
                    .into(),
            ),
            Err(e) => Status::Unknown(e.lines().next().unwrap_or_default().to_string()),
        },
    }
}

fn service_status(has_unit: bool, up: bool) -> Status {
    match (has_unit, up) {
        (true, true) => Status::Done("unit installed, daemon answering".into()),
        (true, false) => Status::Unmet("unit installed, daemon not answering".into()),
        (false, _) => Status::Unmet("no unit installed".into()),
    }
}

/// `paired` and `report` are a dry-run `token pair` for this project: `Ok(false)` is a failure or
/// an unreachable machine, and a "would mint" line is a token still to make.
fn tokens_status(paired: Result<bool, String>, report: &str) -> Status {
    match paired {
        Err(e) => Status::Unknown(e),
        Ok(false) => {
            Status::Unknown("a machine could not be read (see `bridle token pair`)".into())
        }
        Ok(true) if report.contains("would mint") => {
            Status::Unmet("tokens missing on some machine".into())
        }
        Ok(true) => Status::Done("every token present".into()),
    }
}

fn mail_status(state: MailState, in_mail_projects: bool, has_bridge: bool) -> Status {
    match state {
        MailState::Off => Status::Skipped(MAIL_OFF.into()),
        MailState::NotConfigured => Status::Skipped("no [mail] section in config.toml".into()),
        MailState::NotSetUp(why) => Status::Skipped(format!("this machine is not set up ({why})")),
        MailState::Ready if !in_mail_projects => {
            Status::Skipped("project is not in [mail] projects".into())
        }
        MailState::Ready if has_bridge => Status::Done("bridge unit installed".into()),
        MailState::Ready => Status::Unmet("bridge unit not installed".into()),
    }
}

struct Real5<'a> {
    cli: &'a Cli,
    ctl: &'a dyn Ctl,
    home: PathBuf,
    sys: crate::project_start::Sys,
    project: String,
    repo: PathBuf,
    ws: Workspace,
}

const STEPS: [&str; 5] = ["register", "state", "service", "tokens", "mail"];

impl Real5<'_> {
    fn fleet(&self) -> Result<Fleet, String> {
        let map = MachineMap::load(&self.home).map_err(|e| e.to_string())?;
        Ok(Fleet::from_config(&map))
    }

    async fn daemon_up(&self) -> bool {
        let Some(d) = list_registry()
            .into_iter()
            .find(|d| d.project == self.project)
        else {
            return false;
        };
        Client::new_with_timeout(d.url, None, Duration::from_secs(2))
            .health()
            .await
            .is_ok()
    }
}

impl Steps for Real5<'_> {
    fn names(&self) -> Vec<&'static str> {
        STEPS.to_vec()
    }

    async fn check(&mut self, i: usize) -> Status {
        match i {
            0 => {
                let fleet = match self.fleet() {
                    Ok(f) => f,
                    Err(e) => return Status::Unknown(format!("config.toml: {e}")),
                };
                let placed = fleet.placed.get(&self.project).cloned();
                let counted = if placed.is_some() {
                    let home = self.home.clone();
                    // `sync` shells out to ssh: keep it off the runtime.
                    tokio::task::spawn_blocking(move || {
                        sync_counting(&fleet, &Real { home }, false, &mut std::io::sink())
                    })
                    .await
                    .unwrap_or((false, 0))
                } else {
                    (true, 0)
                };
                register_status(placed, counted)
            }
            1 => state_status(
                local_owner(&self.repo).await,
                &owner_names(&self.home),
                is_new_project(&self.repo).await,
            ),
            2 => service_status(
                self.sys
                    .units(&self.project)
                    .iter()
                    .any(|u| u.what == "daemon"),
                self.daemon_up().await,
            ),
            3 => {
                if !self.daemon_up().await {
                    return Status::Unmet("the daemon is not running yet".into());
                }
                let Ok(fleet) = self.fleet() else {
                    return Status::Unknown("config.toml unreadable".into());
                };
                let opts = Options {
                    projects: vec![self.project.clone()],
                    dry_run: true,
                    ..Default::default()
                };
                let (paired, report) = tokio::task::spawn_blocking(move || {
                    let mut buf = Vec::new();
                    let r = pair(&opts, &fleet, &Exec, &mut buf);
                    (r, String::from_utf8_lossy(&buf).into_owned())
                })
                .await
                .unwrap_or((Err("token check did not finish".into()), String::new()));
                tokens_status(paired, &report)
            }
            _ => {
                let text =
                    std::fs::read_to_string(self.home.join("config.toml")).unwrap_or_default();
                mail_status(
                    mail_state(&self.home, &self.sys.home_dir, false),
                    crate::project_mail::mails(&text, &self.project),
                    self.sys
                        .units(&self.project)
                        .iter()
                        .any(|u| u.what == "mail bridge"),
                )
            }
        }
    }

    async fn apply(&mut self, i: usize, out: &mut dyn Write) -> Result<String, String> {
        match i {
            0 => {
                let fleet = self.fleet()?;
                if fleet.placed.contains_key(&self.project) {
                    let home = self.home.clone();
                    let ok = tokio::task::spawn_blocking(move || {
                        sync(&fleet, &Real { home }, true, &mut std::io::stdout())
                    })
                    .await
                    .map_err(|e| e.to_string())?;
                    if !ok {
                        return Err("some machines could not be read or written; fix that or run `bridle projects sync --apply`".into());
                    }
                } else {
                    let args = ProjectAddArgs {
                        name: self.project.clone(),
                        machine: None,
                        port: None,
                    };
                    tokio::task::block_in_place(|| add(&args)).map_err(|e| e.to_string())?;
                }
                Ok("registered and synced".into())
            }
            // The daemon's first start makes `bridle/state` and claims the owner (serve).
            1 => Ok("left to the daemon's first start (next step)".into()),
            2 => {
                crate::project_start::start_service(
                    self.cli,
                    &self.sys,
                    &self.project,
                    &self.repo,
                    &self.ws,
                    self.ctl,
                    out,
                )
                .await
                .map_err(|e| e.to_string())?;
                Ok("daemon service installed and running".into())
            }
            3 => {
                crate::project_start::make_tokens(&self.home, &self.project, out)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("tokens present on every machine".into())
            }
            _ => {
                crate::project_start::bring_mail(
                    self.cli,
                    &self.sys,
                    self.ctl,
                    &self.home,
                    &self.repo,
                    &self.project,
                    None,
                    out,
                )
                .map_err(|e| {
                    format!(
                        "{e}; fix it, then run `bridle mail install --project {}`",
                        self.project
                    )
                })?;
                Ok("mail bridge installed".into())
            }
        }
    }
}

/// The clone: `--repo`, else the running daemon's for `name`, else the current directory.
fn repo_for(args: &ProjectSetupArgs, name: Option<&str>) -> anyhow::Result<PathBuf> {
    use anyhow::Context;
    let repo = match (&args.repo, name) {
        (Some(r), _) => r.clone(),
        (None, Some(n)) => match list_registry().into_iter().find(|d| d.project == n) {
            Some(d) => PathBuf::from(d.repo),
            None if Path::new(".git").exists() => std::env::current_dir()?,
            None => anyhow::bail!(
                "no running daemon for {n} to find its clone: run this inside the clone, or pass --repo"
            ),
        },
        (None, None) => std::env::current_dir().context("current directory")?,
    };
    repo.canonicalize()
        .with_context(|| format!("repo path {}", repo.display()))
}

pub async fn run(cli: &Cli, args: &ProjectSetupArgs) -> Result<(), CliError> {
    let name = args.name.clone().or_else(|| cli.project.clone());
    let repo = repo_for(args, name.as_deref())?;
    let project = name.unwrap_or_else(|| {
        repo.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    let mut steps = Real5 {
        cli,
        ctl: &RealCtl,
        home: bridle_home(),
        sys: crate::project_start::Sys::current(),
        project: project.clone(),
        ws: Workspace::new(repo.clone(), args.workspace.clone()),
        repo,
    };
    drive(&mut steps, &project, args.check, &mut std::io::stdout()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner(host: &str, released: bool) -> Option<Owner> {
        Some(Owner {
            host: host.into(),
            since: "2026-10-16T00:00:00Z".into(),
            released: released.then(|| "2026-10-16T01:00:00Z".to_string()),
        })
    }

    fn names() -> Vec<String> {
        vec!["dalek".to_string(), "dalek.local".to_string()]
    }

    #[test]
    fn register_reads_the_entry_and_the_sync() {
        let here = Some(("dalek".to_string(), 7409));
        assert!(matches!(register_status(None, (true, 0)), Status::Unmet(_)));
        assert!(matches!(
            register_status(here.clone(), (true, 0)),
            Status::Done(t) if t.contains("dalek:7409")
        ));
        assert!(matches!(
            register_status(here.clone(), (true, 2)),
            Status::Unmet(t) if t.contains("2 entries")
        ));
        assert!(matches!(
            register_status(here, (false, 0)),
            Status::Unknown(_)
        ));
    }

    #[test]
    fn state_follows_the_owner_the_way_project_start_does() {
        assert!(matches!(
            state_status(owner("dalek", false), &names(), Ok(false)),
            Status::Done(_)
        ));
        assert!(matches!(
            state_status(owner("dalek", true), &names(), Ok(false)),
            Status::Blocked(t) if t.contains("take-over")
        ));
        assert!(matches!(
            state_status(owner("remus", false), &names(), Ok(false)),
            Status::Blocked(t) if t.contains("owned by remus")
        ));
        assert!(matches!(
            state_status(None, &names(), Ok(true)),
            Status::Unmet(_)
        ));
        assert!(matches!(
            state_status(None, &names(), Ok(false)),
            Status::Blocked(_)
        ));
        assert!(matches!(
            state_status(None, &names(), Err("origin unreachable".into())),
            Status::Unknown(_)
        ));
    }

    #[test]
    fn service_needs_the_unit_and_an_answering_daemon() {
        assert!(matches!(service_status(true, true), Status::Done(_)));
        assert!(matches!(service_status(true, false), Status::Unmet(_)));
        assert!(matches!(service_status(false, false), Status::Unmet(_)));
    }

    #[test]
    fn tokens_are_missing_when_the_dry_run_would_mint() {
        assert!(matches!(
            tokens_status(Ok(true), "remus peer: kept 2\n"),
            Status::Done(_)
        ));
        assert!(matches!(
            tokens_status(Ok(true), "peer: would mint on the receiver for a -> b/p\n"),
            Status::Unmet(_)
        ));
        assert!(matches!(
            tokens_status(Ok(true), "would mint: orchestrator -> d/p\n"),
            Status::Unmet(_)
        ));
        assert!(matches!(tokens_status(Ok(false), ""), Status::Unknown(_)));
        assert!(matches!(
            tokens_status(Err("bad".into()), ""),
            Status::Unknown(_)
        ));
    }

    #[test]
    fn mail_is_skipped_unless_this_machine_does_it_for_this_project() {
        assert!(matches!(
            mail_status(MailState::Off, true, false),
            Status::Skipped(_)
        ));
        assert!(matches!(
            mail_status(MailState::NotConfigured, true, false),
            Status::Skipped(_)
        ));
        assert!(matches!(
            mail_status(MailState::NotSetUp("missing AWS credentials".into()), true, false),
            Status::Skipped(t) if t.contains("AWS")
        ));
        assert!(matches!(
            mail_status(MailState::Ready, false, false),
            Status::Skipped(_)
        ));
        assert!(matches!(
            mail_status(MailState::Ready, true, false),
            Status::Unmet(_)
        ));
        assert!(matches!(
            mail_status(MailState::Ready, true, true),
            Status::Done(_)
        ));
    }

    /// A fake: each step is `todo` until applied, then `ok`; `fail` makes one apply fail.
    struct Fake {
        state: Vec<Status>,
        fail: Option<usize>,
        applied: Vec<usize>,
    }

    impl Fake {
        fn new(state: Vec<Status>) -> Self {
            Fake {
                state,
                fail: None,
                applied: Vec::new(),
            }
        }
    }

    impl Steps for Fake {
        fn names(&self) -> Vec<&'static str> {
            vec!["one", "two", "three"]
        }
        async fn check(&mut self, i: usize) -> Status {
            self.state[i].clone()
        }
        async fn apply(&mut self, i: usize, _out: &mut dyn Write) -> Result<String, String> {
            if self.fail == Some(i) {
                return Err("run `fix-it`".into());
            }
            self.applied.push(i);
            self.state[i] = Status::Done("made".into());
            Ok("made".into())
        }
    }

    fn done() -> Status {
        Status::Done("there".into())
    }
    fn todo() -> Status {
        Status::Unmet("missing".into())
    }

    async fn go(f: &mut Fake, check_only: bool) -> (Result<(), CliError>, String) {
        let mut out = Vec::new();
        let r = drive(f, "p", check_only, &mut out).await;
        (r, String::from_utf8(out).expect("utf8"))
    }

    #[tokio::test]
    async fn check_prints_the_checklist_and_changes_nothing() {
        let mut f = Fake::new(vec![done(), todo(), Status::Unknown("no ssh".into())]);
        let (r, text) = go(&mut f, true).await;
        assert!(f.applied.is_empty());
        assert_eq!(r.expect_err("incomplete").exit_code(), 1);
        assert!(text.contains("one ... ok: there"), "{text}");
        assert!(text.contains("two ... todo: missing"), "{text}");
        assert!(text.contains("three ... unknown: no ssh"), "{text}");
    }

    #[tokio::test]
    async fn check_of_a_finished_project_exits_zero() {
        let mut f = Fake::new(vec![done(), Status::Skipped("off".into()), done()]);
        let (r, text) = go(&mut f, true).await;
        r.expect("complete");
        assert!(text.contains("fully set up"), "{text}");
    }

    #[tokio::test]
    async fn setup_applies_the_unmet_steps_in_order_and_a_second_run_does_nothing() {
        let mut f = Fake::new(vec![todo(), done(), todo()]);
        let (r, text) = go(&mut f, false).await;
        r.expect("done");
        assert_eq!(f.applied, vec![0, 2]);
        assert!(text.contains("Done: p is fully set up"), "{text}");
        let (r, text) = go(&mut f, false).await;
        r.expect("nothing to do");
        assert_eq!(f.applied, vec![0, 2], "second run applied nothing");
        assert!(text.contains("nothing to do"), "{text}");
    }

    #[tokio::test]
    async fn the_first_failure_stops_with_the_fix_and_the_later_steps_wait() {
        let mut f = Fake::new(vec![todo(), todo(), todo()]);
        f.fail = Some(1);
        let (r, _) = go(&mut f, false).await;
        let msg = r.expect_err("stopped").to_string();
        assert!(msg.contains("stopped at two"), "{msg}");
        assert!(msg.contains("run `fix-it`"), "{msg}");
        assert!(msg.contains("same command again"), "{msg}");
        assert_eq!(f.applied, vec![0]);
    }

    #[tokio::test]
    async fn a_blocked_step_stops_before_anything_after_it() {
        let mut f = Fake::new(vec![todo(), Status::Blocked("take it over".into()), todo()]);
        let (r, _) = go(&mut f, false).await;
        let msg = r.expect_err("blocked").to_string();
        assert!(msg.contains("take it over"), "{msg}");
        assert_eq!(f.applied, vec![0]);
    }

    #[tokio::test]
    async fn an_unknown_step_is_not_done_and_the_run_exits_non_zero() {
        let mut f = Fake::new(vec![
            todo(),
            done(),
            Status::Unknown("remus: no answer".into()),
        ]);
        let (r, _) = go(&mut f, false).await;
        let msg = r.expect_err("unknown").to_string();
        assert!(
            msg.contains("could not tell") && msg.contains("remus: no answer"),
            "{msg}"
        );
        assert_eq!(f.applied, vec![0], "the known-unmet step was still made");
    }
}
