//! Which machine each project lives on, from `~/.bridle/config.toml` (ticket k7mw):
//!
//! ```toml
//! [machine]
//! name = "mbp"                 # this machine
//!
//! [machines]
//! mbp = "jb-mbp"               # the host to reach each machine by
//! nuc = "nuc"
//!
//! [projects]
//! meta-notes = { machine = "nuc", port = 7402 }
//! ```
//!
//! Written by hand, the same file on every box, and trusted: nothing probes.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::discovery::DiscoveryError;

/// Where one project's daemon is: a machine named in `[machines]`, and its port.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProjectPlace {
    pub machine: String,
    pub port: u16,
}

/// The `[machine] name`, `[machines]` and `[projects]` parts of the machine config.
/// Every other section is the daemon's and is ignored here.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct MachineMap {
    #[serde(default)]
    pub machine: MachineSelf,
    #[serde(default)]
    pub machines: BTreeMap<String, String>,
    #[serde(default)]
    pub projects: BTreeMap<String, ProjectPlace>,
}

/// `[machine]`: this machine's own name, a key of `[machines]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct MachineSelf {
    #[serde(default)]
    pub name: Option<String>,
    /// Overrides the machine's home project (default `<name>-home`), br-7ytg.
    #[serde(default)]
    pub home_project: Option<String>,
}

/// A project's daemon on another machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub machine: String,
    pub url: String,
}

/// The names this machine answers to in a project's `owner.toml` (53gy): `[machine] name` first
/// (the one `[projects]` uses, and the one recorded), then the `hostname`, which older bridles
/// recorded and so still counts as this machine. Never empty.
pub fn owner_names(home: &Path) -> Vec<String> {
    let mut names: Vec<String> = MachineMap::load(home)
        .ok()
        .and_then(|m| m.machine.name)
        .into_iter()
        .collect();
    let host = std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(n) = host
        && !names.contains(&n)
    {
        names.push(n);
    }
    if names.is_empty() {
        names.push("unknown".to_string());
    }
    names
}

/// The machine's home project: the one project where the machine's orchestrator lives
/// (br-7ytg). `<name>-home` unless `[machine] home_project` says otherwise.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct MachineHome {
    /// `[machine] name`, else the hostname; None when neither is known.
    pub machine: Option<String>,
    /// `[machine] home_project`, if set.
    pub override_: Option<String>,
    /// The override, else `<machine>-home`; None when there is no machine name either.
    pub home: Option<String>,
}

/// The one function everything asks for the machine's home project.
pub fn machine_home(bridle_home: &Path) -> MachineHome {
    let map = MachineMap::load(bridle_home).unwrap_or_default();
    let machine = map
        .machine
        .name
        .clone()
        .or_else(|| owner_names(bridle_home).into_iter().next())
        .filter(|n| n != "unknown");
    let override_ = map.machine.home_project.filter(|h| !h.is_empty());
    let home = override_
        .clone()
        .or_else(|| machine.as_ref().map(|m| format!("{m}-home")));
    MachineHome {
        machine,
        override_,
        home,
    }
}

impl MachineHome {
    /// Whether a project of that name runs here: listed in `[projects]` on this machine or in
    /// the local daemon registry.
    pub fn exists(&self, bridle_home: &Path) -> bool {
        let Some(home) = self.home.as_deref() else {
            return false;
        };
        let listed = MachineMap::load(bridle_home).is_ok_and(|m| {
            m.projects
                .get(home)
                .is_some_and(|p| Some(&p.machine) == self.machine.as_ref())
        });
        listed
            || crate::discovery::list_registry()
                .iter()
                .any(|d| d.project == home)
    }

    /// Warn-only check for a command that launches or addresses the orchestrator in `project`
    /// (None: it would pick one itself). `command` is what to run instead, with `{home}` for the
    /// home project. Nothing refuses yet (br-7ytg slice 1).
    pub fn warning(
        &self,
        bridle_home: &Path,
        project: Option<&str>,
        command: &str,
    ) -> Option<String> {
        let Some(home) = self.home.as_deref() else {
            return Some(
                "warning: this machine has no name, so no home project: set `[machine] name` in \
                 ~/.bridle/config.toml (the home is `<name>-home`), or `[machine] home_project`"
                    .to_string(),
            );
        };
        if !self.exists(bridle_home) {
            return Some(format!(
                "warning: the machine's home project '{home}' does not exist here: create it \
                 (a clone named {home}, `bridle init`, `bridle serve`) or set `[machine] \
                 home_project` in ~/.bridle/config.toml to an existing project"
            ));
        }
        match project {
            Some(p) if p != home => Some(format!(
                "warning: the machine's orchestrator belongs in its home project '{home}', not \
                 '{p}': run `{}`. This still works for now; a later release refuses.",
                command.replace("{home}", home)
            )),
            _ => None,
        }
    }
}

/// Projects of the local registry whose config has `[orchestrator] enabled = true`, other than
/// `home`: each would run an orchestrator supervisor outside the home.
pub fn non_home_orchestrators(home: Option<&str>, extra_repo: Option<&Path>) -> Vec<String> {
    let mut repos: Vec<(String, std::path::PathBuf)> = crate::discovery::list_registry()
        .into_iter()
        .map(|d| (d.project, std::path::PathBuf::from(d.repo)))
        .collect();
    if let Some(r) = extra_repo {
        let name = r
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !repos.iter().any(|(_, p)| p == r) {
            repos.push((name, r.to_path_buf()));
        }
    }
    let mut out: Vec<String> = repos
        .into_iter()
        .filter(|(p, _)| Some(p.as_str()) != home)
        .filter(|(_, repo)| orchestrator_enabled_in(repo))
        .map(|(p, _)| p)
        .collect();
    out.sort();
    out.dedup();
    out
}

