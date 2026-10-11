//! Mail to a principal on another daemon (3haz, docs/design/agent-host/principals.md,
//! "Mail between daemons"). The sender's own daemon accepts the message at once, keeps it in its
//! outbox and forwards it to the destination daemon with a peer token. The receiving side is
//! `POST /v1/forward` in `server.rs`; it spots a repeat by the message's origin, so a try whose
//! acknowledgement was lost does no harm when it is made again.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use tokio::time::Instant;

use bridle_api::Client;
use bridle_api::client::ClientError;
use bridle_api::discovery;
use bridle_api::machines::MachineMap;
use bridle_api::types::{
    ForwardAck, ForwardRequest, HelloRequest, MessageKind, MessageState, OutboxEntry,
    PeerWatchRequest, Task, When,
};

use crate::store::{OutboxRow, Store};
use crate::supervisor::{AgentManager, ToTarget};

/// One try's cap, so a daemon that accepts the connection and never answers can't hold the queue.
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(30);

/// How often the retry loop looks at the queues. It is also the yardstick for noticing a sleep:
/// a wall clock that moved on by much more than this since the last look means the machine slept.
pub const TICK: Duration = Duration::from_secs(15);

/// A wall-clock gap between two looks beyond which the machine is taken to have slept.
const SLEEP_GAP: Duration = Duration::from_secs(60);

/// A message queued this long without getting through is reported to its sender (once); it keeps
/// being retried.
pub const STUCK_AFTER: Duration = Duration::from_secs(30 * 60);

/// A message queued this long without getting through is also reported to the human through the
/// aide (once). `[messages] undelivered_report_mins` changes it.
pub const REPORT_AFTER: Duration = Duration::from_secs(60 * 60);

/// How long `POST /v1/outbox` waits for the first try before answering "queued".
pub const FIRST_TRY_WAIT: Duration = Duration::from_secs(3);

/// The wait before the next try of a destination's head message, once `attempts` tries have
/// failed: the first try is at once, then 30 s, 2 m, and every 5 m after that. Messages never
/// expire.
pub fn retry_delay(attempts: i64) -> Duration {
    match attempts {
        ..=1 => Duration::from_secs(30),
        2 => Duration::from_secs(120),
        _ => Duration::from_secs(300),
    }
}

/// Hands one forward to the destination daemon: `(url, peer token, request)`. A seam so the
/// retry schedule can be tested without sockets.
pub type Transport = Arc<
    dyn Fn(String, String, ForwardRequest) -> BoxFuture<'static, Result<ForwardAck, ClientError>>
        + Send
        + Sync,
>;

/// Tells a local principal something: `(the sender's id on this daemon, text)`. Delivery is the
/// daemon's `system` voice; see [`notifier`].
pub type Notifier = Arc<dyn Fn(String, String) -> BoxFuture<'static, ()> + Send + Sync>;

type WallClock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

fn http_transport() -> Transport {
    Arc::new(|url, token, req| {
        Box::pin(async move { Client::new(url, Some(token)).forward(&req).await })
    })
}

/// The production [`Notifier`]: a note from `system` to the local sender (human, external
/// principal or agent). A sender that is gone is dropped with a log line.
pub fn notifier(store: Store, manager: AgentManager) -> Notifier {
    Arc::new(move |sender, body| {
        let store = store.clone();
        let manager = manager.clone();
        Box::pin(async move {
            let target = if sender == "human" {
                ToTarget::Human
            } else if sender.starts_with("external:") || sender.starts_with("human@") {
                ToTarget::External(sender)
            } else {
                let name = sender.strip_prefix("agent:").unwrap_or(&sender);
                match store.get_agent(name).await {
                    Ok(Some(a)) => ToTarget::Agent(a.id),
                    _ => {
                        tracing::warn!(sender, "outbox notice dropped: the sender is gone");
                        return;
                    }
                }
            };
            if let Err(e) = manager
                .send(
                    "system".to_string(),
                    target,
                    MessageKind::Note,
                    body,
                    When::Now,
                    None,
                )
                .await
            {
                tracing::warn!(error = %e, "outbox notice not delivered");
            }
        })
    })
}

