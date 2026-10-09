//! design/specs/project-resolution.md: runs every project-scoped command of the real binary,
//! subcommands included, from a temp workspace and checks which project it acted on. The tree is
//! read from `--help`, so a command wired wrong fails here, and a new one must be classified.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    /// Reaches a daemon: observed as a connection to the project's daemon URL.
    Daemon,
    /// Starts a claude session: observed as `$BRIDLE_PROJECT` in the stub claude.
    Session,
    /// Writes something named for the project; a repo not yet served is named for its folder.
    Names,
    Skip,
}
use Kind::*;

/// Longest matching prefix wins. A command matching none fails `every_command_is_classified`.
const CLASSES: &[(&str, Kind, &str)] = &[
    (
        "daemon launchd",
        Skip,
        "installs a system service; the plist naming is covered by launchd.rs unit tests",
    ),
    (
        "daemon systemd",
        Skip,
        "installs a system service; the unit naming is covered by systemd.rs unit tests",
    ),
    (
        "daemon serve",
        Skip,
        "creates the project: its name is the repo's",
    ),
    ("daemon init", Skip, "creates a project in the cwd"),
    (
        "daemon doctor",
        Skip,
        "checks the machine and the cwd's repo",
    ),
    (
        "daemon rebuild",
        Skip,
        "rebuilds the binary in the cwd's repo",
    ),
    ("daemon list", Skip, "lists every project's daemon"),
    ("daemon", Daemon, ""),
    ("agent", Daemon, ""),
    ("docs", Skip, "static text"),
    (
        "link",
        Skip,
        "config lookup; only a ticket link needs --project, covered by link_test.rs",
    ),
    ("gateway", Skip, "serves every project"),
    (
        "mail run",
        Skip,
        "a long-running bridge; reads its own config",
    ),
    (
        "mail install",
        Skip,
        "writes a service file in the user's home; the project naming is covered by mail_install.rs unit tests",
    ),
    (
        "mail uninstall",
        Skip,
        "removes a service file from the user's home; see mail install",
    ),
    (
        "tui",
        Skip,
        "needs a terminal; covered by the daemon discovery tests",
    ),
    ("completions", Skip, "prints a shell script"),
    ("workflow", Skip, "reads and writes the cwd's repo files"),
    ("pane", Skip, "tmux"),
    ("machine", Skip, "machine setup"),
    ("focus", Skip, "the human's own focus gate"),
    (
        "orchestrator note-session",
        Skip,
        "runs from a hook, in the agent's own env",
    ),
    ("session orchestrator", Session, ""),
    ("session advisor", Session, ""),
    ("session aide", Session, ""),
    ("session", Daemon, ""),
    (
        "advisor",
        Skip,
        "prints a command line; see the advisor tests",
    ),
    ("ticket new", Names, ""),
    ("ticket check", Skip, "reads the cwd's repo"),
    ("ticket set", Skip, "edits the cwd's repo"),
    ("ticket resolve", Skip, "edits the cwd's repo"),
    ("ticket task", Daemon, ""),
    ("ticket submit", Daemon, ""),
    (
        "migrate",
        Skip,
        "migrates the cwd's repo; --project only picks a registry entry for --all",
    ),
    ("status", Daemon, ""),
    ("agents", Daemon, ""),
    ("send", Daemon, ""),
    ("inbox", Daemon, ""),
    ("schedule add", Daemon, ""),
    ("schedule list", Daemon, ""),
    ("schedule rm", Daemon, ""),
    ("events", Daemon, ""),
    ("wait", Daemon, ""),
    ("usage", Daemon, ""),
    ("review now", Daemon, ""),
    ("review", Skip, "edits the documents of the cwd's repo"),
    ("usage cost", Skip, "reads the role prompts; no daemon"),
    ("token", Daemon, ""),
    (
        "token pair",
        Skip,
        "pairs every project on the machines in config.toml; --project is not its input",
    ),
    ("task", Daemon, ""),
    ("probe", Daemon, ""),
    ("port", Daemon, ""),
    ("queue", Daemon, ""),
    ("report", Daemon, ""),
    ("orchestrator", Daemon, ""),
];