fn orchestrator_enabled_in(repo: &Path) -> bool {
    std::fs::read_to_string(repo.join(".bridle/config.toml"))
        .ok()
        .and_then(|t| t.parse::<toml::Table>().ok())
        .and_then(|t| t.get("orchestrator")?.get("enabled")?.as_bool())
        .unwrap_or(false)
}

impl MachineMap {
    /// Reads `<home>/config.toml`; a missing file is an empty map.
    pub fn load(home: &Path) -> Result<Self, DiscoveryError> {
        let path = home.join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).map_err(|e| {
                DiscoveryError::Message(format!("{}: invalid machine config: {e}", path.display()))
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(DiscoveryError::io(&path, e)),
        }
    }

    /// The daemon of `project` if the config puts it on a machine other than this one;
    /// `None` for an unlisted project or one on this machine (use the local registry).
    pub fn remote(&self, project: &str) -> Result<Option<Remote>, DiscoveryError> {
        let Some(place) = self.projects.get(project) else {
            return Ok(None);
        };
        let Some(this) = self.machine.name.as_deref() else {
            return Err(DiscoveryError::Message(format!(
                "project '{project}' is listed in [projects] but this machine has no name: set \
                 `[machine] name = \"...\"` in ~/.bridle/config.toml"
            )));
        };
        if place.machine == this {
            return Ok(None);
        }
        let host = self.machines.get(&place.machine).ok_or_else(|| {
            DiscoveryError::Message(format!(
                "project '{project}' is on machine '{}', which is not in [machines]",
                place.machine
            ))
        })?;
        Ok(Some(Remote {
            machine: place.machine.clone(),
            url: format!("http://{host}:{}", place.port),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CFG: &str = r#"
        workflow = "x"
        [budget]
        five_hour = 0.5
        [machine]
        name = "mbp"
        [machines]
        mbp = "jb-mbp"
        nuc = "nuc"
        [projects]
        bridle = { machine = "mbp", port = 7401 }
        meta-notes = { machine = "nuc", port = 7402 }
    "#;

    #[test]
    fn parses_and_ignores_other_sections() {
        let m: MachineMap = toml::from_str(CFG).unwrap();
        assert_eq!(m.machine.name.as_deref(), Some("mbp"));
        assert_eq!(m.machines["nuc"], "nuc");
        assert_eq!(m.projects["meta-notes"].port, 7402);
    }

    #[test]
    fn remote_project_gets_host_and_port() {
        let m: MachineMap = toml::from_str(CFG).unwrap();
        let r = m.remote("meta-notes").unwrap().unwrap();
        assert_eq!(r.machine, "nuc");
        assert_eq!(r.url, "http://nuc:7402");
    }

    #[test]
    fn local_and_unlisted_projects_use_the_registry() {
        let m: MachineMap = toml::from_str(CFG).unwrap();
        assert_eq!(m.remote("bridle").unwrap(), None);
        assert_eq!(m.remote("other").unwrap(), None);
    }

    #[test]
    fn a_listed_project_needs_this_machines_name() {
        let m: MachineMap =
            toml::from_str("[projects]\np = { machine = \"nuc\", port = 1 }").unwrap();
        assert!(m.remote("p").is_err());
    }

    #[test]
    fn unknown_machine_is_an_error() {
        let m: MachineMap = toml::from_str(
            "[machine]\nname = \"mbp\"\n[projects]\np = { machine = \"nuc\", port = 1 }",
        )
        .unwrap();
        assert!(
            m.remote("p")
                .unwrap_err()
                .to_string()
                .contains("[machines]")
        );
    }

    fn home_dir(cfg: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.toml"), cfg).unwrap();
        dir
    }

    #[test]
    fn home_defaults_to_the_machine_name_and_the_config_overrides_it() {
        let d = home_dir("[machine]\nname = \"mbp\"\n");
        let h = machine_home(d.path());
        assert_eq!(h.home.as_deref(), Some("mbp-home"));
        assert_eq!(h.override_, None);
        let d = home_dir("[machine]\nname = \"mbp\"\nhome_project = \"base\"\n");
        let h = machine_home(d.path());
        assert_eq!(h.home.as_deref(), Some("base"));
        assert_eq!(h.override_.as_deref(), Some("base"));
    }

    #[test]
    fn a_non_home_project_gets_a_warning_naming_the_home_command() {
        let d = home_dir(
            "[machine]\nname = \"mbp\"\n[machines]\nmbp = \"h\"\n[projects]\nmbp-home = { machine = \"mbp\", port = 1 }\n",
        );
        let h = machine_home(d.path());
        let w = h
            .warning(d.path(), Some("bridle"), "bridle up --project {home}")
            .expect("warns");
        assert!(
            w.contains("'mbp-home'") && w.contains("bridle up --project mbp-home"),
            "{w}"
        );
        assert_eq!(h.warning(d.path(), Some("mbp-home"), "x"), None);
    }

    #[test]
    fn a_missing_home_says_how_to_create_one() {
        let d = home_dir("[machine]\nname = \"mbp\"\n");
        let w = machine_home(d.path())
            .warning(d.path(), None, "x")
            .expect("warns");
        assert!(
            w.contains("does not exist") && w.contains("bridle init"),
            "{w}"
        );
    }

    #[test]
    fn orchestrator_enabled_reads_the_projects_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".bridle")).unwrap();
        assert!(!orchestrator_enabled_in(dir.path()));
        std::fs::write(
            dir.path().join(".bridle/config.toml"),
            "[orchestrator]\nenabled = true\n",
        )
        .unwrap();
        assert!(orchestrator_enabled_in(dir.path()));
    }

    #[test]
    fn missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(MachineMap::load(dir.path()).unwrap(), MachineMap::default());
    }
}
