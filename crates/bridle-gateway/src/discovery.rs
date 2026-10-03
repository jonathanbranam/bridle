//! Finds every defined daemon the way `bridle --project` does and asks each, concurrently and
//! with a short timeout, whether it answers. An unanswering daemon is a result, never an error.
//! See docs/design/human-web-ui.md sections 1 and 5.

use std::collections::BTreeMap;
use std::time::Duration;

use bridle_api::client::Client;
use bridle_api::discovery::{bridle_home, list_registry};
use bridle_api::machines::MachineMap;
use bridle_api::types::DaemonInfo;
use serde::Serialize;
use ts_rs::TS;

/// Long enough for a daemon on a LAN or tailnet, short enough that a sleeping laptop doesn't
/// hold up the page.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// One project's daemon, as the gateway saw it just now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ProjectStatus {
    pub project: String,
    /// The machine it lives on; `None` when that is this machine and the config doesn't name it.
    pub machine: Option<String>,
    pub url: Option<String>,
    pub reachable: bool,
    /// Why it is unreachable, for the UI to show; absent when reachable.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Projects {
    pub projects: Vec<ProjectStatus>,
}

/// A daemon to probe, before probing.
pub(crate) struct Target {
    pub project: String,
    pub machine: Option<String>,
    pub url: Option<String>,
    /// The daemon's workspace, known only for one on this machine; where its human token lives.
    pub workspace: Option<String>,
    /// Set when the config alone says it can't be reached.
    pub problem: Option<String>,
}

/// Discovers from this machine's registry and `~/.bridle/config.toml`, and probes.
pub async fn discover() -> Projects {
    probe_all(current_targets().await, PROBE_TIMEOUT).await
}

/// Every daemon this machine knows of, unprobed.
pub(crate) async fn current_targets() -> Vec<Target> {
    let machines = match MachineMap::load(&bridle_home()) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("machine config: {e}");
            MachineMap::default()
        }
    };
    // `list_registry` reads and prunes files: blocking work.
    let registry = tokio::task::spawn_blocking(list_registry)
        .await
        .unwrap_or_default();
    targets(&machines, registry)
}

/// The injectable core: the same inputs `discover` reads, so a test needs no home directory.
pub async fn discover_from(
    machines: &MachineMap,
    registry: Vec<DaemonInfo>,
    timeout: Duration,
) -> Projects {
    probe_all(targets(machines, registry), timeout).await
}

async fn probe_all(targets: Vec<Target>, timeout: Duration) -> Projects {
    let probes = targets.into_iter().map(|t| probe(t, timeout));
    let mut projects = futures::future::join_all(probes).await;
    projects.sort_by(|a, b| a.project.cmp(&b.project));
    Projects { projects }
}

fn targets(machines: &MachineMap, registry: Vec<DaemonInfo>) -> Vec<Target> {
    let mut by_project: BTreeMap<String, Target> = BTreeMap::new();
    for d in registry {
        by_project.insert(
            d.project.clone(),
            Target {
                project: d.project,
                machine: machines.machine.name.clone(),
                url: Some(d.url),
                workspace: Some(d.workspace),
                problem: None,
            },
        );
    }
    for (project, place) in &machines.projects {
        match machines.remote(project) {
            Ok(Some(remote)) => {
                by_project.insert(
                    project.clone(),
                    Target {
                        project: project.clone(),
                        machine: Some(remote.machine),
                        url: Some(remote.url),
                        workspace: None,
                        problem: None,
                    },
                );
            }
            // On this machine: the registry entry above is the daemon; none means it isn't running.
            Ok(None) => {
                by_project.entry(project.clone()).or_insert_with(|| Target {
                    project: project.clone(),
                    machine: Some(place.machine.clone()),
                    url: None,
                    workspace: None,
                    problem: Some("no daemon is running for it".to_string()),
                });
            }
            Err(e) => {
                by_project.insert(
                    project.clone(),
                    Target {
                        project: project.clone(),
                        machine: Some(place.machine.clone()),
                        url: None,
                        workspace: None,
                        problem: Some(e.to_string()),
                    },
                );
            }
        }
    }
    by_project.into_values().collect()
}

async fn probe(target: Target, timeout: Duration) -> ProjectStatus {
    let Target {
        project,
        machine,
        url,
        problem,
        ..
    } = target;
    let outcome = match (&url, problem) {
        (_, Some(problem)) => Err(problem),
        (Some(url), None) => Client::new_with_timeout(url.clone(), None, timeout)
            .health()
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        (None, None) => Err("no address".to_string()),
    };
    ProjectStatus {
        project,
        machine,
        url,
        reachable: outcome.is_ok(),
        reason: outcome.err(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, routing::get};
    use serde_json::json;

    async fn fake(delay: Duration) -> String {
        let app = Router::new().route(
            "/v1/health",
            get(move || async move {
                tokio::time::sleep(delay).await;
                Json(json!({"ok": true, "version": "t", "agent_count": 0}))
            }),
        );
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        url
    }

    fn info(project: &str, url: String) -> DaemonInfo {
        serde_json::from_value(json!({
            "project": project, "workspace": "/w", "repo": "/r", "url": url,
            "pid": 1, "started_at": "2026-01-01T00:00:00Z", "version": "t"
        }))
        .expect("daemon info")
    }

    #[tokio::test]
    async fn up_down_and_slow_daemons_are_reported_not_errors() {
        let up = fake(Duration::ZERO).await;
        let slow = fake(Duration::from_secs(5)).await;
        // A port nobody listens on.
        let down = {
            let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
            format!("http://{}", l.local_addr().expect("addr"))
        };
        let registry = vec![info("a-up", up), info("b-down", down), info("c-slow", slow)];
        let got = discover_from(&MachineMap::default(), registry, Duration::from_millis(500)).await;
        let seen: Vec<_> = got
            .projects
            .iter()
            .map(|p| (p.project.as_str(), p.reachable))
            .collect();
        assert_eq!(seen, [("a-up", true), ("b-down", false), ("c-slow", false)]);
        assert!(got.projects[1].reason.is_some());
        assert!(got.projects[0].reason.is_none());
    }

    #[tokio::test]
    async fn remote_projects_come_from_the_machine_config() {
        let up = fake(Duration::ZERO).await;
        let port: u16 = up.rsplit(':').next().expect("port").parse().expect("u16");
        let cfg = format!(
            "[machine]\nname = \"mbp\"\n[machines]\nmbp = \"x\"\nnuc = \"127.0.0.1\"\n\
             [projects]\nfar = {{ machine = \"nuc\", port = {port} }}\n\
             idle = {{ machine = \"mbp\", port = 1 }}\n\
             lost = {{ machine = \"gone\", port = 1 }}\n"
        );
        let machines: MachineMap = toml::from_str(&cfg).expect("config");
        let got = discover_from(&machines, vec![], Duration::from_millis(500)).await;
        let by: BTreeMap<_, _> = got
            .projects
            .iter()
            .map(|p| (p.project.as_str(), p))
            .collect();
        assert!(by["far"].reachable);
        assert_eq!(by["far"].machine.as_deref(), Some("nuc"));
        assert!(!by["idle"].reachable);
        assert!(
            by["lost"]
                .reason
                .as_deref()
                .expect("reason")
                .contains("[machines]")
        );
    }
}
