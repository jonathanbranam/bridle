//! Collects the human's interactions from every daemon the gateway knows and keeps the merge in
//! its own store (ticket u6w9, docs/design/human-web-ui.md "Human time"). Every 5 minutes it asks
//! each machine for its prompt log (`GET /v1/interactions`) and each project's daemon for the
//! messages the human sent; an unreachable one is reported and its data catches up later, since
//! the daemons' records are append-only.

use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bridle_api::client::Client;
use bridle_api::types::{InteractionEvent, InteractionsQuery, MessageQuery};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::discovery::{PROBE_TIMEOUT, Target, current_targets};
use crate::intervals::{Event, Kind};
use crate::items::{Source, source};

pub const POLL_EVERY: Duration = Duration::from_secs(300);

/// One stored line. `key` is what makes a record the same one seen twice: machine, session and
/// time for a log line, machine, project and id for a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Record {
    key: String,
    machine: String,
    project: String,
    agent: String,
    session: String,
    at: DateTime<Utc>,
    reply: bool,
}

/// The merged records, in a JSONL file under the bridle home; appended to, never rewritten.
#[derive(Clone)]
pub struct Store {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    path: Option<PathBuf>,
    keys: HashSet<String>,
    records: Vec<Record>,
    /// Who the last poll couldn't read.
    unreachable: Vec<String>,
}

impl Store {
    /// Opens `path`, reading what is there; a line that doesn't parse is skipped.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };
        let mut inner = Inner {
            path: Some(path.to_path_buf()),
            keys: HashSet::new(),
            records: Vec::new(),
            unreachable: Vec::new(),
        };
        for r in text
            .lines()
            .filter_map(|l| serde_json::from_str::<Record>(l).ok())
        {
            if inner.keys.insert(r.key.clone()) {
                inner.records.push(r);
            }
        }
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    /// A store that keeps nothing on disk, for tests.
    pub fn in_memory() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                path: None,
                keys: HashSet::new(),
                records: Vec::new(),
                unreachable: Vec::new(),
            })),
        }
    }

    /// Adds the records not seen before; returns how many were new.
    fn merge(&self, records: Vec<Record>) -> std::io::Result<usize> {
        let mut inner = self.inner.lock().expect("store lock");
        let fresh: Vec<Record> = records
            .into_iter()
            .filter(|r| inner.keys.insert(r.key.clone()))
            .collect();
        if let (Some(path), false) = (&inner.path, fresh.is_empty()) {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            for r in &fresh {
                writeln!(
                    f,
                    "{}",
                    serde_json::to_string(r).map_err(std::io::Error::other)?
                )?;
            }
        }
        let n = fresh.len();
        inner.records.extend(fresh);
        Ok(n)
    }

    /// Everything collected, for the interval math.
    pub fn events(&self) -> Vec<Event> {
        let inner = self.inner.lock().expect("store lock");
        inner
            .records
            .iter()
            .map(|r| Event {
                machine: r.machine.clone(),
                project: r.project.clone(),
                agent: r.agent.clone(),
                session: r.session.clone(),
                at: r.at,
                kind: if r.reply { Kind::Reply } else { Kind::Prompt },
            })
            .collect()
    }

    /// Machines (or projects) the last poll couldn't read, for the reports to flag.
    pub fn unreachable(&self) -> Vec<String> {
        self.inner.lock().expect("store lock").unreachable.clone()
    }

    /// A store holding `events`, for tests of what is built on it.
    #[cfg(test)]
    pub(crate) fn from_events(events: &[Event], unreachable: &[&str]) -> Self {
        let store = Self::in_memory();
        let records = events
            .iter()
            .enumerate()
            .map(|(i, e)| Record {
                key: format!("t{i}"),
                machine: e.machine.clone(),
                project: e.project.clone(),
                agent: e.agent.clone(),
                session: e.session.clone(),
                at: e.at,
                reply: e.kind == Kind::Reply,
            })
            .collect();
        store.merge(records).expect("merge");
        store.inner.lock().expect("store lock").unreachable =
            unreachable.iter().map(|s| s.to_string()).collect();
        store
    }

    /// The newest record of a machine: where its next poll can start.
    fn newest(&self, machine: &str) -> Option<DateTime<Utc>> {
        let inner = self.inner.lock().expect("store lock");
        inner
            .records
            .iter()
            .filter(|r| r.machine == machine)
            .map(|r| r.at)
            .max()
    }
}

