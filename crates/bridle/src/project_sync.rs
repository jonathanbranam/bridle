//! `bridle projects sync` (ticket 2wnn): compare the `[machines]` and `[projects]` lists of
//! `~/.bridle/config.toml` (name -> host; project -> machine, port) across this machine and every `[machines]` entry, and with
//! `--apply` update the others to match this one. This machine wins; nothing is ever deleted.
//! The file is hand-edited, so edits go through toml_edit and leave comments and other tables
//! alone. Remote files are read and written over plain ssh (write to a temp file, then rename).

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

use crate::token_pair::{Fleet, Where};

/// Reads and writes a machine's `config.toml` text. A missing file reads as empty.
pub trait Files {
    fn read(&self, at: &Where) -> Result<String, String>;
    fn write(&self, at: &Where, text: &str) -> Result<(), String>;
}

/// The real thing: the local file, and `ssh <host>` with the remote shell expanding `$HOME`.
pub struct Real {
    pub home: PathBuf,
}

const REMOTE_FILE: &str = "$HOME/.bridle/config.toml";

/// How long one ssh call may take in all. Tailscale SSH in "check" mode waits for a browser
/// login that BatchMode does not stop, and ConnectTimeout only covers the connect.
pub(crate) const SSH_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) fn ssh(host: &str, script: &str, stdin: Option<&str>) -> Result<String, String> {
    ssh_with("ssh", host, script, stdin, SSH_TIMEOUT)
}

fn ssh_with(
    program: &str,
    host: &str,
    script: &str,
    stdin: Option<&str>,
    timeout: Duration,
) -> Result<String, String> {
    let mut cmd = Command::new(program);
    // No prompt can be answered here: fail rather than hang.
    cmd.args([
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=10",
        host,
        script,
    ]);
    run_timed(cmd, host, stdin, timeout)
}

/// Runs an ssh command with `stdin`, killing it after `timeout`. The error names `host`.
pub(crate) fn run_timed(
    cmd: Command,
    host: &str,
    stdin: Option<&str>,
    timeout: Duration,
) -> Result<String, String> {
    let (code, stdout, stderr) = run_timed_raw(cmd, host, stdin, timeout)?;
    if code == 0 {
        Ok(String::from_utf8_lossy(&stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&stderr);
        let err = err.trim();
        Err(match err.lines().last() {
            Some(l) => l.to_string(),
            None => format!("ssh failed (exit {code})"),
        })
    }
}

/// `run_timed` for callers that need the exit code and both streams. Err is only a start
/// failure or the timeout.
pub(crate) fn run_timed_raw(
    mut cmd: Command,
    host: &str,
    stdin: Option<&str>,
    timeout: Duration,
) -> Result<(i32, Vec<u8>, Vec<u8>), String> {
    let mut child = cmd
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start ssh: {e}"))?;
    if let (Some(text), Some(mut pipe)) = (stdin, child.stdin.take()) {
        pipe.write_all(text.as_bytes())
            .map_err(|e| format!("write to ssh: {e}"))?;
    }
    let readers: Vec<_> = [
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    ]
    .into_iter()
    .map(|p| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = p {
                let _ = p.read_to_end(&mut buf);
            }
            buf
        })
    })
    .collect();
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait().map_err(|e| format!("ssh: {e}"))? {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "no answer from `{host}` after {}s (ssh may be waiting for a login, e.g. \
                     Tailscale SSH check mode); try `ssh {host}` by hand",
                    timeout.as_secs()
                ));
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    let mut bufs = readers.into_iter().map(|h| h.join().unwrap_or_default());
    let (stdout, stderr) = (
        bufs.next().unwrap_or_default(),
        bufs.next().unwrap_or_default(),
    );
    Ok((status.code().unwrap_or(255), stdout, stderr))
}

impl Files for Real {
    fn read(&self, at: &Where) -> Result<String, String> {
        match at {
            Where::Local => match std::fs::read_to_string(self.home.join("config.toml")) {
                Ok(t) => Ok(t),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
                Err(e) => Err(e.to_string()),
            },
            Where::Ssh(host) => ssh(
                host,
                &format!("if [ -f \"{REMOTE_FILE}\" ]; then cat \"{REMOTE_FILE}\"; fi"),
                None,
            ),
        }
    }

