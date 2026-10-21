//! `bridle dev site up | down | list`: a throwaway second site (daemons, gateway, optional UI
//! dev server) in `.agent-site/<name>/`, on its own `BRIDLE_HOME`, ports and login, so an agent
//! can click through the web UI without touching the human's. See docs/design/dev-site.md.
//!
//! Stopping (rule `no-kill-by-name`): only the pids recorded in `site.json`, and only while
//! each one's start time and argv still match what was recorded.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::cli::{Cli, DevAction, DevArgs, SiteAction, SiteDownArgs, SiteUpArgs};
use crate::error::CliError;

const SITES_DIR: &str = ".agent-site";
const USERNAME: &str = "agent";
const START_WAIT: Duration = Duration::from_secs(120);
const STOP_WAIT: Duration = Duration::from_secs(60);

/// (project, path in the repo, contents) for `fixtures/dev-site/`; add a new file here when you add one there.
const FIXTURE_FILES: &[(&str, &str, &str)] = &[
    (
        "acme-docs",
        ".bridle/config.toml",
        include_str!("../fixtures/dev-site/acme-docs/.bridle/config.toml"),
    ),
    (
        "acme-docs",
        "README.md",
        include_str!("../fixtures/dev-site/acme-docs/README.md"),
    ),
    (
        "acme-docs",
        "docs/design/site.md",
        include_str!("../fixtures/dev-site/acme-docs/docs/design/site.md"),
    ),
    (
        "acme-docs",
        "docs/tickets/open/add-search-to-the-docs-ad1a.md",
        include_str!(
            "../fixtures/dev-site/acme-docs/docs/tickets/open/add-search-to-the-docs-ad1a.md"
        ),
    ),
    (
        "acme-docs",
        "docs/tickets/open/fix-broken-anchors-ad2b.md",
        include_str!("../fixtures/dev-site/acme-docs/docs/tickets/open/fix-broken-anchors-ad2b.md"),
    ),
    (
        "acme-docs",
        "docs/tickets/resolved/set-up-the-site-ad3c.md",
        include_str!(
            "../fixtures/dev-site/acme-docs/docs/tickets/resolved/set-up-the-site-ad3c.md"
        ),
    ),
    (
        "acme-shop",
        ".bridle/config.toml",
        include_str!("../fixtures/dev-site/acme-shop/.bridle/config.toml"),
    ),
    (
        "acme-shop",
        "README.md",
        include_str!("../fixtures/dev-site/acme-shop/README.md"),
    ),
    (
        "acme-shop",
        "docs/design/checkout.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/design/checkout.md"),
    ),
    (
        "acme-shop",
        "docs/design/specs/cart.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/design/specs/cart.md"),
    ),
    (
        "acme-shop",
        "docs/notes/api-overview.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/notes/api-overview.md"),
    ),
    (
        "acme-shop",
        "docs/tickets/open/bump-dependencies-sh9c.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/tickets/open/bump-dependencies-sh9c.md"),
    ),
    (
        "acme-shop",
        "docs/tickets/open/cart-totals-round-wrongly-sh4a.md",
        include_str!(
            "../fixtures/dev-site/acme-shop/docs/tickets/open/cart-totals-round-wrongly-sh4a.md"
        ),
    ),
    (
        "acme-shop",
        "docs/tickets/open/checkout-flow-epic-sh1e.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/tickets/open/checkout-flow-epic-sh1e.md"),
    ),
    (
        "acme-shop",
        "docs/tickets/open/document-the-api-sh8a.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/tickets/open/document-the-api-sh8a.md"),
    ),
    (
        "acme-shop",
        "docs/tickets/open/fix-cart-rounding-sh7b.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/tickets/open/fix-cart-rounding-sh7b.md"),
    ),
    (
        "acme-shop",
        "docs/tickets/open/guest-checkout-sh3c.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/tickets/open/guest-checkout-sh3c.md"),
    ),
    (
        "acme-shop",
        "docs/tickets/open/survey-search-options-sh6f.md",
        include_str!(
            "../fixtures/dev-site/acme-shop/docs/tickets/open/survey-search-options-sh6f.md"
        ),
    ),
    (
        "acme-shop",
        "docs/tickets/open/which-payment-provider-sh5d.md",
        include_str!(
            "../fixtures/dev-site/acme-shop/docs/tickets/open/which-payment-provider-sh5d.md"
        ),
    ),
    (
        "acme-shop",
        "docs/tickets/resolved/add-wishlist-sh2c.md",
        include_str!("../fixtures/dev-site/acme-shop/docs/tickets/resolved/add-wishlist-sh2c.md"),
    ),
    (
        "acme-shop",
        "docs/tickets/resolved/incident-checkout-down-sh0a.md",
        include_str!(
            "../fixtures/dev-site/acme-shop/docs/tickets/resolved/incident-checkout-down-sh0a.md"
        ),
    ),
    (
        "acme-shop",
        "docs/tickets/resolved/login-loops-on-expired-session-sh9d.md",
        include_str!(
            "../fixtures/dev-site/acme-shop/docs/tickets/resolved/login-loops-on-expired-session-sh9d.md"
        ),
    ),
];