#[derive(Clone)]
pub struct Outbox {
    store: Store,
    /// The machine's `~/.bridle`: `config.toml` says where daemons are, `credentials.toml` holds
    /// the peer tokens.
    home: PathBuf,
    project: String,
    /// One flush at a time per destination keeps delivery in order.
    locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
    /// When a destination's head message may next be tried. In memory only: after a restart
    /// everything is tried at once, which is what the start-up flush wants anyway.
    next_try: Arc<Mutex<HashMap<String, Instant>>>,
    transport: Transport,
    notify: Notifier,
    wall: WallClock,
    /// The wall clock at the previous `tick`.
    last_look: Arc<Mutex<DateTime<Utc>>>,
    stuck_after: Duration,
    report_after: Duration,
}

impl Outbox {
    pub fn new(store: Store, home: PathBuf, project: String, notify: Notifier) -> Self {
        Self {
            store,
            home,
            project,
            locks: Default::default(),
            next_try: Default::default(),
            transport: http_transport(),
            notify,
            wall: Arc::new(Utc::now),
            last_look: Arc::new(Mutex::new(Utc::now())),
            stuck_after: STUCK_AFTER,
            report_after: REPORT_AFTER,
        }
    }

    pub fn with_report_after(mut self, report_after: Duration) -> Self {
        self.report_after = report_after;
        self
    }

    #[cfg(test)]
    fn with_transport(mut self, transport: Transport) -> Self {
        self.transport = transport;
        self
    }

    #[cfg(test)]
    fn with_wall(mut self, wall: WallClock) -> Self {
        *self.last_look.lock().expect("look") = wall();
        self.wall = wall;
        self
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

    /// This daemon's project: how a peer names it as a watcher's home.
    pub fn project(&self) -> &str {
        &self.project
    }

    /// Asks `project`'s daemon, with our peer token, to start or stop a watch for one of our
    /// principals. Not queued: the caller needs the answer (no such task, no peer token), and
    /// a watch is state, not mail.
    pub async fn peer_watch(
        &self,
        project: &str,
        req: &PeerWatchRequest,
    ) -> Result<Task, ClientError> {
        let other = ClientError::Unreachable;
        self.check_destination(project).map_err(other)?;
        let url = self.resolve(project).map_err(other)?;
        let token = self.peer_token(project).map_err(other)?;
        match tokio::time::timeout(ATTEMPT_TIMEOUT, Client::new(url, token).peer_watch(req)).await {
            Ok(r) => r,
            Err(_) => Err(other(format!("'{project}' did not answer in time"))),
        }
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
            let failure = outcome.clone().err();
            if let Err(e) = self.store.outbox_finish(&row.id, outcome).await {
                tracing::warn!(project, error = %e, "recording an outbox try failed");
                return;
            }
            match failure {
                None => {
                    self.next_try
                        .lock()
                        .expect("next_try poisoned")
                        .remove(project);
                }
                // A permanent refusal is recorded, the sender told, and the queue moves on.
                Some((why, true)) => {
                    self.tell_sender(
                        &row,
                        format!(
                            "Your message to {} on '{}' was refused and will not be retried: \
                             {why} (outbox {}).",
                            row.to, row.project, row.id
                        ),
                    )
                    .await;
                }
                // A transient one stops here and waits out the backoff.
                Some((_, false)) => {
                    let wait = retry_delay(row.attempts + 1);
                    self.next_try
                        .lock()
                        .expect("next_try poisoned")
                        .insert(project.to_string(), Instant::now() + wait);
                    return;
                }
            }
        }
    }

