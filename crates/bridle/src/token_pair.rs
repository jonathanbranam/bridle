//! `bridle token pair`: set up role tokens between machines over ssh, like `ssh-copy-id`
//! (docs/tickets/open/pair-machines-token-setup-over-ssh-sk7p.md, design/specs/token-pairing.md).
//!
//! The machine it runs on brokers: for each (holder machine, daemon machine, project, role) it
//! asks the holder whether its credentials entry still works, and if not has the daemon's
//! machine mint a token and the holder store it. Every step is a hidden `bridle token pair-*`
//! helper run locally or as `ssh <host> bridle ...`. A token only ever travels as a helper's
//! stdout and the next helper's stdin: never argv, a log, our output or a file but the
//! holder's `credentials.toml`.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::process::{Command, Stdio};

use clap::ValueEnum;

/// A role whose external session or service holds a token in `credentials.toml`.
pub struct TokenRole {
    pub name: &'static str,
    /// Only held on other machines, as `human@<machine>`: the local human uses the workspace
    /// token file.
    pub remote_only: bool,
    /// Where its visitor token's replies go when the holder machine has no project of the same
    /// name as the daemon's project (br-eurm): the project the role lives in.
    pub home: &'static str,
}

pub const HUMAN: &str = "human";
pub const ORCHESTRATOR: &str = "orchestrator";
pub const ADVISOR: &str = "advisor";
pub const AIDE: &str = "aide";
pub const MAIL: &str = "mail";

/// THE list of roles `bridle token pair` pairs, and the only roles a launcher may set
/// `BRIDLE_AS` to (a test scans the sources). A new role's launcher adds it here; the human
/// then runs `bridle token pair` once and the role is paired everywhere.
///
/// product-manager is not listed: it is a named advisor (`advisor/product-manager`) and shares
/// the `advisor` token, so pairing `advisor` pairs it. When it gets its own identity it joins
/// this list like any new role.
pub const TOKEN_ROLES: &[TokenRole] = &[
    TokenRole {
        name: HUMAN,
        remote_only: true,
        home: "bridle",
    },
    TokenRole {
        name: ORCHESTRATOR,
        remote_only: false,
        home: "bridle",
    },
    TokenRole {
        name: ADVISOR,
        remote_only: false,
        home: "bridle",
    },
    TokenRole {
        name: AIDE,
        remote_only: false,
        home: "bridle",
    },
    TokenRole {
        name: MAIL,
        remote_only: false,
        home: "bridle",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TokenType {
    Role,
    Peer,
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub machines: Vec<String>,
    pub projects: Vec<String>,
    pub roles: Vec<String>,
    pub tokens: Vec<TokenType>,
    pub rotate: bool,
    pub dry_run: bool,
}

/// Where a helper runs: this process's machine, or another over ssh to this host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Where {
    Local,
    Ssh(String),
}

/// Runs `bridle <args>` somewhere, feeding it `stdin`, and returns its stdout. The error is
/// the reason, from stderr: helpers never print a token there.
pub trait Runner {
    fn run(&self, at: &Where, args: &[String], stdin: Option<&str>) -> Result<String, String>;
}

/// Appended to the remote PATH: a non-interactive ssh shell often lacks where `just install`
/// and `cargo install` put bridle.
const REMOTE_PATH: &str = r#"PATH="$PATH:$HOME/.local/bin:$HOME/.cargo/bin""#;

/// Single-quote one word for a remote POSIX shell.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// The one remote command line that runs `bridle <args>` over ssh, finding bridle in
/// `~/.local/bin` and `~/.cargo/bin` as well as the PATH.
pub fn remote_bridle_cmd(args: &[String]) -> String {
    let quoted: Vec<String> = args.iter().map(|a| sh_quote(a)).collect();
    format!("{REMOTE_PATH}; exec bridle {}", quoted.join(" "))
}

/// The reason for a failed remote run: 127 is the shell not finding bridle, 255 is ssh itself.
pub fn remote_failure(host: &str, code: i32, stderr: &str) -> String {
    let last = stderr.trim().lines().last().unwrap_or("");
    match code {
        127 => format!("bridle not found on {host} (looked on PATH, ~/.local/bin, ~/.cargo/bin)"),
        255 if last.is_empty() => format!("{host} unreachable: ssh failed"),
        255 => format!("{host} unreachable: {last}"),
        _ if last.is_empty() => format!("exit {code}"),
        _ => last.to_string(),
    }
}

/// The real runner: the running binary locally, `ssh <host> <remote_bridle_cmd>` elsewhere.
pub struct Exec;

impl Runner for Exec {
    fn run(&self, at: &Where, args: &[String], stdin: Option<&str>) -> Result<String, String> {
        if let Where::Ssh(host) = at {
            // Bounded: Tailscale SSH check mode would otherwise wait for a browser login.
            let mut c = Command::new("ssh");
            c.args([
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                host,
                &remote_bridle_cmd(args),
            ]);
            let input = stdin.map(|t| format!("{t}\n"));
            let (code, out, err) = crate::project_sync::run_timed_raw(
                c,
                host,
                input.as_deref(),
                crate::project_sync::SSH_TIMEOUT,
            )?;
            return if code == 0 {
                Ok(String::from_utf8_lossy(&out).into_owned())
            } else {
                Err(remote_failure(host, code, &String::from_utf8_lossy(&err)))
            };
        }
        let mut cmd = match at {
            Where::Local => {
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let mut c = Command::new(exe);
                c.args(args);
                c
            }
            Where::Ssh(_) => unreachable!("returned above"),
        };
        cmd.stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| format!("cannot start: {e}"))?;
        if let (Some(text), Some(mut pipe)) = (stdin, child.stdin.take()) {
            pipe.write_all(text.as_bytes())
                .and_then(|()| pipe.write_all(b"\n"))
                .map_err(|e| format!("write to helper: {e}"))?;
        }
        let out = child
            .wait_with_output()
            .map_err(|e| format!("wait for helper: {e}"))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            let err = err.trim();
            Err(if err.is_empty() {
                format!("exit {}", out.status.code().unwrap_or(-1))
            } else {
                err.lines().last().unwrap_or(err).to_string()
            })
        }
    }
}

/// The machine config the plan needs (`~/.bridle/config.toml`).
#[derive(Clone)]
pub struct Fleet {
    /// This machine's `[machine] name` (`local` if unnamed).
    pub local: String,
    /// `[machines]`: machine name to host.
    pub hosts: BTreeMap<String, String>,
    /// `[projects]`: project to (machine, port).
    pub placed: BTreeMap<String, (String, u16)>,
}

impl Fleet {
    pub fn from_config(map: &bridle_api::machines::MachineMap) -> Self {
        Fleet {
            local: map.machine.name.clone().unwrap_or_else(|| "local".into()),
            hosts: map.machines.clone(),
            placed: map
                .projects
                .iter()
                .map(|(p, pl)| (p.clone(), (pl.machine.clone(), pl.port)))
                .collect(),
        }
    }

    fn at(&self, machine: &str) -> Where {
        if machine == self.local {
            Where::Local
        } else {
            Where::Ssh(self.hosts.get(machine).cloned().unwrap_or_default())
        }
    }
}

/// Names and hosts go into a remote shell command line: keep them plain.
fn plain(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':' | '/'))
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[derive(Default)]
struct Tally {
    minted: usize,
    kept: usize,
    would: usize,
    failed: usize,
    lines: Vec<String>,
}