/// A process the site started, with what identifies it later: a pid alone can be reused.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Proc {
    pub role: String,
    pub pid: i32,
    /// `ps -o lstart=` output at start.
    pub started: String,
    /// `ps -o command=` output at start.
    pub argv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub project: String,
    pub daemon_port: u16,
    pub tickets_open: usize,
    pub tickets_resolved: usize,
    pub seeded_tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub name: String,
    pub dir: PathBuf,
    pub gateway_port: u16,
    pub ui_port: Option<u16>,
    /// Where to open it: the UI when there is one, else the gateway.
    pub url: String,
    pub username: String,
    pub password: String,
    pub fixtures: Vec<Fixture>,
    pub procs: Vec<Proc>,
}

pub async fn run(cli: &Cli, args: &DevArgs) -> Result<(), CliError> {
    let DevAction::Site(site) = &args.action;
    match &site.action {
        SiteAction::Up(a) => up(cli, a).await,
        SiteAction::Down(a) => down(a),
        SiteAction::List => list(cli),
    }
    .map_err(CliError::from)
}

// ---------- processes ----------

fn ps_field(pid: i32, field: &str) -> Option<String> {
    let out = Command::new("ps")
        .args(["-o", field, "-p", &pid.to_string()])
        .output()
        .ok()?;
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (out.status.success() && !s.is_empty()).then_some(s)
}

fn record(role: &str, pid: i32) -> Result<Proc> {
    Ok(Proc {
        role: role.to_string(),
        pid,
        started: ps_field(pid, "lstart=")
            .ok_or_else(|| anyhow!("{role} (pid {pid}) is not running"))?,
        argv: ps_field(pid, "command=")
            .ok_or_else(|| anyhow!("{role} (pid {pid}) is not running"))?,
    })
}

#[derive(Debug, PartialEq, Eq)]
enum Liveness {
    Gone,
    /// The pid is alive but is not the process we recorded.
    Reused,
    Ours,
}

fn liveness(p: &Proc) -> Liveness {
    match (ps_field(p.pid, "lstart="), ps_field(p.pid, "command=")) {
        (Some(s), Some(a)) if s == p.started && a == p.argv => Liveness::Ours,
        (None, _) | (_, None) => Liveness::Gone,
        _ => Liveness::Reused,
    }
}

