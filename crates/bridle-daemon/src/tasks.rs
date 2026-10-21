//! Ticket records: the SQLite fast index (`crate::store`) and the state
//! branch (`crate::state_branch`) kept in step, plus an in-memory cache that
//! carries the body/thread the database doesn't (docs/design/storage.md).
//! Scoped to this build's states: `open`, `planned`, `claimed`, `dropped`,
//! `reopened`; `in_review`, `integrated` and `accepted` arrive with later
//! tasks.
//!
//! Edges (coordination.md) and `ready` (roles-and-lifecycle.md, "ready is
//! computed") live here too, on the same manager, per that design: an edge
//! is durable the same way a task is (database fast index + state branch,
//! written in the same logical operation), and `ready` needs both the task
//! cache and the edge cache to answer.
//!
//! Claims are durable too now: `claim_task`/`release_task` write to `Store`
//! *and* enqueue the full current claim set to the state branch's
//! `claims.toml`, so `bridle daemon rebuild` restores who's working what (storage.md,
//! "claims"). The claiming agent's own activity (the same signal
//! `supervisor.rs`'s stall check watches) still stands in for a lease
//! renewal, so [`TicketManager::tick_claim_lease_check`] can release a stale
//! claim without a separate heartbeat call.
//!
//! The queue (roles-and-lifecycle.md, "the queue") is a separate record this
//! manager also owns: an ordered list of tiers of task ids, PM-written,
//! durable on the state branch's `queue.toml` alone — there's no SQLite
//! table for it, so [`TicketManager::open`] hydrates the in-memory cache
//! straight from that file, the same way `rebuild_from_state_branch`
//! rebuilds the task/edge/open-question caches from their own files.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bridle_api::types::{
    AgentState, Edge, EdgeKind, Handover, Impact, MessageKind, MessageState, OpenQuestion,
    PrincipalId, ThreadEntry, ThreadEntryKind, Ticket, TicketKind, TicketPriority, TicketSize,
    TicketState, UNKNOWN_CREATOR, When,
};
use chrono::{DateTime, Utc};

/// Prefix of the thread note `skip_settle` records; `note_task` refuses it
/// from a plain note so nobody can forge a skip.
const SETTLE_SKIP_PREFIX: &str = "settle skipped by ";

/// The human's principal id; their comments and edits restart the settle clock.
const HUMAN: &str = "human";

use bridle_api::types::settle_clock_text;

/// A heuristic: an agent's thread entry that carries the human's words
/// ("From the human, via ..." or "human via <agent>", rule human-via-agent)
/// restarts the settle clock like the human's own entry. A plain text match.
fn is_relayed_human_word(body: &str) -> bool {
    let b = body.to_lowercase();
    b.starts_with("from the human") || b.contains("human via ")
}

/// Shape check only: `r-`/`s-`/`g-`/`a-` plus hex digits.
fn valid_spec_id(id: &str) -> bool {
    id.split_once('-').is_some_and(|(p, hex)| {
        matches!(p, "r" | "s" | "g" | "a")
            && !hex.is_empty()
            && hex.bytes().all(|b| b.is_ascii_hexdigit())
    })
}

use crate::state_branch::{ClaimRecord, StateBranch, StateBranchError};
use crate::store::{NewMessage, RecipientKind, Store, StoreError, TicketRow};

/// A claim's claimant and when it was made; the in-memory mirror of a
/// `claims` row (`Store::list_claims`).
type ClaimEntry = (PrincipalId, chrono::DateTime<Utc>);

#[derive(Debug, thiserror::Error)]
pub enum TicketError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<StoreError> for TicketError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::NotFound(m) => TicketError::NotFound(m),
            StoreError::Conflict(m) => TicketError::Conflict(m),
            other => TicketError::Internal(other.to_string()),
        }
    }
}

impl From<StateBranchError> for TicketError {
    fn from(e: StateBranchError) -> Self {
        TicketError::Internal(e.to_string())
    }
}

#[derive(Clone)]
pub struct TicketManager {
    store: Store,
    state: StateBranch,
    /// The id prefix new tasks get (storage.md: `<prefix>-<4 chars of the ticket alphabet>`).
    prefix: String,
    /// Full records, including body/thread, which the database doesn't
    /// carry. Hydrated from the state branch at [`TicketManager::open`], kept
    /// current on every write.
    cache: Arc<Mutex<HashMap<String, Ticket>>>,
    /// Every edge. Unlike `cache`, the database row *is* the full record
    /// (there's no extra body/thread to hydrate from the state branch), so
    /// this is loaded straight from `Store::list_edges` at `open`.
    edges: Arc<Mutex<Vec<Edge>>>,
    /// Ticket id -> the id of its open question's message, for the tasks that
    /// currently have one. Mirrors `Store::list_open_questions`, loaded at
    /// `open`, so `has_open_questions` (called from the sync `is_ready`) can
    /// answer without a database round trip.
    open_questions: Arc<Mutex<HashMap<String, String>>>,
    /// Ticket id -> (claimant, claimed at), for every currently claimed task.
    /// Mirrors `Store::list_claims`, loaded at `open`. Like
    /// `open_questions` it is a SQLite table, but the whole set is also
    /// mirrored to `claims.toml` on the state branch and restored by a
    /// rebuild (storage.md); this is also the source of
    /// `Ticket::claimed_by`/`claimed_at` on the cached task.
    claims: Arc<Mutex<HashMap<String, ClaimEntry>>>,
    /// How long a claim survives without the claiming agent's own activity
    /// before [`TicketManager::tick_claim_lease_check`] releases it.
    claim_lease_after: chrono::Duration,
    /// The queue: an ordered list of tiers, each a list of task ids
    /// (roles-and-lifecycle.md, "the queue"). Loaded from `queue.toml` at
    /// [`TicketManager::open`]; there's no SQLite table backing it, so this
    /// cache — kept current on every [`TicketManager::set_queue`] — is the
    /// only in-memory copy.
    queue: Arc<Mutex<Vec<Vec<String>>>>,
    /// The settle period (`[tasks] settle`, ny9u); zero turns it off. Set by
    /// [`TicketManager::with_settle`]; `open` leaves it off.
    settle: chrono::Duration,
    /// The project checkout's `docs/tickets`, to read a version 2 ticket's stages (br-fyww).
    /// Set by [`TicketManager::with_tickets_dir`]; `open` leaves it unset.
    tickets_dir: Option<std::path::PathBuf>,
}

/// Adds `who` unless already there; whether the list changed.
fn add_watcher(watchers: &mut Vec<PrincipalId>, who: &PrincipalId) -> bool {
    if watchers.contains(who) {
        return false;
    }
    watchers.push(who.clone());
    true
}

/// Who is told about a change to `task`: its watchers. The one place that answers it, so the
/// notification code doesn't re-derive it from the creator and claimer.
pub fn notified_of(task: &Ticket) -> Vec<PrincipalId> {
    task.watchers.clone()
}

impl TicketManager {
    /// The id prefix new tasks (and tickets) get.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Sets the settle period. Not in `open`'s arguments so every existing
    /// caller keeps today's behaviour until it opts in.
    pub fn with_settle(mut self, settle: std::time::Duration) -> Self {
        self.settle = chrono::Duration::from_std(settle).unwrap_or(chrono::Duration::zero());
        self
    }