    /// One look at the queues: a machine that slept gets its peers pinged and everything tried
    /// at once; otherwise each destination whose backoff has run out is tried; then senders of
    /// long-stuck messages are told.
    pub async fn tick(&self) {
        let now = (self.wall)();
        let gap = {
            let mut last = self.last_look.lock().expect("look poisoned");
            std::mem::replace(&mut *last, now)
        };
        let slept = (now - gap).to_std().is_ok_and(|g| g > SLEEP_GAP);
        if slept {
            tracing::info!(
                "the clock jumped: the machine slept; greeting peers, flushing the outbox"
            );
            self.next_try.lock().expect("next_try poisoned").clear();
            tokio::spawn(self.clone().ping_peers());
        }
        match self.store.outbox_queued_projects().await {
            Ok(projects) => {
                for project in projects {
                    if self.due(&project) {
                        self.flush(&project).await;
                    }
                }
            }
            Err(e) => tracing::warn!(error = %e, "reading the outbox failed"),
        }
        self.notify_stuck().await;
        self.report_to_aide().await;
    }

    /// The outbox row `id` as its sender sees it. A message the destination has accepted is
    /// `arrived` until that daemon says its recipient received it, then `delivered`; the
    /// destination is asked each time (it is the one that knows).
    pub async fn entry(&self, id: &str) -> Result<Option<OutboxEntry>, crate::store::StoreError> {
        let Some(mut entry) = self.store.outbox_entry(id).await? else {
            return Ok(None);
        };
        if entry.state != "delivered" {
            return Ok(Some(entry));
        }
        entry.state = "arrived".to_string();
        match self.remote_stage(&entry).await {
            Ok(stage) => entry.state = stage,
            Err(why) => {
                entry.last_error = Some(format!("could not ask '{}': {why}", entry.project))
            }
        }
        Ok(Some(entry))
    }

    /// The least advanced stage of the destination's copies of `entry`.
    async fn remote_stage(&self, entry: &OutboxEntry) -> Result<String, String> {
        let url = self.resolve(&entry.project)?;
        let token = self
            .peer_token(&entry.project)?
            .ok_or_else(|| "no peer token".to_string())?;
        let client = Client::new(url, Some(token));
        let mut stage = "delivered";
        for id in &entry.remote_ids {
            let asked = tokio::time::timeout(ATTEMPT_TIMEOUT, client.forward_state(id)).await;
            let state = asked
                .map_err(|_| "timed out".to_string())?
                .map_err(|e| e.to_string())?
                .state;
            match state {
                MessageState::Dropped => return Ok("failed".to_string()),
                MessageState::Delivered | MessageState::Read => {}
                _ => stage = "arrived",
            }
        }
        Ok(stage.to_string())
    }

