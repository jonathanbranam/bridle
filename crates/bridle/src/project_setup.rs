//! `bridle project setup` (br-xtgf, epic br-dffz option C): one command that brings a project
//! with a clone to fully set up on this machine. A fixed list of steps, each with a `check` that
//! reads the real state and an `apply` that calls the existing command's code (`projects add`,
//! `project start`, `token pair`, `mail install`). The checklist is printed first, the unmet
//! steps are applied in order, and the first failure stops the run with the fix. Nothing is
//! stored: a second run reads the same state and does only what is missing.

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use bridle_api::Client;
use bridle_api::discovery::{bridle_home, list_registry};
use bridle_api::machines::{MachineMap, owner_names};
use bridle_daemon::paths::Workspace;
use bridle_daemon::state_branch::{Owner, is_new_project, local_owner};

use crate::cli::{Cli, InitArgs, ProjectAddArgs, ProjectSetupArgs};
use crate::error::CliError;
use crate::mail_install::{MAIL_OFF, MailState, mail_state};
use crate::project_stop::{Ctl, RealCtl};
use crate::project_sync::{Real, add, sync, sync_counting};
use crate::token_pair::{Exec, Fleet, Options, TOKEN_ROLES, TokenType, pair};

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
    /// The existing command that makes step `i` by hand (the "next:" line).
    fn command(&self, i: usize) -> String;
    /// Asks what the unmet steps need answered, before any is applied. Missing answers with no
    /// terminal are an `Usage` error (exit 2) listing all of them.
    async fn questions(&mut self, _statuses: &[Status]) -> Result<(), CliError> {
        Ok(())
    }
}

/// The command for the first step that is not complete: the one thing to do next.
fn next_line<S: Steps>(steps: &S, statuses: &[Status]) -> Option<String> {
    let i = statuses.iter().position(|s| !s.complete())?;
    Some(format!("next: {}", steps.command(i)))
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
        if let Some(n) = next_line(steps, &statuses) {
            let _ = writeln!(out, "{n}");
        }
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
    steps.questions(&statuses).await?;
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

/// A project with commits: where bridle's own files go (rule existing-projects: the default is
/// a separate branch, never the project's own).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Main,
    Adopt,
}

const ADOPT: &str = "bridle-adopt";

/// Where an answer comes from: a flag was already applied by the caller; else `-y` takes the
/// default, else the terminal is asked, else it is reported missing (`None`).
struct Asker {
    yes: bool,
    terminal: bool,
    input: Box<dyn BufRead>,
    out: Box<dyn Write>,
}

impl Asker {
    /// Never answers: for the read-only checks behind `print_next`.
    fn none() -> Self {
        Asker {
            yes: false,
            terminal: false,
            input: Box::new(std::io::empty()),
            out: Box::new(std::io::sink()),
        }
    }

    fn ask(&mut self, question: &str, default: &str) -> Option<String> {
        if self.yes {
            let _ = writeln!(self.out, "{question} {default} (default)");
            return Some(default.to_string());
        }
        if !self.terminal {
            return None;
        }
        let _ = write!(self.out, "{question} [{default}] ");
        let _ = self.out.flush();
        let mut line = String::new();
        let _ = self.input.read_line(&mut line);
        let line = line.trim();
        Some(if line.is_empty() { default } else { line }.to_string())
    }
}

const CLONE: usize = 0;
const INIT: usize = 1;
const REGISTER: usize = 2;
const STATE: usize = 3;
const SERVICE: usize = 4;
const TOKENS: usize = 5;
const MAIL: usize = 6;
const AIDE: usize = 7;

const STEPS: [&str; 8] = [
    "clone",
    "init",
    "register",
    "state",
    "service",
    "tokens",
    "mail",
    "aide token",
];

struct Setup<'a> {
    cli: &'a Cli,
    ctl: &'a dyn Ctl,
    home: PathBuf,
    sys: crate::project_start::Sys,
    project: String,
    /// The clone's main checkout; for a clone still to make, where it will go.
    repo: PathBuf,
    workspace: Option<PathBuf>,
    cloned: bool,
    remote: Option<String>,
    /// The clone's place was given (`--root`) or is the clone's own: nothing to ask.
    root_given: bool,
    /// The repo (or the remote) has commits; read by the clone and init checks.
    commits: bool,
    on_adopt: bool,
    choice: Option<Choice>,
    /// Whether to make the aide token; asked when that step is unmet.
    aide: bool,
    asker: Asker,
}