fn class_of(leaf: &str) -> Option<(Kind, &'static str)> {
    CLASSES
        .iter()
        .filter(|(p, _, _)| leaf == *p || leaf.starts_with(&format!("{p} ")))
        .max_by_key(|(p, _, _)| p.len())
        .map(|(_, k, why)| (*k, *why))
}

fn bridle() -> Command {
    Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
}

fn help(path: &[String]) -> String {
    let out = bridle().args(path).arg("--help").output().unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn walk(path: &mut Vec<String>, leaves: &mut Vec<Vec<String>>) {
    let text = help(path);
    let mut subs = Vec::new();
    let mut in_cmds = false;
    for l in text.lines() {
        if l == "Commands:" {
            in_cmds = true;
        } else if in_cmds && l.starts_with("  ") {
            if let Some(n) = l.split_whitespace().next()
                && n != "help"
            {
                subs.push(n.to_string());
            }
        } else if in_cmds {
            break;
        }
    }
    if subs.is_empty() {
        leaves.push(path.clone());
    }
    for s in subs {
        path.push(s);
        walk(path, leaves);
        path.pop();
    }
}

/// Every leaf command, as the arguments to reach it.
fn leaves() -> &'static Vec<Vec<String>> {
    static L: OnceLock<Vec<Vec<String>>> = OnceLock::new();
    L.get_or_init(|| {
        let mut out = Vec::new();
        walk(&mut Vec::new(), &mut out);
        out
    })
}

/// Dummy values for what the command requires: the first allowed value for an enum, else `1`
/// (which a string, a number and an id all accept).
fn required_args(path: &[String]) -> Vec<String> {
    // Usage lines that clap prints with alternatives, or commands that insist on an optional one.
    const GIVEN: &[(&str, &[&str])] = &[
        ("task conflict resolve", &["--compatible", "r", "1"]),
        ("task summary", &["--text", "t", "1"]),
        ("task dep add", &["1", "--to", "2"]),
        ("task dep rm", &["1", "--to", "2"]),
        ("task search", &["word"]),
        ("probe", &["target"]),
        ("agent wake", &["external:advisor"]),
        ("send", &["human", "hi"]),
        ("token create", &["x"]),
        (
            "schedule add",
            &["--at", "2099-01-01 09:00", "--message", "hi"],
        ),
        ("ticket new", &["-k", "chore", "a title"]),
        ("task comment", &["1", "hi"]),
        ("ticket task", &["abcd"]),
        ("usage budget max-workers", &["2"]),
        ("orchestrator handover write", &["--file", "-"]),
    ];
    if let Some((_, a)) = GIVEN.iter().find(|(n, _)| *n == leaf_name(path)) {
        return a.iter().map(|s| s.to_string()).collect();
    }
    let text = help(path);
    let usage = text.lines().find(|l| l.starts_with("Usage:")).unwrap();
    let mut args = Vec::new();
    let mut toks = usage.split_whitespace().peekable();
    while let Some(t) = toks.next() {
        let name = if t.starts_with('<') && !t.starts_with("<COMMAND") {
            Some((false, t))
        } else if t.starts_with("--") && toks.peek().is_some_and(|n| n.starts_with('<')) {
            Some((true, toks.next().unwrap()))
        } else {
            None
        };
        let Some((is_opt, tok)) = name else { continue };
        let value = text
            .lines()
            .filter(|l| !l.starts_with("Usage:") && l.contains(tok))
            .find_map(|l| {
                let (_, rest) = l.split_once("[possible values: ")?;
                Some(rest.split([',', ']']).next()?.trim().to_string())
            })
            .unwrap_or_else(|| "1".into());
        if is_opt {
            args.push(t.to_string());
        }
        args.push(value);
    }
    args
}

