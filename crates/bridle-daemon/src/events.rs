//! The event bus: every mutation the daemon records goes through one
//! `emit()` that appends to the store *and* broadcasts to live SSE
//! subscribers, so `GET /v1/events/stream` never misses an event between
//! subscribing and backfilling (docs/design/agent-host/api.md).

use bridle_api::types::{Event, PrincipalId};
use serde_json::Value;
use tokio::sync::broadcast;

use crate::store::{Store, StoreError};

const BUS_CAPACITY: usize = 4096;

#[derive(Clone)]
pub struct Emitter {
    store: Store,
    bus: broadcast::Sender<Event>,
}

impl Emitter {
    pub fn new(store: Store) -> Self {
        let (bus, _rx) = broadcast::channel(BUS_CAPACITY);
        Self { store, bus }
    }

    pub async fn emit(
        &self,
        kind: &str,
        actor: PrincipalId,
        agent: Option<String>,
        data: Value,
    ) -> Result<Event, StoreError> {
        let ev = self.store.append_event(kind, actor, agent, data).await?;
        // No receivers is fine (nobody is streaming right now); the event is
        // already durable in the store.
        let _ = self.bus.send(ev.clone());
        Ok(ev)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.bus.subscribe()
    }
}
