//! Daemon discovery: workspace layout, `daemon.json`, the machine registry
//! and token/endpoint resolution. Pure filesystem + env logic, shared by the
//! CLI and (by design) the daemon itself. See docs/design/agent-host/daemon.md (discovery)
//! and docs/design/agent-host/principals.md (token choice).

use std::path::{Path, PathBuf};
use std::{fs, io};

use nix::errno::Errno;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use serde::Serialize;
use thiserror::Error;

use crate::types::DaemonInfo;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// A resolution failure with an actionable, user-facing message.
    #[error("{0}")]
    Message(String),
    #[error("{path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("{path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl DiscoveryError {
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    fn json(path: &Path, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// Read-only access to process environment variables, injectable so tests
/// don't have to mutate the real process environment: edition 2024 makes
/// `std::env::set_var` unsafe, and `unsafe` is forbidden in this workspace.
pub trait Env {
    fn var(&self, key: &str) -> Option<String>;
}

/// Reads the real process environment.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessEnv;

impl Env for ProcessEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

impl<F> Env for F
where
    F: Fn(&str) -> Option<String>,
{
    fn var(&self, key: &str) -> Option<String> {
        self(key)
    }
}

// ---------- layout ----------

/// `$BRIDLE_HOME`, else `~/.bridle`.
pub fn bridle_home() -> PathBuf {
    if let Some(h) = std::env::var("BRIDLE_HOME").ok().filter(|s| !s.is_empty()) {
        return PathBuf::from(h);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    PathBuf::from(home).join(".bridle")
}

/// `~/.bridle/daemons`, where every daemon registers itself.
pub fn registry_dir() -> PathBuf {
    bridle_home().join("daemons")
}

/// `<workspace>/.bridle`.
pub fn state_dir(workspace: &Path) -> PathBuf {
    workspace.join(".bridle")
}

/// `<workspace>/.bridle/daemon.json`.
pub fn daemon_json_path(workspace: &Path) -> PathBuf {
    state_dir(workspace).join("daemon.json")
}

/// `<workspace>/.bridle/tokens/human`.
pub fn human_token_path(workspace: &Path) -> PathBuf {
    state_dir(workspace).join("tokens").join("human")
}

/// `~/.bridle/statusline.token`: the per-user token Claude Code's statusLine
/// command reads for bridle's own counts, regardless of which project
/// workspace it's running in. See docs/design/cli.md.
pub fn statusline_token_path() -> PathBuf {
    bridle_home().join("statusline.token")
}

/// `~/.bridle/context/<session_id>`: the session's latest context size in
/// tokens, written by `bridle statusline` for scripts (the orchestrator's
/// watcher) that need it without a daemon call. Ids with anything but
/// `[A-Za-z0-9_-]` are refused so a session id can't escape the directory.
pub fn session_context_path(session_id: &str) -> Option<PathBuf> {
    let ok = !session_id.is_empty()
        && session_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    ok.then(|| bridle_home().join("context").join(session_id))
}

/// Walk up from `start` to the first ancestor (inclusive) containing
/// `.bridle/daemon.json`, returning that ancestor.
pub fn find_workspace(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if daemon_json_path(&dir).is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

// ---------- daemon.json ----------

pub fn read_daemon_json(workspace: &Path) -> Result<DaemonInfo, DiscoveryError> {
    let path = daemon_json_path(workspace);
    let bytes = fs::read(&path).map_err(|e| DiscoveryError::io(&path, e))?;
    serde_json::from_slice(&bytes).map_err(|e| DiscoveryError::json(&path, e))
}

/// Atomic: write a tmp file in the same directory, then rename over the
/// target, so a reader never sees a half-written `daemon.json`.
pub fn write_daemon_json(workspace: &Path, info: &DaemonInfo) -> Result<(), DiscoveryError> {
    let dir = state_dir(workspace);
    fs::create_dir_all(&dir).map_err(|e| DiscoveryError::io(&dir, e))?;
    atomic_write_json(&daemon_json_path(workspace), info)
}

fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), DiscoveryError> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("tmp");
    let tmp = dir.join(format!(".{file_name}.tmp-{}", std::process::id()));
    let body = serde_json::to_vec_pretty(value).map_err(|e| DiscoveryError::json(path, e))?;
    fs::write(&tmp, &body).map_err(|e| DiscoveryError::io(&tmp, e))?;
    fs::rename(&tmp, path).map_err(|e| DiscoveryError::io(path, e))?;
    Ok(())
}

// ---------- registry (~/.bridle/daemons/<project>.json) ----------

/// Write (or overwrite) this daemon's registry entry.
pub fn write_registry(info: &DaemonInfo) -> Result<(), DiscoveryError> {
    write_registry_in(&registry_dir(), info)
}

fn write_registry_in(dir: &Path, info: &DaemonInfo) -> Result<(), DiscoveryError> {
    fs::create_dir_all(dir).map_err(|e| DiscoveryError::io(dir, e))?;
    atomic_write_json(&dir.join(format!("{}.json", info.project)), info)
}

/// Remove this project's registry entry, if present.
pub fn remove_registry(project: &str) -> Result<(), DiscoveryError> {
    remove_registry_in(&registry_dir(), project)
}

fn remove_registry_in(dir: &Path, project: &str) -> Result<(), DiscoveryError> {
    let path = dir.join(format!("{project}.json"));
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(DiscoveryError::io(&path, e)),
    }
}