impl<'a> Setup<'a> {
    fn new(
        cli: &'a Cli,
        project: &str,
        repo: PathBuf,
        remote: Option<String>,
        asker: Asker,
    ) -> Self {
        Setup {
            cli,
            ctl: &RealCtl,
            home: bridle_home(),
            sys: crate::project_start::Sys::current(),
            project: project.to_string(),
            cloned: repo.join(".git").exists(),
            repo,
            workspace: None,
            remote,
            root_given: true,
            commits: false,
            on_adopt: false,
            choice: None,
            aide: true,
            asker,
        }
    }
}

impl Setup<'_> {
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

    /// A dry-run `token pair` for this project, read as a status.
    async fn pair_status(&self, roles: Vec<String>, tokens: Vec<TokenType>) -> Status {
        if !self.daemon_up().await {
            return Status::Unmet("the daemon is not running yet".into());
        }
        let Ok(fleet) = self.fleet() else {
            return Status::Unknown("config.toml unreadable".into());
        };
        let opts = Options {
            projects: vec![self.project.clone()],
            roles,
            tokens,
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

    async fn check_clone(&mut self) -> Status {
        if self.cloned {
            let origin = git(&self.repo, &["remote", "get-url", "origin"]).await.ok();
            return match (&self.remote, origin) {
                (Some(want), Some(have)) if want.trim_end_matches('/') != have.trim() => {
                    Status::Blocked(format!(
                        "{} has origin {}, not {want}: pass the right URL or none",
                        self.repo.display(),
                        have.trim()
                    ))
                }
                _ => Status::Done(format!("clone at {}", self.repo.display())),
            };
        }
        let Some(url) = self.remote.clone() else {
            return Status::Blocked(
                "no clone here and no remote: make an empty repo for the project and pass its URL (`bridle project setup <URL>`)".into(),
            );
        };
        if std::fs::read_dir(&self.repo).is_ok_and(|mut d| d.next().is_some()) {
            return Status::Blocked(format!(
                "{} exists and is not empty: pass --root with another place, or clone into it yourself",
                self.repo.display()
            ));
        }
        match git(Path::new("."), &["ls-remote", "--heads", &url]).await {
            Ok(heads) => {
                self.commits = !heads.trim().is_empty();
                Status::Unmet(format!("clone {url} into {}", self.repo.display()))
            }
            Err(e) => Status::Blocked(format!("remote {url} is not reachable: {e}")),
        }
    }

    async fn check_init(&mut self) -> Status {
        if !self.cloned {
            return Status::Unmet("after the clone (if it has no .bridle/config.toml)".into());
        }
        if self.repo.join(".bridle/config.toml").is_file() {
            // A failed commit leaves the files made but not committed: not done.
            let dirty = git(
                &self.repo,
                &["status", "--porcelain", "--", ".bridle/config.toml"],
            )
            .await
            .is_ok_and(|o| !o.trim().is_empty());
            return init_status(dirty);
        }
        self.commits = git(&self.repo, &["rev-parse", "--verify", "-q", "HEAD"])
            .await
            .is_ok();
        self.on_adopt = git(&self.repo, &["symbolic-ref", "--short", "HEAD"])
            .await
            .is_ok_and(|b| b.trim() == ADOPT);
        Status::Unmet("no .bridle/config.toml".into())
    }

    /// Makes `.bridle/config.toml` and the workflow files and commits them, on `bridle-adopt`
    /// when the project has commits and adopt was chosen (the default).
    async fn apply_init(&mut self) -> Result<String, String> {
        let repo = self.repo.clone();
        if repo.join(".bridle/config.toml").is_file() {
            // Made by an earlier run whose commit failed: commit it, don't scaffold again.
            return self
                .commit_init(&repo, "already scaffolded; files committed")
                .await;
        }
        let adopt = self.commits && self.choice != Some(Choice::Main);
        if adopt && !self.on_adopt {
            let b = ADOPT;
            if git(&repo, &["checkout", "-q", "-b", b]).await.is_err() {
                git(&repo, &["checkout", "-q", b]).await?;
            }
        }
        let args = InitArgs {
            repo: Some(repo.clone()),
            name: Some(self.project.clone()),
            integration: adopt.then(|| ADOPT.to_string()),
            stack: None,
        };
        let r2 = repo.clone();
        tokio::task::block_in_place(|| -> anyhow::Result<()> {
            crate::init::scaffold(&r2, &args)?;
            if !r2.join("workflow/base").is_dir() {
                crate::vendor::init_vendor(&r2)?;
            }
            Ok(())
        })
        .map_err(|e| format!("{e:#}"))?;
        self.commit_init(
            &repo,
            &if adopt {
                format!("files committed on the new branch {ADOPT}")
            } else {
                "files committed".into()
            },
        )
        .await
    }

    async fn commit_init(&self, repo: &Path, done: &str) -> Result<String, String> {
        let paths: Vec<&str> = [".bridle", ".gitignore", "workflow"]
            .into_iter()
            .filter(|p| repo.join(p).exists())
            .collect();
        let mut add = vec!["add", "-A", "--"];
        add.extend(paths);
        git(repo, &add).await?;
        git(
            repo,
            &[
                "commit",
                "-q",
                "-m",
                "bridle: set up the project (bridle project setup)",
            ],
        )
        .await
        .map_err(|e| format!("{e}; commit .bridle, .gitignore and workflow yourself"))?;
        Ok(done.into())
    }
}

fn init_status(uncommitted: bool) -> Status {
    if uncommitted {
        Status::Unmet(".bridle/config.toml is made but not committed".into())
    } else {
        Status::Done(".bridle/config.toml present".into())
    }
}

/// The clone's own git config remembers a declined aide token: real state on this machine,
/// no file of ours, and it does not dirty the tree.
const DECLINED_KEY: &str = "bridle.setup.aide-token";

fn aide_status(token: Status, declined: bool, project: &str) -> Status {
    match token {
        Status::Done(_) => Status::Done("aide token present".into()),
        Status::Unmet(_) if declined => Status::Skipped(format!(
            "declined; make it with `bridle token pair --projects {project} --roles {AIDE_ROLE}`"
        )),
        other => other,
    }
}

/// Runs `git` in `dir` without ever prompting; the `Err` is git's first stderr line.
async fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let dir = dir.to_path_buf();
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    tokio::task::spawn_blocking(move || {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(&args)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| format!("cannot run git: {e}"))?;
        if out.status.success() {
            return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
        }
        let err = String::from_utf8_lossy(&out.stderr);
        Err(format!(
            "git {}: {}",
            args.first().map(String::as_str).unwrap_or_default(),
            err.lines().next().unwrap_or("failed").trim()
        ))
    })
    .await
    .map_err(|e| e.to_string())?
}