    /// Sets the checkout's tickets folder, so `done` and `reopen` can tell a prepared version 2
    /// ticket from one still being prepared.
    pub fn with_tickets_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.tickets_dir = Some(dir);
        self
    }

    /// Why `id` is a version 2 ticket still being prepared, if it is.
    fn unprepared_v2(&self, id: &str) -> Option<String> {
        crate::stage_watch::unprepared(self.tickets_dir.as_deref()?, id)
    }

    /// Whether `id` is a version 2 ticket that is fully prepared (nothing to re-prepare).
    pub fn is_prepared_v2(&self, id: &str) -> bool {
        self.tickets_dir
            .as_deref()
            .is_some_and(|d| crate::stage_watch::is_prepared(d, id))
    }

    /// Loads every task the database knows about, hydrating each from its
    /// state-branch file. A task whose file is missing (a crash between
    /// `insert_task` and the first flush that would have written it) falls
    /// back to an empty body/thread built from the database row alone,
    /// rather than failing daemon startup over it.
    pub async fn open(
        store: Store,
        state: StateBranch,
        prefix: String,
        claim_lease_after: std::time::Duration,
    ) -> Result<Self, TicketError> {
        let rows = store.list_tasks().await?;
        let mut cache = HashMap::with_capacity(rows.len());
        let mut unrecorded = Vec::new();
        for row in rows {
            let row_state = row.state;
            let mut task = state.read_task(&row.id).unwrap_or(Ticket {
                id: row.id.clone(),
                title: row.title,
                kind: row.kind,
                state: row.state,
                body: String::new(),
                thread: Vec::new(),
                created_at: row.created_at,
                updated_at: row.updated_at,
                created_by: row
                    .created_by
                    .clone()
                    .unwrap_or_else(|| UNKNOWN_CREATOR.to_string()),
                watchers: Vec::new(),
                claimed_by: None,
                claimed_at: None,
                components: Vec::new(),
                size: None,
                priority: TicketPriority::default(),
                priority_at: None,
                branch: None,
                commit: None,
                summary: None,
                ticket: None,
                parent: None,
                impact: Impact::default(),
                settle_until: None,
                held_by: None,
                held_reason: None,
                held_until: None,
                held_at: None,
            });
            // The database state is written synchronously by every transition; the task
            // file lags it by up to a flush, so a restart in between must trust the row
            // (br-w6ww: the file still said `planned` for a claimed task, and the claims
            // sweep below dropped every agent's claim).
            task.state = row_state;
            // Migration (vk3y): the old `original id:` first body line becomes the `ticket`
            // field. Idempotent: once the line is gone there is nothing to move.
            if task.ticket.is_none()
                && let (Some(ticket), body) = split_legacy_origin(&task.body)
            {
                task.ticket = Some(ticket);
                task.body = body;
                if let Err(e) = state.enqueue_task(&task) {
                    tracing::warn!(task = %task.id, "moving the original id line to the ticket field: {e}");
                }
            }
            // The database (backfilled from events) wins; where it's unknown, the task file's
            // value (which may itself have fallen back to the thread) fills it in. Either
            // direction is best effort: it must not fail start-up.
            match &row.created_by {
                Some(by) if *by != task.created_by => {
                    task.created_by = by.clone();
                    if let Err(e) = state.enqueue_task(&task) {
                        tracing::warn!(task = %task.id, "recording created_by in the task file: {e}");
                    }
                }
                None if task.created_by != UNKNOWN_CREATOR => {
                    if let Err(e) = store.fill_task_created_by(&task.id, &task.created_by).await {
                        tracing::warn!(task = %task.id, "recording created_by: {e}");
                    }
                }
                _ => {}
            }
            // A task file with no watchers record (or none at all) is backfilled below, once
            // the claims are laid on.
            if !state.task_records_watchers(&task.id) {
                unrecorded.push(task.id.clone());
            }
            cache.insert(task.id.clone(), task);
        }
        let edges = store.list_edges().await?;
        let open_questions = store
            .list_open_questions()
            .await?
            .into_iter()
            .map(|q| (q.task_id, q.message_id))
            .collect();
        // A claim on a task that isn't `claimed` (say, left behind by an
        // older `drop`) is stale: drop the row rather than list the task as
        // claimed forever.
        let mut claims: HashMap<String, ClaimEntry> = HashMap::new();
        for c in store.list_claims().await? {
            if cache
                .get(&c.task_id)
                .is_some_and(|t| t.state == TicketState::Claimed)
            {
                claims.insert(c.task_id, (c.claimed_by, c.claimed_at));
            } else {
                store.delete_claim(&c.task_id).await?;
            }
        }
        for (task_id, (claimant, claimed_at)) in &claims {
            if let Some(task) = cache.get_mut(task_id) {
                task.claimed_by = Some(claimant.clone());
                task.claimed_at = Some(*claimed_at);
            }
        }
        // Watchers backfill: the creator and the current claimer. Best effort, like the
        // `created_by` fill above: a failure logs and leaves the list as it is.
        for id in unrecorded {
            if let Some(task) = cache.get_mut(&id) {
                if task.created_by != UNKNOWN_CREATOR {
                    add_watcher(&mut task.watchers, &task.created_by.clone());
                }
                if let Some(claimant) = task.claimed_by.clone() {
                    add_watcher(&mut task.watchers, &claimant);
                }
                if let Err(e) = state.enqueue_task(task) {
                    tracing::warn!(task = %id, "backfilling watchers: {e}");
                }
            }
        }
        let queue = state.read_queue();
        Ok(TicketManager {
            store,
            state,
            prefix,
            cache: Arc::new(Mutex::new(cache)),
            edges: Arc::new(Mutex::new(edges)),
            open_questions: Arc::new(Mutex::new(open_questions)),
            claims: Arc::new(Mutex::new(claims)),
            claim_lease_after: chrono::Duration::from_std(claim_lease_after)
                .unwrap_or_else(|_| chrono::Duration::zero()),
            queue: Arc::new(Mutex::new(queue)),
            settle: chrono::Duration::zero(),
            tickets_dir: None,
        })
    }

    /// Reconstructs `tasks`, `edges`, `open_questions` and `claims` from the
    /// state branch alone: the migration path for a fresh clone with no
    /// `bridle.db` (docs/design/overview.md, "`bridle daemon rebuild` recreates the
    /// database from the project's state branch"). Refuses — rather than
    /// silently overwriting — if the database already has any rows in these
    /// tables; a rebuild is a from-nothing reconstruction, not a merge. The
    /// queue cache is re-read from `queue.toml` too, the same as any other
    /// startup, since it never lived in SQLite to begin with (there's
    /// nothing to refuse-over for it).
    ///
    /// An open question's `message_id` in the rebuilt `open_questions` row
    /// doesn't point at a real `messages` row: messages don't survive on the
    /// state branch at all (overview.md's durability table lists them as
    /// SQLite-only, "no durability, by design"), only the thread entry that
    /// records the question's body/from/timestamp does. Rather than leave
    /// the column unfillable, this synthesizes a stand-in id from the task
    /// id; nothing downstream looks a message up by this id after a
    /// rebuild, since there's no message row behind it to find.
    pub async fn rebuild_from_state_branch(&self) -> Result<(), TicketError> {
        // Handover notes aren't in the refusal check below (a note alone is no reason to
        // refuse); `restore_handover` skips any already there.
        for h in self.state.list_handovers()? {
            self.store.restore_handover(&h).await?;
        }
        if !self.store.list_tasks().await?.is_empty()
            || !self.store.list_edges().await?.is_empty()
            || !self.store.list_open_questions().await?.is_empty()
            || !self.store.list_claims().await?.is_empty()
        {
            return Err(TicketError::Conflict(
                "database already has tasks, edges, open questions or claims; refusing to rebuild over it"
                    .to_string(),
            ));
        }

        let tasks = self.state.list_tasks()?;
        let mut cache = HashMap::with_capacity(tasks.len());
        let mut open_questions = HashMap::new();
        for task in tasks {
            self.store
                .insert_task_row(&TicketRow {
                    id: task.id.clone(),
                    title: task.title.clone(),
                    kind: task.kind,
                    state: task.state,
                    created_at: task.created_at,
                    updated_at: task.updated_at,
                    created_by: (task.created_by != UNKNOWN_CREATOR)
                        .then(|| task.created_by.clone()),
                })
                .await?;
            if let Some(entry) = task
                .thread
                .iter()
                .rev()
                .find(|e| matches!(e.kind, ThreadEntryKind::Question | ThreadEntryKind::Answer))
                && entry.kind == ThreadEntryKind::Question
            {
                let message_id = format!("m-rebuilt-{}", task.id);
                self.store
                    .insert_open_question(&task.id, &message_id, &entry.from, entry.at)
                    .await?;
                open_questions.insert(task.id.clone(), message_id);
            }
            cache.insert(task.id.clone(), task);
        }

        let edges = self.state.list_edges()?;
        for edge in &edges {
            self.store.insert_edge_row(edge).await?;
        }

        let claim_records = self.state.list_claims()?;
        let mut claims = HashMap::with_capacity(claim_records.len());
        for c in &claim_records {
            self.store
                .insert_claim(&c.task_id, &c.claimed_by, c.claimed_at)
                .await?;
            // `claim_task` sets `tasks.state = Claimed` synchronously in
            // SQLite, never on the state branch (storage.md, "claims"): the
            // task file this rebuild just restored the row from still says
            // whatever pre-claim state it was last flushed at (`planned`),
            // so this mirrors that same synchronous update here too.
            self.store
                .set_task_state(&c.task_id, TicketState::Claimed)
                .await?;
            if let Some(task) = cache.get_mut(&c.task_id) {
                task.state = TicketState::Claimed;
                task.claimed_by = Some(c.claimed_by.clone());
                task.claimed_at = Some(c.claimed_at);
            }
            claims.insert(c.task_id.clone(), (c.claimed_by.clone(), c.claimed_at));
        }

        *self.cache.lock().expect("task cache lock") = cache;
        *self.edges.lock().expect("edges lock") = edges;
        *self.open_questions.lock().expect("open questions lock") = open_questions;
        *self.claims.lock().expect("claims lock") = claims;
        *self.queue.lock().expect("queue lock") = self.state.read_queue();
        Ok(())
    }

    /// Queues a handover note for the state branch's next flush.
    pub fn enqueue_handover(&self, h: &Handover) -> Result<(), TicketError> {
        Ok(self.state.enqueue_handover(h)?)
    }

    /// Writes every note in SQLite that the state branch doesn't have yet (notes from before
    /// this existed). Idempotent: a note already on the branch is skipped.
    pub async fn backfill_handovers(&self) -> Result<(), TicketError> {
        for h in self.store.list_handovers().await? {
            if !self.state.has_handover(&h.id) {
                self.state.enqueue_handover(&h)?;
            }
        }
        Ok(())
    }

    /// `bridle daemon rebuild --from-origin`: see [`StateBranch::fetch_from_origin`].
    pub async fn fetch_state_from_origin(&self) -> crate::state_branch::FetchOutcome {
        StateBranch::fetch_from_origin(self.state.dir()).await
    }

    fn put(&self, task: Ticket) -> Ticket {
        self.cache
            .lock()
            .expect("task cache lock")
            .insert(task.id.clone(), task.clone());
        self.decorate(task)
    }

    /// Fills in the computed `settle_until`; the cache (and so the state
    /// branch) never holds it.
    fn decorate(&self, mut task: Ticket) -> Ticket {
        task.settle_until = self.settle_until(&task, Utc::now());
        task
    }

    /// When work on `task` (claim, plan, spawn) may start on the settle clock,
    /// if that's after `now`: `max(created_at, the human's latest thread
    /// entry) + settle`, unless a skip note is in the thread. The human's
    /// entries include relays of their words (see [`is_relayed_human_word`]).
    /// Any state, pending included (br-85hs). Computed from the thread alone,
    /// so old tasks need no migration (ny9u).
    pub fn settle_until(&self, task: &Ticket, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        if self.settle.is_zero() {
            return None;
        }
        if task
            .thread
            .iter()
            .any(|e| e.body.starts_with(SETTLE_SKIP_PREFIX))
        {
            return None;
        }
        let start = task
            .thread
            .iter()
            .filter(|e| e.from == HUMAN || is_relayed_human_word(&e.body))
            .map(|e| e.at)
            .fold(task.created_at, DateTime::max);
        Some(start + self.settle).filter(|until| *until > now)
    }

    /// Refuses with the time the hold ends if `id` is still settling: the
    /// check for work that isn't a claim (planning, spawning an agent for it).
    pub fn require_settled(&self, id: &str) -> Result<(), TicketError> {
        let task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        match task.settle_until {
            Some(until) => Err(TicketError::Conflict(format!(
                "task {id} is settling until {}",
                settle_clock_text(until)
            ))),
            None => Ok(()),
        }
    }

    /// Records that the settle period was skipped, so the task counts as
    /// settled. Who may is the caller's check; `reason` is required.
    pub async fn skip_settle(
        &self,
        id: &str,
        by: &PrincipalId,
        reason: &str,
    ) -> Result<Ticket, TicketError> {
        if reason.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "skipping the settle period requires a reason".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        let now = Utc::now();
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: by.clone(),
            body: format!("{SETTLE_SKIP_PREFIX}{by}: {}", reason.trim()),
            at: now,
            for_human: false,
        });
        task.updated_at = now;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// [`TicketManager::new_task_by`] with an unknown creator; for tests and callers that have no
    /// principal.
    pub async fn new_task(
        &self,
        title: &str,
        kind: TicketKind,
        body: String,
        components: Vec<String>,
        size: Option<TicketSize>,
    ) -> Result<Ticket, TicketError> {
        self.new_task_by(title, kind, body, components, size, UNKNOWN_CREATOR)
            .await
    }

    pub async fn new_task_by(
        &self,
        title: &str,
        kind: TicketKind,
        body: String,
        components: Vec<String>,
        size: Option<TicketSize>,
        created_by: &str,
    ) -> Result<Ticket, TicketError> {
        self.new_task_for_ticket(title, kind, body, components, size, created_by, None)
            .await
    }

    /// [`TicketManager::new_task_by`] for a task made from `ticket`; the task's id takes the
    /// ticket's id when free. An older client that sends the `original id:` first body line
    /// instead of `ticket` is read the same way (the line leaves the body).
    #[allow(clippy::too_many_arguments)]
    pub async fn new_task_for_ticket(
        &self,
        title: &str,
        kind: TicketKind,
        body: String,
        components: Vec<String>,
        size: Option<TicketSize>,
        created_by: &str,
        ticket: Option<String>,
    ) -> Result<Ticket, TicketError> {
        if title.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "title must not be empty".to_string(),
            ));
        }
        let (legacy, body) = split_legacy_origin(&body);
        let ticket = ticket
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .or(legacy)
            // A qualified id (`br-vdu8`) names the same ticket as the bare one, and the row id
            // is `<prefix>-<ticket>`: strip our own prefix so it isn't doubled. Another
            // project's prefix is kept as given; it is not ours to rewrite.
            .map(|t| {
                match t
                    .strip_prefix(self.prefix.as_str())
                    .and_then(|r| r.strip_prefix('-'))
                {
                    Some(bare) if !bare.is_empty() => bare.to_string(),
                    _ => t,
                }
            });
        let ticket_ref = ticket.as_deref();
        let row = self
            .store
            .insert_task_wanting(&self.prefix, title, kind, created_by, ticket_ref)
            .await?;
        let task = Ticket {
            id: row.id,
            title: row.title,
            kind: row.kind,
            state: row.state,
            body,
            thread: Vec::new(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row
                .created_by
                .unwrap_or_else(|| UNKNOWN_CREATOR.to_string()),
            watchers: if created_by == UNKNOWN_CREATOR {
                Vec::new()
            } else {
                vec![created_by.to_string()]
            },
            claimed_by: None,
            claimed_at: None,
            components,
            size,
            priority: TicketPriority::default(),
            priority_at: None,
            branch: None,
            commit: None,
            summary: None,
            ticket,
            parent: None,
            impact: Impact::default(),
            settle_until: None,
            held_by: None,
            held_reason: None,
            held_until: None,
            held_at: None,
        };
        self.state.enqueue_task(&task)?;
        self.state
            .enqueue_event(&task.id, "", task.state.as_str(), "human", task.created_at);
        Ok(self.put(task))
    }

    /// Makes `child` a task split from `parent` (`task new --from`): records the link on the
    /// child, gives it the parent's watchers (the creator, already a watcher, stays), and notes
    /// the child on the parent's thread. Fails `NotFound` for an unknown parent, before the
    /// child is touched; call [`TicketManager::get_task`] on the parent first to fail early.
    pub async fn split_from(
        &self,
        child: &str,
        parent: &str,
        by: &PrincipalId,
    ) -> Result<Ticket, TicketError> {
        let mut parent_task = self
            .get_task(parent)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {parent}")))?;
        let mut task = self
            .get_task(child)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {child}")))?;
        let now = Utc::now();
        for w in &parent_task.watchers {
            add_watcher(&mut task.watchers, w);
        }
        task.parent = Some(parent.to_string());
        task.updated_at = now;
        self.state.enqueue_task(&task)?;
        parent_task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: by.clone(),
            body: format!("split off {child}: {}", task.title),
            at: now,
            for_human: false,
        });
        parent_task.updated_at = now;
        self.state.enqueue_task(&parent_task)?;
        self.put(parent_task);
        Ok(self.put(task))
    }

    /// Sets the priority, recording who and when in the thread. The caller emits the event.
    pub async fn set_priority(
        &self,
        id: &str,
        priority: TicketPriority,
        actor: &PrincipalId,
    ) -> Result<(Ticket, TicketPriority), TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        let from = task.priority;
        if from == priority {
            return Ok((task, from));
        }
        task.priority = priority;
        task.updated_at = Utc::now();
        task.priority_at = Some(task.updated_at);
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: format!("priority: {from} -> {priority}"),
            at: task.updated_at,
            for_human: false,
        });
        self.state.enqueue_task(&task)?;
        Ok((self.put(task), from))
    }

    /// Sets the kind, only while the task is `pending` or `open` (a planned task's kind is frozen, and a
    /// reopened one was planned before). The caller emits the event.
    pub async fn set_kind(
        &self,
        id: &str,
        kind: TicketKind,
        actor: &PrincipalId,
    ) -> Result<(Ticket, TicketKind), TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if !matches!(task.state, TicketState::Pending | TicketState::Open) {
            return Err(TicketError::Conflict(format!(
                "{id} is {}; a task's kind can only change while it is pending or open",
                task.state
            )));
        }
        let from = task.kind;
        if from == kind {
            return Ok((task, from));
        }
        self.store.set_task_kind(id, kind).await?;
        task.kind = kind;
        task.updated_at = Utc::now();
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: format!("kind: {from} -> {kind}"),
            at: task.updated_at,
            for_human: false,
        });
        self.state.enqueue_task(&task)?;
        Ok((self.put(task), from))
    }

    /// Records the task's creation in its thread (who, when, priority); used for human to-dos.
    pub async fn note_created(
        &self,
        id: &str,
        priority: TicketPriority,
        actor: &PrincipalId,
    ) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        task.priority = priority;
        task.priority_at = Some(Utc::now());
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: format!("created for the human, priority {priority}"),
            at: Utc::now(),
            for_human: false,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    pub fn get_task(&self, id: &str) -> Option<Ticket> {
        let task = self.cache.lock().expect("task cache lock").get(id).cloned();
        task.map(|t| self.decorate(t))
    }

    pub fn list_tasks(&self) -> Vec<Ticket> {
        let cache = self.cache.lock().expect("task cache lock");
        let mut tasks: Vec<Ticket> = cache.values().cloned().map(|t| self.decorate(t)).collect();
        tasks.sort_by_key(|t| t.created_at);
        tasks
    }

    /// Sets the task's ticket link; no thread note, it is metadata.
    pub async fn set_ticket(&self, id: &str, ticket: &str) -> Result<Ticket, TicketError> {
        let ticket = ticket.trim();
        if ticket.is_empty() {
            return Err(TicketError::BadRequest(
                "ticket must not be empty".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if task.ticket.as_deref() == Some(ticket) {
            return Ok(task);
        }
        task.ticket = Some(ticket.to_string());
        task.updated_at = Utc::now();
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Changes `title`, `body`, `components` and/or `size`. Neither changes the task's state.
    /// A human edit of the title or body is recorded as a thread note, which
    /// restarts the settle clock.
    pub async fn edit_task(
        &self,
        id: &str,
        actor: &PrincipalId,
        title: Option<String>,
        body: Option<String>,
        components: Option<Vec<String>>,
        size: Option<TicketSize>,
    ) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if let Some(title) = &title
            && title.trim().is_empty()
        {
            return Err(TicketError::BadRequest(
                "title must not be empty".to_string(),
            ));
        }
        if title.is_none() && body.is_none() && components.is_none() && size.is_none() {
            return Ok(task);
        }
        if actor == HUMAN && (title.is_some() || body.is_some()) {
            task.thread.push(ThreadEntry {
                kind: ThreadEntryKind::Note,
                from: actor.clone(),
                body: "edited the title or body".to_string(),
                at: Utc::now(),
                for_human: false,
            });
        }
        if let Some(title) = title {
            self.store.set_task_title(id, &title).await?;
            task.title = title;
        }
        if let Some(body) = body {
            task.body = body;
        }
        if let Some(components) = components {
            task.components = components;
        }
        if let Some(size) = size {
            task.size = if size == TicketSize::None {
                None
            } else {
                Some(size)
            };
        }
        task.updated_at = Utc::now();
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `pending` -> `open`: someone with the authority (the human's approval, or the manager's
    /// own small fix) has said this task is ready for the PM. Every task is created `pending`.
    pub async fn ready_task(&self, id: &str, actor: &PrincipalId) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if task.state != TicketState::Pending {
            return Err(TicketError::Conflict(format!(
                "task {id} is {}, not pending; only a pending task can be readied",
                task.state
            )));
        }
        self.transition(&mut task, TicketState::Open, actor).await?;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `open` -> `pending`, for a task nobody planned in time (xz4f): the daemon's own
    /// correction, so it says why in the thread. Skips a task that has gained an open question
    /// or has left `open` since the caller looked.
    pub async fn unready_stale(
        &self,
        id: &str,
        actor: &PrincipalId,
        reason: &str,
    ) -> Result<Option<Ticket>, TicketError> {
        let Some(mut task) = self.get_task(id) else {
            return Ok(None);
        };
        if task.state != TicketState::Open || self.has_open_questions(&task) {
            return Ok(None);
        }
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: reason.to_string(),
            at: Utc::now(),
            for_human: false,
        });
        self.transition(&mut task, TicketState::Pending, actor)
            .await?;
        self.state.enqueue_task(&task)?;
        Ok(Some(self.put(task)))
    }

    /// [`TicketManager::new_task`] then [`TicketManager::ready_task`], for tests that start from
    /// an `open` task.
    pub async fn new_open_task(
        &self,
        title: &str,
        kind: TicketKind,
        body: String,
        components: Vec<String>,
        size: Option<TicketSize>,
    ) -> Result<Ticket, TicketError> {
        let task = self.new_task(title, kind, body, components, size).await?;
        self.ready_task(&task.id, &UNKNOWN_CREATOR.to_string())
            .await
    }

    /// `open` or `reopened` -> `planned`: the PM has decided this task is ready to
    /// build. Fails with `Conflict` from any other state (already `planned`,
    /// `claimed`, `dropped`, ...).
    pub async fn plan_task(&self, id: &str, actor: &PrincipalId) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        // `reopened` is a way back into the queue for every kind (an incident that
        // recurred goes active again the same way); it must not be a dead end.
        if !matches!(task.state, TicketState::Open | TicketState::Reopened) {
            return Err(TicketError::Conflict(format!(
                "task {id} is {}, not open or reopened; only an open or reopened task can be planned",
                task.state
            )));
        }
        self.transition(&mut task, TicketState::Planned, actor)
            .await?;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `open`, `planned` or `claimed` -> `dropped` (releasing any claim), recording `reason` in the thread.
    pub async fn drop_task(
        &self,
        id: &str,
        reason: &str,
        actor: &PrincipalId,
    ) -> Result<Ticket, TicketError> {
        if reason.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "dropping a task requires a reason".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if task.state == TicketState::Dropped {
            return Err(TicketError::Conflict(format!(
                "task {id} is already dropped"
            )));
        }
        if task.state == TicketState::Claimed {
            self.end_claim(id, TicketState::Dropped).await?;
            task.claimed_by = None;
            task.claimed_at = None;
        }
        self.transition(&mut task, TicketState::Dropped, actor)
            .await?;
        clear_held(&mut task);
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: format!("dropped: {reason}"),
            at: task.updated_at,
            for_human: false,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `open`, `planned` or `claimed` -> `integrated`, recording the merge
    /// `commit` in the thread. A claim is released first. `blocks` edges
    /// out of the task resolve, and it leaves the queue and `ready`.
    pub async fn done_task(
        &self,
        id: &str,
        commit: &str,
        branch: Option<&str>,
        actor: &PrincipalId,
    ) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        // A human to-do has no code to land, so the human finishes it bare.
        let bare_ok = (task.state == TicketState::Claimed
            && task.claimed_by.as_deref() == Some("human"))
            || task.kind == TicketKind::Incident;
        if commit.trim().is_empty() && !bare_ok {
            return Err(TicketError::BadRequest(
                "marking a task done requires a commit".to_string(),
            ));
        }
        // Landing a stage branch does not finish the ticket (br-fyww).
        if let Some(why) = self.unprepared_v2(id) {
            return Err(TicketError::Conflict(format!(
                "{id} is a version 2 ticket still being prepared ({why}); landing a stage does not finish it"
            )));
        }
        match task.state {
            TicketState::Dropped | TicketState::Integrated => {
                return Err(TicketError::Conflict(format!(
                    "task {id} is already {}",
                    task.state
                )));
            }
            // A held task is paused by a person: unhold it first.
            TicketState::Held => {
                return Err(TicketError::Conflict(format!(
                    "task {id} is held; unhold it before marking it done"
                )));
            }
            // Tolerates a missing claims row (br-w6ww), so a task already stuck can close.
            TicketState::Claimed => {
                self.end_claim(id, TicketState::Integrated).await?;
                task.claimed_by = None;
                task.claimed_at = None;
            }
            _ => {}
        }
        self.transition(&mut task, TicketState::Integrated, actor)
            .await?;
        let branch = branch.map(str::trim).filter(|b| !b.is_empty());
        let commit = commit.trim();
        task.commit = (!commit.is_empty()).then(|| commit.to_string());
        task.branch = branch.map(str::to_string);
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: match (commit.is_empty(), branch) {
                (true, _) => "done".to_string(),
                (false, Some(b)) => format!("integrated: {commit} (branch {b})"),
                (false, None) => format!("integrated: {commit}"),
            },
            for_human: false,
            at: task.updated_at,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Records how the task was implemented, replacing any earlier summary.
    /// Allowed in any state: a worker writes it before the manager marks
    /// the task done.
    pub async fn set_summary(&self, id: &str, text: &str) -> Result<Ticket, TicketError> {
        if text.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "summary must not be empty".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        task.summary = Some(text.trim().to_string());
        task.updated_at = Utc::now();
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Replaces the task's declared impact (impact-and-conflicts.md). Only
    /// while the task can still be worked: pending, open, planned or claimed.
    pub async fn set_impact(&self, id: &str, impact: Impact) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if !matches!(
            task.state,
            TicketState::Pending | TicketState::Open | TicketState::Planned | TicketState::Claimed
        ) {
            return Err(TicketError::Conflict(format!(
                "task {id} is {}; impact can only be set on a pending, open, planned or claimed task",
                task.state
            )));
        }
        for id in impact
            .modify
            .iter()
            .chain(&impact.add_under)
            .chain(&impact.remove)
        {
            if !valid_spec_id(id) {
                return Err(TicketError::BadRequest(format!(
                    "bad spec id {id:?}: expected r-, s-, g- or a- plus hex digits"
                )));
            }
        }
        if impact.files.iter().any(|f| f.trim().is_empty()) {
            return Err(TicketError::BadRequest(
                "file globs must not be empty".to_string(),
            ));
        }
        task.impact = impact;
        task.updated_at = Utc::now();
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `dropped` or `integrated` -> `reopened`, or -> `pending` for a version 2 ticket still
    /// being prepared, so the stage watch writes its next stage (br-fyww). Any other starting
    /// state is a conflict.
    pub async fn reopen_task(&self, id: &str, actor: &PrincipalId) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if !matches!(task.state, TicketState::Dropped | TicketState::Integrated) {
            return Err(TicketError::Conflict(format!(
                "task {id} is {}; only a dropped or integrated task can be reopened",
                task.state
            )));
        }
        let to = if self.unprepared_v2(id).is_some() {
            TicketState::Pending
        } else {
            TicketState::Reopened
        };
        self.transition(&mut task, to, actor).await?;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `planned` -> `held`: a person pauses the task. Not claimable or startable until
    /// `unhold_task`. The reason is required and goes in the thread.
    pub async fn hold_task(
        &self,
        id: &str,
        reason: &str,
        until: Option<&str>,
        actor: &PrincipalId,
    ) -> Result<Ticket, TicketError> {
        if reason.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "holding a task requires a reason".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if task.state != TicketState::Planned {
            return Err(TicketError::Conflict(format!(
                "task {id} is {}; only a planned task can be held (release a claim first)",
                task.state
            )));
        }
        self.transition(&mut task, TicketState::Held, actor).await?;
        let until = until.map(str::trim).filter(|u| !u.is_empty());
        task.held_by = Some(actor.clone());
        task.held_reason = Some(reason.trim().to_string());
        task.held_until = until.map(str::to_string);
        task.held_at = Some(task.updated_at);
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: match until {
                Some(u) => format!("held: {} (until {u})", reason.trim()),
                None => format!("held: {}", reason.trim()),
            },
            at: task.updated_at,
            for_human: false,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `held` -> `planned`.
    pub async fn unhold_task(&self, id: &str, actor: &PrincipalId) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if task.state != TicketState::Held {
            return Err(TicketError::Conflict(format!(
                "task {id} is {}; only a held task can be unheld",
                task.state
            )));
        }
        self.transition(&mut task, TicketState::Planned, actor)
            .await?;
        clear_held(&mut task);
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: "unheld".to_string(),
            at: task.updated_at,
            for_human: false,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `claimed` or `planned` -> `built`: the work is finished on `branch` at `commit` and
    /// waits to land. A claim is released. Only `done_task` or `send_back_task` leave it.
    pub async fn built_task(
        &self,
        id: &str,
        branch: &str,
        commit: &str,
        actor: &PrincipalId,
    ) -> Result<Ticket, TicketError> {
        let (branch, commit) = (branch.trim(), commit.trim());
        if branch.is_empty() || commit.is_empty() {
            return Err(TicketError::BadRequest(
                "marking a task built requires a branch and a commit".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        match task.state {
            TicketState::Planned => {}
            TicketState::Claimed => {
                self.end_claim(id, TicketState::Built).await?;
                task.claimed_by = None;
                task.claimed_at = None;
            }
            other => {
                return Err(TicketError::Conflict(format!(
                    "task {id} is {other}; only a claimed or planned task can be built"
                )));
            }
        }
        self.transition(&mut task, TicketState::Built, actor)
            .await?;
        task.branch = Some(branch.to_string());
        task.commit = Some(commit.to_string());
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: format!("built: {commit} (branch {branch})"),
            at: task.updated_at,
            for_human: false,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// `built` -> `planned`, keeping `branch` for the next worker. Records
    /// `reason` in the thread.
    pub async fn send_back_task(
        &self,
        id: &str,
        reason: &str,
        actor: &PrincipalId,
    ) -> Result<Ticket, TicketError> {
        if reason.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "sending a task back requires a reason".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if task.state != TicketState::Built {
            return Err(TicketError::Conflict(format!(
                "task {id} is {}; only a built task can be sent back",
                task.state
            )));
        }
        self.transition(&mut task, TicketState::Planned, actor)
            .await?;
        // The branch stays for the next worker; the commit goes, or the start-up migration
        // would take this planned task for finished work again.
        task.commit = None;
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: actor.clone(),
            body: format!("sent back: {}", reason.trim()),
            at: task.updated_at,
            for_human: false,
        });
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Migration (stx8): a `planned` task with a branch and commit is finished work waiting to
    /// land, so it becomes `built`. Idempotent: once moved, nothing matches. Run at start-up.
    pub async fn migrate_planned_with_branch_to_built(&self) -> Result<Vec<String>, TicketError> {
        let ids: Vec<String> = self
            .list_tasks()
            .into_iter()
            .filter(|t| t.state == TicketState::Planned && t.branch.is_some() && t.commit.is_some())
            .map(|t| t.id)
            .collect();
        for id in &ids {
            let mut task = self.get_task(id).expect("listed task is cached");
            self.store.set_task_state(id, TicketState::Built).await?;
            task.state = TicketState::Built;
            task.updated_at = Utc::now();
            self.state.enqueue_task(&task)?;
            self.put(task);
        }
        Ok(ids)
    }

    async fn transition(
        &self,
        task: &mut Ticket,
        to: TicketState,
        actor: &PrincipalId,
    ) -> Result<(), TicketError> {
        let from = task.state;
        self.store.set_task_state(&task.id, to).await?;
        task.state = to;
        task.updated_at = Utc::now();
        self.state
            .enqueue_event(&task.id, from.as_str(), to.as_str(), actor, task.updated_at);
        Ok(())
    }

    // ---------- edges ----------

    /// Adds a `from`-`kind`->`to` edge. Both tasks must exist, and an edge
    /// can't connect a task to itself; the underlying unique constraint
    /// (coordination.md's table has no notion of a duplicate edge) turns a
    /// repeat of the same triple into a conflict.
    pub async fn add_edge(
        &self,
        from: &str,
        to: &str,
        kind: EdgeKind,
    ) -> Result<Edge, TicketError> {
        if from == to {
            return Err(TicketError::BadRequest(
                "an edge cannot connect a task to itself".to_string(),
            ));
        }
        if self.get_task(from).is_none() {
            return Err(TicketError::NotFound(format!("no such task: {from}")));
        }
        if self.get_task(to).is_none() {
            return Err(TicketError::NotFound(format!("no such task: {to}")));
        }
        let edge = self.store.insert_edge(from, to, kind).await?;
        let edges = {
            let mut g = self.edges.lock().expect("edges lock");
            g.push(edge.clone());
            g.clone()
        };
        self.state.enqueue_edges(&edges)?;
        Ok(edge)
    }

    /// Removes the edge identified by the `(from, to, kind)` triple; not
    /// found if no such edge exists.
    pub async fn remove_edge(
        &self,
        from: &str,
        to: &str,
        kind: EdgeKind,
    ) -> Result<(), TicketError> {
        self.store.delete_edge(from, to, kind).await?;
        let edges = {
            let mut g = self.edges.lock().expect("edges lock");
            g.retain(|e| !(e.from == from && e.to == to && e.kind == kind));
            g.clone()
        };
        self.state.enqueue_edges(&edges)?;
        Ok(())
    }

    pub fn list_edges(&self) -> Vec<Edge> {
        self.edges.lock().expect("edges lock").clone()
    }

    // ---------- ready ----------

    /// A blocker is unresolved unless it's `dropped` or `integrated`:
    /// `accepted` doesn't exist yet, so for now anything else — including a
    /// blocker this manager doesn't know about — still counts as blocking
    /// (roles-and-lifecycle.md, "ready is computed"). Documented here rather
    /// than left implicit, since it's a deliberate simplification this
    /// build makes, not the final rule.
    fn blocker_is_resolved(&self, blocker_id: &str) -> bool {
        self.get_task(blocker_id)
            .is_some_and(|b| matches!(b.state, TicketState::Dropped | TicketState::Integrated))
    }

    /// `planned`, no open `blocks` edge naming an unresolved blocker, no
    /// unanswered question, and settled.
    pub fn is_ready(&self, task: &Ticket) -> bool {
        self.is_ready_unsettled(task) && self.settle_until(task, Utc::now()).is_none()
    }

    /// [`TicketManager::is_ready`] ignoring the settle period.
    pub(crate) fn is_ready_unsettled(&self, task: &Ticket) -> bool {
        // An incident is nobody's to build or claim (incidents.md).
        if task.state != TicketState::Planned || task.kind == TicketKind::Incident {
            return false;
        }
        if self.has_open_questions(task) {
            return false;
        }
        let edges = self.edges.lock().expect("edges lock");
        !edges.iter().any(|e| {
            e.kind == EdgeKind::Blocks && e.to == task.id && !self.blocker_is_resolved(&e.from)
        })
    }

    /// `true` while `task` has an unanswered question (docs/design/coordination.md,
    /// "Questions do not stop work").
    fn has_open_questions(&self, task: &Ticket) -> bool {
        self.open_questions
            .lock()
            .expect("open questions lock")
            .contains_key(&task.id)
    }

    // ---------- questions ----------

    /// Asks a question against `task`: appends a `question` thread entry,
    /// inserts the durable message (`to_kind` `task`, so it's addressed to
    /// the task rather than an agent or `human`), and indexes the task as
    /// blocked until answered. Fails with `Conflict` if the task already has
    /// an open question — one at a time, per `open_questions`'s primary key.
    pub async fn ask_question(
        &self,
        id: &str,
        from: &PrincipalId,
        body: &str,
    ) -> Result<Ticket, TicketError> {
        if body.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "a question requires a body".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        // Checked here too, not just left to the store's unique-constraint
        // conflict below: this cache is the source `is_ready` reads, so a
        // question already recorded here must not have its store insert
        // half-succeed while this half is skipped.
        if self.has_open_questions(&task) {
            return Err(TicketError::Conflict(format!(
                "task {id} already has an open question"
            )));
        }
        let message = self
            .store
            .insert_message(NewMessage {
                from: from.clone(),
                to: id.to_string(),
                to_kind: RecipientKind::Ticket,
                kind: MessageKind::Question,
                body: body.to_string(),
                reply_to: None,
                when: When::Now,
                // Delivery here means "durably attached to the task", which
                // already happened synchronously (this message *is* the
                // thread entry below); there's no agent stdin to write to
                // and no further transition this state machine models.
                state: MessageState::Delivered,
            })
            .await?;
        self.store
            .insert_open_question(id, &message.id, from, message.created_at)
            .await?;
        self.open_questions
            .lock()
            .expect("open questions lock")
            .insert(id.to_string(), message.id.clone());
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Question,
            from: from.clone(),
            body: body.to_string(),
            at: message.created_at,
            for_human: false,
        });
        task.updated_at = message.created_at;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Answers `task`'s open question: appends an `answer` thread entry,
    /// clears the open-questions index, and re-enables readiness. Fails
    /// with `Conflict` if the task has no open question.
    pub async fn answer_question(
        &self,
        id: &str,
        from: &PrincipalId,
        body: &str,
    ) -> Result<Ticket, TicketError> {
        if body.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "an answer requires a body".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        let question_message_id = self
            .open_questions
            .lock()
            .expect("open questions lock")
            .get(id)
            .cloned()
            .ok_or_else(|| TicketError::Conflict(format!("task {id} has no open question")))?;
        let message = self
            .store
            .insert_message(NewMessage {
                from: from.clone(),
                to: id.to_string(),
                to_kind: RecipientKind::Ticket,
                kind: MessageKind::Answer,
                body: body.to_string(),
                reply_to: Some(question_message_id),
                when: When::Now,
                state: MessageState::Delivered,
            })
            .await?;
        self.store.delete_open_question(id).await?;
        self.open_questions
            .lock()
            .expect("open questions lock")
            .remove(id);
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Answer,
            from: from.clone(),
            body: body.to_string(),
            at: message.created_at,
            for_human: false,
        });
        task.updated_at = message.created_at;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Adds a plain note to `task`'s thread: no open-question bookkeeping,
    /// no effect on readiness (docs/design/coordination.md, Messages). The
    /// smallest useful message-to-a-task primitive; `ask_question` builds
    /// the blocking variant on the same shape.
    pub async fn note_task(
        &self,
        id: &str,
        from: &PrincipalId,
        body: &str,
    ) -> Result<Ticket, TicketError> {
        self.note_task_as(id, from, body, false).await
    }

    /// [`Self::note_task`]; `for_human` marks the note as the human's words relayed by `from`
    /// (br-ggbx). The caller has checked that `from` may relay.
    pub async fn note_task_as(
        &self,
        id: &str,
        from: &PrincipalId,
        body: &str,
        for_human: bool,
    ) -> Result<Ticket, TicketError> {
        if body.trim().is_empty() {
            return Err(TicketError::BadRequest(
                "a note requires a body".to_string(),
            ));
        }
        if body.starts_with(SETTLE_SKIP_PREFIX) {
            return Err(TicketError::BadRequest(
                "that note is reserved for `task skip-settle`".to_string(),
            ));
        }
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        let message = self
            .store
            .insert_message(NewMessage {
                from: from.clone(),
                to: id.to_string(),
                to_kind: RecipientKind::Ticket,
                kind: MessageKind::Note,
                body: body.to_string(),
                reply_to: None,
                when: When::Now,
                state: MessageState::Delivered,
            })
            .await?;
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: from.clone(),
            body: body.to_string(),
            at: message.created_at,
            for_human,
        });
        task.updated_at = message.created_at;
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    // ---------- watchers ----------

    /// Adds (`watch`) or removes `who` from the task's watchers, recorded in its thread.
    /// Idempotent: a change that is already true is a no-op, with no thread entry. Returns the
    /// task and whether anything changed.
    pub async fn set_watching(
        &self,
        id: &str,
        who: &PrincipalId,
        watch: bool,
    ) -> Result<(Ticket, bool), TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        let changed = if watch {
            add_watcher(&mut task.watchers, who)
        } else {
            let before = task.watchers.len();
            task.watchers.retain(|w| w != who);
            task.watchers.len() != before
        };
        if !changed {
            return Ok((task, false));
        }
        let now = Utc::now();
        task.thread.push(ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: who.clone(),
            body: format!(
                "{} the task",
                if watch {
                    "watching"
                } else {
                    "stopped watching"
                }
            ),
            for_human: false,
            at: now,
        });
        task.updated_at = now;
        self.state.enqueue_task(&task)?;
        Ok((self.put(task), true))
    }

    // ---------- claims ----------

    /// Enqueues the full current claim set to the state branch's
    /// `claims.toml`, mirroring [`TicketManager::claims`] the way
    /// `add_edge`/`remove_edge` mirror the edge cache to `edges.toml`. Called
    /// with the claims lock already released, after every claim/release, so
    /// `bridle daemon rebuild` can restore current claims (storage.md, "claims").
    fn enqueue_claims_snapshot(&self) -> Result<(), TicketError> {
        let records: Vec<ClaimRecord> = self
            .claims
            .lock()
            .expect("claims lock")
            .iter()
            .map(|(task_id, (claimed_by, claimed_at))| ClaimRecord {
                task_id: task_id.clone(),
                claimed_by: claimed_by.clone(),
                claimed_at: *claimed_at,
            })
            .collect();
        Ok(self.state.enqueue_claims(&records)?)
    }

    /// Claims `id` for `by`: `planned` -> `claimed`. Fails with `Conflict`
    /// if the task isn't ready right now — not planned, blocked, or already
    /// claimed (claiming again would need it to still be `planned`, which
    /// `is_ready` already requires). Writes synchronously to the database,
    /// like every other task state change, and enqueues the claim set's new
    /// state to the state branch for the next flush (storage.md, "claims").
    pub async fn claim_task(&self, id: &str, by: &PrincipalId) -> Result<Ticket, TicketError> {
        self.claim_inner(id, by, true).await
    }

    /// [`TicketManager::claim_task`] without the settle check: for the human's
    /// own to-dos, which are created already claimed.
    pub async fn claim_task_unsettled(
        &self,
        id: &str,
        by: &PrincipalId,
    ) -> Result<Ticket, TicketError> {
        self.claim_inner(id, by, false).await
    }

    async fn claim_inner(
        &self,
        id: &str,
        by: &PrincipalId,
        must_be_settled: bool,
    ) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if !self.is_ready_unsettled(&task) {
            return Err(TicketError::Conflict(format!(
                "task {id} is not ready to claim"
            )));
        }
        if must_be_settled && let Some(until) = task.settle_until {
            return Err(TicketError::Conflict(format!(
                "task {id} is settling until {}",
                settle_clock_text(until)
            )));
        }
        let now = Utc::now();
        self.store.insert_claim(id, by, now).await?;
        self.store.set_task_state(id, TicketState::Claimed).await?;
        self.claims
            .lock()
            .expect("claims lock")
            .insert(id.to_string(), (by.clone(), now));
        self.enqueue_claims_snapshot()?;
        task.state = TicketState::Claimed;
        task.updated_at = now;
        task.claimed_by = Some(by.clone());
        task.claimed_at = Some(now);
        add_watcher(&mut task.watchers, by);
        self.state.enqueue_task(&task)?;
        Ok(self.put(task))
    }

    /// Releases `id`'s claim: `claimed` -> `planned`. Fails with `Conflict`
    /// if `by` isn't the current claimant (including if the task isn't
    /// claimed at all).
    pub async fn release_task(&self, id: &str, by: &PrincipalId) -> Result<Ticket, TicketError> {
        {
            let claims = self.claims.lock().expect("claims lock");
            match claims.get(id) {
                Some((claimant, _)) if claimant == by => {}
                Some(_) => {
                    return Err(TicketError::Conflict(format!(
                        "task {id} is claimed by another agent"
                    )));
                }
                None => {
                    return Err(TicketError::Conflict(format!("task {id} is not claimed")));
                }
            }
        }
        self.release_claim(id).await
    }

    /// Removes `id`'s claim row and moves the task to `to` in one database
    /// transaction (a stop between the two writes once left a task `claimed`
    /// with no claim, br-w6ww), then drops the in-memory entry and the
    /// state-branch record. A missing claims row is not an error.
    async fn end_claim(&self, id: &str, to: TicketState) -> Result<(), TicketError> {
        self.store.end_claim(id, to).await?;
        self.claims.lock().expect("claims lock").remove(id);
        self.enqueue_claims_snapshot()
    }

    /// The shared release path for an explicit `release_task` and automatic
    /// lease expiry: clears the claim and transitions the task back to
    /// `planned`, enqueuing the claim set's new state to the state branch
    /// the same way [`TicketManager::claim_task`] does.
    async fn release_claim(&self, id: &str) -> Result<Ticket, TicketError> {
        let mut task = self
            .get_task(id)
            .ok_or_else(|| TicketError::NotFound(format!("no such task: {id}")))?;
        if task.state != TicketState::Claimed {
            return Err(TicketError::Conflict(format!("task {id} is not claimed")));
        }
        self.end_claim(id, TicketState::Planned).await?;
        task.state = TicketState::Planned;
        task.updated_at = Utc::now();
        task.claimed_by = None;
        task.claimed_at = None;
        Ok(self.put(task))
    }

    /// Releases any claim whose claiming agent has gone quiet past
    /// `claim_lease_after`, using the same activity signal the supervisor's
    /// stall check watches (`last_event_at.or(turn_started_at)`,
    /// `supervisor.rs::tick_stall_check`) rather than a separate renewal
    /// call: a claim is alive as long as its claimant is. `now` is passed in
    /// rather than read live, so tests can inject a stale claimant without
    /// waiting out the real lease.
    pub async fn tick_claim_lease_check(&self, now: chrono::DateTime<Utc>) {
        let claimed: Vec<(String, PrincipalId)> = self
            .claims
            .lock()
            .expect("claims lock")
            .iter()
            .map(|(id, (by, _))| (id.clone(), by.clone()))
            .collect();
        for (task_id, claimant) in claimed {
            // Agent principals are `agent:<name>` (principals.md); `agents`
            // rows are keyed by id/name, not this prefixed form (mirrors the
            // same lookup in server.rs).
            let name = claimant.strip_prefix("agent:").unwrap_or(&claimant);
            let agent = match self.store.get_agent(name).await {
                Ok(Some(agent)) => agent,
                // The worker was removed: nobody will build this, so free it.
                Ok(None) if claimant.starts_with("agent:") => {
                    let _ = self.release_claim(&task_id).await;
                    continue;
                }
                // Not an agent claimant (e.g. `human`), or a lookup error:
                // nothing to measure activity against, so leave the claim as
                // is rather than guessing.
                _ => continue,
            };
            // br-4n3v: a live agent can be silent for longer than the lease (a
            // long `just check`), and a restart marks every agent `lost` with
            // its pre-restart `last_event_at`, so the first sweep after it saw
            // them all as stale. Only a claimant that has stopped for good
            // loses its claim; `lost` is about to be resumed.
            if !matches!(
                agent.state,
                AgentState::Stopped | AgentState::Exited | AgentState::Crashed
            ) {
                continue;
            }
            let Some(last) = agent.last_event_at.or(agent.turn_started_at) else {
                continue;
            };
            if now - last < self.claim_lease_after {
                continue;
            }
            // Only a `claimed` task moves back to `planned`; anything else
            // (e.g. dropped) has a stale claim that isn't ours to act on.
            let _ = self.release_claim(&task_id).await;
        }
    }

    /// Incidents that are `planned`, oldest first, for `bridle status`.
    pub fn active_incidents(&self) -> Vec<bridle_api::types::IncidentSummary> {
        self.list_tasks()
            .into_iter()
            .filter(|t| t.kind == TicketKind::Incident && t.state == TicketState::Planned)
            .map(|t| bridle_api::types::IncidentSummary {
                id: t.id,
                title: t.title,
                since: t.updated_at,
            })
            .collect()
    }

    pub fn ready_tasks(&self) -> Vec<Ticket> {
        self.list_tasks()
            .into_iter()
            .filter(|t| self.is_ready(t))
            .collect()
    }

    // ---------- queue ----------

    /// The queue's tiers in rank order, each a list of task ids
    /// (roles-and-lifecycle.md, "the queue"). Empty until the PM sets one; a
    /// task not listed in any tier is backlog.
    pub fn queue_tiers(&self) -> Vec<Vec<String>> {
        self.queue.lock().expect("queue lock").clone()
    }

    /// [`TicketManager::queue_tiers`] without integrated tasks (and any tier
    /// that leaves empty): merged work is done, not queued. The stored
    /// queue keeps them until the PM's next `set_queue`.
    pub fn live_queue_tiers(&self) -> Vec<Vec<String>> {
        self.queue_tiers()
            .into_iter()
            .map(|tier| {
                tier.into_iter()
                    .filter(|id| {
                        self.get_task(id)
                            .is_none_or(|t| t.state != TicketState::Integrated)
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|tier| !tier.is_empty())
            .collect()
    }

    /// Validates `tiers` against the task cache: every id must name a task
    /// that exists, no id may repeat across tiers (a task holds one rank),
    /// and no tier may be empty (an empty tier isn't a rank, it's nothing).
    fn validate_queue(&self, tiers: &[Vec<String>]) -> Result<(), TicketError> {
        let mut seen = std::collections::HashSet::new();
        for tier in tiers {
            if tier.is_empty() {
                return Err(TicketError::BadRequest(
                    "a tier must name at least one task".to_string(),
                ));
            }
            for id in tier {
                let Some(task) = self.get_task(id) else {
                    return Err(TicketError::NotFound(format!("no such task: {id}")));
                };
                if task.kind == TicketKind::Incident {
                    return Err(TicketError::BadRequest(format!(
                        "task {id} is an incident; incidents aren't queued"
                    )));
                }
                if !seen.insert(id) {
                    return Err(TicketError::Conflict(format!(
                        "task {id} appears more than once in the queue"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Replaces the whole queue: the PM's one write primitive, covering
    /// reorder/add/remove alike (resend the full tier list in the shape it
    /// should be). Durable the same way a task edit is: enqueued for the
    /// state branch's next flush, with `actor` recorded on the accompanying
    /// event line, and the in-memory cache updated immediately so a
    /// following read sees it before that flush happens.
    pub async fn set_queue(
        &self,
        tiers: Vec<Vec<String>>,
        actor: &PrincipalId,
    ) -> Result<Vec<Vec<String>>, TicketError> {
        self.validate_queue(&tiers)?;
        self.state.enqueue_queue(&tiers)?;
        self.state.enqueue_queue_event(actor, Utc::now());
        *self.queue.lock().expect("queue lock") = tiers.clone();
        Ok(tiers)
    }

    /// Appends one new tier, ranked after every existing one: sugar over
    /// `set_queue` for the common case of adding work at the back of the
    /// queue without resending the tiers already there.
    pub async fn add_queue_tier(
        &self,
        tasks: Vec<String>,
        actor: &PrincipalId,
    ) -> Result<Vec<Vec<String>>, TicketError> {
        let mut tiers = self.queue_tiers();
        tiers.push(tasks);
        self.set_queue(tiers, actor).await
    }

    /// The highest tier with at least one startable task (ready: planned,
    /// deps met, no open question, and — since `is_ready` requires
    /// `Planned` — necessarily unclaimed), and just that tier's startable
    /// tasks. Empty if no tier has one, including when the queue itself is
    /// empty: the manager takes from the next tier down rather than idling
    /// on a blocked one (roles-and-lifecycle.md, "the queue"), and a task
    /// outside the queue entirely is backlog, never returned here.
    pub fn highest_startable_tier(&self) -> Vec<Ticket> {
        let ready_ids: std::collections::HashSet<String> =
            self.ready_tasks().into_iter().map(|t| t.id).collect();
        for tier in self.queue_tiers() {
            let startable: Vec<Ticket> = tier
                .iter()
                .filter(|id| ready_ids.contains(*id))
                .filter_map(|id| self.get_task(id))
                .collect();
            if !startable.is_empty() {
                return startable;
            }
        }
        Vec::new()
    }

    /// Every task with an unanswered question, oldest first: for `bridle
    /// inbox` (coordination.md, "Questions do not stop work"). Reads the
    /// asker/body/age off the cached task's thread rather than the store, the
    /// same way the rest of this manager avoids a database round trip for
    /// data it already holds.
    pub fn list_open_questions(&self) -> Vec<OpenQuestion> {
        let open = self.open_questions.lock().expect("open questions lock");
        let cache = self.cache.lock().expect("task cache lock");
        let mut questions: Vec<OpenQuestion> = open
            .keys()
            .filter_map(|task_id| {
                let task = cache.get(task_id)?;
                let entry = task
                    .thread
                    .iter()
                    .rev()
                    .find(|e| e.kind == ThreadEntryKind::Question)?;
                Some(OpenQuestion {
                    task_id: task_id.clone(),
                    asked_by: entry.from.clone(),
                    body: entry.body.clone(),
                    asked_at: entry.at,
                })
            })
            .collect();
        questions.sort_by_key(|q| q.asked_at);
        questions
    }

    /// Flushes the state branch's pending writes. Exposed so the periodic
    /// tick and, later, an immediate-flush caller (e.g. on `accept`) share
    /// one code path (docs/design/storage.md, "The state branch").
    pub async fn flush_now(&self) -> Result<(), TicketError> {
        Ok(self.state.flush_now().await?)
    }

    /// One last push after the shutdown flush (bounded; see `StateBranch::push_on_shutdown`).
    pub async fn push_on_shutdown(&self, timeout: std::time::Duration) {
        self.state.push_on_shutdown(timeout).await
    }

    pub fn state_push_status(&self) -> Option<bridle_api::types::StatePushStatus> {
        self.state.push_status()
    }
}

/// Before the `ticket` front-matter field, a task made from a ticket started its body
/// `original id: <ticket>`. Returns that ticket id (if the first line is such a line) and the
/// body without the line.
fn clear_held(task: &mut Ticket) {
    task.held_by = None;
    task.held_reason = None;
    task.held_until = None;
    task.held_at = None;
}

fn split_legacy_origin(body: &str) -> (Option<String>, String) {
    const PREFIX: &str = "original id: ";
    let (first, rest) = body.split_once('\n').unwrap_or((body, ""));
    match first.strip_prefix(PREFIX).map(str::trim) {
        Some(t) if !t.is_empty() => (Some(t.to_string()), rest.to_string()),
        _ => (None, body.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{NewAgent, Store};
    use std::path::Path;
    use tokio::process::Command;

    async fn init_repo(dir: &Path) {
        std::fs::create_dir_all(dir).expect("mkdir repo");
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["init", "-q", "-b", "main"])
            .output()
            .await
            .expect("git init");
        assert!(out.status.success());
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.email=t@e.com",
                "-c",
                "user.name=T",
                "commit",
                "--allow-empty",
                "-q",
                "-m",
                "init",
            ])
            .output()
            .await
            .expect("git commit");
        assert!(out.status.success());
    }

    fn agent() -> PrincipalId {
        "agent:w".to_string()
    }

    async fn manager() -> (TicketManager, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TicketManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");
        (tm, tmp)
    }

    #[tokio::test]
    async fn new_task_is_open_and_visible_before_any_flush() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task(
                "Add foo",
                TicketKind::Feature,
                "a description".to_string(),
                vec![],
                None,
            )
            .await
            .expect("new task");
        assert!(task.id.starts_with("tw-"));
        assert_eq!(task.state, TicketState::Open);
        assert_eq!(task.body, "a description");

        let fetched = tm.get_task(&task.id).expect("get");
        assert_eq!(fetched.title, "Add foo");
        assert_eq!(tm.list_tasks().len(), 1);
    }

    #[tokio::test]
    async fn new_task_ids_use_the_ticket_alphabet() {
        let (tm, _tmp) = manager().await;
        for _ in 0..40 {
            let t = tm
                .new_open_task("A", TicketKind::Chore, String::new(), vec![], None)
                .await
                .expect("new task");
            let tail = t.id.strip_prefix("tw-").expect("prefix");
            assert_eq!(tail.len(), 4);
            assert!(
                tail.bytes()
                    .all(|b| b"abcdefghjkmnpqrstuvwxyz23456789".contains(&b))
            );
        }
    }

    #[tokio::test]
    async fn a_tickets_first_task_takes_its_id_and_the_second_gets_a_fresh_one() {
        let (tm, _tmp) = manager().await;
        let body = "original id: k7tm\ndocs/tickets/open/x-k7tm.md".to_string();
        let first = tm
            .new_open_task("A", TicketKind::Feature, body.clone(), vec![], None)
            .await
            .expect("first");
        assert_eq!(first.id, "tw-k7tm");
        let second = tm
            .new_open_task("B", TicketKind::Feature, body, vec![], None)
            .await
            .expect("second");
        assert_ne!(second.id, "tw-k7tm");
        assert!(second.id.starts_with("tw-"));
    }

    #[tokio::test]
    async fn edit_changes_title_and_body_without_changing_state() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");

        let edited = tm
            .edit_task(
                &task.id,
                &agent(),
                Some("Add foo, better".to_string()),
                None,
                None,
                None,
            )
            .await
            .expect("edit title");
        assert_eq!(edited.title, "Add foo, better");
        assert_eq!(edited.state, TicketState::Open);

        let edited = tm
            .edit_task(
                &task.id,
                &agent(),
                None,
                Some("new body".to_string()),
                None,
                None,
            )
            .await
            .expect("edit body");
        assert_eq!(edited.body, "new body");
        assert_eq!(edited.title, "Add foo, better");
    }

    #[tokio::test]
    async fn a_qualified_ticket_is_stored_bare_and_another_prefix_is_kept() {
        let (tm, _tmp) = manager().await;
        let own = tm
            .new_task_for_ticket(
                "T",
                TicketKind::Feature,
                String::new(),
                vec![],
                None,
                "a",
                Some("tw-abcd".into()),
            )
            .await
            .expect("own");
        assert_eq!(own.ticket.as_deref(), Some("abcd"));
        assert_eq!(own.id, "tw-abcd");
        let other = tm
            .new_task_for_ticket(
                "T",
                TicketKind::Feature,
                String::new(),
                vec![],
                None,
                "a",
                Some("ui-wxyz".into()),
            )
            .await
            .expect("other");
        assert_eq!(other.ticket.as_deref(), Some("ui-wxyz"));
    }

    #[tokio::test]
    async fn a_ticket_task_has_the_field_and_no_body_line_and_edit_keeps_it() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_task_for_ticket(
                "T",
                TicketKind::Feature,
                "docs/tickets/open/x-abcd.md".to_string(),
                vec![],
                None,
                "human",
                Some("abcd".to_string()),
            )
            .await
            .expect("new task");
        assert_eq!(task.ticket.as_deref(), Some("abcd"));
        assert_eq!(task.id, "tw-abcd");
        assert_eq!(task.body, "docs/tickets/open/x-abcd.md");
        let edited = tm
            .edit_task(
                &task.id,
                &agent(),
                None,
                Some("brief".to_string()),
                None,
                None,
            )
            .await
            .expect("edit");
        assert_eq!(edited.body, "brief");
        assert_eq!(edited.ticket.as_deref(), Some("abcd"));
        let set = tm.set_ticket(&task.id, "wxyz").await.expect("set");
        assert_eq!(set.ticket.as_deref(), Some("wxyz"));
        // A task without a ticket stays without one.
        let plain = tm
            .new_open_task("P", TicketKind::Feature, "plain".to_string(), vec![], None)
            .await
            .expect("new task");
        assert_eq!(plain.ticket, None);
    }

    #[tokio::test]
    async fn an_old_client_origin_line_becomes_the_field_on_create() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task(
                "T",
                TicketKind::Feature,
                "original id: abcd\nold".to_string(),
                vec![],
                None,
            )
            .await
            .expect("new task");
        assert_eq!(task.ticket.as_deref(), Some("abcd"));
        assert_eq!(task.body, "old");
    }

    #[tokio::test]
    async fn start_moves_an_original_id_line_into_the_ticket_field_once() {
        let (tm, tmp) = manager().await;
        let mut task = tm
            .new_open_task("T", TicketKind::Feature, "x".to_string(), vec![], None)
            .await
            .expect("new task");
        let plain = tm
            .new_open_task("P", TicketKind::Feature, "plain".to_string(), vec![], None)
            .await
            .expect("new task");
        // The state a pre-vk3y daemon left behind.
        task.body = "original id: abcd\ndocs/x.md\n\nbrief".to_string();
        tm.state.enqueue_task(&task).expect("enqueue");
        tm.state.flush_now().await.expect("flush");
        let reopen = || async {
            let store = Store::open(tmp.path().join("bridle.db"))
                .await
                .expect("open store");
            let state = StateBranch::open(&tmp.path().join("repo"), &tmp.path().join("state"))
                .await
                .expect("open state branch");
            let tm = TicketManager::open(
                store,
                state,
                "tw".to_string(),
                std::time::Duration::from_secs(600),
            )
            .await
            .expect("reopen");
            tm.state.flush_now().await.expect("flush");
            tm
        };
        let tm2 = reopen().await;
        let migrated = tm2.get_task(&task.id).expect("task");
        assert_eq!(migrated.ticket.as_deref(), Some("abcd"));
        assert_eq!(migrated.body, "docs/x.md\n\nbrief");
        assert_eq!(tm2.get_task(&plain.id).expect("plain").ticket, None);
        // It reached the state branch, and a second start changes nothing.
        let on_disk = tm2.state.read_task(&task.id).expect("file");
        assert_eq!(on_disk.ticket.as_deref(), Some("abcd"));
        let tm3 = reopen().await;
        let again = tm3.get_task(&task.id).expect("task");
        assert_eq!(again.ticket, migrated.ticket);
        assert_eq!(again.body, migrated.body);
        assert_eq!(again.updated_at, migrated.updated_at);
    }

    #[tokio::test]
    async fn edit_rejects_a_blank_title() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        let err = tm
            .edit_task(
                &task.id,
                &agent(),
                Some("   ".to_string()),
                None,
                None,
                None,
            )
            .await
            .expect_err("blank title");
        assert!(matches!(err, TicketError::BadRequest(_)));
    }

    #[tokio::test]
    async fn edit_clears_size_with_none() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task(
                "Add foo",
                TicketKind::Feature,
                String::new(),
                vec![],
                Some(TicketSize::M),
            )
            .await
            .expect("new task");
        assert_eq!(task.size, Some(TicketSize::M));

        let edited = tm
            .edit_task(&task.id, &agent(), None, None, None, Some(TicketSize::None))
            .await
            .expect("clear size");
        assert_eq!(edited.size, None);
    }

    #[tokio::test]
    async fn drop_requires_a_reason_and_reopen_only_applies_to_dropped() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");

        let err = tm
            .drop_task(&task.id, "", &"human".to_string())
            .await
            .expect_err("empty reason");
        assert!(matches!(err, TicketError::BadRequest(_)));

        let err = tm
            .reopen_task(&task.id, &"human".to_string())
            .await
            .expect_err("not dropped yet");
        assert!(matches!(err, TicketError::Conflict(_)));

        let dropped = tm
            .drop_task(&task.id, "budget cut", &"human".to_string())
            .await
            .expect("drop");
        assert_eq!(dropped.state, TicketState::Dropped);
        assert_eq!(dropped.thread.len(), 1);
        assert_eq!(dropped.thread[0].body, "dropped: budget cut");

        let err = tm
            .drop_task(&task.id, "again", &"human".to_string())
            .await
            .expect_err("already dropped");
        assert!(matches!(err, TicketError::Conflict(_)));

        let reopened = tm
            .reopen_task(&task.id, &"human".to_string())
            .await
            .expect("reopen");
        assert_eq!(reopened.state, TicketState::Reopened);

        let err = tm
            .reopen_task(&task.id, &"human".to_string())
            .await
            .expect_err("not dropped any more");
        assert!(matches!(err, TicketError::Conflict(_)));
    }

    #[tokio::test]
    async fn plan_moves_open_to_planned_and_rejects_every_other_state() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        assert_eq!(task.state, TicketState::Open);

        let planned = tm
            .plan_task(&task.id, &"human".to_string())
            .await
            .expect("plan");
        assert_eq!(planned.state, TicketState::Planned);

        // Already planned: not open any more.
        let err = tm
            .plan_task(&task.id, &"human".to_string())
            .await
            .expect_err("already planned");
        assert!(matches!(err, TicketError::Conflict(_)));

        // Claimed: also not open.
        tm.claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim");
        let err = tm
            .plan_task(&task.id, &"human".to_string())
            .await
            .expect_err("claimed, not open");
        assert!(matches!(err, TicketError::Conflict(_)));

        // Dropped and reopened: a reopened task can be planned again.
        tm.release_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("release");
        tm.drop_task(&task.id, "no longer needed", &"human".to_string())
            .await
            .expect("drop");
        tm.reopen_task(&task.id, &"human".to_string())
            .await
            .expect("reopen");
        let replanned = tm
            .plan_task(&task.id, &"human".to_string())
            .await
            .expect("reopened can be planned");
        assert_eq!(replanned.state, TicketState::Planned);
    }

    #[tokio::test]
    async fn a_reopened_task_can_be_dropped() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("t", TicketKind::Fix, String::new(), vec![], None)
            .await
            .expect("new");
        let human = "human".to_string();
        tm.drop_task(&task.id, "first", &human).await.expect("drop");
        tm.reopen_task(&task.id, &human).await.expect("reopen");
        let dropped = tm
            .drop_task(&task.id, "again", &human)
            .await
            .expect("drop reopened");
        assert_eq!(dropped.state, TicketState::Dropped);
    }

    #[tokio::test]
    async fn plan_of_an_unknown_id_is_not_found() {
        let (tm, _tmp) = manager().await;
        let err = tm
            .plan_task("tw-nope", &"human".to_string())
            .await
            .expect_err("no such task");
        assert!(matches!(err, TicketError::NotFound(_)));
    }

    #[tokio::test]
    async fn add_edge_rejects_self_loops_and_unknown_tasks_and_duplicates() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_open_task("A", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_open_task("B", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");

        let err = tm
            .add_edge(&a.id, &a.id, EdgeKind::Blocks)
            .await
            .expect_err("self loop");
        assert!(matches!(err, TicketError::BadRequest(_)));

        let err = tm
            .add_edge("tw-nope", &b.id, EdgeKind::Blocks)
            .await
            .expect_err("unknown from");
        assert!(matches!(err, TicketError::NotFound(_)));

        let err = tm
            .add_edge(&a.id, "tw-nope", EdgeKind::Blocks)
            .await
            .expect_err("unknown to");
        assert!(matches!(err, TicketError::NotFound(_)));

        let edge = tm
            .add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("add edge");
        assert_eq!(edge.from, a.id);
        assert_eq!(edge.to, b.id);
        assert_eq!(tm.list_edges(), vec![edge]);

        let err = tm
            .add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect_err("duplicate edge");
        assert!(matches!(err, TicketError::Conflict(_)));
    }

    #[tokio::test]
    async fn remove_edge_drops_it_and_errors_when_missing() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_open_task("A", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_open_task("B", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");
        tm.add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("add edge");

        tm.remove_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("remove edge");
        assert!(tm.list_edges().is_empty());

        let err = tm
            .remove_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect_err("already removed");
        assert!(matches!(err, TicketError::NotFound(_)));
    }

    /// A shortcut for tests that don't care about `plan_task` itself, just
    /// getting a task to `planned` to exercise `is_ready`/`ready_tasks`/etc.
    /// on top of it: reaches into the private cache directly rather than
    /// going through `plan_task`'s actor/event bookkeeping, which those
    /// tests have no use for.
    fn force_planned(tm: &TicketManager, id: &str) {
        let mut cache = tm.cache.lock().expect("task cache lock");
        cache.get_mut(id).expect("task in cache").state = TicketState::Planned;
    }

    #[tokio::test]
    async fn a_prepared_version_2_ticket_opens_itself_and_a_version_1_does_not() {
        let (tm, tmp) = manager().await;
        let v2 = tm
            .new_task("V2", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("v2");
        let v1 = tm
            .new_task("V1", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("v1");
        let open = tmp.path().join("tickets/open");
        std::fs::create_dir_all(&open).expect("dir");
        let file = |id: &str, extra: &str| format!("---\nid: {id}\ntitle: t\n{extra}---\n\nbody\n");
        std::fs::write(
            open.join("a.md"),
            file(&v2.id, "version: 2\nprepared: true\n"),
        )
        .expect("v2");
        std::fs::write(open.join("b.md"), file(&v1.id, "prepared: true\n")).expect("v1");
        let (_, _, readied) = tm.sync_ticket_links(&tmp.path().join("tickets")).await;
        assert_eq!(readied.len(), 1);
        let v2 = tm.get_task(&v2.id).expect("v2 row");
        assert_eq!(v2.state, TicketState::Open);
        assert!(v2.thread.iter().any(|e| e.from == "bridle"));
        assert_eq!(
            tm.get_task(&v1.id).expect("v1 row").state,
            TicketState::Pending
        );
    }

    #[tokio::test]
    async fn integrating_a_blocker_frees_the_blocked_task_and_leaves_the_queue() {
        let (tm, _tmp) = manager().await;
        let human = "human".to_string();
        let blocker = tm
            .new_open_task("Blocker", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new blocker");
        let blocked = tm
            .new_open_task("Blocked", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new blocked");
        tm.add_edge(&blocker.id, &blocked.id, EdgeKind::Blocks)
            .await
            .expect("add edge");
        force_planned(&tm, &blocker.id);
        force_planned(&tm, &blocked.id);
        tm.set_queue(
            vec![vec![blocker.id.clone()], vec![blocked.id.clone()]],
            &human,
        )
        .await
        .expect("set queue");
        assert!(!tm.is_ready(&tm.get_task(&blocked.id).expect("blocked")));

        let err = tm
            .done_task(&blocker.id, " ", None, &human)
            .await
            .expect_err("empty commit");
        assert!(matches!(err, TicketError::BadRequest(_)));

        let done = tm
            .done_task(&blocker.id, "abc123", None, &human)
            .await
            .expect("done");
        assert_eq!(done.state, TicketState::Integrated);
        assert_eq!(done.thread[0].body, "integrated: abc123");
        assert!(tm.is_ready(&tm.get_task(&blocked.id).expect("blocked")));
        let ready: Vec<String> = tm.ready_tasks().into_iter().map(|t| t.id).collect();
        assert_eq!(ready, vec![blocked.id.clone()]);
        assert_eq!(tm.live_queue_tiers(), vec![vec![blocked.id.clone()]]);

        let err = tm
            .done_task(&blocker.id, "def456", None, &human)
            .await
            .expect_err("already integrated");
        assert!(matches!(err, TicketError::Conflict(_)));

        let reopened = tm.reopen_task(&blocker.id, &human).await.expect("reopen");
        assert_eq!(reopened.state, TicketState::Reopened);
        assert_eq!(tm.live_queue_tiers().len(), 2);
    }

    #[tokio::test]
    async fn done_records_branch_and_commit_and_summary_replaces() {
        let (tm, _tmp) = manager().await;
        let human = "human".to_string();
        let t = tm
            .new_open_task("T", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new");
        let err = tm.set_summary(&t.id, "  ").await.expect_err("empty");
        assert!(matches!(err, TicketError::BadRequest(_)));
        tm.set_summary(&t.id, "first").await.expect("summary");
        let s = tm.set_summary(&t.id, "second\n").await.expect("replace");
        assert_eq!(s.summary.as_deref(), Some("second"));
        let done = tm
            .done_task(&t.id, "abc123", Some("bridle/t"), &human)
            .await
            .expect("done");
        assert_eq!(done.branch.as_deref(), Some("bridle/t"));
        assert_eq!(done.commit.as_deref(), Some("abc123"));
        assert_eq!(done.summary.as_deref(), Some("second"));
        assert_eq!(done.thread[0].body, "integrated: abc123 (branch bridle/t)");
    }

    #[tokio::test]
    async fn ready_tasks_excludes_unplanned_and_blocked_tasks() {
        let (tm, _tmp) = manager().await;
        let blocker = tm
            .new_open_task("Blocker", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new blocker");
        let blocked = tm
            .new_open_task("Blocked", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new blocked");
        let unplanned = tm
            .new_open_task(
                "Unplanned",
                TicketKind::Feature,
                String::new(),
                vec![],
                None,
            )
            .await
            .expect("new unplanned");

        tm.add_edge(&blocker.id, &blocked.id, EdgeKind::Blocks)
            .await
            .expect("add edge");

        // Still open, not planned: none of the three are ready yet.
        assert!(tm.ready_tasks().is_empty());

        force_planned(&tm, &blocker.id);
        force_planned(&tm, &blocked.id);
        force_planned(&tm, &unplanned.id);

        // `blocked` is planned but its blocker isn't dropped, so it's still
        // not ready. `blocker` and `unplanned` (misleadingly named now that
        // it's planned too) have no blocks-edge pointing at them, so both
        // are ready.
        let ready_ids: Vec<String> = tm.ready_tasks().into_iter().map(|t| t.id).collect();
        assert!(!ready_ids.contains(&blocked.id));
        assert!(ready_ids.contains(&unplanned.id));
        assert!(ready_ids.contains(&blocker.id));

        // Dropping the blocker resolves it, which frees the blocked task.
        tm.drop_task(&blocker.id, "done another way", &"human".to_string())
            .await
            .expect("drop blocker");
        let ready_ids: Vec<String> = tm.ready_tasks().into_iter().map(|t| t.id).collect();
        assert!(ready_ids.contains(&blocked.id));
    }

    #[tokio::test]
    async fn set_queue_validates_unknown_ids_duplicates_and_empty_tiers() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_open_task("A", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_open_task("B", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");

        let err = tm
            .set_queue(vec![vec!["tw-nope".to_string()]], &"human".to_string())
            .await
            .expect_err("unknown task id");
        assert!(matches!(err, TicketError::NotFound(_)));

        let err = tm
            .set_queue(
                vec![vec![a.id.clone()], vec![a.id.clone()]],
                &"human".to_string(),
            )
            .await
            .expect_err("same task in two tiers");
        assert!(matches!(err, TicketError::Conflict(_)));

        let err = tm
            .set_queue(vec![vec![]], &"human".to_string())
            .await
            .expect_err("empty tier");
        assert!(matches!(err, TicketError::BadRequest(_)));

        let tiers = tm
            .set_queue(vec![vec![a.id.clone(), b.id.clone()]], &"human".to_string())
            .await
            .expect("valid queue");
        assert_eq!(tiers, vec![vec![a.id.clone(), b.id.clone()]]);
        assert_eq!(tm.queue_tiers(), tiers);
    }

    #[tokio::test]
    async fn add_queue_tier_appends_after_whatever_is_already_there() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_open_task("A", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_open_task("B", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");

        tm.add_queue_tier(vec![a.id.clone()], &"human".to_string())
            .await
            .expect("first tier");
        let tiers = tm
            .add_queue_tier(vec![b.id.clone()], &"human".to_string())
            .await
            .expect("second tier");
        assert_eq!(tiers, vec![vec![a.id.clone()], vec![b.id.clone()]]);
    }

    /// The core dispatch rule: the manager takes from the highest tier with
    /// a startable task, skipping a tier that's stuck on a dependency rather
    /// than idling on it (roles-and-lifecycle.md, "the queue").
    #[tokio::test]
    async fn highest_startable_tier_skips_a_tier_blocked_on_a_dependency() {
        let (tm, _tmp) = manager().await;
        let blocker = tm
            .new_open_task("Blocker", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new blocker");
        let blocked = tm
            .new_open_task("Blocked", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new blocked");
        let next = tm
            .new_open_task("Next", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new next");
        tm.add_edge(&blocker.id, &blocked.id, EdgeKind::Blocks)
            .await
            .expect("add edge");
        force_planned(&tm, &blocker.id);
        force_planned(&tm, &blocked.id);
        force_planned(&tm, &next.id);

        // Tier 1's only task is blocked on tier 1's own blocker... but the
        // blocker isn't itself in the queue (it's backlog): tier 1 has
        // nothing startable, so the highest startable tier is tier 2.
        tm.set_queue(
            vec![vec![blocked.id.clone()], vec![next.id.clone()]],
            &"human".to_string(),
        )
        .await
        .expect("set queue");
        let top: Vec<String> = tm
            .highest_startable_tier()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(top, vec![next.id.clone()]);

        // Resolving the blocker frees tier 1's task, which now outranks
        // tier 2 again.
        tm.drop_task(&blocker.id, "done another way", &"human".to_string())
            .await
            .expect("drop blocker");
        let top: Vec<String> = tm
            .highest_startable_tier()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(top, vec![blocked.id.clone()]);
    }

    #[tokio::test]
    async fn highest_startable_tier_is_empty_with_no_queue_or_nothing_startable() {
        let (tm, _tmp) = manager().await;
        assert!(tm.highest_startable_tier().is_empty());

        // A task exists and is even ready, but it's backlog: not in any
        // tier, so it's never returned here.
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));
        assert!(tm.highest_startable_tier().is_empty());
    }

    #[tokio::test]
    async fn queue_survives_a_flush_and_a_fresh_manager_hydrating_from_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TicketManager::open(
            store.clone(),
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let a = tm
            .new_open_task("A", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_open_task("B", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new b");
        tm.set_queue(
            vec![vec![a.id.clone()], vec![b.id.clone()]],
            &"human".to_string(),
        )
        .await
        .expect("set queue");
        tm.flush_now().await.expect("flush");

        let tm2 = TicketManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager");
        assert_eq!(
            tm2.queue_tiers(),
            vec![vec![a.id.clone()], vec![b.id.clone()]]
        );
    }

    #[tokio::test]
    async fn get_edit_and_drop_on_an_unknown_id_is_not_found() {
        let (tm, _tmp) = manager().await;
        assert!(tm.get_task("tw-nope").is_none());
        assert!(matches!(
            tm.edit_task("tw-nope", &agent(), Some("x".to_string()), None, None, None)
                .await
                .expect_err("no such task"),
            TicketError::NotFound(_)
        ));
        assert!(matches!(
            tm.drop_task("tw-nope", "why", &"human".to_string())
                .await
                .expect_err("no such task"),
            TicketError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn state_survives_a_flush_and_a_fresh_manager_hydrating_from_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TicketManager::open(
            store.clone(),
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let task = tm
            .new_open_task(
                "Add foo",
                TicketKind::Feature,
                "a description".to_string(),
                vec![],
                None,
            )
            .await
            .expect("new task");
        tm.flush_now().await.expect("flush");

        // A fresh manager over the same store and state branch, as a
        // restarted daemon would build, hydrates the full record (title
        // *and* body) from the flushed file, not just the database row.
        let tm2 = TicketManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager");
        let rehydrated = tm2.get_task(&task.id).expect("rehydrated");
        assert_eq!(rehydrated.title, "Add foo");
        assert_eq!(rehydrated.body, "a description");
    }

    async fn reopen_manager(store: Store, state: StateBranch) -> TicketManager {
        TicketManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager")
    }

    #[tokio::test]
    async fn a_task_filed_from_a_ticket_is_watched_by_whoever_filed_it() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = reopen_manager(store, state).await;
        let aide = "external:aide".to_string();
        let task = tm
            .new_task_for_ticket(
                "T",
                TicketKind::Feature,
                "docs/tickets/open/t-abcd.md".to_string(),
                vec![],
                None,
                &aide,
                Some("abcd".to_string()),
            )
            .await
            .expect("new task");
        assert_eq!(task.ticket.as_deref(), Some("abcd"));
        assert_eq!(task.watchers, vec![aide.clone()]);
        assert_eq!(notified_of(&task), vec![aide]);
    }

    #[tokio::test]
    async fn split_from_links_inherits_watchers_and_notes_the_parent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = reopen_manager(store.clone(), state.clone()).await;

        let parent = tm
            .new_task_by(
                "P",
                TicketKind::Feature,
                String::new(),
                vec![],
                None,
                "advisor",
            )
            .await
            .expect("parent");
        let orch = "orchestrator".to_string();
        let child = tm
            .new_task_by("C", TicketKind::Feature, String::new(), vec![], None, &orch)
            .await
            .expect("child");
        let child = tm
            .split_from(&child.id, &parent.id, &orch)
            .await
            .expect("split");
        assert_eq!(child.parent.as_deref(), Some(parent.id.as_str()));
        assert_eq!(child.watchers, vec![orch.clone(), "advisor".to_string()]);
        let parent = tm.get_task(&parent.id).expect("parent");
        let note = parent.thread.last().expect("a note on the parent");
        assert_eq!(note.from, orch);
        assert!(note.body.contains(&child.id), "{}", note.body);
        // The link outlives a restart.
        state.flush_now().await.expect("flush");
        let tm2 = reopen_manager(store, state).await;
        assert_eq!(
            tm2.get_task(&child.id).expect("child").parent.as_deref(),
            Some(parent.id.as_str())
        );
        assert!(matches!(
            tm2.split_from(&child.id, "nope", &orch).await,
            Err(TicketError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn watchers_creator_claimer_watch_unwatch_rebuild_and_backfill() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = reopen_manager(store.clone(), state.clone()).await;

        let task = tm
            .new_task_by(
                "T",
                TicketKind::Feature,
                String::new(),
                vec![],
                None,
                "human",
            )
            .await
            .expect("new task");
        assert_eq!(task.watchers, vec!["human".to_string()]);
        let anon = tm
            .new_open_task("A", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        assert!(
            anon.watchers.is_empty(),
            "an unknown creator isn't a watcher"
        );

        tm.ready_task(&task.id, &"human".to_string())
            .await
            .expect("ready");
        tm.plan_task(&task.id, &"human".to_string())
            .await
            .expect("plan");
        let claimed = tm.claim_task(&task.id, &agent()).await.expect("claim");
        assert_eq!(claimed.watchers, vec!["human".to_string(), agent()]);
        assert_eq!(notified_of(&claimed), claimed.watchers);

        // Idempotent, and recorded in the thread only when something changed.
        let (_, changed) = tm
            .set_watching(&task.id, &agent(), true)
            .await
            .expect("watch");
        assert!(!changed);
        let (t, changed) = tm
            .set_watching(&task.id, &agent(), false)
            .await
            .expect("unwatch");
        assert!(changed);
        assert_eq!(t.watchers, vec!["human".to_string()]);
        assert!(
            t.thread
                .last()
                .expect("entry")
                .body
                .contains("stopped watching")
        );
        let (t, _) = tm
            .set_watching(&task.id, &"agent:w2".to_string(), true)
            .await
            .expect("watch");
        assert!(t.thread.last().expect("entry").body.contains("watching"));
        assert_eq!(t.thread.len(), 2);
        let (t, _) = tm
            .set_watching(&anon.id, &"human".to_string(), false)
            .await
            .expect("noop");
        assert!(t.thread.is_empty());
        tm.flush_now().await.expect("flush");

        // A restart keeps the list, including an emptied one (not mistaken for an old record).
        tm.set_watching(&anon.id, &"human".to_string(), true)
            .await
            .expect("watch");
        tm.set_watching(&anon.id, &"human".to_string(), false)
            .await
            .expect("unwatch");
        tm.flush_now().await.expect("flush");
        let tm2 = reopen_manager(store.clone(), state.clone()).await;
        assert_eq!(
            tm2.get_task(&task.id).expect("task").watchers,
            vec!["human".to_string(), "agent:w2".to_string()]
        );
        assert!(tm2.get_task(&anon.id).expect("anon").watchers.is_empty());

        // A rebuild from the state branch alone restores them.
        let fresh = Store::open(tmp.path().join("bridle2.db"))
            .await
            .expect("store");
        let tm3 = reopen_manager(fresh, state.clone()).await;
        tm3.rebuild_from_state_branch().await.expect("rebuild");
        assert_eq!(
            tm3.get_task(&task.id).expect("task").watchers,
            vec!["human".to_string(), "agent:w2".to_string()]
        );

        // Old records (no watchers line) are backfilled with the creator and the claimer.
        let file = state.dir().join("tracking").join(format!("{}.md", task.id));
        let text = std::fs::read_to_string(&file).expect("read");
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("watchers"))
            .map(|l| format!("{l}\n"))
            .collect();
        std::fs::write(&file, old).expect("write");
        let tm4 = reopen_manager(store.clone(), state.clone()).await;
        assert_eq!(
            tm4.get_task(&task.id).expect("task").watchers,
            vec!["human".to_string(), agent()]
        );
        // Idempotent: the backfill recorded itself, and a later removal sticks.
        tm4.set_watching(&task.id, &agent(), false)
            .await
            .expect("unwatch");
        tm4.flush_now().await.expect("flush");
        let tm5 = reopen_manager(store, state).await;
        assert_eq!(
            tm5.get_task(&task.id).expect("task").watchers,
            vec!["human".to_string()]
        );
    }

    #[tokio::test]
    async fn asking_a_question_blocks_ready_and_answering_unblocks_it() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));

        let asked = tm
            .ask_question(&task.id, &"agent:w1".to_string(), "which endpoint?")
            .await
            .expect("ask question");
        assert_eq!(asked.thread.len(), 1);
        assert_eq!(asked.thread[0].kind, ThreadEntryKind::Question);
        assert_eq!(asked.thread[0].from, "agent:w1");
        assert_eq!(asked.thread[0].body, "which endpoint?");
        assert!(
            !tm.ready_tasks().iter().any(|t| t.id == task.id),
            "an open question excludes a task from ready"
        );

        // A second question while one is already open is a conflict.
        let err = tm
            .ask_question(&task.id, &"human".to_string(), "another one?")
            .await
            .expect_err("already has an open question");
        assert!(matches!(err, TicketError::Conflict(_)));

        let answered = tm
            .answer_question(&task.id, &"human".to_string(), "the v1 endpoint")
            .await
            .expect("answer question");
        assert_eq!(answered.thread.len(), 2);
        assert_eq!(answered.thread[1].kind, ThreadEntryKind::Answer);
        assert_eq!(answered.thread[1].from, "human");
        assert_eq!(answered.thread[1].body, "the v1 endpoint");
        assert!(
            tm.ready_tasks().iter().any(|t| t.id == task.id),
            "answering the question re-enables readiness"
        );

        // Answering again with nothing open is a conflict.
        let err = tm
            .answer_question(&task.id, &"human".to_string(), "still there?")
            .await
            .expect_err("no open question left");
        assert!(matches!(err, TicketError::Conflict(_)));
    }

    #[tokio::test]
    async fn ask_and_answer_reject_blank_bodies() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");

        let err = tm
            .ask_question(&task.id, &"human".to_string(), "   ")
            .await
            .expect_err("blank question body");
        assert!(matches!(err, TicketError::BadRequest(_)));

        tm.ask_question(&task.id, &"human".to_string(), "real question")
            .await
            .expect("ask question");
        let err = tm
            .answer_question(&task.id, &"human".to_string(), "")
            .await
            .expect_err("blank answer body");
        assert!(matches!(err, TicketError::BadRequest(_)));
    }

    #[tokio::test]
    async fn note_adds_a_thread_entry_without_affecting_readiness() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));

        let noted = tm
            .note_task(&task.id, &"agent:w1".to_string(), "fyi, started on this")
            .await
            .expect("note task");
        assert_eq!(noted.thread.len(), 1);
        assert_eq!(noted.thread[0].kind, ThreadEntryKind::Note);
        assert_eq!(noted.thread[0].from, "agent:w1");
        assert_eq!(noted.thread[0].body, "fyi, started on this");
        assert!(
            tm.ready_tasks().iter().any(|t| t.id == task.id),
            "a note doesn't block readiness like a question does"
        );

        let noted = tm
            .note_task(&task.id, &"human".to_string(), "sounds good")
            .await
            .expect("note task from human");
        assert_eq!(noted.thread.len(), 2);
        assert_eq!(noted.thread[1].from, "human");
        assert_eq!(noted.thread[1].body, "sounds good");
    }

    #[tokio::test]
    async fn note_rejects_a_blank_body() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        let err = tm
            .note_task(&task.id, &"human".to_string(), "   ")
            .await
            .expect_err("blank note body");
        assert!(matches!(err, TicketError::BadRequest(_)));
    }

    #[tokio::test]
    async fn an_open_question_survives_a_flush_and_a_fresh_manager_hydrating_from_the_database() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TicketManager::open(
            store.clone(),
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        tm.ask_question(&task.id, &"human".to_string(), "which endpoint?")
            .await
            .expect("ask question");
        tm.flush_now().await.expect("flush");

        // A fresh manager, as a restarted daemon would build, hydrates the
        // open-questions index from the database (not the state branch, which
        // has no separate index of its own) and the thread entry from the
        // flushed task file.
        let tm2 = TicketManager::open(
            store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager");
        let rehydrated = tm2.get_task(&task.id).expect("rehydrated");
        assert_eq!(rehydrated.thread.len(), 1);
        assert_eq!(rehydrated.thread[0].kind, ThreadEntryKind::Question);
        assert!(
            !tm2.is_ready(&rehydrated),
            "the open question survives a restart and still blocks ready"
        );
    }

    #[tokio::test]
    async fn claim_blocks_ready_and_release_unblocks_it() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        assert!(tm.ready_tasks().iter().any(|t| t.id == task.id));
        assert_eq!(task.claimed_by, None, "unclaimed on creation");

        let claimed = tm
            .claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim task");
        assert_eq!(claimed.state, TicketState::Claimed);
        assert_eq!(claimed.claimed_by.as_deref(), Some("agent:w1"));
        assert!(claimed.claimed_at.is_some());
        assert!(
            !tm.ready_tasks().iter().any(|t| t.id == task.id),
            "a claimed task drops out of ready"
        );

        // Claiming an already-claimed task is a conflict.
        let err = tm
            .claim_task(&task.id, &"agent:w2".to_string())
            .await
            .expect_err("already claimed");
        assert!(matches!(err, TicketError::Conflict(_)));

        let released = tm
            .release_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("release task");
        assert_eq!(released.state, TicketState::Planned);
        assert_eq!(released.claimed_by, None, "cleared on release");
        assert_eq!(released.claimed_at, None);
        assert!(
            tm.ready_tasks().iter().any(|t| t.id == task.id),
            "releasing the claim re-enables readiness"
        );

        // Releasing again with nothing claimed is a conflict.
        let err = tm
            .release_task(&task.id, &"agent:w1".to_string())
            .await
            .expect_err("no longer claimed");
        assert!(matches!(err, TicketError::Conflict(_)));
    }

    /// br-w6ww: the database state is written synchronously, the task file
    /// lags (and never records `claimed`), so a restart must trust the row.
    #[tokio::test]
    async fn a_claim_survives_a_restart_and_the_task_can_then_be_done() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let open = || {
            TicketManager::open(
                store.clone(),
                state.clone(),
                "tw".to_string(),
                std::time::Duration::from_secs(600),
            )
        };
        let tm = open().await.expect("open");
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        let human = "human".to_string();
        tm.plan_task(&task.id, &human).await.expect("plan");
        let w1 = "agent:w1".to_string();
        tm.claim_task(&task.id, &w1).await.expect("claim");
        // No flush: the restart comes before the task file catches up.
        let tm2 = open().await.expect("reopen");
        assert_eq!(store.list_claims().await.expect("claims").len(), 1);
        let t = tm2.get_task(&task.id).expect("task");
        assert_eq!(t.state, TicketState::Claimed);
        assert_eq!(t.claimed_by.as_deref(), Some(w1.as_str()));
        let done = tm2
            .done_task(&task.id, "abc123", None, &w1)
            .await
            .expect("done after restart");
        assert_eq!(done.state, TicketState::Integrated);
    }

    /// br-w6ww: a task `claimed` with no claims row (stuck by an older
    /// build) can still be closed with a commit.
    #[tokio::test]
    async fn a_claimed_task_with_no_claims_row_can_be_done_with_a_commit() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);
        let w1 = "agent:w1".to_string();
        tm.claim_task(&task.id, &w1).await.expect("claim");
        tm.store.delete_claim(&task.id).await.expect("drop row");
        tm.claims.lock().expect("claims lock").remove(&task.id);
        let done = tm
            .done_task(&task.id, "abc123", None, &w1)
            .await
            .expect("done");
        assert_eq!(done.state, TicketState::Integrated);
        assert_eq!(done.claimed_by, None);
    }

    /// A human to-do: the lease check never releases the human's claim, and
    /// the human finishes it with no commit.
    #[tokio::test]
    async fn human_claim_survives_the_lease_check_and_is_done_without_a_commit() {
        let (tm, _tmp) = manager().await;
        let t = tm
            .new_open_task(
                "[at restart] tokens",
                TicketKind::Feature,
                String::new(),
                vec![],
                None,
            )
            .await
            .expect("new");
        force_planned(&tm, &t.id);
        let human = "human".to_string();
        tm.claim_task(&t.id, &human).await.expect("claim");
        tm.tick_claim_lease_check(Utc::now() + chrono::Duration::days(30))
            .await;
        assert_eq!(
            tm.get_task(&t.id).expect("task").state,
            TicketState::Claimed
        );

        let done = tm.done_task(&t.id, "", None, &human).await.expect("done");
        assert_eq!(done.state, TicketState::Integrated);
        assert_eq!(done.commit, None);
    }

    #[tokio::test]
    async fn done_without_a_commit_is_refused_for_an_agent_claim() {
        let (tm, _tmp) = manager().await;
        let t = tm
            .new_open_task("A", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new");
        force_planned(&tm, &t.id);
        tm.claim_task(&t.id, &"agent:w1".to_string())
            .await
            .expect("claim");
        let err = tm
            .done_task(&t.id, "", None, &"human".to_string())
            .await
            .expect_err("needs a commit");
        assert!(matches!(err, TicketError::BadRequest(_)));
    }

    /// `server.rs::list_tasks` filters `?claimed_by=` by matching
    /// `Ticket::claimed_by` verbatim against the already-resolved id (`me` is
    /// resolved to the caller's own `PrincipalId` before this filter ever
    /// runs); this exercises that same filter directly against the field.
    #[tokio::test]
    async fn claimed_by_filters_to_the_matching_claimant() {
        let (tm, _tmp) = manager().await;
        let a = tm
            .new_open_task("A", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_open_task("B", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new b");
        force_planned(&tm, &a.id);
        force_planned(&tm, &b.id);

        tm.claim_task(&a.id, &"agent:w1".to_string())
            .await
            .expect("claim a");
        tm.claim_task(&b.id, &"human".to_string())
            .await
            .expect("claim b");

        let claimed_by = |who: &str| -> Vec<String> {
            tm.list_tasks()
                .into_iter()
                .filter(|t| t.claimed_by.as_deref() == Some(who))
                .map(|t| t.id)
                .collect()
        };
        assert_eq!(claimed_by("agent:w1"), vec![a.id.clone()]);
        assert_eq!(claimed_by("human"), vec![b.id.clone()]);
        assert!(claimed_by("agent:w2").is_empty());
    }

    #[tokio::test]
    async fn claim_by_another_agent_is_rejected() {
        let (tm, _tmp) = manager().await;
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);

        tm.claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim task");

        let err = tm
            .release_task(&task.id, &"agent:w2".to_string())
            .await
            .expect_err("not the claimant");
        assert!(matches!(err, TicketError::Conflict(_)));

        // Still claimed by w1, untouched by w2's rejected release.
        assert!(!tm.ready_tasks().iter().any(|t| t.id == task.id));
    }

    async fn claimed_task(tm: &TicketManager, agent: &str) -> Ticket {
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(tm, &task.id);
        tm.claim_task(&task.id, &format!("agent:{agent}"))
            .await
            .expect("claim task")
    }

    #[tokio::test]
    async fn dropping_a_claimed_task_releases_its_claim() {
        let (tm, _tmp) = manager().await;
        let task = claimed_task(&tm, "w1").await;

        let dropped = tm
            .drop_task(&task.id, "obsolete", &"human".to_string())
            .await
            .expect("drop claimed task");
        assert_eq!(dropped.state, TicketState::Dropped);
        assert_eq!(dropped.claimed_by, None);
        assert!(tm.store.list_claims().await.expect("claims").is_empty());
        assert!(tm.claims.lock().expect("claims lock").is_empty());
    }

    #[tokio::test]
    async fn the_lease_check_never_moves_a_task_that_is_not_claimed() {
        let (tm, _tmp) = manager().await;
        let task = claimed_task(&tm, "w1").await;
        // A stale claim on a dropped task, as older versions could leave.
        tm.cache
            .lock()
            .expect("task cache lock")
            .get_mut(&task.id)
            .expect("task in cache")
            .state = TicketState::Dropped;
        assert!(matches!(
            tm.release_claim(&task.id).await,
            Err(TicketError::Conflict(_))
        ));
        tm.tick_claim_lease_check(Utc::now() + chrono::Duration::days(1))
            .await;
        assert_eq!(
            tm.get_task(&task.id).expect("task").state,
            TicketState::Dropped
        );
    }

    #[tokio::test]
    async fn open_discards_claims_whose_task_is_not_claimed() {
        let (tm, _tmp) = manager().await;
        let task = claimed_task(&tm, "w1").await;
        tm.store
            .set_task_state(&task.id, TicketState::Dropped)
            .await
            .expect("set state");

        let tm2 = TicketManager::open(
            tm.store.clone(),
            tm.state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("reopen task manager");
        assert!(tm.store.list_claims().await.expect("claims").is_empty());
        assert!(tm2.claims.lock().expect("claims lock").is_empty());
        assert_eq!(tm2.get_task(&task.id).expect("task").claimed_by, None);
    }

    #[tokio::test]
    async fn a_stale_claim_expires_and_releases_the_task_back_to_ready() {
        let (tm, _tmp) = manager().await;
        let store = tm.store.clone();
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");
        force_planned(&tm, &task.id);

        let agent = store
            .insert_agent(NewAgent {
                name: "w1".to_string(),
                role: "worker".to_string(),
                model: "sonnet".to_string(),
                session_id: "sess-1".to_string(),
                workdir_kind: "worktree".to_string(),
                cwd: "/ws/wt/w1".to_string(),
                worktree: Some("/ws/wt/w1".to_string()),
                branch: Some("bridle/w1".to_string()),
                created_by: "human".to_string(),
                extra_allowed_tools: Vec::new(),
                extra_env: Vec::new(),
                components: Vec::new(),
            })
            .await
            .expect("insert agent");
        let last_activity = Utc::now();
        store
            .touch_agent(&agent.id, last_activity)
            .await
            .expect("touch agent");

        tm.claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim task");
        assert!(!tm.ready_tasks().iter().any(|t| t.id == task.id));

        // Well within the lease: a lease-check tick leaves the claim alone.
        tm.tick_claim_lease_check(last_activity + chrono::Duration::seconds(1))
            .await;
        assert!(!tm.ready_tasks().iter().any(|t| t.id == task.id));

        // br-4n3v: a lost (restarting) or running claimant keeps its claim however quiet.
        let late = last_activity + tm.claim_lease_after + chrono::Duration::seconds(1);
        for state in [AgentState::Lost, AgentState::Working, AgentState::Idle] {
            store
                .set_agent_state(&agent.id, state)
                .await
                .expect("set state");
            tm.tick_claim_lease_check(late).await;
            assert!(
                !tm.ready_tasks().iter().any(|t| t.id == task.id),
                "a {state:?} claimant keeps the claim"
            );
        }

        // Stopped for good and past the lease with no further activity: the tick releases it.
        store
            .set_agent_state(&agent.id, AgentState::Stopped)
            .await
            .expect("set state");
        tm.tick_claim_lease_check(
            last_activity + tm.claim_lease_after + chrono::Duration::seconds(1),
        )
        .await;
        assert!(
            tm.ready_tasks().iter().any(|t| t.id == task.id),
            "a stale claim's lease expiry re-enables readiness"
        );
    }

    /// br-4n3v: another ticket landing, and a restart, leave a worker's claim and its
    /// own `built` working.
    #[tokio::test]
    async fn a_claim_survives_another_ticket_landing_and_built_still_works() {
        let (tm, _tmp) = manager().await;
        let human = "human".to_string();
        let w = "agent:w1".to_string();
        let a = planned(&tm, "A").await;
        let b = planned(&tm, "B").await;
        tm.claim_task(&a.id, &w).await.expect("claim a");
        tm.claim_task(&b.id, &"agent:w2".to_string())
            .await
            .expect("claim b");
        tm.built_task(&b.id, "bridle/w2", "abc", &"agent:w2".to_string())
            .await
            .expect("b built");
        tm.done_task(&b.id, "abc", Some("bridle/w2"), &human)
            .await
            .expect("b lands");
        let got = tm.get_task(&a.id).expect("a");
        assert_eq!(got.state, TicketState::Claimed);
        assert_eq!(got.claimed_by.as_deref(), Some("agent:w1"));
        tm.built_task(&a.id, "bridle/w1", "def", &w)
            .await
            .expect("w1's own built");
    }

    #[tokio::test]
    async fn a_claim_survives_a_flush_and_a_fresh_manager_hydrating_from_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TicketManager::open(
            store,
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let task = tm
            .new_task_by(
                "Add foo",
                TicketKind::Feature,
                String::new(),
                vec![],
                None,
                "external:advisor",
            )
            .await
            .expect("new task");
        tm.ready_task(&task.id, &"human".to_string())
            .await
            .expect("ready");
        tm.plan_task(&task.id, &"human".to_string())
            .await
            .expect("plan");
        tm.claim_task(&task.id, &"agent:w1".to_string())
            .await
            .expect("claim");
        tm.flush_now().await.expect("flush");

        // A fresh database, over the same already-flushed state branch, as
        // `bridle daemon rebuild` runs against: unlike before durable claims, this
        // restores the claim too, not just the task.
        let fresh_store = Store::open(tmp.path().join("bridle2.db"))
            .await
            .expect("open fresh store");
        let tm2 = TicketManager::open(
            fresh_store.clone(),
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open fresh task manager");
        tm2.rebuild_from_state_branch().await.expect("rebuild");

        let rehydrated = tm2.get_task(&task.id).expect("rehydrated");
        assert_eq!(rehydrated.state, TicketState::Claimed);
        assert_eq!(rehydrated.claimed_by.as_deref(), Some("agent:w1"));
        assert!(rehydrated.claimed_at.is_some());
        assert_eq!(rehydrated.created_by, "external:advisor");
        let row = fresh_store
            .get_task(&task.id)
            .await
            .expect("row")
            .expect("row");
        assert_eq!(row.created_by.as_deref(), Some("external:advisor"));

        let db_claims = fresh_store.list_claims().await.expect("db claims");
        assert_eq!(db_claims.len(), 1);
        assert_eq!(db_claims[0].task_id, task.id);
        assert_eq!(db_claims[0].claimed_by, "agent:w1");
    }

    #[tokio::test]
    async fn impact_is_validated_state_gated_and_survives_rebuild() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TicketManager::open(
            store,
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");
        let task = tm
            .new_open_task("Add foo", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new task");

        let impact = Impact {
            modify: vec!["s-b310".to_string()],
            add_under: vec!["r-7fa2".to_string()],
            remove: vec!["a-0c".to_string()],
            files: vec!["client/**".to_string()],
        };
        tm.set_impact(&task.id, impact.clone()).await.expect("set");
        let bad = Impact {
            modify: vec!["x-12".to_string()],
            ..Impact::default()
        };
        assert!(matches!(
            tm.set_impact(&task.id, bad).await,
            Err(TicketError::BadRequest(_))
        ));
        assert_eq!(tm.get_task(&task.id).expect("task").impact, impact);

        tm.flush_now().await.expect("flush");
        let fresh_store = Store::open(tmp.path().join("bridle2.db"))
            .await
            .expect("open fresh store");
        let tm2 = TicketManager::open(
            fresh_store,
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open fresh task manager");
        tm2.rebuild_from_state_branch().await.expect("rebuild");
        assert_eq!(tm2.get_task(&task.id).expect("rebuilt").impact, impact);

        tm2.drop_task(&task.id, "no", &"human".to_string())
            .await
            .expect("drop");
        assert!(matches!(
            tm2.set_impact(&task.id, impact).await,
            Err(TicketError::Conflict(_))
        ));
    }

    #[tokio::test]
    async fn rebuild_reconstructs_tasks_edges_and_open_questions_from_the_state_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let state = StateBranch::open(&repo, &tmp.path().join("state"))
            .await
            .expect("open state branch");
        let tm = TicketManager::open(
            store,
            state.clone(),
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open task manager");

        let a = tm
            .new_open_task("A", TicketKind::Chore, "body a".to_string(), vec![], None)
            .await
            .expect("new a");
        let b = tm
            .new_open_task("B", TicketKind::Feature, "body b".to_string(), vec![], None)
            .await
            .expect("new b");
        let c = tm
            .new_open_task("C", TicketKind::Feature, String::new(), vec![], None)
            .await
            .expect("new c");
        tm.add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("add edge");

        force_planned(&tm, &c.id);
        tm.ask_question(&c.id, &"human".to_string(), "which endpoint?")
            .await
            .expect("ask c");

        force_planned(&tm, &b.id);
        tm.ask_question(&b.id, &"agent:w1".to_string(), "old question?")
            .await
            .expect("ask b");
        tm.answer_question(&b.id, &"human".to_string(), "answered")
            .await
            .expect("answer b");

        tm.flush_now().await.expect("flush");

        // A fresh, empty database over the same (already flushed) state
        // branch, as the migration story describes: clone, start the
        // daemon, `bridle daemon rebuild`.
        let fresh_store = Store::open(tmp.path().join("bridle2.db"))
            .await
            .expect("open fresh store");
        let tm2 = TicketManager::open(
            fresh_store.clone(),
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open fresh task manager");
        tm2.rebuild_from_state_branch().await.expect("rebuild");

        let mut tasks = tm2.list_tasks();
        tasks.sort_by(|x, y| x.id.cmp(&y.id));
        assert_eq!(tasks.len(), 3);

        let got_a = tasks.iter().find(|t| t.id == a.id).expect("a rebuilt");
        assert_eq!(got_a.title, "A");
        assert_eq!(got_a.body, "body a");
        assert_eq!(got_a.state, TicketState::Open);

        let got_b = tasks.iter().find(|t| t.id == b.id).expect("b rebuilt");
        assert_eq!(got_b.state, TicketState::Planned);
        assert_eq!(
            got_b.thread.len(),
            2,
            "b's question and its answer both survive"
        );
        assert!(
            !tm2.list_open_questions().iter().any(|q| q.task_id == b.id),
            "b's question was answered before the rebuild, so it has no open question"
        );

        let got_c = tasks.iter().find(|t| t.id == c.id).expect("c rebuilt");
        assert_eq!(got_c.state, TicketState::Planned);
        assert!(
            !tm2.is_ready(got_c),
            "c's open question survives the rebuild and still blocks ready"
        );

        let edges = tm2.list_edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, a.id);
        assert_eq!(edges[0].to, b.id);
        assert_eq!(edges[0].kind, EdgeKind::Blocks);

        let open_qs = tm2.list_open_questions();
        assert_eq!(open_qs.len(), 1);
        assert_eq!(open_qs[0].task_id, c.id);
        assert_eq!(open_qs[0].asked_by, "human");
        assert_eq!(open_qs[0].body, "which endpoint?");

        // The database rows themselves, not just the in-memory cache this
        // process built while rebuilding.
        let db_tasks = fresh_store.list_tasks().await.expect("db tasks");
        assert_eq!(db_tasks.len(), 3);
        let db_edges = fresh_store.list_edges().await.expect("db edges");
        assert_eq!(db_edges.len(), 1);
        let db_open_qs = fresh_store
            .list_open_questions()
            .await
            .expect("db open questions");
        assert_eq!(db_open_qs.len(), 1);
        assert_eq!(db_open_qs[0].task_id, c.id);
    }

    #[tokio::test]
    async fn rebuild_restores_handovers_with_their_seq_and_new_notes_do_not_collide() {
        let (tm, tmp) = manager().await;
        let store = tm.store.clone();
        for body in ["one", "two", "three"] {
            let h = store
                .insert_handover("orchestrator", "p", body, "human")
                .await
                .expect("insert");
            tm.enqueue_handover(&h).expect("enqueue");
        }
        tm.flush_now().await.expect("flush");

        let fresh = Store::open(tmp.path().join("fresh.db"))
            .await
            .expect("store");
        let state = StateBranch::open(&tmp.path().join("repo"), &tmp.path().join("state"))
            .await
            .expect("state");
        let tm2 = TicketManager::open(
            fresh.clone(),
            state,
            "tw".to_string(),
            std::time::Duration::from_secs(600),
        )
        .await
        .expect("open");
        tm2.rebuild_from_state_branch().await.expect("rebuild");
        let list = fresh.list_handovers().await.expect("list");
        let ids: Vec<_> = list.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, ["h-0003", "h-0002", "h-0001"]);
        assert_eq!(list[0].body, "three");
        let next = fresh
            .insert_handover("orchestrator", "p", "four", "human")
            .await
            .expect("insert");
        assert_eq!(next.id, "h-0004");
    }

    #[tokio::test]
    async fn backfill_writes_only_the_missing_notes() {
        let (tm, _tmp) = manager().await;
        let h1 = tm
            .store
            .insert_handover("orchestrator", "p", "one", "human")
            .await
            .expect("insert");
        tm.backfill_handovers().await.expect("backfill");
        tm.flush_now().await.expect("flush");
        assert!(tm.state.has_handover(&h1.id));
        // Nothing missing: nothing pending, so no new commit.
        tm.backfill_handovers().await.expect("backfill again");
        tm.flush_now().await.expect("flush");
        assert_eq!(tm.state.list_handovers().expect("list").len(), 1);
    }

    #[tokio::test]
    async fn rebuild_refuses_against_an_already_populated_database() {
        let (tm, _tmp) = manager().await;
        tm.new_open_task("A", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new a");
        tm.flush_now().await.expect("flush");

        let err = tm
            .rebuild_from_state_branch()
            .await
            .expect_err("must refuse to rebuild over an already-populated database");
        assert!(matches!(err, TicketError::Conflict(_)));

        // Untouched: still exactly the one task from before the refused
        // rebuild attempt.
        assert_eq!(tm.list_tasks().len(), 1);
    }

    // ---------- settle period (ny9u) ----------

    /// A manager with a 5 minute settle period and a planned task created
    /// `age` ago (the injected clock: shifting `created_at`/thread times back).
    async fn settling(age_mins: i64) -> (TicketManager, Ticket, tempfile::TempDir) {
        let (tm, tmp) = manager().await;
        let tm = tm.with_settle(std::time::Duration::from_secs(300));
        let t = tm
            .new_open_task("T", TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new");
        force_planned(&tm, &t.id);
        shift_back(&tm, &t.id, age_mins);
        (tm, t, tmp)
    }

    fn shift_back(tm: &TicketManager, id: &str, mins: i64) {
        let mut cache = tm.cache.lock().expect("task cache lock");
        let t = cache.get_mut(id).expect("task");
        let d = chrono::Duration::minutes(mins);
        t.created_at -= d;
        for e in &mut t.thread {
            e.at -= d;
        }
    }

    fn ready(tm: &TicketManager, id: &str) -> bool {
        tm.is_ready(&tm.get_task(id).expect("task"))
    }

    #[tokio::test]
    async fn a_new_task_is_not_ready_until_the_period_passes() {
        let (tm, t, _tmp) = settling(0).await;
        assert!(!ready(&tm, &t.id));
        let task = tm.get_task(&t.id).expect("task");
        let until = task.settle_until.expect("settling");
        assert!(tm.settle_until(&task, until).is_none());
        assert!(
            tm.settle_until(&task, until - chrono::Duration::seconds(1))
                .is_some()
        );
        // Same task, but created 6 minutes ago.
        let (tm, t, _tmp) = settling(6).await;
        assert!(ready(&tm, &t.id));
        assert!(tm.get_task(&t.id).expect("task").settle_until.is_none());
    }

    #[tokio::test]
    async fn a_relayed_human_word_restarts_the_clock_and_a_plain_agent_note_does_not() {
        let (tm, t, _tmp) = settling(6).await;
        tm.require_settled(&t.id).expect("settled");
        tm.note_task(&t.id, &agent(), "fyi, tests pass")
            .await
            .expect("agent note");
        tm.require_settled(&t.id).expect("still settled");
        tm.note_task(&t.id, &agent(), "human via advisor: keep it in the backlog")
            .await
            .expect("relay");
        let err = tm.require_settled(&t.id).expect_err("held");
        assert!(err.to_string().contains("settling until"), "{err}");

        let (tm, t, _tmp) = settling(6).await;
        tm.note_task(&t.id, &agent(), "From the human, via advisor: wait")
            .await
            .expect("relay");
        assert!(tm.require_settled(&t.id).is_err());
    }

    #[tokio::test]
    async fn only_a_human_comment_or_edit_restarts_the_clock() {
        let (tm, t, _tmp) = settling(6).await;
        tm.note_task(&t.id, &agent(), "fyi")
            .await
            .expect("agent note");
        assert!(ready(&tm, &t.id));
        tm.note_task(&t.id, &"human".to_string(), "wait")
            .await
            .expect("human note");
        assert!(!ready(&tm, &t.id));

        let (tm, t, _tmp) = settling(6).await;
        tm.edit_task(&t.id, &agent(), Some("x".into()), None, None, None)
            .await
            .expect("agent edit");
        assert!(ready(&tm, &t.id));
        tm.edit_task(
            &t.id,
            &"human".to_string(),
            Some("y".into()),
            None,
            None,
            None,
        )
        .await
        .expect("human edit");
        let task = tm.get_task(&t.id).expect("task");
        assert!(task.settle_until.is_some());
        assert_eq!(task.thread.last().expect("note").from, "human");
    }

    #[tokio::test]
    async fn skip_settle_is_recorded_needs_a_reason_and_cannot_be_forged() {
        let (tm, t, _tmp) = settling(0).await;
        let by = "human".to_string();
        assert!(matches!(
            tm.skip_settle(&t.id, &by, " ").await,
            Err(TicketError::BadRequest(_))
        ));
        let forged = tm
            .note_task(&t.id, &agent(), "settle skipped by me: sure")
            .await;
        assert!(matches!(forged, Err(TicketError::BadRequest(_))));
        assert!(!ready(&tm, &t.id));
        let task = tm
            .skip_settle(&t.id, &by, "human asked")
            .await
            .expect("skip");
        assert!(task.settle_until.is_none());
        assert!(
            task.thread
                .last()
                .expect("entry")
                .body
                .contains("human asked")
        );
        assert!(ready(&tm, &t.id));
    }

    #[tokio::test]
    async fn settle_zero_turns_it_off() {
        let (tm, t, _tmp) = settling(0).await;
        assert!(!ready(&tm, &t.id));
        let tm = tm.with_settle(std::time::Duration::ZERO);
        assert!(ready(&tm, &t.id));
    }

    #[tokio::test]
    async fn claim_refuses_while_settling_but_not_for_a_human_todo() {
        let (tm, t, _tmp) = settling(0).await;
        let err = tm.claim_task(&t.id, &agent()).await.expect_err("settling");
        assert!(err.to_string().contains("settling until"), "{err}");
        tm.claim_task_unsettled(&t.id, &"human".to_string())
            .await
            .expect("human to-do claim");
        let (tm, t, _tmp) = settling(6).await;
        tm.claim_task(&t.id, &agent()).await.expect("settled claim");
    }
    async fn planned(tm: &TicketManager, title: &str) -> Ticket {
        let t = tm
            .new_open_task(title, TicketKind::Chore, String::new(), vec![], None)
            .await
            .expect("new");
        tm.plan_task(&t.id, &"human".to_string())
            .await
            .expect("plan")
    }

    #[tokio::test]
    async fn hold_blocks_claiming_until_unheld_and_is_recorded() {
        let (tm, _tmp) = manager().await;
        let human = "human".to_string();
        let t = planned(&tm, "A").await;
        let err = tm
            .hold_task(&t.id, " ", None, &human)
            .await
            .expect_err("no reason");
        assert!(matches!(err, TicketError::BadRequest(_)));
        let held = tm
            .hold_task(&t.id, "wait for design", Some("human reviews"), &human)
            .await
            .expect("hold");
        assert_eq!(held.state, TicketState::Held);
        assert_eq!(held.held_by.as_deref(), Some("human"));
        assert_eq!(held.held_reason.as_deref(), Some("wait for design"));
        assert_eq!(held.held_until.as_deref(), Some("human reviews"));
        assert!(held.held_at.is_some());
        assert!(
            held.thread
                .last()
                .expect("note")
                .body
                .starts_with("held: wait for design")
        );
        assert!(!tm.is_ready(&held));
        let err = tm
            .claim_task(&t.id, &"agent:w1".to_string())
            .await
            .expect_err("held");
        assert!(matches!(err, TicketError::Conflict(_)));
        // Only planned can be held; held cannot be done.
        assert!(matches!(
            tm.hold_task(&t.id, "again", None, &human).await,
            Err(TicketError::Conflict(_))
        ));
        assert!(matches!(
            tm.done_task(&t.id, "abc", None, &human).await,
            Err(TicketError::Conflict(_))
        ));
        let back = tm.unhold_task(&t.id, &human).await.expect("unhold");
        assert_eq!(back.state, TicketState::Planned);
        assert!(back.held_by.is_none() && back.held_reason.is_none() && back.held_at.is_none());
        assert!(tm.is_ready(&back));
        assert!(matches!(
            tm.unhold_task(&t.id, &human).await,
            Err(TicketError::Conflict(_))
        ));
        // Drop is allowed from held.
        tm.hold_task(&t.id, "again", None, &human)
            .await
            .expect("hold");
        let dropped = tm
            .drop_task(&t.id, "not needed", &human)
            .await
            .expect("drop");
        assert_eq!(dropped.state, TicketState::Dropped);
        assert!(dropped.held_by.is_none());
    }

    #[tokio::test]
    async fn built_from_claimed_and_planned_leaves_only_by_done_or_send_back() {
        let (tm, _tmp) = manager().await;
        let human = "human".to_string();
        let w = "agent:w1".to_string();
        let a = planned(&tm, "A").await;
        let b = planned(&tm, "B").await;
        tm.add_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("edge");

        // The worker's finish: claimed -> built releases the lease.
        tm.claim_task(&a.id, &w).await.expect("claim");
        assert!(matches!(
            tm.built_task(&a.id, "", "abc", &w).await,
            Err(TicketError::BadRequest(_))
        ));
        let built = tm
            .built_task(&a.id, "bridle/a", "abc123", &w)
            .await
            .expect("built");
        assert_eq!(built.state, TicketState::Built);
        assert!(built.claimed_by.is_none());
        assert_eq!(built.branch.as_deref(), Some("bridle/a"));
        assert_eq!(built.commit.as_deref(), Some("abc123"));
        assert!(tm.release_task(&a.id, &w).await.is_err(), "lease is gone");
        // Built does not resolve blocks edges and is not claimable.
        assert!(!tm.is_ready(&tm.get_task(&b.id).expect("b")));
        assert!(matches!(
            tm.claim_task(&a.id, &w).await,
            Err(TicketError::Conflict(_))
        ));
        assert!(matches!(
            tm.hold_task(&a.id, "x", None, &human).await,
            Err(TicketError::Conflict(_))
        ));
        assert!(matches!(
            tm.built_task(&a.id, "b", "c", &human).await,
            Err(TicketError::Conflict(_))
        ));
        // Send-back keeps the branch and clears the commit.
        assert!(matches!(
            tm.send_back_task(&a.id, " ", &human).await,
            Err(TicketError::BadRequest(_))
        ));
        let sent = tm
            .send_back_task(&a.id, "fix tests", &human)
            .await
            .expect("send back");
        assert_eq!(sent.state, TicketState::Planned);
        assert_eq!(sent.branch.as_deref(), Some("bridle/a"));
        assert!(sent.commit.is_none());
        assert!(
            sent.thread
                .last()
                .expect("note")
                .body
                .contains("sent back: fix tests")
        );
        assert!(matches!(
            tm.send_back_task(&a.id, "again", &human).await,
            Err(TicketError::Conflict(_))
        ));
        // Parked work: planned -> built, then landed.
        let parked = tm
            .built_task(&a.id, "bridle/a", "def456", &human)
            .await
            .expect("parked");
        assert_eq!(parked.state, TicketState::Built);
        let done = tm
            .done_task(&a.id, "def456", Some("bridle/a"), &human)
            .await
            .expect("done");
        assert_eq!(done.state, TicketState::Integrated);
        assert!(tm.is_ready(&tm.get_task(&b.id).expect("b")));
        // Drop is allowed from built.
        tm.claim_task(&b.id, &w).await.expect("claim b");
        tm.built_task(&b.id, "bridle/b", "bbb", &w)
            .await
            .expect("built b");
        let dropped = tm.drop_task(&b.id, "obsolete", &human).await.expect("drop");
        assert_eq!(dropped.state, TicketState::Dropped);
    }

    #[tokio::test]
    async fn migration_moves_planned_with_branch_and_commit_to_built_once() {
        let (tm, _tmp) = manager().await;
        let human = "human".to_string();
        let with = planned(&tm, "with").await;
        let without = planned(&tm, "without").await;
        let branch_only = planned(&tm, "branch only").await;
        // A planned task that carries a landing record (as older builds left them).
        for (id, branch, commit) in [
            (&with.id, Some("bridle/w"), Some("abc")),
            (&branch_only.id, Some("bridle/b"), None),
        ] {
            let mut t = tm.get_task(id).expect("task");
            t.branch = branch.map(str::to_string);
            t.commit = commit.map(str::to_string);
            tm.put(t);
        }
        let moved = tm
            .migrate_planned_with_branch_to_built()
            .await
            .expect("migrate");
        assert_eq!(moved, vec![with.id.clone()]);
        assert_eq!(tm.get_task(&with.id).expect("t").state, TicketState::Built);
        assert_eq!(
            tm.get_task(&without.id).expect("t").state,
            TicketState::Planned
        );
        assert_eq!(
            tm.get_task(&branch_only.id).expect("t").state,
            TicketState::Planned
        );
        // Rerun is a no-op.
        assert!(
            tm.migrate_planned_with_branch_to_built()
                .await
                .expect("rerun")
                .is_empty()
        );
        // The state is durable in the database, not just the cache.
        assert_eq!(
            tm.store
                .list_tasks()
                .await
                .expect("rows")
                .iter()
                .find(|r| r.id == with.id)
                .expect("row")
                .state,
            TicketState::Built
        );
        let _ = human;
    }
}