/// Every registered daemon on this machine, pruning (and deleting the
/// registry file for) any entry whose pid is no longer alive.
pub fn list_registry() -> Vec<DaemonInfo> {
    list_registry_in(&registry_dir())
}

fn list_registry_in(dir: &Path) -> Vec<DaemonInfo> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else { continue };
        let Ok(info) = serde_json::from_slice::<DaemonInfo>(&bytes) else {
            continue;
        };
        if pid_alive(info.pid) {
            out.push(info);
        } else {
            let _ = fs::remove_file(&path);
        }
    }
    out
}

/// `kill(pid, 0)`: `ESRCH` means gone, `EPERM` means it exists but is owned
/// by someone else (treated as alive), anything else is treated as dead.
fn pid_alive(pid: i32) -> bool {
    match kill(Pid::from_raw(pid), None) {
        Ok(()) => true,
        Err(Errno::EPERM) => true,
        Err(_) => false,
    }
}

// ---------- endpoint + token resolution ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub url: String,
    pub workspace: Option<PathBuf>,
    pub project: Option<String>,
    /// The machine the project's daemon is on, when the machine config puts it on
    /// another machine than this one (k7mw); `None` for a local daemon or a bare url.
    pub machine: Option<String>,
}

/// Order: `--url`, `$BRIDLE_URL`, `--project`/`$BRIDLE_PROJECT` via the
/// registry, then a cwd walk to `.bridle/daemon.json`.
pub fn resolve_endpoint(
    url_flag: Option<&str>,
    project_flag: Option<&str>,
    cwd: &Path,
    env: &impl Env,
) -> Result<Endpoint, DiscoveryError> {
    let machines = crate::machines::MachineMap::load(&bridle_home())?;
    resolve_endpoint_with(&machines, url_flag, project_flag, cwd, env)
}

fn resolve_endpoint_with(
    machines: &crate::machines::MachineMap,
    url_flag: Option<&str>,
    project_flag: Option<&str>,
    cwd: &Path,
    env: &impl Env,
) -> Result<Endpoint, DiscoveryError> {
    if let Some(url) = url_flag {
        return Ok(Endpoint {
            url: url.to_string(),
            workspace: None,
            project: None,
            machine: None,
        });
    }
    if let Some(url) = env.var("BRIDLE_URL") {
        return Ok(Endpoint {
            url,
            workspace: None,
            project: None,
            machine: None,
        });
    }
    let project = project_flag
        .map(str::to_string)
        .or_else(|| env.var("BRIDLE_PROJECT"));
    if let Some(project) = project {
        if let Some(remote) = machines.remote(&project)? {
            return Ok(Endpoint {
                url: remote.url,
                workspace: None,
                project: Some(project),
                machine: Some(remote.machine),
            });
        }
        return list_registry()
            .into_iter()
            .find(|d| d.project == project)
            .map(|d| Endpoint {
                url: d.url,
                workspace: Some(PathBuf::from(d.workspace)),
                project: Some(d.project),
                machine: None,
            })
            .ok_or_else(|| {
                DiscoveryError::Message(format!(
                    "no daemon registered for project '{project}'; check `bridle daemons`"
                ))
            });
    }
    if let Some(workspace) = find_workspace(cwd) {
        let info = read_daemon_json(&workspace)?;
        return Ok(Endpoint {
            url: info.url,
            workspace: Some(workspace),
            project: Some(info.project),
            machine: None,
        });
    }
    Err(DiscoveryError::Message(
        "no bridle daemon found: pass --url, set $BRIDLE_URL, use --project/$BRIDLE_PROJECT \
         (see `bridle daemons`), or run inside a workspace created by `bridle serve`"
            .to_string(),
    ))
}

