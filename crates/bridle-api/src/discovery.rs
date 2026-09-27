//! Daemon discovery: workspace layout, `daemon.json`, the machine registry
//! and token/endpoint resolution. Pure filesystem + env logic, shared by the
//! CLI and (by design) the daemon itself. See docs/agent-host.md §2.1, §3.1
//! and §5.2.

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
    fn io(path: &Path, source: io::Error) -> Self {
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
}

/// Order: `--url`, `$BRIDLE_URL`, `--project`/`$BRIDLE_PROJECT` via the
/// registry, then a cwd walk to `.bridle/daemon.json`.
pub fn resolve_endpoint(
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
        });
    }
    if let Some(url) = env.var("BRIDLE_URL") {
        return Ok(Endpoint {
            url,
            workspace: None,
            project: None,
        });
    }
    let project = project_flag
        .map(str::to_string)
        .or_else(|| env.var("BRIDLE_PROJECT"));
    if let Some(project) = project {
        return list_registry()
            .into_iter()
            .find(|d| d.project == project)
            .map(|d| Endpoint {
                url: d.url,
                workspace: Some(PathBuf::from(d.workspace)),
                project: Some(d.project),
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
        });
    }
    Err(DiscoveryError::Message(
        "no bridle daemon found: pass --url, set $BRIDLE_URL, use --project/$BRIDLE_PROJECT \
         (see `bridle daemons`), or run inside a workspace created by `bridle serve`"
            .to_string(),
    ))
}

/// §5.2: `--token`/`$BRIDLE_TOKEN`, else (only if `$CLAUDECODE` is unset) the
/// human token file in `workspace`, else an error.
pub fn resolve_token(
    token_flag: Option<&str>,
    workspace: Option<&Path>,
    env: &impl Env,
) -> Result<Option<String>, DiscoveryError> {
    if let Some(t) = token_flag {
        return Ok(Some(t.to_string()));
    }
    if let Some(t) = env.var("BRIDLE_TOKEN") {
        return Ok(Some(t));
    }
    if env.var("CLAUDECODE").is_some() {
        return Err(DiscoveryError::Message(
            "running inside Claude Code ($CLAUDECODE is set), so the human token is never \
             used implicitly: set $BRIDLE_TOKEN"
                .to_string(),
        ));
    }
    let Some(workspace) = workspace else {
        return Err(DiscoveryError::Message(
            "no workspace found to read the human token from: set $BRIDLE_TOKEN".to_string(),
        ));
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
            resolve_token(Some("flag"), None, &empty_env()).unwrap(),
            Some("flag".to_string())
        );
        let env = MapEnv(std::collections::HashMap::from([(
            "BRIDLE_TOKEN",
            "envtok",
        )]));
        assert_eq!(
            resolve_token(None, None, &env).unwrap(),
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

        let tok = resolve_token(None, Some(ws), &empty_env()).unwrap();
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
        let err = resolve_token(None, Some(ws), &env).unwrap_err();
        assert!(err.to_string().contains("BRIDLE_TOKEN"));
    }

    #[test]
    fn resolve_token_errors_when_no_token_file_exists() {
        let root = tempdir().unwrap();
        let err = resolve_token(None, Some(root.path()), &empty_env()).unwrap_err();
        assert!(err.to_string().contains("BRIDLE_TOKEN"));
    }
}