/// What one poll found.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Polled {
    /// Records not seen before.
    pub new: usize,
    /// Machines (or projects, for their messages) that couldn't be read.
    pub unreachable: Vec<String>,
}

/// A daemon to read, or why it can't be.
type Daemon = (Source, Result<Client, String>);

const LOCAL: &str = "local";

/// Polls every daemon this machine knows of, once.
pub async fn poll(store: &Store) -> Polled {
    let targets = current_targets().await;
    // Token lookup reads files: blocking work.
    let sources = tokio::task::spawn_blocking(move || {
        targets.into_iter().map(|t: Target| source(t)).collect()
    })
    .await
    .unwrap_or_default();
    poll_from(store, sources, PROBE_TIMEOUT).await
}

pub(crate) async fn poll_from(store: &Store, sources: Vec<Source>, timeout: Duration) -> Polled {
    let clients: Vec<Daemon> = sources
        .into_iter()
        .map(|s| {
            let client = match (&s.problem, &s.url) {
                (Some(p), _) => Err(p.clone()),
                (None, None) => Err("no address".to_string()),
                (None, Some(url)) => Ok(Client::new_with_timeout(
                    url.clone(),
                    s.token.clone(),
                    timeout,
                )),
            };
            (s, client)
        })
        .collect();
    let machine_of = |s: &Source| s.machine.clone().unwrap_or_else(|| LOCAL.to_string());

    // The prompt log is machine-wide, so one daemon answering is enough for its machine.
    let mut by_machine: BTreeMap<String, Vec<&Daemon>> = BTreeMap::new();
    for c in &clients {
        by_machine.entry(machine_of(&c.0)).or_default().push(c);
    }
    let logs = by_machine.into_iter().map(|(machine, daemons)| async move {
        let query = InteractionsQuery {
            since: store.newest(&machine),
        };
        for (s, client) in daemons {
            let Ok(client) = client else { continue };
            if let Ok(lines) = client.interactions(&query).await {
                let records: Vec<Record> = lines
                    .into_iter()
                    .map(|l| {
                        let session = l.session.unwrap_or_default();
                        let reply = l.event == InteractionEvent::Reply;
                        let machine = l.machine.unwrap_or_else(|| machine.clone());
                        Record {
                            key: format!("{machine}|{session}|{}|{}", l.at.to_rfc3339(), reply),
                            project: l.project.unwrap_or_else(|| s.project.clone()),
                            agent: l.role.unwrap_or_default(),
                            machine,
                            session,
                            at: l.at,
                            reply,
                        }
                    })
                    .collect();
                return Ok(records);
            }
        }
        Err(machine)
    });
    // Messages are per project; each is a point prompt to its recipient, in a session of its own.
    let messages = clients.iter().map(|(s, client)| async move {
        let Ok(client) = client else {
            return Err(s.project.clone());
        };
        let query = MessageQuery {
            from: Some("human".to_string()),
            ..Default::default()
        };
        let list = client
            .list_messages(&query)
            .await
            .map_err(|_| s.project.clone())?;
        let machine = machine_of(s);
        Ok(list
            .into_iter()
            .map(|m| Record {
                key: format!("{machine}|{}|{}", s.project, m.id),
                machine: machine.clone(),
                project: s.project.clone(),
                session: format!("message:{}", m.to),
                agent: m.to,
                at: m.created_at,
                reply: false,
            })
            .collect::<Vec<_>>())
    });
    let (logs, messages) = futures::future::join(
        futures::future::join_all(logs),
        futures::future::join_all(messages),
    )
    .await;

    let mut polled = Polled::default();
    let mut records = Vec::new();
    for r in logs {
        match r {
            Ok(rs) => records.extend(rs),
            Err(machine) => polled.unreachable.push(machine),
        }
    }
    for r in messages {
        match r {
            Ok(rs) => records.extend(rs),
            Err(project) => polled.unreachable.push(project),
        }
    }
    match store.merge(records) {
        Ok(n) => polled.new = n,
        Err(e) => tracing::warn!("storing interactions: {e}"),
    }
    polled.unreachable.sort();
    polled.unreachable.dedup();
    store.inner.lock().expect("store lock").unreachable = polled.unreachable.clone();
    polled
}