/// `~/.bridle/credentials.toml`: one table per external principal, one key per
/// project, holding that principal's token for the project's daemon on this machine.
/// A sub-table `[principal.<machine>]` holds the same for that machine's daemons (k7mw).
pub fn credentials_path() -> PathBuf {
    bridle_home().join("credentials.toml")
}

/// Reads the credentials file; a missing file is an empty table. Refuses a file
/// that group or others can access, since it holds live tokens.
fn read_credentials(path: &Path) -> Result<toml::Table, DiscoveryError> {
    use std::os::unix::fs::PermissionsExt;
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(toml::Table::new()),
        Err(e) => return Err(DiscoveryError::io(path, e)),
    };
    if meta.permissions().mode() & 0o077 != 0 {
        return Err(DiscoveryError::Message(format!(
            "{} is accessible to other users (mode {:o}); refusing to read it: run `chmod 600 {}`",
            path.display(),
            meta.permissions().mode() & 0o777,
            path.display()
        )));
    }
    let text = fs::read_to_string(path).map_err(|e| DiscoveryError::io(path, e))?;
    text.parse().map_err(|e| {
        DiscoveryError::Message(format!("{}: invalid credentials file: {e}", path.display()))
    })
}

fn write_credentials(path: &Path, table: &toml::Table) -> Result<(), DiscoveryError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| DiscoveryError::io(dir, e))?;
    }
    // Written to a 0600 temp file and renamed, so the tokens are never on disk
    // with a looser mode and a crash can't leave a half-written file.
    let tmp = path.with_extension("toml.tmp");
    let _ = fs::remove_file(&tmp);
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)
        .map_err(|e| DiscoveryError::io(&tmp, e))?;
    f.write_all(table.to_string().as_bytes())
        .map_err(|e| DiscoveryError::io(&tmp, e))?;
    fs::rename(&tmp, path).map_err(|e| DiscoveryError::io(path, e))
}