    /// Tells the human, through the aide, of each message undelivered past `report_after`. Once
    /// per message: it is marked before it is sent.
    async fn report_to_aide(&self) {
        let cutoff = (self.wall)()
            - chrono::Duration::from_std(self.report_after).unwrap_or(chrono::Duration::MAX);
        let rows = match self.store.outbox_aide_due(cutoff).await {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!(error = %e, "reading unreported outbox entries failed");
                return;
            }
        };
        for row in rows {
            if self.store.outbox_mark_aide(&row.id).await.is_err() {
                continue;
            }
            let why = row.last_error.as_deref().unwrap_or("no answer yet");
            (self.notify)(
                "external:aide".to_string(),
                format!(
                    "For the human: {}'s message to {} on '{}' has been undelivered for over {}                      min: {why}. It is still being retried (outbox {}).",
                    row.from,
                    row.to,
                    row.project,
                    self.report_after.as_secs() / 60,
                    row.id
                ),
            )
            .await;
        }
    }

    fn due(&self, project: &str) -> bool {
        self.next_try
            .lock()
            .expect("next_try poisoned")
            .get(project)
            .is_none_or(|at| *at <= Instant::now())
    }

    /// A peer said it is back: try its queue now, backoff or not.
    pub async fn peer_is_back(&self, project: &str) {
        self.next_try
            .lock()
            .expect("next_try poisoned")
            .remove(project);
        self.flush(project).await;
    }

    /// Greets every daemon the machine config lists (that we hold a peer token for) so each
    /// flushes what it has queued for us. Best effort: a peer that is down hears nothing, and
    /// greets us when it comes up.
    pub async fn ping_peers(self) {
        let Ok(machines) = MachineMap::load(&self.home) else {
            return;
        };
        for project in machines.projects.keys() {
            if *project == self.project {
                continue;
            }
            let (Ok(url), Ok(Some(token))) = (self.resolve(project), self.peer_token(project))
            else {
                continue;
            };
            let hello = HelloRequest {
                daemon: self.project.clone(),
            };
            let client = Client::new(url, Some(token));
            match tokio::time::timeout(ATTEMPT_TIMEOUT, client.hello(&hello)).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => tracing::debug!(project, error = %e, "peer greeting not heard"),
                Err(_) => tracing::debug!(project, "peer greeting timed out"),
            }
        }
    }

    async fn notify_stuck(&self) {
        let cutoff = (self.wall)()
            - chrono::Duration::from_std(self.stuck_after).unwrap_or(chrono::Duration::MAX);
        let rows = match self.store.outbox_stuck(cutoff).await {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!(error = %e, "reading stuck outbox entries failed");
                return;
            }
        };
        for row in rows {
            // Marked first: the notice is sent at most once, even if the send fails.
            if self.store.outbox_mark_stuck(&row.id).await.is_err() {
                continue;
            }
            let why = row.last_error.as_deref().unwrap_or("no answer yet");
            self.tell_sender(
                &row,
                format!(
                    "Your message to {} on '{}' has been queued for {} min and is not delivered \
                     yet: {why}. It is still being retried (outbox {}).",
                    row.to,
                    row.project,
                    self.stuck_after.as_secs() / 60,
                    row.id
                ),
            )
            .await;
        }
    }

    /// Says `text` to the local sender: its label carries this machine's name, which goes.
    async fn tell_sender(&self, row: &OutboxRow, text: String) {
        let suffix = format!("@{}", self.machine_name());
        let local = row.from.strip_suffix(&suffix).unwrap_or(&row.from);
        (self.notify)(local.to_string(), text).await;
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
        match tokio::time::timeout(ATTEMPT_TIMEOUT, (self.transport)(url, token, req)).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

    /// A sender with a fake peer ("beta" on machine m2): its transport log, an up/down switch
    /// and the notices it sent, all on tokio's paused clock.
    struct Rig {
        outbox: Outbox,
        store: Store,
        up: Arc<AtomicBool>,
        /// `(seconds since the start, origin id)` of every try, delivered or not.
        tries: Arc<Mutex<Vec<(u64, String)>>>,
        delivered: Arc<Mutex<Vec<String>>>,
        notices: Arc<Mutex<Vec<(String, String)>>>,
        /// Added to the wall clock: a sleep, or the passing of half an hour.
        skew_secs: Arc<AtomicI64>,
        start: Instant,
        _tmp: tempfile::TempDir,
    }

    async fn rig() -> Rig {
        let tmp = tempfile::tempdir().expect("tmp");
        let home = tmp.path().to_path_buf();
        std::fs::write(
            home.join("config.toml"),
            "[machine]\nname = \"m1\"\n[machines]\nm1 = \"127.0.0.1\"\nm2 = \"127.0.0.1\"\n\
             [projects]\nbeta = { machine = \"m2\", port = 1 }\n",
        )
        .expect("config");
        bridle_api::discovery::store_credential(
            &home.join("credentials.toml"),
            "peer",
            "beta",
            "t",
        )
        .expect("credentials");
        let store = Store::open(home.join("state.db")).await.expect("store");
        let up = Arc::new(AtomicBool::new(false));
        let tries: Arc<Mutex<Vec<(u64, String)>>> = Default::default();
        let delivered: Arc<Mutex<Vec<String>>> = Default::default();
        let notices: Arc<Mutex<Vec<(String, String)>>> = Default::default();
        let skew_secs = Arc::new(AtomicI64::new(0));
        let start = Instant::now();
        let transport: Transport = {
            let (up, tries, delivered) = (up.clone(), tries.clone(), delivered.clone());
            Arc::new(move |_url, _token, req| {
                let (up, tries, delivered) = (up.clone(), tries.clone(), delivered.clone());
                Box::pin(async move {
                    let at = start.elapsed().as_secs();
                    tries
                        .lock()
                        .expect("tries")
                        .push((at, req.origin_id.clone()));
                    if req.to == "agent:nobody" && up.load(Ordering::SeqCst) {
                        return Err(ClientError::Api {
                            status: 404,
                            code: "not_found".into(),
                            message: "no such recipient: agent:nobody".into(),
                        });
                    }
                    if !up.load(Ordering::SeqCst) {
                        return Err(ClientError::Unreachable("connection refused".into()));
                    }
                    delivered
                        .lock()
                        .expect("delivered")
                        .push(req.origin_id.clone());
                    Ok(ForwardAck {
                        message_ids: vec![format!("m-{}", req.origin_id)],
                    })
                })
            })
        };
        let notify: Notifier = {
            let notices = notices.clone();
            Arc::new(move |sender, text| {
                let notices = notices.clone();
                Box::pin(async move { notices.lock().expect("notices").push((sender, text)) })
            })
        };
        let wall: WallClock = {
            let skew = skew_secs.clone();
            Arc::new(move || Utc::now() + chrono::Duration::seconds(skew.load(Ordering::SeqCst)))
        };
        let outbox = Outbox::new(store.clone(), home, "alpha".to_string(), notify)
            .with_transport(transport)
            .with_wall(wall);
        Rig {
            outbox,
            store,
            up,
            tries,
            delivered,
            notices,
            skew_secs,
            start,
            _tmp: tmp,
        }
    }

    impl Rig {
        async fn queue(&self, from: &str, to: &str) -> String {
            self.store
                .outbox_enqueue(OutboxRow {
                    id: String::new(),
                    project: "beta".to_string(),
                    from: from.to_string(),
                    to: to.to_string(),
                    kind: MessageKind::Note,
                    body: "hi".to_string(),
                    reply_to: None,
                    when: When::Now,
                    attempts: 0,
                    last_error: None,
                })
                .await
                .expect("enqueue")
        }

        /// Moves the paused clock on `secs` and takes one look at the queues.
        async fn step(&self, secs: u64) {
            tokio::time::advance(Duration::from_secs(secs)).await;
            self.outbox.tick().await;
        }

        fn elapsed(&self) -> u64 {
            self.start.elapsed().as_secs()
        }

        fn try_times(&self) -> Vec<u64> {
            self.tries
                .lock()
                .expect("tries")
                .iter()
                .map(|t| t.0)
                .collect()
        }
    }

    #[test]
    fn the_backoff_is_30s_then_2m_then_5m() {
        let secs: Vec<_> = (1..=5).map(|n| retry_delay(n).as_secs()).collect();
        assert_eq!(secs, [30, 120, 300, 300, 300]);
    }

    #[tokio::test(start_paused = true)]
    async fn a_down_peer_is_retried_on_the_schedule_and_the_message_is_kept() {
        let r = rig().await;
        r.queue("agent:w1@m1", "external:advisor").await;
        r.outbox.tick().await;
        for _ in 0..80 {
            r.step(10).await;
        }
        // At once, then 30 s, 2 m, 5 m, 5 m after each failure.
        assert_eq!(r.try_times(), [0, 30, 150, 450, 750]);
        assert!(r.delivered.lock().expect("d").is_empty());
        assert_eq!(r.store.outbox_queued_projects().await.expect("q"), ["beta"]);
    }

    #[tokio::test(start_paused = true)]
    async fn a_peer_that_says_it_is_back_gets_its_queue_at_once_once_each_and_in_order() {
        let r = rig().await;
        let first = r.queue("agent:w1@m1", "external:advisor").await;
        let second = r.queue("agent:w1@m1", "external:advisor").await;
        r.outbox.tick().await;
        // Only the head is tried while the peer is down: nothing overtakes it.
        assert_eq!(r.try_times(), [0]);
        r.step(10).await;
        assert_eq!(r.try_times(), [0], "still inside the backoff");

        r.up.store(true, Ordering::SeqCst);
        r.outbox.peer_is_back("beta").await;
        assert_eq!(r.elapsed(), 10, "at once, not after the backoff");
        assert_eq!(*r.delivered.lock().expect("d"), [first, second]);
        // Nothing left to send, so no later look tries anything again.
        r.step(600).await;
        assert_eq!(r.delivered.lock().expect("d").len(), 2);
        assert_eq!(r.tries.lock().expect("t").len(), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn a_clock_jump_is_a_sleep_and_flushes_inside_the_backoff() {
        let r = rig().await;
        r.queue("agent:w1@m1", "external:advisor").await;
        r.outbox.tick().await;
        r.up.store(true, Ordering::SeqCst);
        r.step(10).await;
        assert!(
            r.delivered.lock().expect("d").is_empty(),
            "inside the backoff"
        );
        // The laptop slept an hour: the wall clock jumped, the monotonic one barely moved.
        r.skew_secs.store(3600, Ordering::SeqCst);
        r.step(10).await;
        assert_eq!(r.delivered.lock().expect("d").len(), 1);
        assert_eq!(r.elapsed(), 20);
    }

    #[tokio::test(start_paused = true)]
    async fn a_refusal_for_good_tells_the_sender_once_and_the_queue_moves_on() {
        let r = rig().await;
        r.up.store(true, Ordering::SeqCst);
        r.queue("agent:w1@m1", "agent:nobody").await;
        let next = r.queue("agent:w1@m1", "external:advisor").await;
        r.outbox.tick().await;
        r.step(60).await;
        assert_eq!(*r.delivered.lock().expect("d"), [next]);
        let notices = r.notices.lock().expect("n").clone();
        assert_eq!(notices.len(), 1, "{notices:?}");
        let (to, text) = &notices[0];
        assert_eq!(to, "agent:w1", "the machine label is dropped");
        for part in ["beta", "agent:nobody", "no such recipient"] {
            assert!(text.contains(part), "{part} missing from {text}");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn a_message_stuck_half_an_hour_tells_the_sender_once_and_keeps_retrying() {
        let r = rig().await;
        r.queue("external:orchestrator@m1", "external:advisor")
            .await;
        r.outbox.tick().await;
        r.step(60).await;
        assert!(r.notices.lock().expect("n").is_empty());

        r.skew_secs.store(31 * 60, Ordering::SeqCst);
        r.outbox.tick().await;
        r.step(600).await;
        r.step(600).await;
        let notices = r.notices.lock().expect("n").clone();
        assert_eq!(notices.len(), 1, "{notices:?}");
        assert_eq!(notices[0].0, "external:orchestrator");
        assert!(notices[0].1.contains("connection refused"), "{notices:?}");
        // Still queued, and still being tried.
        assert!(r.try_times().len() > 2, "{:?}", r.try_times());
        assert_eq!(r.store.outbox_queued_projects().await.expect("q"), ["beta"]);
    }

    #[tokio::test(start_paused = true)]
    async fn a_message_stuck_an_hour_is_reported_to_the_human_through_the_aide_once() {
        let r = rig().await;
        r.queue("external:orchestrator@m1", "external:advisor")
            .await;
        r.outbox.tick().await;
        r.skew_secs.store(61 * 60, Ordering::SeqCst);
        r.outbox.tick().await;
        r.step(600).await;
        r.outbox.tick().await;
        let aide: Vec<_> = r
            .notices
            .lock()
            .expect("n")
            .iter()
            .filter(|(to, _)| to == "external:aide")
            .cloned()
            .collect();
        assert_eq!(aide.len(), 1, "{aide:?}");
        assert!(aide[0].1.contains("For the human"), "{aide:?}");
    }
}
