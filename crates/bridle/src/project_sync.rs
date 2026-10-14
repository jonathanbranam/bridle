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
}

impl Entry {
    fn show(&self) -> String {
        match self {
            Entry::Host(h) => h.clone(),
            Entry::Place(m, p) => format!("{m}:{p}"),
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

/// Compares (and with `apply`, updates) every machine. Returns whether every machine could be
/// read and, when applying, written.
pub fn sync(fleet: &Fleet, files: &dyn Files, apply: bool, out: &mut dyn Write) -> bool {
    let mut ok = true;
    let source = list_of(fleet);
    let mut names: Vec<String> = vec![fleet.local.clone()];
    let mut lists: Vec<Option<List>> = vec![Some(source.clone())];
    let mut texts: Vec<String> = vec![String::new()];
    let mut selves: Vec<Option<String>> = vec![Some(fleet.local.clone())];
    for (name, host) in &fleet.hosts {
        if *name == fleet.local {
            continue;
        }
        names.push(name.clone());
        let got = files
            .read(&Where::Ssh(host.clone()))
            .and_then(|t| parse(&t).map(|(l, n)| (t, l, n)));
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
            let Where::Ssh(h) = at else {
                return Ok(String::new());
            };
            self.0
                .borrow()
                .get(h)
                .cloned()
                .ok_or("ssh: connection refused".into())
        }
        fn write(&self, at: &Where, text: &str) -> Result<(), String> {
            let Where::Ssh(h) = at else { return Ok(()) };
            self.0.borrow_mut().insert(h.clone(), text.to_string());
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
}