/// Stores `token` as `principal`'s entry for `project` in the credentials file,
/// keeping every other entry.
pub fn store_credential(
    path: &Path,
    principal: &str,
    project: &str,
    token: &str,
) -> Result<(), DiscoveryError> {
    let mut table = read_credentials(path)?;
    let entry = table
        .entry(principal.to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    let Some(projects) = entry.as_table_mut() else {
        return Err(DiscoveryError::Message(format!(
            "{}: [{principal}] is not a table",
            path.display()
        )));
    };
    projects.insert(project.to_string(), toml::Value::String(token.to_string()));
    write_credentials(path, &table)
}

/// The token this machine's daemons present to `project`'s daemon when forwarding mail: the
/// `[peer]` table's entry for that project (3haz). A missing file or entry is `None`.
pub fn peer_token(credentials: &Path, project: &str) -> Result<Option<String>, DiscoveryError> {
    Ok(read_credentials(credentials)?
        .get("peer")
        .and_then(|v| v.get(project))
        .and_then(|v| v.as_str())
        .map(str::to_string))
}

/// Removes `principal`'s entry for `project`; returns whether there was one.
pub fn remove_credential(
    path: &Path,
    principal: &str,
    project: &str,
) -> Result<bool, DiscoveryError> {
    let mut table = read_credentials(path)?;
    let Some(projects) = table.get_mut(principal).and_then(|v| v.as_table_mut()) else {
        return Ok(false);
    };
    let removed = projects.remove(project).is_some();
    if projects.is_empty() {
        table.remove(principal);
    }
    if removed {
        write_credentials(path, &table)?;
    }
    Ok(removed)
}

/// principals.md: `--token`, `$BRIDLE_TOKEN`, then `$BRIDLE_AS`'s entry for `project` in
/// the credentials file; else (only if `$CLAUDECODE` is unset) the human token file in
/// `workspace`, else an error — except for a read command
/// (`allow_anonymous_read`), which proceeds with no token instead: the daemon
/// already accepts token-less GET/HEAD requests as the synthetic `local`
/// principal (server.rs's `auth_middleware`), so a read gets the same
/// tolerance client-side rather than a client-side error ahead of a request
/// that would have succeeded anyway.
pub fn resolve_token(
    token_flag: Option<&str>,
    workspace: Option<&Path>,
    project: Option<&str>,
    machine: Option<&str>,
    env: &impl Env,
    allow_anonymous_read: bool,
) -> Result<Option<String>, DiscoveryError> {
    resolve_token_in(
        &credentials_path(),
        token_flag,
        workspace,
        project,
        machine,
        env,
        allow_anonymous_read,
    )
}

fn resolve_token_in(
    credentials: &Path,
    token_flag: Option<&str>,
    workspace: Option<&Path>,
    project: Option<&str>,
    machine: Option<&str>,
    env: &impl Env,
    allow_anonymous_read: bool,
) -> Result<Option<String>, DiscoveryError> {
    if let Some(t) = token_flag {
        return Ok(Some(t.to_string()));
    }
    if let Some(t) = env.var("BRIDLE_TOKEN") {
        return Ok(Some(t));
    }
    if let Some(principal) = env.var("BRIDLE_AS").filter(|s| !s.is_empty()) {
        let Some(project) = project else {
            return Err(DiscoveryError::Message(format!(
                "$BRIDLE_AS={principal} but the project is unknown (the daemon was found by \
                 URL): pass --project, or set $BRIDLE_TOKEN"
            )));
        };
        // A visitor's token on another machine's daemon sits under that machine's name.
        let table = read_credentials(credentials)?;
        let mut entry = table.get(&principal);
        if let Some(m) = machine {
            entry = entry.and_then(|v| v.get(m));
        }
        let found = entry
            .and_then(|v| v.get(project))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        return found.map(Some).ok_or_else(|| {
            let (place, section) = match machine {
                Some(m) => (format!(" on machine '{m}'"), format!("[{principal}.{m}]")),
                None => (String::new(), format!("[{principal}]")),
            };
            DiscoveryError::Message(format!(
                "no token for principal '{principal}' on project '{project}'{place} in {}: \
                 mint one as the human and put it under {section}; on this machine \
                 `bridle token create {principal} --project {project}` does it",
                credentials.display()
            ))
        });
    }
    if env.var("CLAUDECODE").is_some() {
        if allow_anonymous_read {
            return Ok(None);
        }
        return Err(DiscoveryError::Message(
            "running inside Claude Code ($CLAUDECODE is set), so the human token is never \
             used implicitly: set $BRIDLE_TOKEN or $BRIDLE_AS"
                .to_string(),
        ));
    }
    let Some(workspace) = workspace else {
        // Last resort, only where this used to be an error: the human's token for another
        // machine's daemon, principal `human@<machine>` (3ehu).
        if let (Some(machine), Some(project)) = (machine, project)
            && let Some(t) = read_credentials(credentials)?
                .get("human")
                .and_then(|v| v.get(machine))
                .and_then(|v| v.get(project))
                .and_then(|v| v.as_str())
        {
            return Ok(Some(t.to_string()));
        }
        let hint = match machine {
            Some(m) => format!(
                ", or put the token under [human.{m}] in {}",
                credentials.display()
            ),
            None => String::new(),
        };
        return Err(DiscoveryError::Message(format!(
            "no workspace found to read the human token from: pass --token (required with --url or $BRIDLE_URL), set $BRIDLE_TOKEN, or $BRIDLE_AS{hint}"
        )));
    };
    let path = human_token_path(workspace);
    match fs::read_to_string(&path) {
        Ok(s) => Ok(Some(s.trim().to_string())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(DiscoveryError::Message(format!(
            "no human token at {}: set $BRIDLE_TOKEN",
            path.display()
        ))),
        Err(e) => Err(DiscoveryError::io(&path, e)),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tempfile::tempdir;

    use super::*;

    fn sample_daemon_info(project: &str, workspace: &Path, pid: i32) -> DaemonInfo {
        DaemonInfo {
            project: project.to_string(),
            workspace: workspace.to_string_lossy().into_owned(),
            repo: workspace.join("repo").to_string_lossy().into_owned(),
            url: "http://127.0.0.1:12345".to_string(),
            pid,
            started_at: Utc::now(),
            version: "0.1.0".to_string(),
        }
    }

    struct MapEnv(std::collections::HashMap<&'static str, &'static str>);

    impl Env for MapEnv {
        fn var(&self, key: &str) -> Option<String> {
            self.0.get(key).map(|s| s.to_string())
        }
    }

    fn empty_env() -> MapEnv {
        MapEnv(std::collections::HashMap::new())
    }

    #[test]
    fn find_workspace_walks_up_from_a_nested_dir() {
        let root = tempdir().unwrap();
        let ws = root.path();
        fs::create_dir_all(state_dir(ws)).unwrap();
        fs::write(daemon_json_path(ws), "{}").unwrap();
        let nested = ws.join("wt").join("w1").join("src");
        fs::create_dir_all(&nested).unwrap();

        assert_eq!(find_workspace(&nested), Some(ws.to_path_buf()));
    }

    #[test]
    fn find_workspace_returns_none_outside_any_workspace() {
        let root = tempdir().unwrap();
        assert_eq!(find_workspace(root.path()), None);
    }

    #[test]
    fn daemon_json_round_trips_atomically() {
        let root = tempdir().unwrap();
        let ws = root.path();
        let info = sample_daemon_info("demo", ws, std::process::id() as i32);
        write_daemon_json(ws, &info).unwrap();
        assert_eq!(read_daemon_json(ws).unwrap(), info);
        // no leftover tmp file
        let leftovers: Vec<_> = fs::read_dir(state_dir(ws))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn registry_prunes_entries_with_a_dead_pid() {
        let dir = tempdir().unwrap();
        let alive = sample_daemon_info(
            "alive-project",
            Path::new("/ws/a"),
            std::process::id() as i32,
        );
        // A pid essentially guaranteed not to exist.
        let dead = sample_daemon_info("dead-project", Path::new("/ws/b"), 999_999);
        write_registry_in(dir.path(), &alive).unwrap();
        write_registry_in(dir.path(), &dead).unwrap();

        let listed = list_registry_in(dir.path());
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].project, "alive-project");
        assert!(!dir.path().join("dead-project.json").exists());
        assert!(dir.path().join("alive-project.json").exists());
    }

    #[test]
    fn remove_registry_is_idempotent() {
        let dir = tempdir().unwrap();
        remove_registry_in(dir.path(), "nope").unwrap();
        let info = sample_daemon_info("p", Path::new("/ws"), std::process::id() as i32);
        write_registry_in(dir.path(), &info).unwrap();
        remove_registry_in(dir.path(), "p").unwrap();
        assert!(list_registry_in(dir.path()).is_empty());
    }

    #[test]
    fn resolve_endpoint_prefers_url_flag() {
        let cwd = tempdir().unwrap();
        let ep = resolve_endpoint(Some("http://x:1"), None, cwd.path(), &empty_env()).unwrap();
        assert_eq!(ep.url, "http://x:1");
        assert_eq!(ep.workspace, None);
    }

    #[test]
    fn resolve_endpoint_falls_back_to_bridle_url_env() {
        let cwd = tempdir().unwrap();
        let env = MapEnv(std::collections::HashMap::from([(
            "BRIDLE_URL",
            "http://y:2",
        )]));
        let ep = resolve_endpoint(None, None, cwd.path(), &env).unwrap();
        assert_eq!(ep.url, "http://y:2");
    }

    #[test]
    fn resolve_endpoint_walks_cwd_when_nothing_else_is_set() {
        let root = tempdir().unwrap();
        let ws = root.path();
        fs::create_dir_all(state_dir(ws)).unwrap();
        let info = sample_daemon_info("demo", ws, std::process::id() as i32);
        write_daemon_json(ws, &info).unwrap();
        let nested = ws.join("wt").join("w1");
        fs::create_dir_all(&nested).unwrap();

        let ep = resolve_endpoint(None, None, &nested, &empty_env()).unwrap();
        assert_eq!(ep.url, info.url);
        assert_eq!(ep.project, Some("demo".to_string()));
    }

    #[test]
    fn resolve_endpoint_errors_with_actionable_message_when_lost() {
        let cwd = tempdir().unwrap();
        let err = resolve_endpoint(None, None, cwd.path(), &empty_env()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--url"));
        assert!(msg.contains("bridle serve"));
    }

    #[test]
    fn resolve_token_prefers_flag_then_env() {
        assert_eq!(
            resolve_token(Some("flag"), None, None, None, &empty_env(), false).unwrap(),
            Some("flag".to_string())
        );
        let env = MapEnv(std::collections::HashMap::from([(
            "BRIDLE_TOKEN",
            "envtok",
        )]));
        assert_eq!(
            resolve_token(None, None, None, None, &env, false).unwrap(),
            Some("envtok".to_string())
        );
    }

    #[test]
    fn resolve_token_reads_human_token_file_when_claudecode_unset() {
        let root = tempdir().unwrap();
        let ws = root.path();
        let token_path = human_token_path(ws);
        fs::create_dir_all(token_path.parent().unwrap()).unwrap();
        fs::write(&token_path, "human-secret\n").unwrap();

        let tok = resolve_token(None, Some(ws), None, None, &empty_env(), false).unwrap();
        assert_eq!(tok, Some("human-secret".to_string()));
    }

    #[test]
    fn resolve_token_refuses_human_token_when_claudecode_is_set() {
        let root = tempdir().unwrap();
        let ws = root.path();
        let token_path = human_token_path(ws);
        fs::create_dir_all(token_path.parent().unwrap()).unwrap();
        fs::write(&token_path, "human-secret").unwrap();

        let env = MapEnv(std::collections::HashMap::from([("CLAUDECODE", "1")]));
        let err = resolve_token(None, Some(ws), None, None, &env, false).unwrap_err();
        assert!(err.to_string().contains("BRIDLE_TOKEN"));
    }

    #[test]
    fn resolve_token_errors_when_no_token_file_exists() {
        let root = tempdir().unwrap();
        let err =
            resolve_token(None, Some(root.path()), None, None, &empty_env(), false).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("BRIDLE_TOKEN") || msg.contains("--token"));
    }

    #[test]
    fn resolve_token_no_workspace_and_no_token_mentions_url_requirement() {
        // When --url is used (workspace is None) and no token is set, the error
        // should mention both --token and that it's required with --url/$BRIDLE_URL
        let err = resolve_token(None, None, None, None, &empty_env(), false).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--token"), "error message: {}", msg);
        assert!(msg.contains("--url"), "error message: {}", msg);
    }

    #[test]
    fn resolve_token_allows_anonymous_read_when_claudecode_is_set() {
        let root = tempdir().unwrap();
        let ws = root.path();
        let token_path = human_token_path(ws);
        fs::create_dir_all(token_path.parent().unwrap()).unwrap();
        fs::write(&token_path, "human-secret").unwrap();

        let env = MapEnv(std::collections::HashMap::from([("CLAUDECODE", "1")]));
        // The human token file must still never be used implicitly under
        // CLAUDECODE: the request goes out with no token at all, not the
        // human's.
        assert_eq!(
            resolve_token(None, Some(ws), None, None, &env, true).unwrap(),
            None
        );
    }

    #[test]
    fn resolve_token_write_still_errors_under_claudecode_even_with_anonymous_read_available() {
        let env = MapEnv(std::collections::HashMap::from([("CLAUDECODE", "1")]));
        let err = resolve_token(None, None, None, None, &env, false).unwrap_err();
        assert!(err.to_string().contains("BRIDLE_TOKEN"));
    }

    fn creds_env(vars: &[(&'static str, &'static str)]) -> MapEnv {
        MapEnv(vars.iter().copied().collect())
    }

    fn creds_file(dir: &Path) -> PathBuf {
        dir.join("credentials.toml")
    }

    #[test]
    fn credentials_pick_order_is_flag_then_token_env_then_bridle_as() {
        let dir = tempdir().unwrap();
        let path = creds_file(dir.path());
        store_credential(&path, "advisor", "demo", "from-file").unwrap();
        let both = creds_env(&[("BRIDLE_TOKEN", "envtok"), ("BRIDLE_AS", "advisor")]);
        let pick = |flag, env: &MapEnv| {
            resolve_token_in(&path, flag, None, Some("demo"), None, env, false).unwrap()
        };
        assert_eq!(pick(Some("flag"), &both), Some("flag".to_string()));
        assert_eq!(pick(None, &both), Some("envtok".to_string()));
        let only_as = creds_env(&[("BRIDLE_AS", "advisor"), ("CLAUDECODE", "1")]);
        assert_eq!(pick(None, &only_as), Some("from-file".to_string()));
    }

    #[test]
    fn bridle_as_with_a_missing_entry_names_file_principal_and_project() {
        let dir = tempdir().unwrap();
        let path = creds_file(dir.path());
        store_credential(&path, "advisor", "demo", "t").unwrap();
        let env = creds_env(&[("BRIDLE_AS", "advisor")]);
        let err =
            resolve_token_in(&path, None, None, Some("other"), None, &env, false).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("credentials.toml"), "{msg}");
        assert!(msg.contains("advisor"), "{msg}");
        assert!(msg.contains("other"), "{msg}");
        let err = resolve_token_in(&path, None, None, None, None, &env, false).unwrap_err();
        assert!(err.to_string().contains("--project"));
    }

    #[test]
    fn store_and_remove_round_trip_keeps_other_entries() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("credentials.toml");
        store_credential(&path, "advisor", "a", "ta").unwrap();
        store_credential(&path, "advisor", "b", "tb").unwrap();
        store_credential(&path, "orchestrator", "a", "to").unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        assert!(remove_credential(&path, "advisor", "a").unwrap());
        assert!(!remove_credential(&path, "advisor", "a").unwrap());
        let table = read_credentials(&path).unwrap();
        assert_eq!(table["advisor"]["b"].as_str(), Some("tb"));
        assert_eq!(table["orchestrator"]["a"].as_str(), Some("to"));
        assert!(table["advisor"].get("a").is_none());

        assert!(remove_credential(&path, "advisor", "b").unwrap());
        assert!(read_credentials(&path).unwrap().get("advisor").is_none());
    }

    #[test]
    fn a_loose_credentials_file_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let path = creds_file(dir.path());
        store_credential(&path, "advisor", "demo", "t").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let env = creds_env(&[("BRIDLE_AS", "advisor")]);
        let err = resolve_token_in(&path, None, None, Some("demo"), None, &env, false).unwrap_err();
        assert!(err.to_string().contains("chmod 600"));
    }

    #[test]
    fn a_project_on_another_machine_routes_to_its_host_and_port() {
        let m: crate::machines::MachineMap = toml::from_str(
            "[machine]\nname = \"mbp\"\n[machines]\nnuc = \"nuc\"\n\
             [projects]\nmeta = { machine = \"nuc\", port = 7402 }",
        )
        .unwrap();
        let cwd = tempdir().unwrap();
        let ep = resolve_endpoint_with(&m, None, Some("meta"), cwd.path(), &empty_env()).unwrap();
        assert_eq!(ep.url, "http://nuc:7402");
        assert_eq!(ep.machine.as_deref(), Some("nuc"));
        assert_eq!(ep.project.as_deref(), Some("meta"));
        // $BRIDLE_PROJECT routes the same way.
        let env = MapEnv(std::collections::HashMap::from([(
            "BRIDLE_PROJECT",
            "meta",
        )]));
        let ep = resolve_endpoint_with(&m, None, None, cwd.path(), &env).unwrap();
        assert_eq!(ep.url, "http://nuc:7402");
    }

    #[test]
    fn a_project_on_this_machine_is_not_routed_remotely() {
        let m: crate::machines::MachineMap = toml::from_str(
            "[machine]\nname = \"mbp\"\n[machines]\nmbp = \"h\"\n\
             [projects]\nzz-no-such-project = { machine = \"mbp\", port = 1 }",
        )
        .unwrap();
        let cwd = tempdir().unwrap();
        let err = resolve_endpoint_with(
            &m,
            None,
            Some("zz-no-such-project"),
            cwd.path(),
            &empty_env(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no daemon registered"));
    }

    #[test]
    fn credentials_pick_the_machine_sub_table_and_old_files_still_work() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("credentials.toml");
        store_credential(&path, "advisor", "bridle", "local-tok").unwrap();
        // Hand-written, as the human pastes it on the other box.
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("\n[advisor.nuc]\nmeta = \"nuc-tok\"\n");
        fs::write(&path, text).unwrap();
        let env = MapEnv(std::collections::HashMap::from([("BRIDLE_AS", "advisor")]));
        let get = |project, machine| {
            resolve_token_in(&path, None, None, Some(project), machine, &env, false)
        };
        assert_eq!(get("bridle", None).unwrap().as_deref(), Some("local-tok"));
        assert_eq!(
            get("meta", Some("nuc")).unwrap().as_deref(),
            Some("nuc-tok")
        );
        assert!(get("meta", None).is_err());
        assert!(get("bridle", Some("nuc")).is_err());
    }

    #[test]
    fn human_on_another_machine_falls_back_to_the_human_machine_entry() {
        let dir = tempdir().unwrap();
        let path = creds_file(dir.path());
        store_credential(&path, "human", "bridle", "local-tok").unwrap();
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("\n[human.nuc]\nmeta = \"nuc-tok\"\n");
        fs::write(&path, text).unwrap();
        let env = empty_env();
        let get = |ws, project, machine| {
            resolve_token_in(&path, None, ws, Some(project), machine, &env, false)
        };
        assert_eq!(
            get(None, "meta", Some("nuc")).unwrap().as_deref(),
            Some("nuc-tok")
        );
        // Missing entry: the error is extended to name the section, not swallowed.
        let msg = get(None, "other", Some("nuc")).unwrap_err().to_string();
        assert!(msg.contains("no workspace found"), "{msg}");
        assert!(msg.contains("[human.nuc]"), "{msg}");
        // Without a machine the old error is unchanged.
        let msg = get(None, "meta", None).unwrap_err().to_string();
        assert!(!msg.contains("[human."), "{msg}");
    }

    #[test]
    fn a_local_project_keeps_the_workspace_token_even_with_human_entries() {
        let dir = tempdir().unwrap();
        let path = creds_file(dir.path());
        fs::write(&path, "[human.nuc]\nbridle = \"nuc-tok\"\n").unwrap();
        fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600)).unwrap();
        let ws = dir.path().join("ws");
        fs::create_dir_all(human_token_path(&ws).parent().unwrap()).unwrap();
        fs::write(human_token_path(&ws), "ws-tok\n").unwrap();
        let env = empty_env();
        let tok = resolve_token_in(&path, None, Some(&ws), Some("bridle"), None, &env, false);
        assert_eq!(tok.unwrap().as_deref(), Some("ws-tok"));
    }

    #[test]
    fn bridle_as_human_already_reads_the_machine_entry() {
        let dir = tempdir().unwrap();
        let path = creds_file(dir.path());
        fs::write(&path, "[human.dalek]\nmeta = \"d-tok\"\n").unwrap();
        fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600)).unwrap();
        let env = creds_env(&[("BRIDLE_AS", "human")]);
        let tok = resolve_token_in(&path, None, None, Some("meta"), Some("dalek"), &env, false);
        assert_eq!(tok.unwrap().as_deref(), Some("d-tok"));
    }
}