/// SIGTERM the recorded processes that are still ours, wait for them, and say what was left alone.
fn stop_procs(procs: &[Proc]) -> Vec<String> {
    use nix::sys::signal::{Signal, kill};
    let mut notes = Vec::new();
    let mut signalled = Vec::new();
    // Reverse start order: UI and gateway before the daemons they talk to.
    for p in procs.iter().rev() {
        match liveness(p) {
            Liveness::Gone => {}
            Liveness::Reused => notes.push(format!(
                "pid {} ({}) now belongs to another process; left alone",
                p.pid, p.role
            )),
            Liveness::Ours => {
                let _ = kill(nix::unistd::Pid::from_raw(p.pid), Signal::SIGTERM);
                signalled.push(p);
            }
        }
    }
    let deadline = Instant::now() + STOP_WAIT;
    for p in signalled {
        while liveness(p) == Liveness::Ours {
            if Instant::now() > deadline {
                notes.push(format!(
                    "pid {} ({}) is still running after SIGTERM",
                    p.pid, p.role
                ));
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    notes
}

/// A child of this process (not detached: bridle's cleanup of an agent reaps a forgotten site),
/// logging to `log`. Agent and project variables are dropped so it never reaches the real site.
fn spawn(
    exe: &Path,
    args: &[&str],
    home: &Path,
    cwd: &Path,
    log: &Path,
    extra_env: &[(&str, String)],
) -> Result<std::process::Child> {
    let out = std::fs::File::create(log).with_context(|| format!("creating {}", log.display()))?;
    let err = out.try_clone()?;
    let mut c = Command::new(exe);
    c.args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err);
    scrub_env(&mut c, home);
    for (k, v) in extra_env {
        c.env(k, v);
    }
    c.spawn()
        .with_context(|| format!("starting {}", exe.display()))
}

fn scrub_env(c: &mut Command, home: &Path) {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("BRIDLE_") {
            c.env_remove(k);
        }
    }
    c.env("BRIDLE_HOME", home);
}

/// A free port: bind 0, read it back, release it. Only for processes that cannot report their
/// own (the gateway, Vite); a lost race then fails loudly.
fn probe_port() -> Result<u16> {
    let l = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(l.local_addr()?.port())
}

fn wait_for<T>(what: &str, mut f: impl FnMut() -> Option<T>) -> Result<T> {
    let deadline = Instant::now() + START_WAIT;
    loop {
        if let Some(v) = f() {
            return Ok(v);
        }
        if Instant::now() > deadline {
            bail!("{what} did not come up within {}s", START_WAIT.as_secs());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn http_ok(url: &str) -> bool {
    Command::new("curl")
        .args(["-sf", "-m", "2", "-o", "/dev/null", url])
        .status()
        .is_ok_and(|s| s.success())
}

// ---------- up ----------

fn random_word() -> String {
    use std::io::Read;
    let mut b = [0u8; 3];
    let _ = std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut b));
    format!("site-{}", hex::encode(b))
}

fn git(dir: &Path, args: &[&str]) -> Result<()> {
    let out = Command::new("git")
        .args([
            "-c",
            "user.name=dev-site",
            "-c",
            "user.email=dev-site@localhost",
        ])
        .args(args)
        .current_dir(dir)
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

/// Writes the embedded tree of one project, as a committed git repo with no remote.
fn make_fixture_repo(root: &Path, project: &str) -> Result<PathBuf> {
    let repo = root.join(project);
    for (p, rel, text) in FIXTURE_FILES.iter().filter(|(p, _, _)| *p == project) {
        let path = repo.join(rel);
        let _ = p;
        std::fs::create_dir_all(path.parent().expect("has a parent"))?;
        std::fs::write(&path, text)?;
    }
    git(&repo, &["init", "-q", "-b", "main"])?;
    git(&repo, &["add", "-A"])?;
    git(&repo, &["commit", "-q", "-m", "fixture project"])?;
    Ok(repo)
}

fn fixture_projects() -> Vec<&'static str> {
    let mut v: Vec<&str> = FIXTURE_FILES.iter().map(|(p, _, _)| *p).collect();
    v.dedup();
    v
}

fn count_tickets(repo: &Path, state: &str) -> usize {
    std::fs::read_dir(repo.join("docs/tickets").join(state))
        .map(|d| d.flatten().count())
        .unwrap_or(0)
}

fn random_password() -> String {
    use std::io::Read;
    let mut b = [0u8; 9];
    let _ = std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut b));
    hex::encode(b)
}

fn write_home(home: &Path, gateway_port: u16, password: &str) -> Result<()> {
    std::fs::create_dir_all(home)?;
    let hash = bridle_gateway::auth::hash_password(password)
        .map_err(|e| anyhow!("hashing the password: {e}"))?;
    // No [machines] or [projects]: the gateway finds the daemons through this home's registry,
    // and nothing leaves the laptop.
    let text = format!(
        "[gateway]\nbind = \"127.0.0.1:{gateway_port}\"\nusername = \"{USERNAME}\"\npassword_hash = \"{hash}\"\n"
    );
    std::fs::write(home.join("config.toml"), text)?;
    Ok(())
}

/// Runs a `bridle` CLI command against one fixture daemon as the human; returns its JSON.
/// Runs in the project's repo (`--for-human` writes its ticket file there), pointed at its
/// daemon by `--url`.
struct Target<'a> {
    exe: &'a Path,
    home: &'a Path,
    repo: &'a Path,
    workspace: &'a Path,
    url: &'a str,
}