impl Steps for Setup<'_> {
    fn names(&self) -> Vec<&'static str> {
        STEPS.to_vec()
    }

    async fn check(&mut self, i: usize) -> Status {
        match i {
            CLONE => self.check_clone().await,
            INIT => self.check_init().await,
            REGISTER => {
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
            STATE if !self.cloned => {
                Status::Unmet("after the clone: the daemon's first start creates it".into())
            }
            STATE => state_status(
                local_owner(&self.repo).await,
                &owner_names(&self.home),
                is_new_project(&self.repo).await,
            ),
            SERVICE => service_status(
                self.sys
                    .units(&self.project)
                    .iter()
                    .any(|u| u.what == "daemon"),
                self.daemon_up().await,
            ),
            TOKENS => {
                let roles = TOKEN_ROLES
                    .iter()
                    .map(|r| r.name)
                    .filter(|n| *n != AIDE_ROLE)
                    .map(String::from)
                    .collect();
                self.pair_status(roles, Vec::new()).await
            }
            MAIL => {
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
            _ => {
                let token = self
                    .pair_status(vec![AIDE_ROLE.into()], vec![TokenType::Role])
                    .await;
                let declined = git(&self.repo, &["config", "--local", "--get", DECLINED_KEY])
                    .await
                    .is_ok_and(|v| v.trim() == "declined");
                aide_status(token, declined, &self.project)
            }
        }
    }

    async fn apply(&mut self, i: usize, out: &mut dyn Write) -> Result<String, String> {
        match i {
            CLONE => {
                let url = self.remote.clone().ok_or("no remote to clone")?;
                if std::fs::read_dir(&self.repo).is_ok_and(|mut d| d.next().is_some()) {
                    return Err(format!(
                        "{} exists and is not empty: pass --root with another place",
                        self.repo.display()
                    ));
                }
                if let Some(parent) = self.repo.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("cannot make {}: {e}", parent.display()))?;
                }
                let dest = self.repo.display().to_string();
                git(Path::new("."), &["clone", "-q", &url, &dest]).await?;
                self.repo = self.repo.canonicalize().map_err(|e| e.to_string())?;
                self.cloned = true;
                Ok(format!("cloned into {}", self.repo.display()))
            }
            INIT => self.apply_init().await,
            REGISTER => {
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
            STATE => Ok("left to the daemon's first start (next step)".into()),
            SERVICE => {
                let ws = Workspace::new(self.repo.clone(), self.workspace.clone());
                crate::project_start::start_service(
                    self.cli,
                    &self.sys,
                    &self.project,
                    &self.repo,
                    &ws,
                    self.ctl,
                    out,
                )
                .await
                .map_err(|e| e.to_string())?;
                Ok("daemon service installed and running".into())
            }
            TOKENS => {
                let roles = TOKEN_ROLES
                    .iter()
                    .map(|r| r.name)
                    .filter(|n| *n != AIDE_ROLE)
                    .map(String::from)
                    .collect();
                self.make_tokens(roles, Vec::new(), out).await?;
                Ok("tokens present on every machine".into())
            }
            MAIL => {
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
            _ if !self.aide => {
                git(&self.repo, &["config", "--local", DECLINED_KEY, "declined"]).await?;
                Ok("skipped: declined (remembered in the clone's git config)".into())
            }
            _ => {
                self.make_tokens(vec![AIDE_ROLE.into()], vec![TokenType::Role], out)
                    .await?;
                Ok("aide token made".into())
            }
        }
    }

    fn command(&self, i: usize) -> String {
        let p = &self.project;
        match i {
            CLONE => match &self.remote {
                Some(u) => format!("git clone {u} {}", self.repo.display()),
                None => "make an empty repo and run bridle project setup <URL>".into(),
            },
            INIT => "bridle init".into(),
            REGISTER => format!("bridle projects add {p}"),
            STATE | SERVICE => "bridle project start".into(),
            TOKENS => format!("bridle token pair --projects {p}"),
            MAIL => format!("bridle mail install --project {p}"),
            _ => format!("bridle token pair --projects {p} --roles {AIDE_ROLE}"),
        }
    }

    async fn questions(&mut self, statuses: &[Status]) -> Result<(), CliError> {
        let unmet = |i: usize| matches!(statuses[i], Status::Unmet(_));
        let mut missing: Vec<String> = Vec::new();
        if unmet(CLONE) && !self.root_given {
            let default = self.repo.display().to_string();
            match self
                .asker
                .ask(&format!("Put the clone of {} at", self.project), &default)
            {
                Some(a) => self.repo = expand(&a, &self.sys.home_dir),
                None => missing.push(format!(
                    "where the clone goes: pass --root DIR (proposed: {default})"
                )),
            }
        }
        if unmet(INIT) && self.commits && self.choice.is_none() && !self.on_adopt {
            let q = format!(
                "{} has commits. Put bridle's files on a new {ADOPT} branch (adopt) or on the project's own branch (main)?",
                self.project
            );
            match self.asker.ask(&q, "adopt").as_deref() {
                Some("adopt") => self.choice = Some(Choice::Adopt),
                Some("main") => self.choice = Some(Choice::Main),
                Some(other) => {
                    return Err(CliError::Usage(format!(
                        "'{other}' is not an answer: say adopt or main"
                    )));
                }
                None => {
                    missing.push("main or adopt: pass --main or --adopt (default adopt)".into())
                }
            }
        }
        if unmet(AIDE) {
            let q = "Make the aide token (for the human's aide session)? [Y/n]";
            match self.asker.ask(q, "Y") {
                Some(a) => self.aide = !matches!(a.to_lowercase().as_str(), "n" | "no"),
                None => missing.push("aide token: pass -y to make it".into()),
            }
        }
        if missing.is_empty() {
            return Ok(());
        }
        Err(CliError::Usage(format!(
            "nothing was changed: no terminal to ask, and these answers are missing:\n  - {}\nanswer them with the flags, or pass -y to take every default",
            missing.join("\n  - ")
        )))
    }
}

impl Setup<'_> {
    /// `token pair` for this project, narrowed to `roles`/`tokens`: no second minting path.
    async fn make_tokens(
        &self,
        roles: Vec<String>,
        tokens: Vec<TokenType>,
        out: &mut dyn Write,
    ) -> Result<(), String> {
        let opts = Options {
            projects: vec![self.project.clone()],
            roles,
            tokens,
            ..Default::default()
        };
        let fleet = self.fleet()?;
        let (r, buf) = tokio::task::spawn_blocking(move || {
            let mut buf = Vec::new();
            let r = pair(&opts, &fleet, &Exec, &mut buf);
            (r, buf)
        })
        .await
        .map_err(|e| e.to_string())?;
        let _ = out.write_all(&buf);
        if r? {
            Ok(())
        } else {
            Err(format!(
                "some tokens were not made (see above; an unreachable machine was skipped): fix it and run `bridle token pair --projects {}`",
                self.project
            ))
        }
    }
}