/// Polls now and then every [`POLL_EVERY`], for as long as the gateway runs.
pub fn spawn(store: Store) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(POLL_EVERY);
        loop {
            tick.tick().await;
            let p = poll(&store).await;
            tracing::info!(new = p.new, unreachable = ?p.unreachable, "polled interactions");
        }
    })
}

/// Where the gateway keeps what it collected.
pub fn store_path(home: &Path) -> PathBuf {
    home.join("gateway-interactions.jsonl")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, routing::get};
    use serde_json::{Value, json};

    async fn fake(log: Vec<Value>, messages: Vec<Value>) -> String {
        let app = Router::new()
            .route("/v1/interactions", get(move || async move { Json(log) }))
            .route("/v1/messages", get(move || async move { Json(messages) }));
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", l.local_addr().expect("addr"));
        tokio::spawn(async move { axum::serve(l, app).await });
        url
    }

    fn src(project: &str, machine: Option<&str>, url: Option<String>) -> Source {
        Source {
            project: project.into(),
            machine: machine.map(str::to_string),
            url,
            token: None,
            problem: None,
        }
    }

    fn line(session: &str, at: &str, event: &str) -> Value {
        json!({"at": at, "event": event, "session": session, "role": "orchestrator",
               "machine": "mbp", "project": "p"})
    }

    fn message(id: &str, to: &str, at: &str) -> Value {
        json!({"id": id, "from": "human", "to": to, "kind": "note", "body": "hi",
               "reply_to": null, "when": "now", "state": "read", "created_at": at,
               "written_at": null, "delivered_at": null, "read_at": null})
    }

    #[tokio::test]
    async fn merges_dedupes_across_daemons_and_polls_and_survives_a_down_machine() {
        let log = vec![
            line("s1", "2026-10-03T12:00:00Z", "prompt"),
            line("s1", "2026-10-03T12:01:00Z", "reply"),
        ];
        let msgs = vec![message("m-1", "agent:w", "2026-10-03T12:05:00Z")];
        // Two projects on one machine serve the same machine-wide log.
        let a = fake(log.clone(), msgs).await;
        let b = fake(log, vec![]).await;
        let down = {
            let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
            format!("http://{}", l.local_addr().expect("addr"))
        };
        let sources = || {
            vec![
                src("p", Some("mbp"), Some(a.clone())),
                src("q", Some("mbp"), Some(b.clone())),
                src("far", Some("nuc"), Some(down.clone())),
            ]
        };
        let store = Store::in_memory();
        let first = poll_from(&store, sources(), Duration::from_millis(500)).await;
        // Two log lines once, one message.
        assert_eq!(first.new, 3);
        assert_eq!(first.unreachable, ["far", "nuc"]);
        let again = poll_from(&store, sources(), Duration::from_millis(500)).await;
        assert_eq!(again.new, 0);
        let events = store.events();
        assert_eq!(events.iter().filter(|e| e.kind == Kind::Reply).count(), 1);
        assert!(
            events
                .iter()
                .any(|e| e.session == "message:agent:w" && e.agent == "agent:w")
        );
    }

    #[test]
    fn the_store_survives_a_restart_and_skips_bad_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("x").join("i.jsonl");
        let store = Store::open(&path).expect("open");
        let rec = Record {
            key: "k".into(),
            machine: "m".into(),
            project: "p".into(),
            agent: "a".into(),
            session: "s".into(),
            at: "2026-10-03T12:00:00Z".parse().expect("time"),
            reply: false,
        };
        assert_eq!(store.merge(vec![rec.clone(), rec]).expect("merge"), 1);
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("open");
        writeln!(f, "not json").expect("write");
        let reopened = Store::open(&path).expect("reopen");
        assert_eq!(reopened.events().len(), 1);
    }
}