fn cli_json(t: &Target, args: &[&str]) -> Result<serde_json::Value> {
    let Target {
        exe,
        home,
        repo,
        workspace,
        url,
    } = *t;
    let token = std::fs::read_to_string(bridle_api::discovery::human_token_path(workspace))
        .context("reading the human token")?;
    let mut c = Command::new(exe);
    c.args(args)
        .args(["--json", "--url", url, "--token", token.trim()])
        .current_dir(repo);
    scrub_env(&mut c, home);
    let out = c.output().context("running bridle")?;
    if !out.status.success() {
        bail!(
            "bridle {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null))
}

/// Tasks in several states, a to-do for the human with an open question, and a message. No agents.
fn seed(t: &Target, project: &str) -> Result<Vec<String>> {
    let run = |args: &[&str]| cli_json(t, args);
    let new = |title: &str, kind: &str| -> Result<String> {
        let v = run(&[
            "task",
            "new",
            title,
            "--kind",
            kind,
            "--body",
            "Seeded by the dev site.",
        ])?;
        v["id"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| anyhow!("no id in {v}"))
    };
    let mut ids = Vec::new();
    let planned = new(&format!("{project}: ready to build"), "feature")?;
    run(&["task", "ready", &planned])?;
    run(&["task", "plan", &planned])?;
    ids.push(planned);
    ids.push(new(&format!("{project}: waiting for approval"), "chore")?);
    let done = new(&format!("{project}: already shipped"), "fix")?;
    run(&["task", "ready", &done])?;
    run(&["task", "plan", &done])?;
    run(&[
        "task",
        "done",
        &done,
        "--commit",
        "0000000000000000000000000000000000000000",
    ])?;
    ids.push(done);
    // `--for-human` prints the new file's path, not JSON, so find the to-do by its title.
    let title = format!("{project}: pick one");
    run(&[
        "task",
        "new",
        &title,
        "--kind",
        "question",
        "--for-human",
        "--body",
        "A to-do for the human.",
    ])?;
    let todo = run(&["task", "list"])?
        .as_array()
        .and_then(|a| a.iter().find(|t| t["title"] == title.as_str()))
        .and_then(|t| t["id"].as_str().map(str::to_string))
        .ok_or_else(|| anyhow!("the to-do was not created"))?;
    run(&[
        "task",
        "ask",
        &todo,
        "Which of the two should we ship first?",
        "--to",
        "human",
    ])?;
    ids.push(todo);
    run(&[
        "send",
        "human",
        &format!("Welcome to the {project} fixture site."),
        "--task",
        &ids[0],
    ])?;
    Ok(ids)
}

async fn up(cli: &Cli, args: &SiteUpArgs) -> Result<()> {
    let name = args
        .name
        .clone()
        .or_else(|| {
            std::env::var("BRIDLE_AGENT_NAME")
                .ok()
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(random_word);
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "site name {name:?} must be letters, digits, - or _"
        ));
    }
    let dir = std::env::current_dir()?.join(SITES_DIR).join(&name);
    if dir.exists() {
        return Err(anyhow!(
            "{} exists; `bridle dev site down {name}` first",
            dir.display()
        ));
    }
    std::fs::create_dir_all(&dir)?;
    let mut procs: Vec<Proc> = Vec::new();
    let mut children: Vec<std::process::Child> = Vec::new();
    match build(args, &name, &dir, &mut procs, &mut children) {
        Ok(site) => {
            let json = serde_json::to_string_pretty(&site)?;
            std::fs::write(dir.join("site.json"), &json)?;
            if cli.json {
                println!("{json}");
            } else {
                println!("site {name} is up: {}", site.url);
                println!("login: {} / {}", site.username, site.password);
                println!("stop it with: bridle dev site down {name}");
            }
            // The children stay in our process tree but must not be waited on or killed on drop.
            std::mem::forget(children);
            Ok(())
        }
        Err(e) => {
            // Half-started: stop what we started, then the directory.
            stop_procs(&procs);
            for mut c in children {
                let _ = c.kill();
                let _ = c.wait();
            }
            let _ = std::fs::remove_dir_all(&dir);
            Err(e.context("bridle dev site up failed; stopped what it had started"))
        }
    }
}

fn start(
    role: &str,
    child: std::process::Child,
    procs: &mut Vec<Proc>,
    children: &mut Vec<std::process::Child>,
) -> Result<()> {
    let pid = i32::try_from(child.id()).context("pid")?;
    children.push(child);
    procs.push(record(role, pid)?);
    Ok(())
}

fn build(
    args: &SiteUpArgs,
    name: &str,
    dir: &Path,
    procs: &mut Vec<Proc>,
    children: &mut Vec<std::process::Child>,
) -> Result<Site> {
    let exe = std::env::current_exe().context("finding the bridle binary")?;
    let home = dir.join("home");
    let password = random_password();
    let gateway_port = probe_port()?;
    write_home(&home, gateway_port, &password)?;

    let mut fixtures = Vec::new();
    for project in fixture_projects() {
        let repo = make_fixture_repo(&dir.join("projects"), project)?;
        let workspace = dir.join("workspaces").join(project);
        std::fs::create_dir_all(&workspace)?;
        let child = spawn(
            &exe,
            &[
                "serve",
                "--repo",
                &repo.to_string_lossy(),
                "--workspace",
                &workspace.to_string_lossy(),
                "--listen",
                "127.0.0.1:0",
            ],
            &home,
            &repo,
            &dir.join(format!("daemon-{project}.log")),
            &[],
        )?;
        start(&format!("daemon-{project}"), child, procs, children)?;
        let port = wait_for(&format!("the {project} daemon"), || {
            let info = bridle_api::discovery::read_daemon_json(&workspace).ok()?;
            let port = info
                .url
                .rsplit(':')
                .next()?
                .trim_end_matches('/')
                .parse::<u16>()
                .ok()?;
            http_ok(&format!("{}/v1/health", info.url.trim_end_matches('/'))).then_some(port)
        })?;
        let url = format!("http://127.0.0.1:{port}");
        let target = Target {
            exe: &exe,
            home: &home,
            repo: &repo,
            workspace: &workspace,
            url: &url,
        };
        let seeded_tasks = seed(&target, project)?;
        fixtures.push(Fixture {
            project: project.to_string(),
            daemon_port: port,
            tickets_open: count_tickets(&repo, "open"),
            tickets_resolved: count_tickets(&repo, "resolved"),
            seeded_tasks,
        });
    }

    let gateway = spawn(
        &exe,
        &["gateway"],
        &home,
        dir,
        &dir.join("gateway.log"),
        &[],
    )?;
    start("gateway", gateway, procs, children)?;
    wait_for("the gateway", || {
        http_ok(&format!("http://127.0.0.1:{gateway_port}/api/v1/health")).then_some(())
    })?;

    let ui_port = if args.no_ui {
        None
    } else {
        start_ui(args, dir, gateway_port, procs, children)?
    };
    let url = format!("http://127.0.0.1:{}", ui_port.unwrap_or(gateway_port));
    Ok(Site {
        name: name.to_string(),
        dir: dir.to_path_buf(),
        gateway_port,
        ui_port,
        url,
        username: USERNAME.to_string(),
        password,
        fixtures,
        procs: procs.clone(),
    })
}

fn start_ui(
    args: &SiteUpArgs,
    dir: &Path,
    gateway_port: u16,
    procs: &mut Vec<Proc>,
    children: &mut Vec<std::process::Child>,
) -> Result<Option<u16>> {
    let ui = match &args.ui {
        Some(d) => d.clone(),
        None => std::env::current_dir()?.join("../bridle-ui"),
    };
    let vite = ui.join("node_modules/vite/bin/vite.js");
    if !vite.is_file() {
        // Not asked for by name: the UI checkout is simply not there (yet), so serve the API alone.
        if args.ui.is_none() {
            eprintln!(
                "no UI dev server at {}; starting the gateway only (--ui DIR names a checkout)",
                vite.display()
            );
            return Ok(None);
        }
        bail!("no UI dev server at {}", vite.display());
    }
    let port = probe_port()?;
    let mut c = Command::new("node");
    c.arg(&vite)
        .current_dir(&ui)
        .stdin(Stdio::null())
        .env("VITE_DEV_PORT", port.to_string())
        .env(
            "VITE_API_TARGET",
            format!("http://127.0.0.1:{gateway_port}"),
        );
    let log = std::fs::File::create(dir.join("ui.log"))?;
    c.stdout(log.try_clone()?).stderr(log);
    start("ui", c.spawn().context("starting node")?, procs, children)?;
    wait_for("the UI dev server", || {
        http_ok(&format!("http://127.0.0.1:{port}/")).then_some(())
    })?;
    Ok(Some(port))
}

// ---------- down, list ----------

fn sites_root() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join(SITES_DIR))
}