const AIDE_ROLE: &str = crate::token_pair::AIDE;

fn expand(answer: &str, home: &Path) -> PathBuf {
    match answer.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None => PathBuf::from(answer),
    }
}

/// A target that is a place to clone from, not a project name: names have none of these.
fn looks_like_url(t: &str) -> bool {
    t.contains("://")
        || (t.contains('@') && t.contains(':'))
        || t.starts_with(['/', '.', '~'])
        || t.ends_with(".git")
}

/// `git@host:org/foo.git` and `/srv/foo.git/` both give `foo`.
fn project_from_remote(url: &str) -> Option<String> {
    let last = url
        .trim_end_matches('/')
        .rsplit(['/', ':'])
        .next()
        .unwrap_or_default();
    let name = last.strip_suffix(".git").unwrap_or(last);
    let plain = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    plain.then(|| name.to_string())
}

/// `<base>/<p>-work/<p>`, where `<base>` is the commonest parent of this machine's existing
/// workspaces, else `~/work`. Only a proposal: it is always shown and confirmed.
fn default_root(project: &str, workspaces: &[PathBuf], home: &Path) -> PathBuf {
    let mut counts: Vec<(PathBuf, usize)> = Vec::new();
    for parent in workspaces.iter().filter_map(|w| w.parent()) {
        match counts.iter_mut().find(|(p, _)| p == parent) {
            Some(c) => c.1 += 1,
            None => counts.push((parent.to_path_buf(), 1)),
        }
    }
    let base = counts
        .iter()
        .enumerate()
        .max_by_key(|(i, (_, n))| (*n, std::cmp::Reverse(*i)))
        .map(|(_, (p, _))| p.clone())
        .unwrap_or_else(|| home.join("work"));
    base.join(format!("{project}-work")).join(project)
}