/// What the options select: the reachable machines with their project daemons. Shared by
/// `pair` and `check`, so the two walk the same entries.
struct Plan {
    do_role: bool,
    do_peer: bool,
    roles: Vec<&'static str>,
    /// machine -> its projects (name, port)
    up: BTreeMap<String, Vec<(String, u16)>>,
    /// Every project on those machines, ignoring `--projects`: the receivers a peer entry may
    /// point at (a new project's daemon forwards to visitors' homes outside the selection).
    all_up: BTreeMap<String, Vec<(String, u16)>>,
    /// (machine, project) pairs that left peer tokens out (`[mail] peers = false`).
    no_peers: BTreeSet<(String, String)>,
    /// False when a machine was unreachable.
    ok: bool,
}

/// A role entry a holder keeps for one project's daemon.
struct RoleEntry<'a> {
    daemon: &'a str,
    project: &'a str,
    port: u16,
    role: &'static str,
    same: bool,
}

impl Plan {
    fn role_entries(&self, holder: &str) -> Vec<RoleEntry<'_>> {
        let mut v = Vec::new();
        for (daemon, projects) in &self.up {
            for (project, port) in projects {
                for role in &self.roles {
                    let remote_only = TOKEN_ROLES.iter().any(|r| r.name == *role && r.remote_only);
                    let same = holder == daemon;
                    if remote_only && same {
                        continue;
                    }
                    v.push(RoleEntry {
                        daemon,
                        project,
                        port: *port,
                        role,
                        same,
                    });
                }
            }
        }
        v
    }

    /// The (daemon, project) pairs `holder` needs a peer token for. Senders are all the
    /// holder's projects, ignoring `--projects`: after a take-over the other machines' entries
    /// toward the moved project are the stale ones (br-eg64).
    fn peer_entries(&self, holder: &str) -> Vec<(&str, &str)> {
        let senders: Vec<&String> = self.all_up[holder]
            .iter()
            .map(|(p, _)| p)
            .filter(|p| !self.no_peers.contains(&(holder.to_string(), (*p).clone())))
            .collect();
        let mut v = Vec::new();
        if senders.is_empty() {
            return v;
        }
        for (daemon, projects) in &self.all_up {
            for (project, _) in projects {
                if self.no_peers.contains(&(daemon.clone(), project.clone())) {
                    continue;
                }
                // A project's only sender being itself needs no token.
                if daemon == holder && senders.len() == 1 && senders[0] == project {
                    continue;
                }
                v.push((daemon.as_str(), project.as_str()));
            }
        }
        v
    }
}

