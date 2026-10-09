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
    },
    TokenRole {
        name: ORCHESTRATOR,
        remote_only: false,
    },
    TokenRole {
        name: ADVISOR,
        remote_only: false,
    },
    TokenRole {
        name: AIDE,
        remote_only: false,
    },
    TokenRole {
        name: MAIL,
        remote_only: false,
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

/// The real runner: the running binary locally, plain `ssh <host> bridle ...` elsewhere.
pub struct Exec;

impl Runner for Exec {
    fn run(&self, at: &Where, args: &[String], stdin: Option<&str>) -> Result<String, String> {
        let mut cmd = match at {
            Where::Local => {
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let mut c = Command::new(exe);
                c.args(args);
                c
            }
            Where::Ssh(host) => {
                let mut c = Command::new("ssh");
                // No prompt can be answered here: fail rather than hang.
                c.args(["-o", "BatchMode=yes", host, "bridle"]).args(args);
                c
            }
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

/// Runs the pairing. Returns whether everything went through (the exit code is non-zero
/// otherwise). Err is a usage problem found before anything ran.
pub fn pair(
    opts: &Options,
    fleet: &Fleet,
    runner: &dyn Runner,
    out: &mut dyn Write,
) -> Result<bool, String> {
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
    let roles: Vec<&str> = TOKEN_ROLES
        .iter()
        .map(|t| t.name)
        .filter(|n| opts.roles.is_empty() || opts.roles.iter().any(|r| r == n))
        .collect();

    // Each reachable machine and its projects (name, port).
    let mut ok = true;
    let mut up: BTreeMap<String, Vec<(String, u16)>> = BTreeMap::new();
    // (machine, project) pairs that left peer tokens out (`[mail] peers = false`).
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
        found.retain(|(p, _)| opts.projects.is_empty() || opts.projects.contains(p));
        found.sort();
        up.insert(m.clone(), found);
    }
    for p in &opts.projects {
        if !up.values().flatten().any(|(f, _)| f == p) && up.len() == machines.len() {
            return Err(format!(
                "no daemon for project '{p}' on the selected machines"
            ));
        }
    }

    for holder in up.keys().filter(|_| do_role) {
        let mut t = Tally::default();
        for (daemon, projects) in &up {
            for (project, port) in projects {
                if !plain(project) {
                    t.failed += 1;
                    t.lines
                        .push(format!("project '{project}' is not a plain name"));
                    continue;
                }
                for role in &roles {
                    let remote_only = TOKEN_ROLES.iter().any(|r| r.name == *role && r.remote_only);
                    let same = holder == daemon;
                    if remote_only && same {
                        continue;
                    }
                    let label = format!("{role} -> {daemon}/{project}");
                    match one(
                        opts, fleet, runner, holder, daemon, project, *port, role, same,
                    ) {
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
            }
        }
        if t.failed > 0 {
            ok = false;
        }
        report(out, opts, holder, "role", &t);
    }

    // Peer tokens: a daemon forwards mail with a token the receiving daemon minted for the
    // sending MACHINE, so each machine with a sending project needs one per receiving project.
    for holder in up.keys().filter(|_| do_peer) {
        let senders: Vec<&String> = up[holder]
            .iter()
            .map(|(p, _)| p)
            .filter(|p| !no_peers.contains(&(holder.clone(), (*p).clone())))
            .collect();
        let mut t = Tally::default();
        for (daemon, projects) in &up {
            for (project, _) in projects {
                if no_peers.contains(&(daemon.clone(), project.clone())) {
                    continue;
                }
                // A project's only sender being itself needs no token.
                if daemon == holder && senders.len() == 1 && senders[0] == project {
                    continue;
                }
                if senders.is_empty() {
                    continue;
                }
                if !plain(project) {
                    t.failed += 1;
                    t.lines
                        .push(format!("project '{project}' is not a plain name"));
                    continue;
                }
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
        }
        if t.failed > 0 {
            ok = false;
        }
        if t.minted + t.kept + t.would + t.failed > 0 {
            report(out, opts, holder, "peer", &t);
        }
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
    if !opts.rotate {
        let held = runner
            .run(
                &holder_at,
                &args(&["--project", project, "token", "pair-peer-held"]),
                None,
            )
            .is_ok();
        let live = held
            && runner
                .run(
                    &daemon_at,
                    &args(&["--project", project, "token", "pair-peer-active", holder]),
                    None,
                )
                .is_ok();
        if live {
            return Ok(Outcome::Kept);
        }
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

enum Outcome {
    Kept,
    Minted,
    Would,
}

#[allow(clippy::too_many_arguments)]
fn one(
    opts: &Options,
    fleet: &Fleet,
    runner: &dyn Runner,
    holder: &str,
    daemon: &str,
    project: &str,
    port: u16,
    role: &str,
    same: bool,
) -> Result<Outcome, String> {
    let holder_at = fleet.at(holder);
    let daemon_at = fleet.at(daemon);
    if !opts.rotate {
        // The holder judges its own entry against the daemon, so no token leaves it.
        let mut a = args(&["--project", project]);
        if !same {
            let host = fleet
                .hosts
                .get(daemon)
                .ok_or_else(|| format!("machine {daemon} has no host in [machines]"))?;
            a.extend(args(&["--url", &format!("http://{host}:{port}")]));
        }
        a.extend(args(&["token", "pair-check", role]));
        if !same {
            a.extend(args(&["--machine", daemon]));
        }
        if runner.run(&holder_at, &a, None).is_ok() {
            return Ok(Outcome::Kept);
        }
    }
    if opts.dry_run {
        return Ok(Outcome::Would);
    }
    let mut mint = args(&["--project", project, "token", "pair-mint", role]);
    if !same {
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
                    if valid
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
                    let mut valid = self.valid.borrow_mut();
                    let set = valid.entry((here, project)).or_default();
                    // A principal has one active token: the old one is revoked.
                    let _ = a[i + 2].as_str();
                    set.insert(tok.clone());
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