fn cached_args(leaf: &[String]) -> Vec<String> {
    static C: OnceLock<BTreeMap<String, Vec<String>>> = OnceLock::new();
    C.get_or_init(|| {
        leaves()
            .iter()
            .map(|l| (leaf_name(l), required_args(l)))
            .collect()
    })[&leaf_name(leaf)]
        .clone()
}

fn leaf_name(path: &[String]) -> String {
    path.join(" ")
}

/// A fake daemon that counts the connections it gets and answers each with a 500.
struct Fake {
    url: String,
    hits: Arc<AtomicUsize>,
}

fn fake_daemon() -> Fake {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    let hits = Arc::new(AtomicUsize::new(0));
    let h = hits.clone();
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            h.fetch_add(1, Ordering::SeqCst);
            let mut s = s;
            let mut buf = [0u8; 2048];
            let _ = s.set_read_timeout(Some(Duration::from_millis(200)));
            let _ = s.read(&mut buf);
            let _ = s.write_all(
                b"HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
            );
        }
    });
    Fake { url, hits }
}

struct World {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    /// A folder inside project x's workspace (the clone).
    inside: PathBuf,
    /// A folder inside no workspace.
    outside: PathBuf,
    x: Fake,
    y: Fake,
    bin: PathBuf,
}

fn info_json(project: &str, ws: &Path, url: &str, pid: u32) -> String {
    format!(
        r#"{{"project":"{project}","workspace":"{}","repo":"","url":"{url}","pid":{pid},"started_at":"2026-01-01T00:00:00Z","version":""}}"#,
        ws.display()
    )
}

fn stub(dir: &Path, name: &str, body: &str) {
    let p = dir.join(name);
    fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

fn git_init(dir: &Path) {
    for args in [
        &["init", "-q", "-b", "main"][..],
        &["add", "-A"],
        &["commit", "-q", "--allow-empty", "-m", "i"],
    ] {
        let st = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .status()
            .unwrap();
        assert!(st.success());
    }
}

fn write_ticket(repo: &Path) {
    let dir = repo.join("docs/tickets/open");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("example-abcd.md"),
        "---\nid: abcd\ntitle: example\nkind: chore\nopened: 2026-01-01\nrepos: []\nchanges: []\nspecs: []\nneeds: []\nsee: []\ntasks: []\n---\n\n## The ask\n\nx\n",
    )
    .unwrap();
}

/// Commands that print what they can without a project, so they don't refuse outside a
/// workspace; they must still reach no daemon there.
const TOLERANT: &[&str] = &["orchestrator prime"];

fn world() -> World {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let (x, y) = (fake_daemon(), fake_daemon());
    // Project x's workspace, with its clone one level down.
    let ws = root.join("ws-x");
    let inside = ws.join("clone");
    fs::create_dir_all(ws.join(".bridle")).unwrap();
    fs::create_dir_all(&inside).unwrap();
    // `ticket task` reads its ticket from the cwd's repo before it asks a daemon.
    write_ticket(&inside);
    // `orchestrator prime` reads its role prompt from the cwd's repo before it asks a daemon.
    let role = inside.join("workflow/base/roles/orchestrator.md");
    fs::create_dir_all(role.parent().unwrap()).unwrap();
    fs::write(&role, "role\n").unwrap();
    git_init(&inside);
    fs::write(
        ws.join(".bridle/daemon.json"),
        info_json("x", &ws, &x.url, std::process::id()),
    )
    .unwrap();
    // Project y is only in the registry, owned by this (live) process.
    fs::create_dir_all(root.join("home/daemons")).unwrap();
    fs::write(
        root.join("home/daemons/y.json"),
        info_json("y", &root.join("ws-y"), &y.url, std::process::id()),
    )
    .unwrap();
    let outside = root.join("fresh-repo");
    fs::create_dir_all(&outside).unwrap();
    write_ticket(&outside);
    let role = outside.join("workflow/base/roles/orchestrator.md");
    fs::create_dir_all(role.parent().unwrap()).unwrap();
    fs::write(&role, "role\n").unwrap();
    git_init(&outside);
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    stub(
        &bin,
        "claude",
        "echo \"PROJECT=$BRIDLE_PROJECT\" >> \"$CLAUDE_REC\"",
    );
    stub(&bin, "tmux", "exit 0");
    World {
        _tmp: tmp,
        root,
        inside,
        outside,
        x,
        y,
        bin,
    }
}