/// The clone: `--repo`, `--root`, the running daemon's for `name`, or (with no remote to clone
/// from) the current directory. `None`: there is none yet.
fn find_clone(
    args: &ProjectSetupArgs,
    name: Option<&str>,
    cloning: bool,
    proposed: Option<PathBuf>,
) -> anyhow::Result<Option<PathBuf>> {
    use anyhow::Context;
    let is_clone = |p: &PathBuf| p.join(".git").exists();
    let found = args
        .repo
        .iter()
        .chain(args.root.iter())
        .chain(proposed.iter())
        .find(|p| is_clone(p))
        .cloned()
        .or_else(|| {
            let n = name?;
            list_registry()
                .into_iter()
                .find(|d| d.project == n)
                .map(|d| PathBuf::from(d.repo))
        });
    let repo = match (found, cloning, name) {
        (Some(r), _, _) => r,
        (None, true, _) => return Ok(None),
        (None, false, Some(n)) if !Path::new(".git").exists() => anyhow::bail!(
            "no running daemon for {n} to find its clone: run this inside the clone, or pass --repo (or the remote's URL to clone it)"
        ),
        (None, false, _) => std::env::current_dir().context("current directory")?,
    };
    Ok(Some(repo.canonicalize().with_context(|| {
        format!("repo path {}", repo.display())
    })?))
}

/// The "next:" line the step commands (`init`, `projects add`, `project start`, `token pair`,
/// `mail install`) end with: the first step of the checklist that is not done, from the same
/// checks. Best effort and read-only; prints nothing when `repo` is not a clone of `project` or
/// everything is done.
pub async fn print_next(cli: &Cli, repo: &Path, project: &str) {
    if let Some(n) = next_in(cli, repo, project).await {
        println!("{n}");
    }
}