    fn write(&self, at: &Where, text: &str) -> Result<(), String> {
        match at {
            Where::Local => {
                let path = self.home.join("config.toml");
                let tmp = self.home.join("config.toml.sync-tmp");
                std::fs::write(&tmp, text)
                    .and_then(|()| std::fs::rename(&tmp, &path))
                    .map_err(|e| e.to_string())
            }
            Where::Ssh(host) => ssh(
                host,
                &format!(
                    "mkdir -p \"$HOME/.bridle\" && cat > \"{REMOTE_FILE}.sync-tmp\" \
                     && mv \"{REMOTE_FILE}.sync-tmp\" \"{REMOTE_FILE}\""
                ),
                Some(text),
            )
            .map(|_| ()),
        }
    }
}

/// One `[machines]` or `[projects]` value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// `[machines]`: name -> host
    Host(String),
    /// `[projects]`: project -> (machine, port)
    Place(String, u16),
    /// `[mail]`: key -> its TOML value, normalised (no comments or line breaks).
    Mail(String),
}

impl Entry {
    fn show(&self) -> String {
        match self {
            Entry::Host(h) => h.clone(),
            Entry::Place(m, p) => format!("{m}:{p}"),
            Entry::Mail(v) => v.clone(),
        }
    }
}

/// (section, key) -> value, for both sections together.
pub type List = BTreeMap<(&'static str, String), Entry>;

fn label(k: &(&'static str, String)) -> String {
    format!("[{}] {}", k.0, k.1)
}

/// This machine's list, from the loaded config.
pub fn list_of(fleet: &Fleet) -> List {
    let hosts = fleet
        .hosts
        .iter()
        .map(|(n, h)| (("machines", n.clone()), Entry::Host(h.clone())));
    let places = fleet
        .placed
        .iter()
        .map(|(p, (m, port))| (("projects", p.clone()), Entry::Place(m.clone(), *port)));
    hosts.chain(places).collect()
}

/// The `[machines]` and `[projects]` entries of a config.toml text, and its `[machine] name`.
pub fn parse(text: &str) -> Result<(List, Option<String>), String> {
    let doc: DocumentMut = text
        .parse()
        .map_err(|e| format!("invalid config.toml: {e}"))?;
    let mut out = List::new();
    if let Some(machines) = doc.get("machines").and_then(Item::as_table_like) {
        for (name, item) in machines.iter() {
            let host = item
                .as_str()
                .ok_or(format!("[machines] {name}: want a host string"))?;
            out.insert(
                ("machines", name.to_string()),
                Entry::Host(host.to_string()),
            );
        }
    }
    if let Some(projects) = doc.get("projects").and_then(Item::as_table_like) {
        for (name, item) in projects.iter() {
            let place = item.as_table_like().and_then(|t| {
                let machine = t.get("machine")?.as_str()?.to_string();
                let port = u16::try_from(t.get("port")?.as_integer()?).ok()?;
                Some(Entry::Place(machine, port))
            });
            let place = place.ok_or(format!("[projects] {name}: want {{ machine, port }}"))?;
            out.insert(("projects", name.to_string()), place);
        }
    }
    let name = doc
        .get("machine")
        .and_then(Item::as_table_like)
        .and_then(|t| t.get("name")?.as_str().map(str::to_string));
    Ok((out, name))
}

fn normalise(v: &Value) -> Value {
    match v {
        Value::Array(a) => Value::Array(a.iter().map(normalise).collect()),
        other => {
            let mut v = other.clone();
            v.decor_mut().clear();
            v
        }
    }
}

/// The `[mail]` section of a config.toml text, key -> value. Empty when there is none.
pub fn parse_mail(text: &str) -> Result<List, String> {
    let doc: DocumentMut = text
        .parse()
        .map_err(|e| format!("invalid config.toml: {e}"))?;
    let mut out = List::new();
    if let Some(mail) = doc.get("mail").and_then(Item::as_table_like) {
        for (key, item) in mail.iter() {
            let v = item
                .as_value()
                .ok_or(format!("[mail] {key}: want a plain value"))?;
            out.insert(
                ("mail", key.to_string()),
                Entry::Mail(normalise(v).to_string()),
            );
        }
    }
    Ok(out)
}

/// What `[mail] projects` must be on every machine (kx5u K6, afzu): every `[projects]` name,
/// `bridle` first, so the same bridge answers mail to unknown names everywhere.
pub fn mail_projects(fleet: &Fleet) -> Entry {
    let mut names = vec!["bridle".to_string()];
    names.extend(fleet.placed.keys().filter(|p| *p != "bridle").cloned());
    let arr: toml_edit::Array = names.iter().map(String::as_str).collect();
    Entry::Mail(normalise(&Value::Array(arr)).to_string())
}

/// The names in a `[mail] projects` entry (empty when it is not a list of strings).
fn mail_names(e: Option<&Entry>) -> Vec<String> {
    let Some(Entry::Mail(v)) = e else {
        return vec![];
    };
    v.parse::<Value>()
        .ok()
        .and_then(|v| {
            v.as_array().map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
        })
        .unwrap_or_default()
}

/// `mail_projects` plus any name some machine already lists that `[projects]` does not: sync
/// adds projects but never drops an enrolment just because the project is not registered yet.
/// Returns the entry and the names it left alone.
fn mail_projects_keeping(fleet: &Fleet, seen: &[Vec<String>]) -> (Entry, Vec<String>) {
    let mut names = mail_names(Some(&mail_projects(fleet)));
    let mut kept = vec![];
    for n in seen.iter().flatten() {
        if !names.contains(n) {
            names.push(n.clone());
            kept.push(n.clone());
        }
    }
    let arr: toml_edit::Array = names.iter().map(String::as_str).collect();
    (Entry::Mail(normalise(&Value::Array(arr)).to_string()), kept)
}

/// One difference between the source list and a target's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diff {
    /// The target lacks the entry.
    Missing(String, Entry),
    /// The target has it with another value: (key, source, target).
    Differs(String, Entry, Entry),
    /// The target has an entry the source lacks; never touched.
    Only(String, Entry),
}

pub fn diff(source: &List, target: &List) -> Vec<((&'static str, String), Diff)> {
    let mut out = Vec::new();
    for (k, s) in source {
        match target.get(k) {
            None => out.push((k.clone(), Diff::Missing(label(k), s.clone()))),
            Some(t) if t != s => {
                out.push((k.clone(), Diff::Differs(label(k), s.clone(), t.clone())));
            }
            Some(_) => {}
        }
    }
    for (k, t) in target {
        if !source.contains_key(k) {
            out.push((k.clone(), Diff::Only(label(k), t.clone())));
        }
    }
    out
}

/// Applies the Missing and Differs entries to the file text, touching nothing else.
pub fn apply_text(text: &str, diffs: &[((&'static str, String), Diff)]) -> Result<String, String> {
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e| format!("invalid config.toml: {e}"))?;
    for ((section, name), d) in diffs {
        let entry = match d {
            Diff::Missing(_, s) | Diff::Differs(_, s, _) => s,
            Diff::Only(..) => continue,
        };
        if !doc.contains_key(section) {
            doc[*section] = Item::Table(Table::new());
        }
        let table = doc[*section]
            .as_table_like_mut()
            .ok_or(format!("[{section}] is not a table"))?;
        match entry {
            Entry::Host(h) => {
                table.insert(name, toml_edit::value(h.as_str()));
            }
            Entry::Mail(v) => {
                let value: Value = v
                    .parse()
                    .map_err(|e| format!("[mail] {name}: bad value {v}: {e}"))?;
                table.insert(name, Item::Value(value));
            }
            Entry::Place(machine, port) => {
                if let Some(t) = table.get_mut(name).and_then(Item::as_table_like_mut) {
                    // Keep the entry's own formatting; only the two values change.
                    t.insert("machine", toml_edit::value(machine.as_str()));
                    t.insert("port", toml_edit::value(i64::from(*port)));
                } else {
                    let mut t = InlineTable::new();
                    t.insert("machine", Value::from(machine.as_str()));
                    t.insert("port", Value::from(i64::from(*port)));
                    table.insert(name, Item::Value(Value::InlineTable(t)));
                }
            }
        }
    }
    Ok(doc.to_string())
}

fn cell(e: Option<&Entry>) -> String {
    e.map_or("-".to_string(), Entry::show)
}

pub fn run(action: &crate::cli::ProjectsAction) -> Result<(), crate::error::CliError> {
    let crate::cli::ProjectsAction::Sync { apply } = action else {
        unreachable!("take-over and stop are dispatched before this")
    };
    let home = bridle_api::discovery::bridle_home();
    let map = bridle_api::machines::MachineMap::load(&home)
        .map_err(|e| crate::error::CliError::Other(e.into()))?;
    let fleet = Fleet::from_config(&map);
    let ok = sync(&fleet, &Real { home }, *apply, &mut std::io::stdout());
    if ok {
        Ok(())
    } else {
        Err(crate::error::CliError::Other(anyhow::anyhow!(
            "some machines could not be read or written"
        )))
    }
}

/// `bridle project add`: list the project in this machine's `[projects]`, then `sync` it (and
/// `[mail] projects`, which follows `[projects]`) to every machine.
pub fn add(args: &crate::cli::ProjectAddArgs) -> Result<(), crate::error::CliError> {
    let home = bridle_api::discovery::bridle_home();
    let map = bridle_api::machines::MachineMap::load(&home)
        .map_err(|e| crate::error::CliError::Other(e.into()))?;
    let fleet = Fleet::from_config(&map);
    let files = Real { home };
    let mut out = std::io::stdout();
    add_with(&fleet, &files, args, &mut out).map_err(crate::error::CliError::Other)
}

/// The first port above every port in `[projects]` (any machine; 7405 may be listed twice), or
/// 7401 when none is listed.
fn next_port(fleet: &Fleet) -> Result<u16, String> {
    match fleet.placed.values().map(|(_, p)| *p).max() {
        None => Ok(7401),
        Some(p) => p
            .checked_add(1)
            .ok_or("no port above the highest in use".into()),
    }
}

fn add_with(
    fleet: &Fleet,
    files: &dyn Files,
    args: &crate::cli::ProjectAddArgs,
    out: &mut dyn Write,
) -> Result<(), anyhow::Error> {
    let name = &args.name;
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        anyhow::bail!("`{name}` is not a project name: use letters, digits, `.`, `_` and `-`");
    }
    if let Some((m, p)) = fleet.placed.get(name) {
        anyhow::bail!("{name} is already in [projects] ({m}:{p}); nothing changed");
    }
    let machine = args.machine.clone().unwrap_or_else(|| fleet.local.clone());
    if !fleet.hosts.contains_key(&machine) {
        let known: Vec<&str> = fleet.hosts.keys().map(String::as_str).collect();
        anyhow::bail!(
            "`{machine}` is not a key of [machines] (known: {}); add it there first",
            known.join(", ")
        );
    }
    let port = match args.port {
        Some(p) => p,
        None => next_port(fleet).map_err(anyhow::Error::msg)?,
    };
    let entry = Entry::Place(machine.clone(), port);
    let diff = Diff::Missing(format!("[projects] {name}"), entry);
    let text = files.read(&Where::Local).map_err(anyhow::Error::msg)?;
    let text =
        apply_text(&text, &[(("projects", name.clone()), diff)]).map_err(anyhow::Error::msg)?;
    files
        .write(&Where::Local, &text)
        .map_err(anyhow::Error::msg)?;
    let _ = writeln!(out, "added [projects] {name} = {machine}:{port}");
    let mut fleet = fleet.clone();
    fleet.placed.insert(name.clone(), (machine, port));
    if !sync(&fleet, files, true, out) {
        anyhow::bail!(
            "{name} is listed here, but some machines could not be read or written; fix it, \
             then run `bridle projects sync --apply`"
        );
    }
    let _ = writeln!(out, "next: `bridle project start`");
    Ok(())
}

/// Compares (and with `apply`, updates) every machine. Returns whether every machine could be
/// read and, when applying, written.
pub fn sync(fleet: &Fleet, files: &dyn Files, apply: bool, out: &mut dyn Write) -> bool {
    let mut ok = true;
    let mut source = list_of(fleet);
    // This machine's [mail] is spread too, with `projects` derived from [projects] (afzu). With
    // no [mail] here there is nothing to spread.
    let local_text = files.read(&Where::Local).unwrap_or_default();
    let local_mail = parse_mail(&local_text).unwrap_or_default();
    let mut mail_source = local_mail.clone();
    let mut names: Vec<String> = vec![fleet.local.clone()];
    let mut lists: Vec<Option<List>> = vec![Some(source.clone())];
    let mut texts: Vec<String> = vec![String::new()];
    let mut selves: Vec<Option<String>> = vec![Some(fleet.local.clone())];
    for (name, host) in &fleet.hosts {
        if *name == fleet.local {
            continue;
        }
        names.push(name.clone());
        let got = files.read(&Where::Ssh(host.clone())).and_then(|t| {
            let mut l = parse(&t)?;
            l.0.extend(parse_mail(&t)?);
            Ok((t, l.0, l.1))
        });
        match got {
            Ok((t, l, n)) => {
                texts.push(t);
                lists.push(Some(l));
                selves.push(n);
            }
            Err(e) => {
                let _ = writeln!(out, "{name}: cannot read config ({e}); skipped");
                ok = false;
                texts.push(String::new());
                lists.push(None);
                selves.push(None);
            }
        }
    }

    // Derive `[mail] projects` now every machine has been read, so no machine's extra entry is lost.
    if !mail_source.is_empty() {
        let key = ("mail", "projects".to_string());
        let seen: Vec<Vec<String>> = std::iter::once(&local_mail)
            .chain(lists.iter().skip(1).flatten())
            .map(|l| mail_names(l.get(&key)))
            .collect();
        let (entry, kept) = mail_projects_keeping(fleet, &seen);
        if !kept.is_empty() {
            let _ = writeln!(
                out,
                "[mail] projects: left {} (not in [projects])",
                kept.join(", ")
            );
        }
        mail_source.insert(("mail", "projects".into()), entry);
    }
    source.extend(mail_source.clone());
    lists[0] = Some(source.clone());

    let _ = writeln!(out, "entry\t{}", names.join("\t"));
    let mut all: Vec<&(&'static str, String)> =
        lists.iter().flatten().flat_map(|l| l.keys()).collect();
    all.sort();
    all.dedup();
    for k in all {
        let cells: Vec<String> = lists
            .iter()
            .map(|l| l.as_ref().map_or("?".to_string(), |l| cell(l.get(k))))
            .collect();
        let _ = writeln!(out, "{}\t{}", label(k), cells.join("\t"));
    }

    for (i, name) in names.iter().enumerate().skip(1) {
        let Some(target) = &lists[i] else { continue };
        let diffs = diff(&source, target);
        if diffs.is_empty() {
            let _ = writeln!(out, "{name}: same");
        }
        for (_, d) in &diffs {
            let _ = match d {
                Diff::Missing(k, s) => writeln!(out, "{name}: missing {k} ({})", cell(Some(s))),
                Diff::Differs(k, s, t) => writeln!(
                    out,
                    "{name}: differs {k} (here {}, there {})",
                    cell(Some(s)),
                    cell(Some(t))
                ),
                Diff::Only(k, t) => {
                    writeln!(out, "{name}: only on {name} {k} ({}); kept", cell(Some(t)))
                }
            };
        }
        let changes = diffs
            .iter()
            .filter(|(_, d)| !matches!(d, Diff::Only(..)))
            .count();
        let mut effective = target.clone();
        if apply && changes > 0 {
            let host = Where::Ssh(fleet.hosts[name].clone());
            let done = apply_text(&texts[i], &diffs).and_then(|t| files.write(&host, &t));
            match done {
                Ok(()) => {
                    for (k, d) in &diffs {
                        if let Diff::Missing(_, s) | Diff::Differs(_, s, _) = d {
                            effective.insert(k.clone(), s.clone());
                        }
                    }
                    let _ = writeln!(out, "{name}: updated {changes} entries");
                }
                Err(e) => {
                    let _ = writeln!(out, "{name}: cannot write config ({e})");
                    ok = false;
                }
            }
        }
        // The machine's own name must be a key of the [machines] it reads.
        if let Some(me) = &selves[i]
            && !effective.contains_key(&("machines", me.clone()))
        {
            let _ = writeln!(
                out,
                "{name}: [machine] name '{me}' is not a key of its [machines]"
            );
        }
    }
    // This machine's own `[mail] projects` is derived too; the other keys here are the source.
    let want = mail_source.get(&("mail", "projects".into()));
    if let Some(want) = want
        && local_mail.get(&("mail", "projects".into())) != Some(want)
    {
        let have = local_mail.get(&("mail", "projects".into()));
        let _ = writeln!(
            out,
            "{}: differs [mail] projects (derived {}, here {})",
            fleet.local,
            cell(Some(want)),
            cell(have)
        );
        if apply {
            let d = [(
                ("mail", "projects".to_string()),
                Diff::Missing(String::new(), want.clone()),
            )];
            let done = apply_text(&local_text, &d).and_then(|t| files.write(&Where::Local, &t));
            match done {
                Ok(()) => {
                    let _ = writeln!(out, "{}: updated 1 entries", fleet.local);
                }
                Err(e) => {
                    let _ = writeln!(out, "{}: cannot write config ({e})", fleet.local);
                    ok = false;
                }
            }
        }
    }
    if !source.contains_key(&("machines", fleet.local.clone())) && fleet.local != "local" {
        let _ = writeln!(
            out,
            "{}: [machine] name '{}' is not a key of its [machines]",
            fleet.local, fleet.local
        );
    }
    ok
}

#[cfg(test)]
mod tests {
    #[test]
    fn ssh_that_never_answers_times_out_naming_the_machine() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fake = dir.path().join("ssh");
        std::fs::write(&fake, "#!/bin/sh\nexec sleep 30\n").expect("write");
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        }
        let err = ssh_with(
            fake.to_str().expect("utf8"),
            "remus",
            "true",
            None,
            Duration::from_millis(300),
        )
        .expect_err("timeout");
        assert!(
            err.contains("`remus`") && err.contains("no answer"),
            "{err}"
        );
    }

    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[test]
    fn a_command_that_never_answers_is_killed_naming_the_host() {
        let mut c = Command::new("sleep");
        c.arg("30");
        let err = run_timed(c, "remus", None, Duration::from_millis(200)).expect_err("timeout");
        assert!(
            err.contains("`remus`") && err.contains("no answer"),
            "{err}"
        );
    }

    struct Fake(RefCell<HashMap<String, String>>);

    impl Fake {
        fn new(files: &[(&str, &str)]) -> Self {
            Fake(RefCell::new(
                files
                    .iter()
                    .map(|(h, t)| (h.to_string(), t.to_string()))
                    .collect(),
            ))
        }
    }

    impl Files for Fake {
        fn read(&self, at: &Where) -> Result<String, String> {
            let h = match at {
                Where::Ssh(h) => h.as_str(),
                Where::Local => return Ok(self.0.borrow().get("mbp").cloned().unwrap_or_default()),
            };
            self.0
                .borrow()
                .get(h)
                .cloned()
                .ok_or("ssh: connection refused".into())
        }
        fn write(&self, at: &Where, text: &str) -> Result<(), String> {
            let h = match at {
                Where::Ssh(h) => h.clone(),
                Where::Local => "mbp".to_string(),
            };
            self.0.borrow_mut().insert(h, text.to_string());
            Ok(())
        }
    }

    const SRC: &str = "[machine]\nname = \"mbp\"\n[machines]\nmbp = \"mbp\"\nnuc = \"nuc.lan\"\n\
        [projects]\na = { machine = \"mbp\", port = 7401 }\nb = { machine = \"nuc\", port = 7402 }\n";

    fn fleet() -> Fleet {
        Fleet {
            local: "mbp".into(),
            hosts: [("mbp", "mbp"), ("nuc", "nuc.lan")]
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .into(),
            placed: [("a", ("mbp", 7401)), ("b", ("nuc", 7402))]
                .map(|(p, (m, n))| (p.to_string(), (m.to_string(), n)))
                .into(),
        }
    }

    fn run(files: &Fake, apply: bool) -> String {
        let mut out = Vec::new();
        sync(&fleet(), files, apply, &mut out);
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn identical_lists_show_no_differences() {
        let f = Fake::new(&[("nuc.lan", SRC)]);
        assert!(run(&f, false).contains("nuc: same"));
    }

    #[test]
    fn missing_and_differing_entries_are_shown_and_untouched_without_apply() {
        let text = "[machine]\nname = \"nuc\"\n[machines]\nnuc = \"x\"\n[projects]\na = { machine = \"mbp\", port = 9 }\nz = { machine = \"nuc\", port = 1 }\n";
        let f = Fake::new(&[("nuc.lan", text)]);
        let out = run(&f, false);
        assert!(
            out.contains("nuc: missing [projects] b (nuc:7402)"),
            "{out}"
        );
        assert!(out.contains("nuc: missing [machines] mbp (mbp)"), "{out}");
        assert!(
            out.contains("nuc: differs [machines] nuc (here nuc.lan, there x)"),
            "{out}"
        );
        assert!(
            out.contains("nuc: differs [projects] a (here mbp:7401, there mbp:9)"),
            "{out}"
        );
        assert!(out.contains("only on nuc [projects] z"), "{out}");
        assert_eq!(f.0.borrow()["nuc.lan"], text);
    }

    #[test]
    fn apply_adds_and_fixes_keeps_comments_and_never_deletes_and_is_idempotent() {
        let text = "# mine\n[machine]\nname = \"nuc\"\n\n[machines]\nold = \"o\" # keep\n\n[projects]\n# why\na = { machine = \"mbp\", port = 9 } # old\nz = { machine = \"nuc\", port = 1 }\n";
        let f = Fake::new(&[("nuc.lan", text)]);
        let out = run(&f, true);
        assert!(out.contains("nuc: updated 4 entries"), "{out}");
        let now = f.0.borrow()["nuc.lan"].clone();
        assert!(
            now.starts_with("# mine\n[machine]\nname = \"nuc\"\n"),
            "{now}"
        );
        assert!(now.contains("# why") && now.contains("# keep"), "{now}");
        let (l, _) = parse(&now).unwrap();
        assert_eq!(
            l[&("projects", "a".to_string())],
            Entry::Place("mbp".into(), 7401)
        );
        assert_eq!(
            l[&("projects", "b".to_string())],
            Entry::Place("nuc".into(), 7402)
        );
        assert_eq!(
            l[&("projects", "z".to_string())],
            Entry::Place("nuc".into(), 1)
        );
        assert_eq!(
            l[&("machines", "nuc".to_string())],
            Entry::Host("nuc.lan".into())
        );
        assert!(l.contains_key(&("machines", "old".to_string())));
        let again = run(&f, true);
        assert!(again.contains("only on nuc [projects] z"), "{again}");
        assert!(!again.contains("updated"), "{again}");
        assert_eq!(f.0.borrow()["nuc.lan"], now);
    }

    #[test]
    fn apply_creates_sections_in_a_file_without_them() {
        let f = Fake::new(&[("nuc.lan", "[machine]\nname = \"nuc\"\n")]);
        run(&f, true);
        let now = f.0.borrow()["nuc.lan"].clone();
        assert!(now.contains("name = \"nuc\""));
        assert_eq!(parse(&now).unwrap().0.len(), 4);
    }

    #[test]
    fn own_name_missing_from_machines_is_flagged() {
        let f = Fake::new(&[("nuc.lan", "[machine]\nname = \"vesta\"\n")]);
        let out = run(&f, true);
        assert!(
            out.contains("nuc: [machine] name 'vesta' is not a key"),
            "{out}"
        );
    }

    #[test]
    fn unreachable_machine_is_reported_and_fails() {
        let f = Fake::new(&[]);
        let mut out = Vec::new();
        assert!(!sync(&fleet(), &f, true, &mut out));
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("nuc: cannot read config")
        );
    }

    #[test]
    fn mail_entry_for_an_unregistered_project_is_kept_everywhere() {
        let local = format!("{SRC}[mail]\nbucket = \"b\"\nprojects = [\"bridle\", \"later\"]\n");
        let there = "[machine]\nname = \"nuc\"\n[mail]\nprojects = [\"bridle\", \"other\"]\n";
        let f = Fake::new(&[("mbp", &local), ("nuc.lan", there)]);
        let out = run(&f, true);
        assert!(
            out.contains("left later, other (not in [projects])"),
            "{out}"
        );
        for h in ["nuc.lan", "mbp"] {
            let now = f.0.borrow()[h].clone();
            assert!(
                now.contains("\"later\"") && now.contains("\"other\""),
                "{h}: {now}"
            );
            assert!(now.contains("\"a\"") && now.contains("\"b\""), "{h}: {now}");
        }
    }

    #[test]
    fn mail_is_spread_with_projects_derived_and_machine_left_alone() {
        let local = format!(
            "{SRC}[mail]\nbucket = \"b\"\nallow = [\"me@example.com\"]\nprojects = [\"b\", \"a\"]\n"
        );
        let there = "[machine]\nname = \"nuc\"\nmail = false\n[mail]\nprojects = [\"a\"]\n";
        let f = Fake::new(&[("mbp", &local), ("nuc.lan", there)]);
        let out = run(&f, false);
        assert!(out.contains("nuc: missing [mail] bucket"), "{out}");
        assert!(out.contains("nuc: differs [mail] projects"), "{out}");
        assert!(out.contains("mbp: differs [mail] projects"), "{out}");
        run(&f, true);
        let want = "projects = [\"bridle\", \"a\", \"b\"]";
        for h in ["nuc.lan", "mbp"] {
            let now = f.0.borrow()[h].clone();
            assert!(now.contains(want), "{h}: {now}");
        }
        let now = f.0.borrow()["nuc.lan"].clone();
        assert!(
            now.contains("mail = false") && now.contains("bucket = \"b\""),
            "{now}"
        );
        assert!(!run(&f, true).contains("differs [mail]"));
    }
    fn add_args(
        name: &str,
        machine: Option<&str>,
        port: Option<u16>,
    ) -> crate::cli::ProjectAddArgs {
        crate::cli::ProjectAddArgs {
            name: name.into(),
            machine: machine.map(Into::into),
            port,
        }
    }

    #[test]
    fn add_picks_the_port_above_the_highest_and_syncs_keeping_comments() {
        let src = format!("# mine\n{SRC}[mail]\nprojects = [\"bridle\"]\n");
        let files = Fake::new(&[("mbp", &src), ("nuc.lan", "[machine]\nname = \"nuc\"\n")]);
        let mut out = Vec::new();
        add_with(&fleet(), &files, &add_args("c", None, None), &mut out).unwrap();
        let mine = files.read(&Where::Local).unwrap();
        assert!(mine.starts_with("# mine\n"), "{mine}");
        assert!(
            mine.contains("c = { machine = \"mbp\", port = 7403 }"),
            "{mine}"
        );
        let there = files.read(&Where::Ssh("nuc.lan".into())).unwrap();
        assert!(
            there.contains("c = { machine = \"mbp\", port = 7403 }"),
            "{there}"
        );
        assert!(there.contains("\"c\""), "mail follows [projects]: {there}");
        assert!(String::from_utf8_lossy(&out).contains("next: `bridle project start`"));
    }

    #[test]
    fn add_refuses_a_listed_name_and_an_unknown_machine() {
        let files = Fake::new(&[("mbp", SRC), ("nuc.lan", "")]);
        let mut out = Vec::new();
        let e = add_with(&fleet(), &files, &add_args("a", None, None), &mut out).unwrap_err();
        assert!(e.to_string().contains("already in [projects]"), "{e}");
        let e = add_with(
            &fleet(),
            &files,
            &add_args("c", Some("zed"), None),
            &mut out,
        )
        .unwrap_err();
        assert!(e.to_string().contains("not a key of [machines]"), "{e}");
        assert_eq!(files.read(&Where::Local).unwrap(), SRC);
    }

    #[test]
    fn add_takes_a_given_machine_and_port() {
        let files = Fake::new(&[("mbp", SRC), ("nuc.lan", "")]);
        let mut out = Vec::new();
        add_with(
            &fleet(),
            &files,
            &add_args("c", Some("nuc"), Some(7500)),
            &mut out,
        )
        .unwrap();
        let mine = files.read(&Where::Local).unwrap();
        assert!(
            mine.contains("c = { machine = \"nuc\", port = 7500 }"),
            "{mine}"
        );
    }
}