struct Ran {
    ok: bool,
    out: String,
    claude: String,
}

/// Runs a leaf from `cwd` in a clean env: no `$BRIDLE_*` except what `extra` sets.
fn run(w: &World, leaf: &[String], cwd: &Path, global: &[&str], extra: &[(&str, &str)]) -> Ran {
    let rec = w.root.join("claude.rec");
    let _ = fs::remove_file(&rec);
    let mut cmd = bridle();
    cmd.args(global)
        .args(leaf)
        .args(cached_args(leaf))
        .current_dir(cwd)
        .env_clear()
        .env(
            "PATH",
            format!(
                "{}:{}",
                w.bin.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("HOME", w.root.join("userhome"))
        .env("BRIDLE_HOME", w.root.join("home"))
        .env("BRIDLE_LAUNCHER_TEST", "1")
        .env("BRIDLE_TOKEN", "t")
        .env("CLAUDE_REC", &rec)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in extra {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().unwrap();
    let start = Instant::now();
    let ok = loop {
        if let Some(st) = child.try_wait().unwrap() {
            break st.success();
        }
        if start.elapsed() > Duration::from_secs(20) {
            let _ = child.kill();
            let _ = child.wait();
            break false;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let mut out = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    Ran {
        ok,
        out,
        claude: fs::read_to_string(&rec).unwrap_or_default(),
    }
}

fn of_kind(k: Kind) -> Vec<&'static Vec<String>> {
    leaves()
        .iter()
        .filter(|l| class_of(&leaf_name(l)).is_some_and(|(c, _)| c == k))
        .collect()
}

/// Which daemon a run reached, as (x hits, y hits) since `before`.
fn hits(w: &World) -> (usize, usize) {
    (
        w.x.hits.load(Ordering::SeqCst),
        w.y.hits.load(Ordering::SeqCst),
    )
}

/// Runs each Daemon and Session leaf and returns what each reached: the project it acted on.
fn acted_on(
    w: &World,
    cwd: &Path,
    global: &[&str],
    extra: &[(&str, &str)],
) -> BTreeMap<String, String> {
    let mut seen = BTreeMap::new();
    for leaf in of_kind(Daemon).into_iter().chain(of_kind(Session)) {
        let name = leaf_name(leaf);
        let before = hits(w);
        let r = run(w, leaf, cwd, global, extra);
        let after = hits(w);
        let project = if class_of(&name).unwrap().0 == Session {
            r.claude
                .lines()
                .find_map(|l| l.strip_prefix("PROJECT="))
                .unwrap_or("")
                .to_string()
        } else if after.0 > before.0 {
            "x".into()
        } else if after.1 > before.1 {
            "y".into()
        } else {
            String::new()
        };
        let project = if project.is_empty() {
            format!("none ({})", r.out.trim().replace('\n', " "))
        } else {
            project
        };
        seen.insert(name, project);
    }
    seen
}

fn assert_all(seen: &BTreeMap<String, String>, want: &str) {
    let bad: Vec<String> = seen
        .iter()
        .filter(|(_, p)| p.as_str() != want)
        .map(|(c, p)| format!("  bridle {c}: {p}"))
        .collect();
    assert!(
        bad.is_empty(),
        "expected every command to act on project {want}; these did not:\n{}",
        bad.join("\n")
    );
    assert!(seen.len() > 50, "only {} commands ran", seen.len());
}

// s-a438: a command run inside a workspace acts on that workspace's project.
#[test]
fn inside_a_workspace_every_command_acts_on_its_project() {
    let w = world();
    let seen = acted_on(&w, &w.inside, &[], &[]);
    assert_all(&seen, "x");
    // The naming commands name the workspace's project too.
    for leaf in of_kind(Names).into_iter() {
        let r = run(&w, leaf, &w.inside, &[], &[]);
        let text = names(&w, leaf, &w.inside, &r);
        assert!(
            text.contains("dev.bridle.x") || text.contains("[x]"),
            "bridle {}: {text}",
            leaf_name(leaf)
        );
    }
}

// s-9bc0: the flag wins over the workspace.
#[test]
fn the_flag_wins_over_the_workspace() {
    let w = world();
    let mut seen = acted_on(&w, &w.inside, &["--project", "y"], &[]);
    // 3haz: `send --project y` is mail for y's daemon, so it is queued on the sender's own
    // daemon (x, the workspace's) and forwarded from there; the CLI never writes to y.
    assert_eq!(seen.remove("send").as_deref(), Some("x"));
    assert_all(&seen, "y");
}

// s-0bd0: the environment variable wins over the workspace.
#[test]
fn the_environment_wins_over_the_workspace() {
    let w = world();
    let seen = acted_on(&w, &w.inside, &[], &[("BRIDLE_PROJECT", "y")]);
    assert_all(&seen, "y");
}

// s-f742: outside any workspace the command refuses.
#[test]
fn outside_a_workspace_every_command_refuses() {
    let w = world();
    let mut bad = Vec::new();
    for leaf in of_kind(Daemon).into_iter().chain(of_kind(Session)) {
        let r = run(&w, leaf, &w.outside, &[], &[]);
        let tolerant = TOLERANT.contains(&leaf_name(leaf).as_str());
        let refused = !r.ok && r.out.contains("--project");
        if !(refused || tolerant && r.ok) || r.claude.contains("PROJECT=") {
            bad.push(format!(
                "  bridle {}: ok={} {}",
                leaf_name(leaf),
                r.ok,
                r.out.trim()
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "these did not refuse naming --project:\n{}",
        bad.join("\n")
    );
    assert_eq!(hits(&w), (0, 0), "a command reached a daemon");
}

// s-c389: a repo not yet served is named after its folder.
#[test]
fn a_repo_not_yet_served_is_named_after_its_folder() {
    let w = world();
    for leaf in of_kind(Names).into_iter() {
        let r = run(&w, leaf, &w.outside, &[], &[]);
        let text = names(&w, leaf, &w.outside, &r);
        assert!(
            text.contains("fresh-repo"),
            "bridle {}: {text}",
            leaf_name(leaf)
        );
        assert!(
            !text.contains("dev.bridle.bridle") && !text.contains("[bridle]"),
            "bridle {}: {text}",
            leaf_name(leaf)
        );
    }
}

/// What a naming command wrote or printed: its output, plus the ticket it minted.
fn names(w: &World, leaf: &[String], cwd: &Path, r: &Ran) -> String {
    let mut text = r.out.clone();
    let _ = w;
    if leaf_name(leaf) == "ticket new" {
        let dir = cwd.join("docs/tickets/open");
        for e in fs::read_dir(dir).into_iter().flatten().flatten() {
            text.push_str(&fs::read_to_string(e.path()).unwrap_or_default());
        }
    }
    text
}

// s-4b92: an unclassified command fails the check.
#[test]
fn every_command_is_classified() {
    let unclassified: Vec<String> = leaves()
        .iter()
        .map(|l| leaf_name(l))
        .filter(|n| class_of(n).is_none())
        .collect();
    assert!(
        unclassified.is_empty(),
        "classify these in CLASSES (tests/project_resolution_test.rs): how do they find their \
         project? {unclassified:?}"
    );
    for (p, k, why) in CLASSES {
        if *k == Skip {
            assert!(!why.is_empty(), "`{p}` is skipped without a reason");
        }
        assert!(
            leaves().iter().any(|l| {
                let n = leaf_name(l);
                n == *p || n.starts_with(&format!("{p} "))
            }),
            "CLASSES lists `{p}`, which is not a command"
        );
    }
}