/// `print_next` for a command that is not given the clone: tries `repo` if given, the current
/// directory, then the clone of a running daemon for `project`.
pub async fn print_next_for(cli: &Cli, project: &str, repo: Option<&Path>) {
    let mut candidates: Vec<PathBuf> = repo.map(Path::to_path_buf).into_iter().collect();
    candidates.extend(std::env::current_dir().ok());
    candidates.extend(
        list_registry()
            .into_iter()
            .find(|d| d.project == project)
            .map(|d| PathBuf::from(d.repo)),
    );
    for c in candidates {
        if let Some(n) = next_in(cli, &c, project).await {
            println!("{n}");
            return;
        }
    }
}

async fn next_in(cli: &Cli, repo: &Path, project: &str) -> Option<String> {
    let repo = repo.canonicalize().ok()?;
    if !repo.join(".git").exists() || repo.file_name().is_none_or(|n| n != project) {
        return None;
    }
    let mut steps = Setup::new(cli, project, repo, None, Asker::none());
    let mut statuses = Vec::new();
    for i in 0..STEPS.len() {
        statuses.push(steps.check(i).await);
    }
    next_line(&steps, &statuses)
}

pub async fn run(cli: &Cli, args: &ProjectSetupArgs) -> Result<(), CliError> {
    let (target_url, target_name) = match args.target.as_deref() {
        Some(t) if looks_like_url(t) => (Some(t.to_string()), None),
        Some(t) => (None, Some(t.to_string())),
        None => (None, None),
    };
    if let (Some(a), Some(b)) = (&target_url, &args.remote)
        && a != b
    {
        return Err(anyhow::anyhow!("two remotes given ({a} and --remote {b}): pass one").into());
    }
    let remote = args.remote.clone().or(target_url);
    let name = target_name
        .or_else(|| remote.as_deref().and_then(project_from_remote))
        .or_else(|| cli.project.clone());
    let home_dir = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let spaces: Vec<PathBuf> = list_registry()
        .into_iter()
        .map(|d| PathBuf::from(d.workspace))
        .collect();
    // A rerun finds the clone an earlier run made at the proposed place.
    let proposed = name.as_deref().map(|n| default_root(n, &spaces, &home_dir));
    let found = find_clone(args, name.as_deref(), remote.is_some(), proposed)?;
    let project = match (&name, &found) {
        (Some(n), _) => n.clone(),
        (None, Some(repo)) => repo
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        (None, None) => {
            return Err(anyhow::anyhow!(
                "cannot tell the project's name from the remote: name it, `bridle project setup <name> --remote <URL>`"
            )
            .into());
        }
    };
    let (repo, root_given) = match found {
        Some(r) => (r, true),
        None => match &args.root {
            Some(r) => (r.clone(), true),
            None => (default_root(&project, &spaces, &home_dir), false),
        },
    };
    let asker = Asker {
        yes: args.yes,
        terminal: std::io::stdin().is_terminal(),
        input: Box::new(std::io::BufReader::new(std::io::stdin())),
        out: Box::new(std::io::stdout()),
    };
    let mut steps = Setup::new(cli, &project, repo, remote, asker);
    steps.root_given = root_given;
    steps.workspace = args.workspace.clone();
    steps.choice = match (args.main, args.adopt) {
        (true, _) => Some(Choice::Main),
        (_, true) => Some(Choice::Adopt),
        _ => None,
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
    fn init_files_made_but_not_committed_are_not_done() {
        assert!(matches!(init_status(true), Status::Unmet(_)));
        assert!(matches!(init_status(false), Status::Done(_)));
    }

    #[test]
    fn a_declined_aide_token_is_skipped_unless_it_exists() {
        let unmet = || Status::Unmet("no token".into());
        assert!(matches!(
            aide_status(unmet(), true, "p"),
            Status::Skipped(t) if t.contains("--projects p --roles aide")
        ));
        assert!(matches!(aide_status(unmet(), false, "p"), Status::Unmet(_)));
        assert!(matches!(
            aide_status(Status::Done("x".into()), true, "p"),
            Status::Done(_)
        ));
        let blocked = Status::Blocked("b".into());
        assert_eq!(aide_status(blocked.clone(), true, "p"), blocked);
    }

    #[tokio::test]
    async fn a_clone_with_uncommitted_init_files_is_seen_through_git() {
        let d = tempfile::tempdir().expect("tmp");
        let run = |args: &[&str]| {
            assert!(
                std::process::Command::new("git")
                    .arg("-C")
                    .arg(d.path())
                    .args(args)
                    .status()
                    .expect("git")
                    .success()
            );
        };
        run(&["init", "-q"]);
        std::fs::create_dir(d.path().join(".bridle")).expect("mkdir");
        std::fs::write(d.path().join(".bridle/config.toml"), "").expect("write");
        let status = |p: &Path| {
            let p = p.to_path_buf();
            async move { git(&p, &["status", "--porcelain", "--", ".bridle/config.toml"]).await }
        };
        assert!(!status(d.path()).await.expect("status").trim().is_empty());
        run(&["add", "-A"]);
        run(&[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-qm",
            "x",
        ]);
        assert!(status(d.path()).await.expect("status").trim().is_empty());
        // The decline marker round-trips through the clone's own git config.
        git(d.path(), &["config", "--local", DECLINED_KEY, "declined"])
            .await
            .expect("set");
        let v = git(d.path(), &["config", "--local", "--get", DECLINED_KEY])
            .await
            .expect("get");
        assert_eq!(v.trim(), "declined");
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

    /// The hint each step command ends with is `next_line` over the real step commands, so it
    /// names the first unmet step: `projects add` leaves the state step, `project start` the
    /// token step, `token pair` the mail step, `mail install` the aide token.
    #[test]
    fn each_command_hint_names_the_first_unmet_step_after_it() {
        use clap::Parser;
        let cli = Cli::parse_from(["bridle", "--project", "demo", "status"]);
        let steps = Setup::new(&cli, "demo", PathBuf::from("/x/demo"), None, Asker::none());
        let after = |first_unmet: usize| {
            let statuses: Vec<Status> = (0..STEPS.len())
                .map(|i| if i < first_unmet { done() } else { todo() })
                .collect();
            next_line(&steps, &statuses)
        };
        assert_eq!(
            after(STATE).as_deref(),
            Some("next: bridle project start"),
            "after projects add"
        );
        assert_eq!(
            after(TOKENS).as_deref(),
            Some("next: bridle token pair --projects demo"),
            "after project start"
        );
        assert_eq!(
            after(MAIL).as_deref(),
            Some("next: bridle mail install --project demo"),
            "after token pair"
        );
        assert_eq!(
            after(AIDE).as_deref(),
            Some("next: bridle token pair --projects demo --roles aide"),
            "after mail install"
        );
        assert_eq!(after(STEPS.len()), None, "everything done");
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
        fn command(&self, i: usize) -> String {
            format!("make-{i}")
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

    #[test]
    fn a_url_is_told_from_a_name_and_gives_its_project() {
        for u in [
            "git@host:org/foo.git",
            "https://h/o/foo",
            "/srv/foo.git/",
            "./foo",
            "~/x/foo",
        ] {
            assert!(looks_like_url(u), "{u}");
            assert_eq!(project_from_remote(u).as_deref(), Some("foo"), "{u}");
        }
        assert!(!looks_like_url("foo-bar"));
        assert_eq!(project_from_remote("https://h/o/a b.git"), None);
    }

    #[test]
    fn the_default_root_follows_the_commonest_parent_else_work() {
        let home = Path::new("/h");
        let spaces = [
            PathBuf::from("/d/a-work"),
            PathBuf::from("/d/b-work"),
            PathBuf::from("/e/c"),
        ];
        assert_eq!(
            default_root("p", &spaces, home),
            PathBuf::from("/d/p-work/p")
        );
        assert_eq!(
            default_root("p", &[], home),
            PathBuf::from("/h/work/p-work/p")
        );
    }

    fn asker(yes: bool, terminal: bool, typed: &str) -> Asker {
        Asker {
            yes,
            terminal,
            input: Box::new(std::io::Cursor::new(typed.to_string().into_bytes())),
            out: Box::new(std::io::sink()),
        }
    }

    #[test]
    fn a_question_takes_the_typed_answer_the_default_on_enter_or_is_missing() {
        assert_eq!(asker(true, false, "").ask("q", "d").as_deref(), Some("d"));
        assert_eq!(asker(false, true, "\n").ask("q", "d").as_deref(), Some("d"));
        assert_eq!(
            asker(false, true, "x\n").ask("q", "d").as_deref(),
            Some("x")
        );
        assert_eq!(asker(false, false, "x\n").ask("q", "d"), None);
    }

    #[test]
    fn missing_answers_are_exit_two() {
        let e = CliError::Usage("x".into());
        assert_eq!(e.exit_code(), 2);
    }
}