fn plan(
    opts: &Options,
    fleet: &Fleet,
    runner: &dyn Runner,
    out: &mut dyn Write,
) -> Result<Plan, String> {
    let tokens: Vec<TokenType> = if opts.tokens.is_empty() {
        vec![TokenType::Role, TokenType::Peer]
    } else {
        opts.tokens.clone()
    };
    let do_role = tokens.contains(&TokenType::Role);
    let do_peer = tokens.contains(&TokenType::Peer);
    if !do_role && !opts.roles.is_empty() {
        return Err("--roles applies to role tokens only: it cannot go with --tokens peer".into());
    }
    for r in &opts.roles {
        if !TOKEN_ROLES.iter().any(|t| t.name == r) {
            let known: Vec<&str> = TOKEN_ROLES.iter().map(|t| t.name).collect();
            return Err(format!("unknown role '{r}' (known: {})", known.join(", ")));
        }
    }
    let mut known_machines: Vec<String> = vec![fleet.local.clone()];
    known_machines.extend(fleet.hosts.keys().filter(|m| **m != fleet.local).cloned());
    for m in &opts.machines {
        if !known_machines.contains(m) {
            return Err(format!(
                "unknown machine '{m}' (known: {})",
                known_machines.join(", ")
            ));
        }
    }
    let machines: Vec<String> = known_machines
        .into_iter()
        .filter(|m| opts.machines.is_empty() || opts.machines.contains(m))
        .collect();
    for name in machines.iter().chain(&opts.projects) {
        if !plain(name) {
            return Err(format!("'{name}' is not a plain name"));
        }
    }
    for host in fleet.hosts.values() {
        if !plain(host) {
            return Err(format!("host '{host}' in [machines] is not a plain name"));
        }
    }
    let roles: Vec<&'static str> = TOKEN_ROLES
        .iter()
        .map(|t| t.name)
        .filter(|n| opts.roles.is_empty() || opts.roles.iter().any(|r| r == n))
        .collect();

    // Each reachable machine and its projects (name, port).
    let mut ok = true;
    let mut up: BTreeMap<String, Vec<(String, u16)>> = BTreeMap::new();
    let mut all_up: BTreeMap<String, Vec<(String, u16)>> = BTreeMap::new();
    let mut no_peers: BTreeSet<(String, String)> = BTreeSet::new();
    for m in &machines {
        let mut found: Vec<(String, u16)> = Vec::new();
        match runner.run(&fleet.at(m), &args(&["token", "pair-projects"]), None) {
            Ok(text) => {
                for line in text.lines() {
                    let mut it = line.split_whitespace();
                    if let (Some(p), Some(port)) = (it.next(), it.next())
                        && let Ok(port) = port.parse()
                    {
                        found.push((p.to_string(), port));
                        if it.next() == Some("false") {
                            no_peers.insert((m.clone(), p.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                writeln!(out, "machine {m}: unreachable, skipped: {e}").ok();
                ok = false;
                continue;
            }
        }
        for (p, (pm, port)) in &fleet.placed {
            if pm == m && !found.iter().any(|(f, _)| f == p) {
                found.push((p.clone(), *port));
            }
        }
        all_up.insert(
            m.clone(),
            found.iter().filter(|(p, _)| plain(p)).cloned().collect(),
        );
        found.retain(|(p, _)| opts.projects.is_empty() || opts.projects.contains(p));
        found.retain(|(p, _)| {
            let good = plain(p);
            if !good {
                writeln!(
                    out,
                    "machine {m}: project '{p}' is not a plain name, skipped"
                )
                .ok();
                ok = false;
            }
            good
        });
        found.sort();
        all_up.entry(m.clone()).and_modify(|v| v.sort());
        up.insert(m.clone(), found);
    }
    for p in &opts.projects {
        if !up.values().flatten().any(|(f, _)| f == p) && up.len() == machines.len() {
            return Err(format!(
                "no daemon for project '{p}' on the selected machines"
            ));
        }
    }

    Ok(Plan {
        do_role,
        do_peer,
        roles,
        up,
        all_up,
        no_peers,
        ok,
    })
}

/// Runs the pairing. Returns whether everything went through (the exit code is non-zero
/// otherwise). Err is a usage problem found before anything ran.
pub fn pair(
    opts: &Options,
    fleet: &Fleet,
    runner: &dyn Runner,
    out: &mut dyn Write,
) -> Result<bool, String> {
    let p = plan(opts, fleet, runner, out)?;
    let mut ok = p.ok;
    let up = &p.up;
    // Peer tokens go first: minting a visitor's role token on a daemon checks that the daemon
    // can forward to the visitor's home, which needs the peer entries made here (br-g9qm).
    // A daemon forwards mail with a token the receiving daemon minted for the
    // sending MACHINE, so each machine with a sending project needs one per receiving project.
    for holder in p.all_up.keys().filter(|_| p.do_peer) {
        let mut t = Tally::default();
        for (daemon, project) in p.peer_entries(holder) {
            let label = format!("{holder} -> {daemon}/{project}");
            match peer_one(opts, fleet, runner, holder, daemon, project) {
                Ok(Outcome::Kept) => t.kept += 1,
                Ok(Outcome::Minted) => {
                    t.minted += 1;
                    t.lines
                        .push(format!("peer: minted on the receiver for {label}"));
                }
                Ok(Outcome::Would) => {
                    t.would += 1;
                    t.lines
                        .push(format!("peer: would mint on the receiver for {label}"));
                }
                Err(e) => {
                    t.failed += 1;
                    t.lines.push(format!("failed: peer {label}: {e}"));
                }
            }
        }
        if t.failed > 0 {
            ok = false;
        }
        if t.minted + t.kept + t.would + t.failed > 0 {
            report(out, opts, holder, "peer", &t);
        }
    }
    for holder in up.keys().filter(|_| p.do_role) {
        let mut t = Tally::default();
        for e in p.role_entries(holder) {
            let label = format!("{} -> {}/{}", e.role, e.daemon, e.project);
            let home = entry_home(holder, &up[holder], &e);
            match one(opts, fleet, runner, holder, &e, home) {
                Ok(Outcome::Kept) => t.kept += 1,
                Ok(Outcome::Minted) => t.minted += 1,
                Ok(Outcome::Would) => {
                    t.would += 1;
                    t.lines.push(format!("would mint: {label}"));
                }
                Err(e) => {
                    t.failed += 1;
                    t.lines.push(format!("failed: {label}: {e}"));
                }
            }
        }
        if t.failed > 0 {
            ok = false;
        }
        report(out, opts, holder, "role", &t);
    }

    // The last word: every entry, checked from its holder against its daemon, whatever the
    // steps above believed (br-su62).
    if !opts.dry_run {
        let results = check_entries(&p, fleet, runner);
        write_check(out, &results, false);
        write_non_forwarding(out, &p, &[]);
        ok &= results.iter().all(|r| r.refused.is_none());
    }
    Ok(ok)
}

fn report(out: &mut dyn Write, opts: &Options, holder: &str, kind: &str, t: &Tally) {
    let mut head = format!("machine {holder} ({kind} tokens): ");
    if opts.dry_run {
        head.push_str(&format!("would mint {}, kept {}", t.would, t.kept));
    } else {
        head.push_str(&format!("minted {}, kept {}", t.minted, t.kept));
    }
    if t.failed > 0 {
        head.push_str(&format!(", failed {}", t.failed));
    }
    writeln!(out, "{head}").ok();
    for l in &t.lines {
        writeln!(out, "  {l}").ok();
    }
}

/// One peer entry: `holder` machine sends to `project` on `daemon`. The receiver mints
/// `peer:<holder>`, the holder stores it as `[peer] <project>` (the direction rule, gdf3).
fn peer_one(
    opts: &Options,
    fleet: &Fleet,
    runner: &dyn Runner,
    holder: &str,
    daemon: &str,
    project: &str,
) -> Result<Outcome, String> {
    let holder_at = fleet.at(holder);
    let daemon_at = fleet.at(daemon);
    if !opts.rotate && check_peer(fleet, runner, holder, daemon, project).is_ok() {
        return Ok(Outcome::Kept);
    }
    if opts.dry_run {
        return Ok(Outcome::Would);
    }
    let token = runner
        .run(
            &daemon_at,
            &args(&["--project", project, "token", "pair-peer-mint", holder]),
            None,
        )
        .map_err(|e| format!("mint on {daemon}: {e}"))?;
    let token = token.trim();
    if token.is_empty() {
        return Err(format!("mint on {daemon}: no token came back"));
    }
    runner
        .run(
            &holder_at,
            &args(&["--project", project, "token", "pair-store", "peer"]),
            Some(token),
        )
        .map_err(|e| format!("store on {holder}: {e}"))?;
    Ok(Outcome::Minted)
}

/// The visitor's home on the holder machine: the orchestrator's `<machine>-home` if the holder
/// has it (br-gr3v), else the project of the same name if the holder has it, else the role's own
/// project, else the holder's first project (a daemon with none cannot receive replies at all).
fn home_for(holder: &str, holder_projects: &[(String, u16)], project: &str, role: &str) -> String {
    let has = |p: &str| holder_projects.iter().any(|(h, _)| h == p);
    if role == ORCHESTRATOR && has(&machine_home(holder)) {
        return machine_home(holder);
    }
    let default = TOKEN_ROLES
        .iter()
        .find(|r| r.name == role)
        .map_or("bridle", |r| r.home);
    if has(project) {
        project.to_string()
    } else if has(default) {
        default.to_string()
    } else {
        holder_projects
            .first()
            .map_or(default.to_string(), |(p, _)| p.clone())
    }
}

fn machine_home(machine: &str) -> String {
    format!("{machine}-home")
}

/// The home a role's token for `e` is minted with, or None for a direct token. A token for
/// another machine's daemon is a visitor. On the holder's own machine only the orchestrator is
/// one, and only on daemons other than its `<machine>-home`: it then holds one waiter there, and
/// the project daemons forward their wakes to it (br-gr3v).
fn entry_home(holder: &str, holder_projects: &[(String, u16)], e: &RoleEntry) -> Option<String> {
    if !e.same {
        return Some(home_for(holder, holder_projects, e.project, e.role));
    }
    let home = machine_home(holder);
    (e.role == ORCHESTRATOR && e.project != home && holder_projects.iter().any(|(p, _)| *p == home))
        .then_some(home)
}

/// The holder's projects whose daemons do not forward to the orchestrator: with no
/// `<machine>-home` its tokens stay direct (br-gr3v), so only the project it waits on is heard.
fn non_forwarding(p: &Plan, holder: &str) -> Vec<String> {
    let Some(projects) = p.up.get(holder) else {
        return Vec::new();
    };
    if !p.do_role
        || !p.roles.contains(&ORCHESTRATOR)
        || projects.iter().any(|(n, _)| *n == machine_home(holder))
    {
        return Vec::new();
    }
    let waits_on = home_for(holder, projects, "bridle", ORCHESTRATOR);
    projects
        .iter()
        .filter(|(n, _)| *n != waits_on)
        .map(|(n, _)| n.clone())
        .collect()
}

/// Says which projects do not forward, and what to do about it (br-8rdn). `results` are the
/// checked entries: on a machine with a home, a local daemon whose orchestrator visitor token is
/// missing or refused does not forward either (br-8rdn, item 3).
fn write_non_forwarding(out: &mut dyn Write, p: &Plan, results: &[Checked]) {
    for holder in p.up.keys() {
        let home = machine_home(holder);
        if !p.up[holder].iter().any(|(n, _)| *n == home) {
            continue;
        }
        let v: Vec<&str> = results
            .iter()
            .filter(|r| {
                r.holder == *holder
                    && r.daemon == *holder
                    && r.kind == ORCHESTRATOR
                    && r.project != home
                    && r.refused.is_some()
            })
            .map(|r| r.project.as_str())
            .collect();
        if !v.is_empty() {
            writeln!(
                out,
                "note: on machine {holder} these daemons do not forward wakes or mail to {home}: {names} \
                 (the orchestrator's visitor token is missing or refused). Run `bridle token pair`.",
                names = v.join(", ")
            )
            .ok();
        }
    }
    for holder in p.up.keys() {
        let v = non_forwarding(p, holder);
        if !v.is_empty() {
            writeln!(
                out,
                "note: machine {holder} has no {home} project, so its orchestrator's tokens stay direct and \
                 these daemons do not forward wakes or mail to it: {names}. Create {home} \
                 (`bridle init`, then start its daemon), then re-run `bridle token pair`.",
                home = machine_home(holder),
                names = v.join(", ")
            )
            .ok();
        }
    }
}

enum Outcome {
    Kept,
    Minted,
    Would,
}

/// Asks `holder` whether its entry for the role works against the daemon it is for. The
/// holder judges its own entry, so no token leaves it. The error is the reason it is refused.
fn check_role(
    fleet: &Fleet,
    runner: &dyn Runner,
    holder: &str,
    e: &RoleEntry,
    home: Option<&str>,
) -> Result<(), String> {
    let mut a = args(&["--project", e.project]);
    // A token held for another machine's daemon is a visitor there, and has a home.
    if !e.same {
        let host = fleet
            .hosts
            .get(e.daemon)
            .ok_or_else(|| format!("machine {} has no host in [machines]", e.daemon))?;
        a.extend(args(&["--url", &format!("http://{host}:{}", e.port)]));
    }
    a.extend(args(&["token", "pair-check", e.role]));
    if !e.same {
        a.extend(args(&["--machine", e.daemon]));
    }
    // A kept visitor token must have the home it would be minted with now (br-8usa).
    if let Some(home) = home {
        a.extend(args(&["--home", home]));
    }
    runner.run(&fleet.at(holder), &a, None).map(|_| ())
}

/// Whether `holder` holds a peer entry for `project` and `daemon` has a live token for it.
fn check_peer(
    fleet: &Fleet,
    runner: &dyn Runner,
    holder: &str,
    daemon: &str,
    project: &str,
) -> Result<(), String> {
    runner
        .run(
            &fleet.at(holder),
            &args(&["--project", project, "token", "pair-peer-held"]),
            None,
        )
        .map_err(|e| format!("holder: {e}"))?;
    runner
        .run(
            &fleet.at(daemon),
            &args(&["--project", project, "token", "pair-peer-active", holder]),
            None,
        )
        .map(|_| ())
        .map_err(|e| format!("daemon: {e}"))
}

/// One checked entry; `refused` holds the reason.
struct Checked {
    holder: String,
    /// A role name, or `peer`.
    kind: String,
    daemon: String,
    project: String,
    refused: Option<String>,
}

fn check_entries(p: &Plan, fleet: &Fleet, runner: &dyn Runner) -> Vec<Checked> {
    let mut v = Vec::new();
    for holder in p.up.keys().filter(|_| p.do_role) {
        for e in p.role_entries(holder) {
            v.push(Checked {
                holder: holder.clone(),
                kind: e.role.to_string(),
                daemon: e.daemon.to_string(),
                project: e.project.to_string(),
                refused: check_role(
                    fleet,
                    runner,
                    holder,
                    &e,
                    entry_home(holder, &p.up[holder], &e).as_deref(),
                )
                .err(),
            });
        }
    }
    for holder in p.all_up.keys().filter(|_| p.do_peer) {
        for (daemon, project) in p.peer_entries(holder) {
            v.push(Checked {
                holder: holder.clone(),
                kind: "peer".into(),
                daemon: daemon.to_string(),
                project: project.to_string(),
                refused: check_peer(fleet, runner, holder, daemon, project).err(),
            });
        }
    }
    v
}

fn write_check(out: &mut dyn Write, results: &[Checked], json: bool) {
    if json {
        let v: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "machine": r.holder,
                    "role": r.kind,
                    "daemon": r.daemon,
                    "project": r.project,
                    "accepted": r.refused.is_none(),
                    "reason": r.refused,
                })
            })
            .collect();
        writeln!(out, "{}", serde_json::Value::Array(v)).ok();
        return;
    }
    for r in results {
        let verdict = match &r.refused {
            None => "accepted".to_string(),
            Some(why) => format!("refused: {}", why.trim()),
        };
        writeln!(
            out,
            "check {} {} -> {}/{}: {verdict}",
            r.holder, r.kind, r.daemon, r.project
        )
        .ok();
    }
}

/// `bridle token check`: read-only. Returns whether every selected entry is accepted.
pub fn check(
    opts: &Options,
    fleet: &Fleet,
    runner: &dyn Runner,
    out: &mut dyn Write,
    json: bool,
) -> Result<bool, String> {
    // In JSON mode the plan's own notes (an unreachable machine) go to stderr, not the document.
    let p = if json {
        plan(opts, fleet, runner, &mut std::io::stderr())?
    } else {
        plan(opts, fleet, runner, out)?
    };
    let results = check_entries(&p, fleet, runner);
    write_check(out, &results, json);
    if !json {
        write_non_forwarding(out, &p, &results);
    }
    Ok(p.ok && results.iter().all(|r| r.refused.is_none()))
}

fn one(
    opts: &Options,
    fleet: &Fleet,
    runner: &dyn Runner,
    holder: &str,
    e: &RoleEntry,
    home: Option<String>,
) -> Result<Outcome, String> {
    let (daemon, project, role) = (e.daemon, e.project, e.role);
    let holder_at = fleet.at(holder);
    let daemon_at = fleet.at(daemon);
    let same = e.same;
    if !opts.rotate && check_role(fleet, runner, holder, e, home.as_deref()).is_ok() {
        return Ok(Outcome::Kept);
    }
    if opts.dry_run {
        return Ok(Outcome::Would);
    }
    let mut mint = args(&["--project", project, "token", "pair-mint", role]);
    if let Some(home) = &home {
        mint.extend(args(&["--for", holder, "--home", home]));
        if same {
            // The machine's own orchestrator keeps full rights here: a direct token that
            // records its home, replacing a visitor token an earlier pair made (br-j86p).
            mint.push("--direct".into());
        }
    } else if !same {
        mint.extend(args(&["--for", holder]));
    }
    let token = runner
        .run(&daemon_at, &mint, None)
        .map_err(|e| format!("mint on {daemon}: {e}"))?;
    let token = token.trim();
    if token.is_empty() {
        return Err(format!("mint on {daemon}: no token came back"));
    }
    let mut store = args(&["--project", project, "token", "pair-store", role]);
    if !same {
        store.extend(args(&["--machine", daemon]));
    }
    runner
        .run(&holder_at, &store, Some(token))
        .map_err(|e| format!("store on {holder}: {e}"))?;
    Ok(Outcome::Minted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// An in-memory fleet: daemons accept the tokens minted for them; holders keep entries.
    type CredKey = (String, String, Option<String>, String);

    #[derive(Default)]
    struct Sim {
        /// machine -> projects
        projects: HashMap<String, Vec<(String, u16)>>,
        /// (daemon machine, project) -> valid tokens
        valid: RefCell<HashMap<(String, String), BTreeSet<String>>>,
        /// (holder, principal, machine, project) -> token
        creds: RefCell<HashMap<CredKey, String>>,
        calls: RefCell<Vec<Vec<String>>>,
        down: Vec<String>,
        /// (machine, project) with `[mail] peers = false`
        no_peers: Vec<(String, String)>,
        /// (daemon machine, receiving project, sending machine) -> live peer token
        peer: RefCell<HashMap<(String, String, String), String>>,
        n: RefCell<u32>,
        /// The daemon turns the orchestrator's token away although it minted it (br-nbgp).
        refuse_after_mint: bool,
        /// token -> the home it was minted with (visitor tokens only)
        homes: RefCell<HashMap<String, Option<String>>>,
        /// tokens minted as visitors for the holder's own machine (what a pair before br-j86p made)
        local_visitors: RefCell<BTreeSet<String>>,
    }

    impl Sim {
        fn machine(at: &Where) -> String {
            match at {
                Where::Local => "mbp".into(),
                Where::Ssh(h) => h.clone(),
            }
        }
    }

    impl Runner for Sim {
        fn run(&self, at: &Where, a: &[String], stdin: Option<&str>) -> Result<String, String> {
            self.calls.borrow_mut().push(a.to_vec());
            let here = Self::machine(at);
            if self.down.contains(&here) {
                return Err("ssh: connection refused".into());
            }
            let val = |k: &str| {
                a.iter()
                    .position(|x| x == k)
                    .and_then(|i| a.get(i + 1))
                    .cloned()
            };
            let project = val("--project").unwrap_or_default();
            let i = a.iter().position(|x| x == "token").unwrap();
            let cmd = a[i + 1].as_str();
            match cmd {
                "pair-projects" => Ok(self
                    .projects
                    .get(&here)
                    .into_iter()
                    .flatten()
                    .map(|(p, port)| {
                        let peers = !self.no_peers.contains(&(here.clone(), p.clone()));
                        format!("{p} {port} {peers}\n")
                    })
                    .collect()),
                "pair-check" => {
                    let machine = val("--machine");
                    let key = (
                        here.clone(),
                        a[i + 2].clone(),
                        machine.clone(),
                        project.clone(),
                    );
                    let daemon = machine.unwrap_or(here);
                    let creds = self.creds.borrow();
                    let tok = creds.get(&key).ok_or("no entry")?;
                    let valid = self.valid.borrow();
                    if val("--machine").is_none() && self.local_visitors.borrow().contains(tok) {
                        return Err("a token on the holder's own machine must be direct".into());
                    }
                    let visitor_ok = (val("--machine").is_none() && val("--home").is_none()) || {
                        let homes = self.homes.borrow();
                        let have = homes.get(tok).and_then(Option::as_ref);
                        have.is_some() && val("--home").is_none_or(|h| have == Some(&h))
                    };
                    if visitor_ok
                        && !(self.refuse_after_mint && a[i + 2] == ORCHESTRATOR)
                        && valid
                            .get(&(daemon, project))
                            .is_some_and(|s| s.contains(tok))
                    {
                        Ok(String::new())
                    } else {
                        Err("rejected".into())
                    }
                }
                "pair-mint" => {
                    *self.n.borrow_mut() += 1;
                    let tok = format!("secret-{}", self.n.borrow());
                    let same_machine = val("--for").as_deref() == Some(here.as_str());
                    let mut valid = self.valid.borrow_mut();
                    let set = valid.entry((here, project)).or_default();
                    // A principal has one active token: the old one is revoked.
                    let _ = a[i + 2].as_str();
                    set.insert(tok.clone());
                    self.homes.borrow_mut().insert(tok.clone(), val("--home"));
                    if same_machine && !a.iter().any(|x| x == "--direct") {
                        self.local_visitors.borrow_mut().insert(tok.clone());
                    }
                    Ok(format!("{tok}\n"))
                }
                "pair-store" => {
                    self.creds.borrow_mut().insert(
                        (here, a[i + 2].clone(), val("--machine"), project),
                        stdin.unwrap().to_string(),
                    );
                    Ok(String::new())
                }
                "pair-peer-held" => {
                    let key = (here, "peer".to_string(), None, project);
                    self.creds
                        .borrow()
                        .contains_key(&key)
                        .then(String::new)
                        .ok_or_else(|| "no peer entry".into())
                }
                "pair-peer-active" => self
                    .peer
                    .borrow()
                    .contains_key(&(here, project, a[i + 2].clone()))
                    .then(String::new)
                    .ok_or_else(|| "no live peer".into()),
                "pair-peer-mint" => {
                    *self.n.borrow_mut() += 1;
                    let tok = format!("secret-{}", self.n.borrow());
                    self.peer
                        .borrow_mut()
                        .insert((here, project, a[i + 2].clone()), tok.clone());
                    Ok(format!("{tok}\n"))
                }
                other => panic!("unexpected helper {other}"),
            }
        }
    }

    fn fleet() -> Fleet {
        Fleet {
            local: "mbp".into(),
            hosts: [("mbp", "mbp"), ("nuc", "nuc")]
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .into(),
            placed: BTreeMap::new(),
        }
    }

    fn sim() -> Sim {
        let mut s = Sim::default();
        s.projects
            .insert("mbp".into(), vec![("bridle".into(), 7401)]);
        s.projects
            .insert("nuc".into(), vec![("notes".into(), 7402)]);
        s
    }

    fn run(s: &Sim, opts: &Options) -> (bool, String) {
        let mut out = Vec::new();
        let ok = pair(opts, &fleet(), s, &mut out).unwrap();
        (ok, String::from_utf8(out).unwrap())
    }

    // s-891b
    // s-89c3: the second run keeps every entry
    #[test]
    fn remote_command_extends_path_and_quotes_every_arg() {
        let a = args(&["token", "pair-mint", "it's $HOME; rm"]);
        assert_eq!(
            remote_bridle_cmd(&a),
            r#"PATH="$PATH:$HOME/.local/bin:$HOME/.cargo/bin"; exec bridle 'token' 'pair-mint' 'it'\''s $HOME; rm'"#
        );
    }

    #[test]
    fn remote_failures_tell_not_found_unreachable_and_bridles_own_error() {
        let nf = remote_failure("nuc", 127, "sh: bridle: not found");
        assert!(nf.starts_with("bridle not found on nuc"), "{nf}");
        let un = remote_failure("nuc", 255, "ssh: connect: Connection refused\n");
        assert_eq!(un, "nuc unreachable: ssh: connect: Connection refused");
        assert_eq!(
            remote_failure("nuc", 1, "warn\nerror: no such project"),
            "error: no such project"
        );
        assert_eq!(remote_failure("nuc", 2, ""), "exit 2");
    }

    #[test]
    fn defaults_pair_every_role_everywhere_and_a_second_run_changes_nothing() {
        let s = sim();
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        // 2 holders x 2 projects x 4 roles, plus `human` for the 2 cross-machine pairs x 1.
        // Per holder: 2 projects x 4 roles + 1 remote human = 9.
        let roles = |s: &Sim| s.creds.borrow().keys().filter(|k| k.1 != "peer").count();
        assert_eq!(roles(&s), 18, "{out}");
        assert!(out.contains("(role tokens): minted 9, kept 0"), "{out}");
        let before: BTreeMap<_, _> = s.creds.borrow().clone().into_iter().collect();
        let (ok, out) = run(&s, &Options::default());
        assert!(ok);
        assert!(out.contains("(role tokens): minted 0, kept 9"), "{out}");
        let after: BTreeMap<_, _> = s.creds.borrow().clone().into_iter().collect();
        assert_eq!(before, after);
        // The local human uses the workspace token: no `[human] project` entry anywhere.
        assert!(
            !s.creds
                .borrow()
                .keys()
                .any(|(_, p, m, _)| p == "human" && m.is_none())
        );
    }

    // br-eurm
    #[test]
    fn home_is_the_same_named_project_else_the_roles_else_any() {
        let p = |names: &[&str]| -> Vec<(String, u16)> {
            names.iter().map(|n| (n.to_string(), 1)).collect()
        };
        assert_eq!(
            home_for("m", &p(&["bridle", "notes"]), "notes", "aide"),
            "notes"
        );
        assert_eq!(
            home_for("m", &p(&["bridle", "x"]), "notes", "aide"),
            "bridle"
        );
        assert_eq!(home_for("m", &p(&["x", "y"]), "notes", "aide"), "x");
        // br-gr3v: the orchestrator's home is <machine>-home when there is one, else as above.
        let orch = |names: &[&str]| home_for("m", &p(names), "bridle", ORCHESTRATOR);
        assert_eq!(orch(&["bridle", "m-home", "x"]), "m-home");
        assert_eq!(orch(&["bridle", "x"]), "bridle");
        assert_eq!(orch(&["x", "y"]), "x");
        // Other roles ignore <machine>-home.
        assert_eq!(
            home_for("m", &p(&["x", "m-home", "bridle"]), "notes", "aide"),
            "bridle"
        );
    }

    /// The pair-mint calls for the orchestrator: (project, --for, --home, --direct).
    fn orch_mints(s: &Sim) -> Vec<(String, Option<String>, Option<String>, bool)> {
        s.calls
            .borrow()
            .iter()
            .filter(|c| c.iter().any(|x| x == "pair-mint") && c.iter().any(|x| x == ORCHESTRATOR))
            .map(|c| {
                let at = |k: &str| c.iter().position(|x| x == k).map(|i| c[i + 1].clone());
                (
                    at("--project").unwrap(),
                    at("--for"),
                    at("--home"),
                    c.iter().any(|x| x == "--direct"),
                )
            })
            .collect()
    }

    // br-gr3v
    #[test]
    fn the_orchestrator_is_homed_in_the_machine_home_and_local_daemons_forward() {
        let mut s = sim();
        s.projects
            .get_mut("mbp")
            .unwrap()
            .extend([("mbp-home".into(), 7410), ("notes".into(), 7403)]);
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        let m = orch_mints(&s);
        let some = |x: &str| Some(x.to_string());
        // mbp's own daemons other than its home: a visitor homed in mbp-home, replacing the
        // direct token.
        for p in ["bridle", "notes"] {
            assert!(
                m.contains(&(p.into(), some("mbp"), some("mbp-home"), true)),
                "{m:?}"
            );
        }
        // Its home daemon keeps a direct token; aide stays direct everywhere on mbp.
        assert!(m.contains(&("mbp-home".into(), None, None, false)), "{m:?}");
        assert!(
            !s.calls
                .borrow()
                .iter()
                .any(|c| c.iter().any(|x| x == "aide")
                    && c.iter().any(|x| x == "pair-mint")
                    && c.iter().any(|x| x == "--direct"))
        );
        // nuc's orchestrator on mbp's daemons is homed in nuc's first project (nuc has no home).
        assert!(m.iter().any(|(p, f, h, _)| p == "bridle"
            && f.as_deref() == Some("nuc")
            && h.as_deref() == Some("notes")));
        // A second run changes nothing.
        let (_, out) = run(&s, &Options::default());
        assert_eq!(out.matches("(role tokens): minted 0").count(), 2, "{out}");
    }

    // br-j86p: a pair before this fix left the orchestrator a visitor on its own machine; a
    // re-pair replaces that token with a direct one that records the home.
    #[test]
    fn a_repair_replaces_a_local_visitor_token_with_a_direct_one() {
        let mut s = sim();
        s.projects
            .get_mut("mbp")
            .unwrap()
            .push(("mbp-home".into(), 7410));
        run(&s, &Options::default());
        let key = (
            "mbp".to_string(),
            ORCHESTRATOR.to_string(),
            None,
            "bridle".to_string(),
        );
        let tok = s.creds.borrow()[&key].clone();
        assert!(!s.local_visitors.borrow().contains(&tok));
        s.local_visitors.borrow_mut().insert(tok.clone());
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        let now = s.creds.borrow()[&key].clone();
        assert_ne!(now, tok, "{out}");
        assert!(!s.local_visitors.borrow().contains(&now));
        assert_eq!(s.homes.borrow()[&now].as_deref(), Some("mbp-home"));
    }

    // br-8rdn
    #[test]
    fn a_machine_with_no_home_names_the_projects_that_do_not_forward() {
        let mut s = sim();
        s.projects
            .get_mut("mbp")
            .unwrap()
            .extend([("notes".into(), 7403)]);
        let (_, out) = run(&s, &Options::default());
        assert!(
            out.contains("machine mbp has no mbp-home project") && out.contains("notes"),
            "{out}"
        );
        assert!(out.contains("re-run `bridle token pair`"), "{out}");
        s.projects
            .get_mut("mbp")
            .unwrap()
            .push(("mbp-home".into(), 7410));
        let (_, out) = run(&s, &Options::default());
        assert!(!out.contains("machine mbp has no"), "{out}");
    }

    // br-8rdn item 3: a machine WITH a home names a daemon whose visitor token is missing.
    #[test]
    fn a_missing_visitor_token_on_a_machine_with_a_home_is_named_by_check() {
        let mut s = sim();
        s.projects
            .get_mut("mbp")
            .unwrap()
            .extend([("mbp-home".into(), 7410), ("notes".into(), 7403)]);
        run(&s, &Options::default());
        let note = |s: &Sim| {
            let mut out = Vec::new();
            check(&Options::default(), &fleet(), s, &mut out, false).unwrap();
            String::from_utf8(out).unwrap()
        };
        assert!(!note(&s).contains("do not forward"), "{}", note(&s));
        s.valid
            .borrow_mut()
            .get_mut(&("mbp".into(), "notes".into()))
            .unwrap()
            .clear();
        let text = note(&s);
        assert!(
            text.contains("on machine mbp these daemons do not forward")
                && text.contains("mbp-home: notes"),
            "{text}"
        );
    }

    // br-gr3v, br-8usa: the home appears after a first pair; a re-pair moves the visitors.
    #[test]
    fn a_repair_moves_the_orchestrators_home_when_the_machine_home_appears() {
        let mut s = sim();
        run(&s, &Options::default());
        let held_by_nuc = |s: &Sim| {
            let tok = s.creds.borrow()[&(
                "nuc".into(),
                ORCHESTRATOR.into(),
                Some("mbp".into()),
                "bridle".into(),
            )]
                .clone();
            s.homes.borrow()[&tok].clone()
        };
        // nuc has no bridle and no home: its first project.
        assert_eq!(held_by_nuc(&s), Some("notes".to_string()));
        s.projects
            .get_mut("nuc")
            .unwrap()
            .push(("nuc-home".into(), 7420));
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        assert_eq!(held_by_nuc(&s), Some("nuc-home".to_string()));
    }

    // br-eurm, s-8088
    #[test]
    fn visitors_are_minted_with_a_home_and_no_home_is_reminted() {
        let mut s = sim();
        // mbp also has `notes`: the same-named project is the home there.
        s.projects
            .get_mut("mbp")
            .unwrap()
            .push(("notes".into(), 7403));
        run(&s, &Options::default());
        let mint_homes = |s: &Sim| -> Vec<(String, String, String)> {
            s.calls
                .borrow()
                .iter()
                .filter(|c| c.iter().any(|x| x == "pair-mint") && c.iter().any(|x| x == "--home"))
                .map(|c| {
                    let at = |k: &str| c[c.iter().position(|x| x == k).unwrap() + 1].clone();
                    (at("--project"), at("pair-mint"), at("--home"))
                })
                .collect()
        };
        let homes = mint_homes(&s);
        // nuc/notes held by mbp: mbp has `notes`.
        assert!(homes.contains(&("notes".into(), "aide".into(), "notes".into())));
        // mbp/bridle held by nuc: nuc has no `bridle` and no home for the role: its first project.
        assert!(homes.contains(&("bridle".into(), "aide".into(), "notes".into())));
        // Every cross-machine mint carried a home.
        let cross = s
            .calls
            .borrow()
            .iter()
            .filter(|c| c.iter().any(|x| x == "pair-mint") && c.iter().any(|x| x == "--for"))
            .count();
        assert_eq!(homes.len(), cross);

        // A visitor minted before homes existed is not working: the plain re-run re-mints it.
        for h in s.homes.borrow_mut().values_mut() {
            *h = None;
        }
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        assert!(out.contains("(role tokens): minted"), "{out}");
        assert!(!out.contains("(role tokens): minted 0"), "{out}");
        // And then a further run changes nothing.
        let before: BTreeMap<_, _> = s.creds.borrow().clone().into_iter().collect();
        let (_, out) = run(&s, &Options::default());
        assert_eq!(out.matches("(role tokens): minted 0").count(), 2, "{out}");
        let after: BTreeMap<_, _> = s.creds.borrow().clone().into_iter().collect();
        assert_eq!(before, after);
    }

    // br-8usa
    #[test]
    fn repairing_a_visitor_whose_home_changed_records_the_new_home() {
        let s = sim();
        run(&s, &Options::default());
        // The visitors' recorded homes were right; now the principal's home moved on.
        for h in s.homes.borrow_mut().values_mut() {
            if h.is_some() {
                *h = Some("old-home".into());
            }
        }
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        assert!(!out.contains("(role tokens): minted 0"), "{out}");
        // Every visitor now has the home it would be minted with, and a further run is a no-op.
        let homes = s.homes.borrow();
        let held: Vec<_> = s
            .creds
            .borrow()
            .iter()
            .filter(|(k, _)| k.2.is_some())
            .map(|(_, t)| homes[t].clone())
            .collect();
        assert!(!held.is_empty());
        assert!(
            held.iter()
                .all(|h| h.is_some() && h.as_deref() != Some("old-home"))
        );
        drop(homes);
        let (_, out) = run(&s, &Options::default());
        assert_eq!(out.matches("(role tokens): minted 0").count(), 2, "{out}");
    }

    // s-7504
    #[test]
    fn selectors_narrow_to_exactly_what_is_named() {
        let s = sim();
        let opts = Options {
            machines: vec!["mbp".into(), "nuc".into()],
            projects: vec!["notes".into()],
            roles: vec!["aide".into()],
            tokens: vec![TokenType::Role],
            ..Default::default()
        };
        let (ok, out) = run(&s, &opts);
        assert!(ok, "{out}");
        let creds = s.creds.borrow();
        assert_eq!(creds.len(), 2, "{out}");
        assert!(
            creds
                .keys()
                .all(|(_, p, _, proj)| p == "aide" && proj == "notes")
        );
        assert!(!out.contains("peer tokens"), "{out}");
    }

    // s-9267
    #[test]
    fn roles_with_only_peer_tokens_is_an_error() {
        let s = sim();
        let opts = Options {
            roles: vec!["aide".into()],
            tokens: vec![TokenType::Peer],
            ..Default::default()
        };
        assert!(pair(&opts, &fleet(), &s, &mut Vec::new()).is_err());
    }

    // s-ac41
    #[test]
    fn a_new_role_in_the_list_is_paired_and_nothing_else_changes() {
        // The list is a const, so the "new role" is simulated by pairing, then dropping one
        // role's entries (as if the role had not existed) and running again.
        let s = sim();
        run(&s, &Options::default());
        let before: HashMap<_, _> = s.creds.borrow().clone();
        s.creds.borrow_mut().retain(|(_, p, _, _), _| p != "aide");
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        let after = s.creds.borrow();
        assert_eq!(after.len(), before.len());
        for (k, v) in before.iter().filter(|(k, _)| k.1 != "aide") {
            assert_eq!(after.get(k), Some(v), "{k:?} changed");
        }
        assert!(after.keys().any(|k| k.1 == "aide"));
    }

    // s-e443
    #[test]
    fn a_down_machine_is_reported_the_rest_goes_on_and_it_fails() {
        let mut s = sim();
        s.down.push("nuc".into());
        let (ok, out) = run(&s, &Options::default());
        assert!(!ok);
        assert!(out.contains("machine nuc: unreachable"), "{out}");
        assert!(out.contains("machine mbp (role tokens): minted 4"), "{out}");
    }

    #[test]
    fn a_broken_entry_is_reminted_and_rotate_replaces_working_ones() {
        let s = sim();
        run(&s, &Options::default());
        s.valid.borrow_mut().clear();
        let (_, out) = run(&s, &Options::default());
        assert!(out.contains("minted 9, kept 0"), "{out}");
        let (_, out) = run(
            &s,
            &Options {
                rotate: true,
                ..Default::default()
            },
        );
        assert!(out.contains("minted 9, kept 0"), "{out}");
    }

    // br-su62
    #[test]
    fn pair_ends_by_checking_every_entry_and_fails_on_a_refusal() {
        let s = sim();
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        assert!(
            out.contains("check mbp orchestrator -> nuc/notes: accepted"),
            "{out}"
        );
        assert!(
            out.contains("check nuc peer -> mbp/bridle: accepted"),
            "{out}"
        );
        // A refused token the keep check cannot see (the daemon turns it away later): the
        // final check names it and the run fails.
        let mut s2 = sim();
        s2.refuse_after_mint = true;
        let (ok, out) = run(&s2, &Options::default());
        assert!(!ok, "{out}");
        assert!(
            out.contains("check mbp orchestrator -> nuc/notes: refused: rejected"),
            "{out}"
        );
    }

    #[test]
    fn a_refused_token_is_reminted_by_a_plain_run() {
        let s = sim();
        run(&s, &Options::default());
        let before = s.creds.borrow()[&(
            "mbp".into(),
            "orchestrator".into(),
            Some("nuc".into()),
            "notes".into(),
        )]
            .clone();
        s.valid
            .borrow_mut()
            .get_mut(&("nuc".into(), "notes".into()))
            .unwrap()
            .remove(&before);
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        let after = s.creds.borrow()[&(
            "mbp".into(),
            "orchestrator".into(),
            Some("nuc".into()),
            "notes".into(),
        )]
            .clone();
        assert_ne!(before, after);
    }

    #[test]
    fn check_is_read_only_and_reports_accepted_refused_and_unreachable() {
        let s = sim();
        run(&s, &Options::default());
        let calls = s.calls.borrow().len();
        let mut out = Vec::new();
        assert!(check(&Options::default(), &fleet(), &s, &mut out, false).unwrap());
        assert!(!String::from_utf8(out).unwrap().contains("refused"));
        s.valid.borrow_mut().clear();
        let mut out = Vec::new();
        assert!(!check(&Options::default(), &fleet(), &s, &mut out, false).unwrap());
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.contains("check mbp advisor -> nuc/notes: refused: rejected"),
            "{text}"
        );
        let mut out = Vec::new();
        check(&Options::default(), &fleet(), &s, &mut out, true).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert!(
            v.as_array()
                .unwrap()
                .iter()
                .any(|e| e["accepted"] == false && e["reason"] == "rejected")
        );
        // Nothing but checks ran after the first pairing.
        assert!(!s.calls.borrow()[calls..].iter().any(|c| {
            c.iter()
                .any(|x| x == "pair-mint" || x == "pair-store" || x == "pair-peer-mint")
        }));
        let mut s2 = sim();
        s2.down.push("nuc".into());
        let mut out = Vec::new();
        assert!(!check(&Options::default(), &fleet(), &s2, &mut out, false).unwrap());
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("machine nuc: unreachable")
        );
    }

    // s-d9c0
    #[test]
    fn dry_run_mints_and_writes_nothing() {
        let s = sim();
        let (ok, out) = run(
            &s,
            &Options {
                dry_run: true,
                ..Default::default()
            },
        );
        assert!(ok);
        assert!(out.contains("would mint 9"), "{out}");
        assert!(s.creds.borrow().is_empty());
        assert!(s.valid.borrow().is_empty());
        assert!(!s.calls.borrow().iter().any(|c| {
            c.iter()
                .any(|x| x.starts_with("pair-m") || x == "pair-store")
        }));
    }

    fn peer_only() -> Options {
        Options {
            tokens: vec![TokenType::Peer],
            ..Default::default()
        }
    }

    fn peer_entries(s: &Sim) -> Vec<(String, String)> {
        let mut v: Vec<_> = s
            .creds
            .borrow()
            .keys()
            .filter(|k| k.1 == "peer")
            .map(|k| (k.0.clone(), k.3.clone()))
            .collect();
        v.sort();
        v
    }

    // s-b3a1: minted on the receiver, written on the sender, keyed by the receiving project
    #[test]
    fn peer_tokens_are_minted_on_the_receiver_and_stored_on_the_sender() {
        let s = sim();
        let (ok, out) = run(&s, &peer_only());
        assert!(ok, "{out}");
        assert_eq!(
            peer_entries(&s),
            [
                ("mbp".into(), "notes".into()),
                ("nuc".into(), "bridle".into())
            ]
        );
        assert!(
            out.contains("peer: minted on the receiver for mbp -> nuc/notes"),
            "{out}"
        );
        // The token for notes was minted on nuc, for sender mbp.
        assert!(
            s.peer
                .borrow()
                .contains_key(&("nuc".into(), "notes".into(), "mbp".into()))
        );
        assert!(
            !s.creds.borrow().keys().any(|k| k.1 != "peer"),
            "no role tokens"
        );
        // Idempotent; --rotate re-mints.
        let (_, out) = run(&s, &peer_only());
        assert!(out.contains("(peer tokens): minted 0, kept 1"), "{out}");
        let rotate = Options {
            rotate: true,
            ..peer_only()
        };
        let (_, out) = run(&s, &rotate);
        assert!(out.contains("(peer tokens): minted 1, kept 0"), "{out}");
    }

    // s-b3a1: a receiver that lost its token (revoked) is re-minted
    #[test]
    fn a_revoked_peer_token_is_reminted() {
        let s = sim();
        run(&s, &peer_only());
        s.peer.borrow_mut().clear();
        let (ok, out) = run(&s, &peer_only());
        assert!(ok);
        assert!(out.contains("(peer tokens): minted 1, kept 0"), "{out}");
    }

    // s-4e7d
    #[test]
    fn peers_false_leaves_a_project_out_both_ways_but_keeps_its_role_tokens() {
        let mut s = sim();
        s.no_peers.push(("nuc".into(), "notes".into()));
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        // notes neither sends (nuc holds nothing) nor receives (mbp holds nothing for it).
        assert!(peer_entries(&s).is_empty(), "{:?}\n{out}", peer_entries(&s));
        let roles = s
            .creds
            .borrow()
            .keys()
            .filter(|k| k.1 != "peer" && k.3 == "notes")
            .count();
        assert!(roles > 0, "role tokens for notes are still minted");
    }

    // s-c92f
    #[test]
    fn peer_dry_run_mints_nothing_and_no_token_leaks() {
        let s = sim();
        let dry = Options {
            dry_run: true,
            ..peer_only()
        };
        let (ok, out) = run(&s, &dry);
        assert!(ok);
        assert!(out.contains("would mint 1"), "{out}");
        assert!(s.creds.borrow().is_empty() && s.peer.borrow().is_empty());
        run(&s, &peer_only());
        for call in s.calls.borrow().iter() {
            assert!(call.iter().all(|a| !a.contains("secret-")), "{call:?}");
        }
    }

    // s-b3a1: two projects on one machine send to each other too (all mail goes via forward)
    #[test]
    fn same_machine_projects_get_peer_tokens() {
        let mut s = sim();
        s.projects
            .get_mut("mbp")
            .unwrap()
            .push(("other".into(), 7403));
        let (_, out) = run(&s, &peer_only());
        assert!(
            peer_entries(&s).contains(&("mbp".into(), "other".into())),
            "{out}"
        );
        assert!(
            peer_entries(&s).contains(&("mbp".into(), "bridle".into())),
            "{out}"
        );
    }

    // br-g9qm: a new project's machine needs peer entries to projects outside --projects, made
    // before the role tokens that depend on them
    #[test]
    fn projects_filter_still_makes_peer_entries_first() {
        let mut s = sim();
        s.projects
            .get_mut("nuc")
            .unwrap()
            .push(("home".into(), 7403));
        let only = Options {
            projects: vec!["home".into()],
            ..Default::default()
        };
        let (ok, out) = run(&s, &only);
        assert!(ok, "{out}");
        assert!(
            peer_entries(&s).contains(&("nuc".into(), "bridle".into())),
            "{out}"
        );
        let peer = out.find("(peer tokens)").expect("peer section");
        let role = out.find("(role tokens)").expect("role section");
        assert!(peer < role, "{out}");
        let (ok, out) = run(&s, &only);
        assert!(ok, "{out}");
        assert!(
            !out.contains("minted 1") && !out.contains("minted 2"),
            "{out}"
        );
    }

    // br-eg64: after a take-over the receiver's database is fresh, so the OTHER machines' peer
    // entries toward the moved project are stale; a run narrowed to it re-mints them
    #[test]
    fn projects_filter_repairs_other_machines_peer_entries_toward_it() {
        let s = sim();
        let (ok, out) = run(&s, &Options::default());
        assert!(ok, "{out}");
        // notes moves: nuc's daemon starts over with no tokens
        s.peer.borrow_mut().retain(|k, _| k.0 != "nuc");
        s.valid.borrow_mut().retain(|k, _| k.0 != "nuc");
        let only = Options {
            projects: vec!["notes".into()],
            ..Default::default()
        };
        let mut out = Vec::new();
        assert!(!check(&only, &fleet(), &s, &mut out, false).unwrap());
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("check mbp peer -> nuc/notes: refused"),
        );
        let (ok, out) = run(&s, &only);
        assert!(ok, "{out}");
        assert!(
            out.contains("peer: minted on the receiver for mbp -> nuc/notes"),
            "{out}"
        );
        let mut out = Vec::new();
        assert!(check(&only, &fleet(), &s, &mut out, false).unwrap());
    }

    // s-f12f
    #[test]
    fn no_token_in_argv_or_output() {
        let s = sim();
        let (_, out) = run(&s, &Options::default());
        assert!(!out.contains("secret-"), "{out}");
        for call in s.calls.borrow().iter() {
            assert!(call.iter().all(|a| !a.contains("secret-")), "{call:?}");
        }
    }

    #[test]
    fn unknown_names_are_refused_up_front() {
        let s = sim();
        for opts in [
            Options {
                roles: vec!["nope".into()],
                ..Default::default()
            },
            Options {
                machines: vec!["nope".into()],
                ..Default::default()
            },
            Options {
                projects: vec!["nope".into()],
                ..Default::default()
            },
        ] {
            assert!(pair(&opts, &fleet(), &s, &mut Vec::new()).is_err());
        }
    }

    // s-d290: launchers may only sign as a listed role, so a new role can't skip the list.
    #[test]
    fn launchers_set_bridle_as_only_to_listed_roles() {
        fn walk(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, files);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    files.push(p);
                }
            }
        }
        let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut files = Vec::new();
        for c in ["bridle", "bridle-daemon", "bridle-mail", "bridle-gateway"] {
            walk(&crates.join(c).join("src"), &mut files);
        }
        // Literal sets: `("BRIDLE_AS", "x")`, `BRIDLE_AS=x` and the plist's `<string>x</string>`.
        let mut seen = 0;
        for f in files {
            // This file spells the patterns out.
            if f.ends_with("token_pair.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&f).unwrap();
            let mut found: Vec<String> = Vec::new();
            for line in text.lines() {
                if let Some(rest) = line.split("(\"BRIDLE_AS\", \"").nth(1) {
                    found.push(rest.split('"').next().unwrap().to_string());
                }
                if let Some(rest) = line.split("BRIDLE_AS=").nth(1)
                    && !line.contains("//")
                {
                    found.push(
                        rest.chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                            .collect(),
                    );
                }
            }
            for role in found {
                seen += 1;
                if role.is_empty() || role.contains('{') {
                    continue;
                }
                assert!(
                    TOKEN_ROLES.iter().any(|r| r.name == role),
                    "{}: BRIDLE_AS set to '{role}', which is not in token_pair::TOKEN_ROLES",
                    f.display()
                );
            }
        }
        assert!(seen > 0, "the scan found no launcher: update the patterns");
    }
}