fn read_site(dir: &Path) -> Result<Site> {
    let p = dir.join("site.json");
    serde_json::from_slice(&std::fs::read(&p).with_context(|| format!("reading {}", p.display()))?)
        .with_context(|| format!("parsing {}", p.display()))
}

fn down(args: &SiteDownArgs) -> Result<()> {
    let root = sites_root()?;
    let dirs: Vec<PathBuf> = if args.all {
        std::fs::read_dir(&root)
            .map(|d| {
                d.flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        let name = match &args.name {
            Some(n) => n.clone(),
            None => std::env::var("BRIDLE_AGENT_NAME")
                .map_err(|_| anyhow!("name the site (`down NAME`) or use --all"))?,
        };
        vec![root.join(name)]
    };
    for dir in dirs {
        if !dir.is_dir() {
            return Err(anyhow!("no site at {}", dir.display()));
        }
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        // A directory with no site.json is a failed or half-removed one: nothing recorded to stop.
        let notes = match read_site(&dir) {
            Ok(site) => stop_procs(&site.procs),
            Err(_) => vec!["no site.json; nothing to stop".to_string()],
        };
        for n in &notes {
            println!("{name}: {n}");
        }
        std::fs::remove_dir_all(&dir).with_context(|| format!("deleting {}", dir.display()))?;
        println!("site {name} removed");
    }
    Ok(())
}

fn list(cli: &Cli) -> Result<()> {
    let root = sites_root()?;
    let mut sites: Vec<Site> = std::fs::read_dir(&root)
        .map(|d| {
            d.flatten()
                .filter_map(|e| read_site(&e.path()).ok())
                .collect()
        })
        .unwrap_or_default();
    sites.sort_by(|a, b| a.name.cmp(&b.name));
    if cli.json {
        let rows: Vec<_> = sites
            .iter()
            .map(|s| serde_json::json!({
                "name": s.name, "url": s.url, "gateway_port": s.gateway_port, "ui_port": s.ui_port,
                "alive": s.procs.iter().map(|p| (p.role.clone(), liveness(p) == Liveness::Ours)).collect::<std::collections::BTreeMap<_, _>>(),
            }))
            .collect();
        println!(
            "{}",
            serde_json::to_string(&rows).map_err(anyhow::Error::from)?
        );
        return Ok(());
    }
    if sites.is_empty() {
        println!("no sites under {}", root.display());
    }
    for s in &sites {
        let alive = s
            .procs
            .iter()
            .filter(|p| liveness(p) == Liveness::Ours)
            .count();
        println!(
            "{}  {}  gateway :{}  {}/{} processes alive",
            s.name,
            s.url,
            s.gateway_port,
            alive,
            s.procs.len()
        );
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sleeper() -> std::process::Child {
        Command::new("sleep").arg("60").spawn().unwrap()
    }

    #[test]
    fn stop_procs_signals_a_recorded_process() {
        let mut child = sleeper();
        let p = record("t", i32::try_from(child.id()).unwrap()).unwrap();
        assert!(stop_procs(&[p]).is_empty());
        // Reaped by wait, so it was SIGTERMed (a still-running sleep would block for 60 s).
        assert!(!child.wait().unwrap().success());
    }

    #[test]
    fn stop_procs_leaves_a_reused_pid_alone() {
        let mut child = sleeper();
        let mut p = record("t", i32::try_from(child.id()).unwrap()).unwrap();
        p.started = "Mon Jan  1 00:00:00 2001".to_string();
        let notes = stop_procs(&[p]);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("left alone"), "{notes:?}");
        assert!(
            child.try_wait().unwrap().is_none(),
            "the process was killed"
        );
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn a_gone_pid_is_not_reported() {
        let mut child = sleeper();
        let p = record("t", i32::try_from(child.id()).unwrap()).unwrap();
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(stop_procs(&[p]).is_empty());
    }

    #[test]
    fn fixtures_are_two_git_repos_with_tickets() {
        let d = tempfile::tempdir().unwrap();
        let projects = fixture_projects();
        assert_eq!(projects.len(), 2, "{projects:?}");
        for project in projects {
            let repo = make_fixture_repo(d.path(), project).unwrap();
            assert!(repo.join(".git").is_dir());
            assert!(count_tickets(&repo, "open") > 0 && count_tickets(&repo, "resolved") > 0);
        }
    }
}
