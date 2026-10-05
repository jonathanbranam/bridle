//! Mail to a principal on another daemon (3haz, docs/design/agent-host/principals.md,
//! "Mail between daemons"). The sender's own daemon accepts the message at once, keeps it in its
//! outbox and forwards it to the destination daemon with a peer token. The receiving side is
//! `POST /v1/forward` in `server.rs`; it spots a repeat by the message's origin, so a try whose
//! acknowledgement was lost does no harm when it is made again.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bridle_api::Client;
use bridle_api::client::ClientError;
use bridle_api::discovery;
use bridle_api::machines::MachineMap;
use bridle_api::types::ForwardRequest;

use crate::store::{OutboxRow, Store};

/// One try's cap, so a daemon that accepts the connection and never answers can't hold the queue.
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct Outbox {
    store: Store,
    /// The machine's `~/.bridle`: `config.toml` says where daemons are, `credentials.toml` holds
    /// the peer tokens.
    home: PathBuf,
    project: String,
    /// One flush at a time per destination keeps delivery in order.
    locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl Outbox {
    pub fn new(store: Store, home: PathBuf, project: String) -> Self {
        Self {
            store,
            home,
            project,
            locks: Default::default(),
        }
    }

    /// This machine's `[machine] name`; `local` when it has none (a daemon pair on one machine
    /// needs no machine config).
    pub fn machine_name(&self) -> String {
        MachineMap::load(&self.home)
            .ok()
            .and_then(|m| m.machine.name)
            .unwrap_or_else(|| "local".to_string())
    }

    /// Where `project`'s daemon is: the machine config for another machine, else this
    /// machine's registry.
    pub fn resolve(&self, project: &str) -> Result<String, String> {
        let machines = MachineMap::load(&self.home).map_err(|e| e.to_string())?;
        if let Some(remote) = machines.remote(project).map_err(|e| e.to_string())? {
            return Ok(remote.url);
        }
        discovery::list_registry()
            .into_iter()
            .find(|d| d.project == project)
            .map(|d| d.url)
            .ok_or_else(|| {
                format!("no daemon known for project '{project}'; check `bridle daemons`")
            })
    }

    fn peer_token(&self, project: &str) -> Result<Option<String>, String> {
        discovery::peer_token(&self.home.join("credentials.toml"), project)
            .map_err(|e| e.to_string())
    }

    /// Refuses a destination that can't be forwarded to: unknown, or with no peer token.
    pub fn check_destination(&self, project: &str) -> Result<(), String> {
        self.resolve(project)?;
        if self.peer_token(project)?.is_none() {
            return Err(format!(
                "no peer token for '{project}': its human runs `bridle token create --peer {}` \
                 there and pastes the token under [peer] in {}",
                self.machine_name(),
                self.home.join("credentials.toml").display()
            ));
        }
        Ok(())
    }

    /// Delivers `project`'s queue, oldest first, until it is empty or a try doesn't get through
    /// (the message stays queued; the order never skips it).
    pub async fn flush(&self, project: &str) {
        let lock = self
            .locks
            .lock()
            .expect("outbox lock map poisoned")
            .entry(project.to_string())
            .or_default()
            .clone();
        let _held = lock.lock().await;
        loop {
            let row = match self.store.outbox_next(project).await {
                Ok(Some(row)) => row,
                Ok(None) => return,
                Err(e) => {
                    tracing::warn!(project, error = %e, "reading the outbox failed");
                    return;
                }
            };
            let outcome = self.try_once(&row).await;
            let delivered = outcome.is_ok();
            if let Err(e) = self.store.outbox_finish(&row.id, outcome).await {
                tracing::warn!(project, error = %e, "recording an outbox try failed");
                return;
            }
            // A permanent refusal is recorded and the queue moves on; a transient one stops here.
            let moved_on = delivered || self.is_failed(&row.id).await;
            if !moved_on {
                return;
            }
        }
    }

    async fn is_failed(&self, id: &str) -> bool {
        matches!(self.store.outbox_state(id).await, Ok(Some((s, _))) if s == "failed")
    }

    /// `Ok(ids)`: the receiver's acknowledgement. `Err((why, permanent))`: not delivered.
    async fn try_once(&self, row: &OutboxRow) -> Result<Vec<String>, (String, bool)> {
        let transient = |e: String| (e, false);
        let url = self.resolve(&row.project).map_err(transient)?;
        let token = self
            .peer_token(&row.project)
            .map_err(transient)?
            .ok_or_else(|| transient(format!("no peer token for '{}'", row.project)))?;
        let req = ForwardRequest {
            origin_machine: self.machine_name(),
            origin_daemon: self.project.clone(),
            origin_id: row.id.clone(),
            from: row.from.clone(),
            to: row.to.clone(),
            body: row.body.clone(),
            kind: row.kind,
            when: row.when,
            reply_to: row.reply_to.clone(),
        };
        let client = Client::new(url, Some(token));
        match tokio::time::timeout(ATTEMPT_TIMEOUT, client.forward(&req)).await {
            Err(_) => Err(transient("timed out".to_string())),
            Ok(Ok(ack)) => Ok(ack.message_ids),
            Ok(Err(ClientError::Api { code, message, .. }))
                if code == "not_found" || code == "bad_request" =>
            {
                Err((message, true))
            }
            Ok(Err(e)) => Err(transient(e.to_string())),
        }
    }
}
