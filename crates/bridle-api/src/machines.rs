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

use serde::Deserialize;

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
}

/// A project's daemon on another machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub machine: String,
    pub url: String,
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

    #[test]
    fn missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(MachineMap::load(dir.path()).unwrap(), MachineMap::default());
    }
}
