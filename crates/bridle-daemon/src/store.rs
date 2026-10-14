//! SQLite-backed store for principals, agents, turns, messages, events and
//! rate limits. See docs/design/storage.md for the schema this implements
//! and docs/design/agent-host/principals.md for principals/tokens.
//!
//! `Store` is a thin, cloneable async handle: every public method runs its
//! SQL on a single connection inside `spawn_blocking`, guarded by a mutex
//! (CLAUDE.md: never block the runtime). The actual SQL lives in the
//! private `sync` module, which is plain, synchronous and independently
//! testable without an async runtime.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bridle_api::types::{
    Agent, AgentState, AgentUsage, Conflict, Edge, EdgeKind, Event, EventQuery, ExitInfo,
    ExternalUsage, Handover, InteractiveUsageRow, Message, MessageKind, MessageState, OutboxEntry,
    OutboxPeer, PortAllocation, PrincipalId, PrincipalKind, RateLimit, RateLimitPoint, Schedule,
    TicketKind, TicketState, TokenCreated, TokenInfo, TokenTotals, Usage, UsageBreakdown,
    UsageBucket, UsageGroup, UsageGroupBy, UsageInstance, UsageSeries, UsageSeriesPoint, When,
};
use chrono::{DateTime, SecondsFormat, SubsecRound, Utc};
use rusqlite::Connection;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// The runtime cancelled a queued blocking task during shutdown
    /// (`JoinError::is_cancelled()`), not a bug: the caller should treat
    /// this like the daemon is unavailable, not a 500.
    #[error("store is shutting down")]
    ShuttingDown,
    /// The database was written by a newer bridle (a copied or restored file, or a downgraded
    /// binary); running on it could corrupt it.
    #[error(
        "the database is schema version {found}, newer than this bridle's {known}: upgrade \
         bridle before opening it"
    )]
    SchemaTooNew { found: usize, known: usize },
}

/// One session's usage in a period (`Store::usage_sessions`): the base rows of the
/// role/name/session grouping. A hosted session is `(agent_id, session_id)`; an external one
/// has no `agent_id` and its name is its identity.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionUsage {
    pub role: String,
    pub name: String,
    /// Empty for turns from before sessions were recorded, and for an external report with none.
    pub session_id: String,
    pub agent_id: Option<String>,
    /// `None` for an external session.
    pub turns: Option<u32>,
    pub tokens: TokenTotals,
    pub cost_usd: f64,
    pub first_context: Option<i64>,
    pub last_context: Option<i64>,
}

/// What a `project stop` leaves in this machine's database (53gy B): the database does not move.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct LeftBehind {
    pub undelivered_mail: i64,
    pub unread_messages: i64,
    pub active_schedules: i64,
    pub agents: i64,
    pub events: i64,
}

/// Counts what stays behind, reading `db` without opening a write or migrating it. `None` when
/// there is no database; a table an older schema lacks counts as 0.
pub fn left_behind(db: &std::path::Path) -> Option<LeftBehind> {
    let conn = Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    let count = |sql: &str| conn.query_row(sql, [], |r| r.get(0)).unwrap_or(0);
    Some(LeftBehind {
        undelivered_mail: count("SELECT COUNT(*) FROM outbox WHERE state = 'queued'"),
        unread_messages: count(
            "SELECT COUNT(*) FROM messages WHERE state NOT IN ('read', 'dropped')",
        ),
        active_schedules: count("SELECT COUNT(*) FROM schedules WHERE state = 'active'"),
        agents: count("SELECT COUNT(*) FROM agents"),
        events: count("SELECT COUNT(*) FROM events"),
    })
}

/// What in `db` makes a project unstable for a move (br-mzy4): claimed or built tickets,
/// running agents, undelivered mail. Read-only like `left_behind`; a missing database or table
/// counts as empty.
pub fn unstable_in_db(db: &std::path::Path) -> Vec<String> {
    let Ok(conn) = Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    else {
        return Vec::new();
    };
    let col = |sql: &str| -> Vec<String> {
        conn.prepare(sql)
            .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect())
            .unwrap_or_default()
    };
    let mut bad = Vec::new();
    for state in ["claimed", "built"] {
        let t = col(&format!(
            "SELECT id FROM tasks WHERE state = '{state}' ORDER BY id"
        ));
        if !t.is_empty() {
            bad.push(format!(
                "{state} ticket(s) not landed or dropped: {}",
                t.join(", ")
            ));
        }
    }
    let a = col(
        "SELECT name FROM agents WHERE state IN ('starting','idle','working','stopping') ORDER BY name",
    );
    if !a.is_empty() {
        bad.push(format!("agent(s) running: {}", a.join(", ")));
    }
    let m = col("SELECT CAST(id AS TEXT) FROM outbox WHERE state = 'queued'");
    if !m.is_empty() {
        bad.push(format!(
            "{} undelivered message(s) in the outbox (something is wrong; find out why before moving)",
            m.len()
        ));
    }
    bad
}

/// The active scheduled messages in `db`, read-only like `left_behind`: `project stop` carries
/// them to the next owner. Empty when there is no database or no such table.
pub fn active_schedules(db: &std::path::Path) -> Vec<Schedule> {
    Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .ok()
        .and_then(|conn| sync::schedules_list(&conn, None, false).ok())
        .unwrap_or_default()
}

/// Turns a `spawn_blocking` `JoinError` into a `StoreError`, or resumes the
/// unwind if the blocking task genuinely panicked (so real bugs still
/// surface as panics rather than being swallowed as `ShuttingDown`).
fn join_error(e: tokio::task::JoinError) -> StoreError {
    match e.try_into_panic() {
        Ok(payload) => std::panic::resume_unwind(payload),
        Err(_) => StoreError::ShuttingDown,
    }
}

/// An authenticated caller. Not a wire type (see `bridle_api::types`):
/// tokens never leave the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub id: PrincipalId,
    pub kind: PrincipalKind,
}

/// An agent's persisted `--allow-tool`/`--env` overrides: the grants and
/// `KEY=VALUE` pairs from its spawn.
pub type AgentOverrides = (Vec<String>, Vec<(String, String)>);

#[derive(Debug, Clone)]
pub struct NewAgent {
    pub name: String,
    pub role: String,
    pub model: String,
    pub session_id: String,
    /// `"worktree"`, `"repo"` or `"path"`.
    pub workdir_kind: String,
    pub cwd: String,
    pub worktree: Option<String>,
    pub branch: Option<String>,
    pub created_by: PrincipalId,
    /// This spawn's `--allow-tool`/`--env` overrides (spike 2ty9/k8dw),
    /// persisted so `renew` and `resume` can reapply them.
    pub extra_allowed_tools: Vec<String>,
    pub extra_env: Vec<(String, String)>,
    pub components: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TurnEnd {
    pub subtype: String,
    pub is_error: bool,
    pub terminal_reason: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    /// This turn's own cost (not the session's cumulative counter); added
    /// to `agents.cost_usd_total` (docs/design/usage-and-budget.md).
    pub cost_total: f64,
    /// `input_tokens + cache_read + cache_write` for this turn: the context
    /// size claude reported at the end of it. Overwrites
    /// `agents.context_tokens`, unlike `cost_total` (agents.md).
    pub context_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningAgent {
    pub id: String,
    /// As stored, e.g. `working`.
    pub state: String,
    pub pid: Option<i32>,
    pub pid_start: Option<String>,
}

/// The fast-index row for a task: `id, title, kind, state, created_at,
/// updated_at` (storage.md). The body and thread live only on the state
/// branch; see `crate::tasks::TicketManager`, which combines this with a file
/// read to answer `show`/`list`.
#[derive(Debug, Clone, PartialEq)]
pub struct TicketRow {
    pub id: String,
    pub title: String,
    pub kind: TicketKind,
    pub state: TicketState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// `None` = creator unknown (a task from before the column, nothing to recover it from).
    pub created_by: Option<String>,
}

/// A message's concrete recipient kind (`messages.to_kind`). The caller
/// states this explicitly rather than the store guessing it from `to`'s
/// shape, since a task id and an agent id are both opaque strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipientKind {
    Human,
    Agent,
    Ticket,
    External,
}

impl RecipientKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Agent => "agent",
            Self::Ticket => "task",
            Self::External => "external",
        }
    }
}

/// One message waiting in (or finished with) this daemon's outbox (3haz).
#[derive(Debug, Clone, PartialEq)]
pub struct OutboxRow {
    /// `o-0007`.
    pub id: String,
    /// The destination daemon's project; one queue per destination.
    pub project: String,
    pub from: PrincipalId,
    pub to: String,
    pub kind: MessageKind,
    pub body: String,
    pub reply_to: Option<String>,
    pub when: When,
    pub attempts: i64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewMessage {
    pub from: PrincipalId,
    /// Concrete recipient: `"human"`, an agent id, or a task id.
    /// Name/`"me"` resolution happens above the store.
    pub to: String,
    pub to_kind: RecipientKind,
    pub kind: MessageKind,
    pub body: String,
    pub reply_to: Option<String>,
    pub when: When,
    /// The message's initial state (`Pending`, `Held` or `Written`,
    /// depending on whether it was delivered immediately). Timestamps for
    /// later transitions are set through `set_message_state`, not here.
    pub state: MessageState,
}

/// The fast-index row for an open question (storage.md, "Questions aren't a
/// separate `questions/…` folder"): enough to list a task's open question in
/// `bridle inbox` without walking the state branch. `message_id` is a
/// pointer into `messages`, which already carries the body, so this table
/// doesn't duplicate it.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenQuestion {
    pub task_id: String,
    pub message_id: String,
    pub asked_by: PrincipalId,
    pub asked_at: DateTime<Utc>,
}

/// The fast-index row for a claim (storage.md, "claims"), mirrored to the
/// state branch's `claims.toml` so `bridle rebuild` can restore it; the
/// ephemeral tables `waits`, `ports`, `impact_cache` still arrive with later
/// tasks. `task_id` is the primary key: a task has at most one claimant at a
/// time. Lease expiry is computed live from the claiming agent's own
/// activity (`agents.last_event_at`/`turn_started_at`), not stored here, so
/// there's nothing to renew on a tick.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub task_id: String,
    pub claimed_by: PrincipalId,
    pub claimed_at: DateTime<Utc>,
}

/// A resolved message query: `to`/`from` are concrete ids, already picked
/// apart from the wire `MessageQuery`'s `"me"` / name lookups.
#[derive(Debug, Clone, Default)]
pub struct ListMessages {
    pub to: Option<String>,
    pub from: Option<String>,
    /// Exclude `read` and `dropped` messages.
    pub unread: bool,
    /// Returns the most recent `limit` messages, oldest first.
    pub limit: Option<u32>,
}

#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

impl Store {
    /// Opens (creating if needed) the SQLite database at `path` in WAL
    /// mode with foreign keys on, and runs any pending migrations.
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let path = path.into();
        let conn = tokio::task::spawn_blocking(move || sync::open(&path))
            .await
            .map_err(join_error)??;
        Ok(Store {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    async fn with_conn<T, F>(&self, f: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, StoreError> + Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().expect("store connection mutex poisoned");
            f(&conn)
        })
        .await
        .map_err(join_error)?
    }

    // ---------- principals / tokens ----------

    pub async fn ensure_human_token(&self) -> Result<Option<String>, StoreError> {
        self.with_conn(sync::ensure_human_token).await
    }

    pub async fn create_external_token(
        &self,
        name: &str,
        home: Option<&str>,
    ) -> Result<TokenCreated, StoreError> {
        let name = name.to_string();
        let home = home.map(str::to_string);
        self.with_conn(move |c| sync::create_external_token(c, &name, home.as_deref()))
            .await
    }

    /// The distinct homes of every visitor token named `<name>@<machine>` that has one.
    pub async fn visitor_homes_named(&self, name: &str) -> Result<Vec<String>, StoreError> {
        let like = format!("external:{name}@%");
        self.with_conn(move |c| sync::visitor_homes_like(c, &like))
            .await
    }

    /// The project a visitor's mail is forwarded to; `None` for a token minted without one.
    pub async fn visitor_home(&self, principal_id: &str) -> Result<Option<String>, StoreError> {
        let id = principal_id.to_string();
        self.with_conn(move |c| sync::visitor_home(c, &id)).await
    }

    /// `agent_id` is accepted for interface symmetry with the rest of the
    /// agent lifecycle calls and for future audit use; the principal itself
    /// is keyed by name (`agent:<name>`, principals.md), like
    /// every other agent-facing reference.
    pub async fn create_agent_token(
        &self,
        agent_id: &str,
        agent_name: &str,
    ) -> Result<String, StoreError> {
        let _ = agent_id;
        let agent_name = agent_name.to_string();
        self.with_conn(move |c| sync::create_agent_token(c, &agent_name))
            .await
    }

    pub async fn authenticate(&self, token: &str) -> Result<Option<Principal>, StoreError> {
        let token = token.to_string();
        self.with_conn(move |c| sync::authenticate(c, &token)).await
    }

    /// Mints `peer:<machine>`; a name already in use is a conflict, like any token.
    pub async fn create_peer_token(
        &self,
        machine: &str,
        replace: bool,
    ) -> Result<TokenCreated, StoreError> {
        let machine = machine.to_string();
        self.with_conn(move |c| sync::create_peer_token(c, &machine, replace))
            .await
    }

    /// Stores a new schedule and returns it with its id.
    pub async fn schedule_add(&self, s: Schedule) -> Result<Schedule, StoreError> {
        self.with_conn(move |c| sync::schedule_add(c, s)).await
    }

    /// `involving` limits to schedules that principal created or that target it; `all`
    /// includes `done` ones.
    pub async fn schedules_list(
        &self,
        involving: Option<&str>,
        all: bool,
    ) -> Result<Vec<Schedule>, StoreError> {
        let involving = involving.map(str::to_string);
        self.with_conn(move |c| sync::schedules_list(c, involving.as_deref(), all))
            .await
    }

    pub async fn schedule_get(&self, id: &str) -> Result<Option<Schedule>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::schedule_get(c, &id)).await
    }

    /// True when it existed.
    pub async fn schedule_remove(&self, id: &str) -> Result<bool, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::schedule_remove(c, &id)).await
    }

    /// Active schedules whose `next_fire_at` is at or before `now`.
    pub async fn schedules_due(&self, now: DateTime<Utc>) -> Result<Vec<Schedule>, StoreError> {
        self.with_conn(move |c| sync::schedules_due(c, now)).await
    }

    /// Records a firing: `next` is the next firing, or none to end a `once`.
    pub async fn schedule_fired(
        &self,
        id: &str,
        at: DateTime<Utc>,
        next: Option<DateTime<Utc>>,
    ) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::schedule_fired(c, &id, at, next))
            .await
    }

    /// Adds a message to the outbox as `queued`; returns its id.
    pub async fn outbox_enqueue(&self, row: OutboxRow) -> Result<String, StoreError> {
        self.with_conn(move |c| sync::outbox_enqueue(c, &row)).await
    }

    /// The highest outbox id number this database ever issued (0 for none), kept by SQLite's
    /// AUTOINCREMENT even for rows since deleted. `project stop` carries it to the next owner.
    pub async fn outbox_high_water(&self) -> Result<i64, StoreError> {
        self.with_conn(|c| {
            Ok(c.query_row(
                "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name = 'outbox'), 0)",
                [],
                |r| r.get(0),
            )?)
        })
        .await
    }

    /// Makes the next outbox id come after number `seq`, so a fresh database does not reissue
    /// ids a receiver has already seen (it dedups on machine, daemon and id). Never lowers it.
    pub async fn outbox_seed_counter(&self, seq: i64) -> Result<(), StoreError> {
        self.with_conn(move |c| {
            let n = c.execute(
                "UPDATE sqlite_sequence SET seq = MAX(seq, ?1) WHERE name = 'outbox'",
                rusqlite::params![seq],
            )?;
            if n == 0 {
                c.execute(
                    "INSERT INTO sqlite_sequence(name, seq) VALUES('outbox', ?1)",
                    rusqlite::params![seq],
                )?;
            }
            Ok(())
        })
        .await
    }

    /// The oldest still-queued message for `project`.
    pub async fn outbox_next(&self, project: &str) -> Result<Option<OutboxRow>, StoreError> {
        let project = project.to_string();
        self.with_conn(move |c| sync::outbox_next(c, &project))
            .await
    }

    /// Records one try: `Ok(ids)` marks it delivered (the receiver's message ids), `Err(e)` with
    /// `permanent` marks it failed, and without it leaves it queued for the next try.
    pub async fn outbox_finish(
        &self,
        id: &str,
        outcome: Result<Vec<String>, (String, bool)>,
    ) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::outbox_finish(c, &id, &outcome))
            .await
    }

    /// Destinations that have something queued.
    pub async fn outbox_queued_projects(&self) -> Result<Vec<String>, StoreError> {
        self.with_conn(sync::outbox_queued_projects).await
    }

    /// `(state, last_error)` of an outbox row.
    pub async fn outbox_info(
        &self,
        id: &str,
    ) -> Result<Option<(String, Option<String>)>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::outbox_info(c, &id)).await
    }

    /// Queued entries older than `cutoff` whose sender has not been told yet.
    pub async fn outbox_stuck(&self, cutoff: DateTime<Utc>) -> Result<Vec<OutboxRow>, StoreError> {
        self.with_conn(move |c| sync::outbox_stuck(c, cutoff)).await
    }

    pub async fn outbox_row(&self, id: &str) -> Result<Option<OutboxRow>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::outbox_row(c, &id)).await
    }

    /// Entries queued since before `cutoff` that have not yet been reported to the aide.
    pub async fn outbox_aide_due(
        &self,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<OutboxRow>, StoreError> {
        self.with_conn(move |c| sync::outbox_aide_due(c, cutoff))
            .await
    }

    pub async fn outbox_mark_aide(&self, id: &str) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::outbox_mark_aide(c, &id))
            .await
    }

    /// One line per destination with mail queued.
    pub async fn outbox_peers(&self) -> Result<Vec<OutboxPeer>, StoreError> {
        self.with_conn(sync::outbox_peers).await
    }

    /// An outbox row as the sender sees it; `arrived` is not decided here (the destination
    /// knows).
    pub async fn outbox_entry(&self, id: &str) -> Result<Option<OutboxEntry>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::outbox_entry(c, &id)).await
    }

    pub async fn outbox_mark_stuck(&self, id: &str) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::outbox_mark_stuck(c, &id))
            .await
    }

    /// `(state, remote ids)` of an outbox row: `queued`, `delivered` or `failed`.
    pub async fn outbox_state(
        &self,
        id: &str,
    ) -> Result<Option<(String, Vec<String>)>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::outbox_state(c, &id)).await
    }

    /// The ids a forwarded message was stored as, if this origin's message was seen before.
    pub async fn forwarded_seen(
        &self,
        origin: &(String, String, String),
    ) -> Result<Option<Vec<String>>, StoreError> {
        let origin = origin.clone();
        self.with_conn(move |c| sync::forwarded_seen(c, &origin))
            .await
    }

    pub async fn forwarded_record(
        &self,
        origin: &(String, String, String),
        ids: &[String],
    ) -> Result<(), StoreError> {
        let (origin, ids) = (origin.clone(), ids.to_vec());
        self.with_conn(move |c| sync::forwarded_record(c, &origin, &ids))
            .await
    }

    pub async fn revoke_principal(&self, id: &str) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::revoke_principal(c, &id))
            .await
    }

    pub async fn list_external_tokens(&self) -> Result<Vec<TokenInfo>, StoreError> {
        self.with_conn(sync::list_external_tokens).await
    }

    /// Whether an active `external:<name>` principal exists (minted with
    /// `bridle token create` and not since revoked) — used to validate
    /// `bridle send external:<name>` without needing the token itself.
    pub async fn external_exists(&self, name: &str) -> Result<bool, StoreError> {
        let name = name.to_string();
        self.with_conn(move |c| sync::external_exists(c, &name))
            .await
    }

    /// Re-addresses `from`'s unread messages whose body starts with `mark` to `to`, taking
    /// `mark` off. Moves nothing the second time, so it is safe to repeat.
    pub async fn move_marked_unread_messages(
        &self,
        from: &str,
        to: &str,
        mark: &str,
    ) -> Result<usize, StoreError> {
        let (from, to, mark) = (from.to_string(), to.to_string(), mark.to_string());
        self.with_conn(move |c| sync::move_marked_unread_messages(c, &from, &to, &mark))
            .await
    }

    /// Marks read the unread restart requests and context-limit notes sent to `to`: once the
    /// session has handed over they are stale, and identities outlive sessions, so the next
    /// session would otherwise be handed them.
    pub async fn close_handover_requests(&self, to: &str) -> Result<usize, StoreError> {
        let to = to.to_string();
        self.with_conn(move |c| sync::close_handover_requests(c, &to))
            .await
    }

    /// Revokes `external:<name>`, failing with `NotFound` if no such
    /// external-token principal exists (an agent's own token isn't revoked
    /// this way; that happens through `rm`).
    pub async fn revoke_external_token(&self, name: &str) -> Result<(), StoreError> {
        let name = name.to_string();
        self.with_conn(move |c| sync::revoke_external_token(c, &name))
            .await
    }

    // ---------- agents ----------

    pub async fn insert_agent(&self, new: NewAgent) -> Result<Agent, StoreError> {
        self.with_conn(move |c| sync::insert_agent(c, &new)).await
    }

    pub async fn get_agent(&self, id_or_name: &str) -> Result<Option<Agent>, StoreError> {
        let id_or_name = id_or_name.to_string();
        self.with_conn(move |c| sync::get_agent(c, &id_or_name))
            .await
    }

    pub async fn get_agent_overrides(&self, id: &str) -> Result<AgentOverrides, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::get_agent_overrides(c, &id))
            .await
    }

    pub async fn list_agents(&self, include_terminal: bool) -> Result<Vec<Agent>, StoreError> {
        self.with_conn(move |c| sync::list_agents(c, include_terminal))
            .await
    }

    pub async fn set_agent_state(&self, id: &str, state: AgentState) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::set_agent_state(c, &id, state))
            .await
    }

    pub async fn set_agent_process(
        &self,
        id: &str,
        pid: Option<i32>,
        pid_start: Option<String>,
    ) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::set_agent_process(c, &id, pid, pid_start.as_deref()))
            .await
    }

    /// Replaces the agent's Claude Code session id, e.g. when `renew` starts
    /// a fresh session in the same worktree/branch instead of resuming the
    /// old one.
    /// Records that the agent's current session has started a turn (so it
    /// exists on disk and `--resume` can find it).
    pub async fn mark_session_started(&self, id: &str) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::mark_session_started(c, &id))
            .await
    }

    pub async fn session_started(&self, id: &str) -> Result<bool, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::session_started(c, &id)).await
    }

    pub async fn set_agent_session(&self, id: &str, session_id: &str) -> Result<(), StoreError> {
        let id = id.to_string();
        let session_id = session_id.to_string();
        self.with_conn(move |c| sync::set_agent_session(c, &id, &session_id))
            .await
    }

    /// Records a hosted session starting (`reason`: spawn, resume or renewal) with the context
    /// renewal limit in force.
    pub async fn session_start(
        &self,
        agent_id: &str,
        session_id: &str,
        reason: &str,
        renewal_limit: f64,
    ) -> Result<(), StoreError> {
        let (a, s, r) = (
            agent_id.to_string(),
            session_id.to_string(),
            reason.to_string(),
        );
        self.with_conn(move |c| sync::session_start(c, &a, &s, &r, renewal_limit))
            .await
    }

    /// Records the agent's open session ending, with its context size at the time.
    pub async fn session_end(
        &self,
        agent_id: &str,
        reason: &str,
        tokens: Option<i64>,
    ) -> Result<(), StoreError> {
        let (a, r) = (agent_id.to_string(), reason.to_string());
        self.with_conn(move |c| sync::session_end(c, &a, &r, tokens))
            .await
    }

    /// The claude session's own running `total_cost_usd` as of its last turn (0 for a fresh
    /// session): the baseline the next turn's cost delta is taken against.
    pub async fn session_cost(&self, id: &str) -> Result<f64, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::session_cost(c, &id)).await
    }

    pub async fn set_session_cost(&self, id: &str, cost: f64) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::set_session_cost(c, &id, cost))
            .await
    }

    pub async fn set_agent_exit(&self, id: &str, exit: ExitInfo) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::set_agent_exit(c, &id, &exit))
            .await
    }

    pub async fn touch_agent(
        &self,
        id: &str,
        last_event_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::touch_agent(c, &id, last_event_at))
            .await
    }

    pub async fn add_turn_start(
        &self,
        id: &str,
        n: u32,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::add_turn_start(c, &id, n, at))
            .await
    }

    pub async fn end_turn(&self, id: &str, n: u32, turn: TurnEnd) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::end_turn(c, &id, n, &turn))
            .await
    }

    pub async fn delete_agent(&self, id: &str) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::delete_agent(c, &id)).await
    }

    pub async fn running_agents(&self) -> Result<Vec<RunningAgent>, StoreError> {
        self.with_conn(sync::running_agents).await
    }

    // ---------- messages ----------

    pub async fn insert_message(&self, new: NewMessage) -> Result<Message, StoreError> {
        self.with_conn(move |c| sync::insert_message(c, &new)).await
    }

    pub async fn get_message(&self, id: &str) -> Result<Option<Message>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::get_message(c, &id)).await
    }

    pub async fn list_messages(&self, query: ListMessages) -> Result<Vec<Message>, StoreError> {
        self.with_conn(move |c| sync::list_messages(c, &query))
            .await
    }

    pub async fn set_message_state(
        &self,
        id: &str,
        state: MessageState,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::set_message_state(c, &id, state, at))
            .await
    }

    /// Marks `id` read and answered by `by` with message `reply` (whose first line is `line`); does nothing if it's
    /// already read.
    pub async fn answer_message(
        &self,
        id: &str,
        by: &str,
        reply: &str,
        line: &str,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let (id, by, reply, line) = (
            id.to_string(),
            by.to_string(),
            reply.to_string(),
            line.to_string(),
        );
        self.with_conn(move |c| sync::answer_message(c, &id, &by, &reply, &line, at))
            .await
    }

    /// Like `set_message_state(id, Written, at)`, but only applies over
    /// `pending`: a message's replay confirmation (`Delivered`) can land
    /// concurrently from the agent's own event stream, on a separate task
    /// from whichever caller wrote the text to stdin, and a plain
    /// unconditional `Written` write can lose that race and land *after*,
    /// silently downgrading `Delivered` back to `Written` forever (nothing
    /// re-checks a message once its replay has already been matched and
    /// removed from the fifo). A no-op, not an error, when it doesn't
    /// apply — `Held`/`Pending`/`Delivered` already reaching the caller
    /// with a different state is an expected outcome here, not a bug.
    pub async fn mark_message_written(
        &self,
        id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::mark_message_written(c, &id, at))
            .await
    }

    pub async fn messages_for_agent(
        &self,
        agent_id: &str,
        states: &[MessageState],
    ) -> Result<Vec<Message>, StoreError> {
        let agent_id = agent_id.to_string();
        let states = states.to_vec();
        self.with_conn(move |c| sync::messages_for_agent(c, &agent_id, &states))
            .await
    }

    /// Links message `id` to the incident `task` it announces.
    pub async fn set_message_incident(&self, id: &str, task: &str) -> Result<(), StoreError> {
        let (id, task) = (id.to_string(), task.to_string());
        self.with_conn(move |c| {
            c.execute(
                "UPDATE messages SET incident_task = ?1 WHERE id = ?2",
                rusqlite::params![task, id],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn messages_for_incident(&self, task: &str) -> Result<Vec<Message>, StoreError> {
        let task = task.to_string();
        self.with_conn(move |c| sync::messages_for_incident(c, &task))
            .await
    }

    /// Rewrites the body of message `id` while it's still `pending` or `held`;
    /// reports whether it did.
    pub async fn rewrite_unwritten_body(&self, id: &str, body: &str) -> Result<bool, StoreError> {
        let (id, body) = (id.to_string(), body.to_string());
        self.with_conn(move |c| {
            let n = c.execute(
                "UPDATE messages SET body = ?1 WHERE id = ?2 AND state IN ('pending', 'held')",
                rusqlite::params![body, id],
            )?;
            Ok(n > 0)
        })
        .await
    }

    /// Unlinks every notice for `task`, so a later activation starts clean.
    pub async fn clear_incident_links(&self, task: &str) -> Result<(), StoreError> {
        let task = task.to_string();
        self.with_conn(move |c| {
            c.execute(
                "UPDATE messages SET incident_task = NULL WHERE incident_task = ?1",
                rusqlite::params![task],
            )?;
            Ok(())
        })
        .await
    }

    // ---------- events ----------

    pub async fn append_event(
        &self,
        kind: &str,
        actor: PrincipalId,
        agent: Option<String>,
        data: serde_json::Value,
    ) -> Result<Event, StoreError> {
        let kind = kind.to_string();
        self.with_conn(move |c| sync::append_event(c, &kind, &actor, agent.as_deref(), &data))
            .await
    }

    pub async fn list_events(&self, query: EventQuery) -> Result<Vec<Event>, StoreError> {
        self.with_conn(move |c| sync::list_events(c, &query)).await
    }

    pub async fn prune_events(&self, older_than: DateTime<Utc>) -> Result<u64, StoreError> {
        self.with_conn(move |c| sync::prune_events(c, older_than))
            .await
    }

    // ---------- tasks ----------

    /// Inserts a new task with a fresh id (`<prefix>-<4 hex chars>`,
    /// storage.md), retrying on the rare id collision.
    pub async fn insert_task(
        &self,
        prefix: &str,
        title: &str,
        kind: TicketKind,
        created_by: &str,
    ) -> Result<TicketRow, StoreError> {
        self.insert_task_wanting(prefix, title, kind, created_by, None)
            .await
    }

    /// [`Store::insert_task`], but with `<prefix>-<ticket>` as the id when `ticket` is given
    /// and no task has it yet (a ticket's first task takes its id; later ones fall back to a
    /// fresh id).
    pub async fn insert_task_wanting(
        &self,
        prefix: &str,
        title: &str,
        kind: TicketKind,
        created_by: &str,
        ticket: Option<&str>,
    ) -> Result<TicketRow, StoreError> {
        let (prefix, title, created_by) = (
            prefix.to_string(),
            title.to_string(),
            created_by.to_string(),
        );
        let wanted = ticket.map(|t| format!("{prefix}-{t}"));
        self.with_conn(move |c| {
            sync::insert_task(c, &prefix, &title, kind, &created_by, wanted.as_deref())
        })
        .await
    }

    pub async fn get_task(&self, id: &str) -> Result<Option<TicketRow>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::get_task(c, &id)).await
    }

    /// Inserts `row` exactly as given, rather than generating a fresh id and
    /// `created_at` the way [`Store::insert_task`] does. Only for `bridle
    /// rebuild`, reconstructing this table's rows from the state branch's
    /// own ids and timestamps.
    pub async fn insert_task_row(&self, row: &TicketRow) -> Result<(), StoreError> {
        let row = row.clone();
        self.with_conn(move |c| sync::insert_task_row(c, &row))
            .await
    }

    pub async fn list_tasks(&self) -> Result<Vec<TicketRow>, StoreError> {
        self.with_conn(sync::list_tasks).await
    }

    /// Records a creator recovered from the state branch for a row the event backfill left
    /// unknown; never overwrites a known one.
    pub async fn fill_task_created_by(&self, id: &str, by: &str) -> Result<(), StoreError> {
        let (id, by) = (id.to_string(), by.to_string());
        self.with_conn(move |c| {
            c.execute(
                "UPDATE tickets SET created_by = ?2 WHERE id = ?1 AND created_by IS NULL",
                rusqlite::params![id, by],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn set_task_title(&self, id: &str, title: &str) -> Result<(), StoreError> {
        let (id, title) = (id.to_string(), title.to_string());
        self.with_conn(move |c| sync::set_task_title(c, &id, &title))
            .await
    }

    pub async fn set_task_kind(&self, id: &str, kind: TicketKind) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::set_task_kind(c, &id, kind))
            .await
    }

    pub async fn set_task_state(&self, id: &str, state: TicketState) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::set_task_state(c, &id, state))
            .await
    }

    // ---------- open questions ----------

    /// Records `task_id` as blocked on an unanswered question, failing with
    /// `Conflict` if it already has one (one open question per task at a
    /// time; `task_id` is the table's primary key).
    pub async fn insert_open_question(
        &self,
        task_id: &str,
        message_id: &str,
        asked_by: &PrincipalId,
        asked_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let (task_id, message_id, asked_by) = (
            task_id.to_string(),
            message_id.to_string(),
            asked_by.clone(),
        );
        self.with_conn(move |c| {
            sync::insert_open_question(c, &task_id, &message_id, &asked_by, asked_at)
        })
        .await
    }

    /// Clears `task_id`'s open question, failing with `NotFound` if it has
    /// none.
    pub async fn delete_open_question(&self, task_id: &str) -> Result<(), StoreError> {
        let task_id = task_id.to_string();
        self.with_conn(move |c| sync::delete_open_question(c, &task_id))
            .await
    }

    pub async fn list_open_questions(&self) -> Result<Vec<OpenQuestion>, StoreError> {
        self.with_conn(sync::list_open_questions).await
    }

    // ---------- claims ----------

    /// Claims `task_id` for `claimed_by`, failing with `Conflict` if it's
    /// already claimed (`task_id` is the table's primary key).
    pub async fn insert_claim(
        &self,
        task_id: &str,
        claimed_by: &PrincipalId,
        claimed_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let (task_id, claimed_by) = (task_id.to_string(), claimed_by.clone());
        self.with_conn(move |c| sync::insert_claim(c, &task_id, &claimed_by, claimed_at))
            .await
    }

    /// Releases `task_id`'s claim, failing with `NotFound` if it has none.
    pub async fn delete_claim(&self, task_id: &str) -> Result<(), StoreError> {
        let task_id = task_id.to_string();
        self.with_conn(move |c| sync::delete_claim(c, &task_id))
            .await
    }

    /// Deletes `task_id`'s claim row (if any) and sets its state, atomically.
    pub async fn end_claim(&self, task_id: &str, state: TicketState) -> Result<(), StoreError> {
        let task_id = task_id.to_string();
        self.with_conn(move |c| sync::end_claim(c, &task_id, state))
            .await
    }

    pub async fn list_claims(&self) -> Result<Vec<Claim>, StoreError> {
        self.with_conn(sync::list_claims).await
    }

    // ---------- edges ----------

    pub async fn insert_edge(
        &self,
        from: &str,
        to: &str,
        kind: EdgeKind,
    ) -> Result<Edge, StoreError> {
        let (from, to) = (from.to_string(), to.to_string());
        self.with_conn(move |c| sync::insert_edge(c, &from, &to, kind))
            .await
    }

    pub async fn delete_edge(
        &self,
        from: &str,
        to: &str,
        kind: EdgeKind,
    ) -> Result<(), StoreError> {
        let (from, to) = (from.to_string(), to.to_string());
        self.with_conn(move |c| sync::delete_edge(c, &from, &to, kind))
            .await
    }

    pub async fn list_edges(&self) -> Result<Vec<Edge>, StoreError> {
        self.with_conn(sync::list_edges).await
    }

    /// Inserts `edge` exactly as given, including its `created_at`, rather
    /// than stamping `now()` the way [`Store::insert_edge`] does. Only for
    /// `bridle rebuild`.
    pub async fn insert_edge_row(&self, edge: &Edge) -> Result<(), StoreError> {
        let edge = edge.clone();
        self.with_conn(move |c| sync::insert_edge_row(c, &edge))
            .await
    }

    // ---------- ports ----------

    /// Records `port` for the owner; `false` if it's already allocated.
    pub async fn insert_port(&self, p: &PortAllocation) -> Result<bool, StoreError> {
        let p = p.clone();
        self.with_conn(move |c| sync::insert_port(c, &p)).await
    }

    /// Inserts a handover note; `id` is `h-<seq>`.
    pub async fn insert_handover(
        &self,
        role: &str,
        project: &str,
        body: &str,
        created_by: &str,
    ) -> Result<Handover, StoreError> {
        let (role, project, body, by) = (
            role.to_string(),
            project.to_string(),
            body.to_string(),
            created_by.to_string(),
        );
        self.with_conn(move |c| sync::insert_handover(c, &role, &project, &body, &by))
            .await
    }

    /// Newest first.
    /// Puts back a note read from the state branch, keeping its id and seq so new notes resume
    /// above the highest (AUTOINCREMENT follows an explicit rowid). A note already there is left.
    pub async fn restore_handover(&self, h: &Handover) -> Result<(), StoreError> {
        let h = h.clone();
        self.with_conn(move |c| sync::restore_handover(c, &h)).await
    }

    pub async fn list_handovers(&self) -> Result<Vec<Handover>, StoreError> {
        self.with_conn(sync::list_handovers).await
    }

    pub async fn get_handover(&self, id: &str) -> Result<Option<Handover>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::get_handover(c, &id)).await
    }

    /// Deletes notes older than `older_than`, always keeping the newest.
    pub async fn prune_handovers(&self, older_than: DateTime<Utc>) -> Result<u64, StoreError> {
        self.with_conn(move |c| sync::prune_handovers(c, older_than))
            .await
    }

    pub async fn list_ports(&self) -> Result<Vec<PortAllocation>, StoreError> {
        self.with_conn(sync::list_ports).await
    }

    /// Frees `port`, returning what was recorded for it.
    pub async fn release_port(&self, port: u16) -> Result<Option<PortAllocation>, StoreError> {
        self.with_conn(move |c| sync::release_port(c, port)).await
    }

    /// Frees every port an agent owns; returns how many.
    pub async fn release_agent_ports(&self, agent: &str) -> Result<usize, StoreError> {
        let agent = agent.to_string();
        self.with_conn(move |c| Ok(c.execute("DELETE FROM ports WHERE agent = ?1", [agent])?))
            .await
    }

    // ---------- conflicts ----------

    /// Opens a conflict for the overlap, or returns `None` if it's already known.
    pub async fn open_conflict(
        &self,
        tasks: &[String; 2],
        kind: &str,
        key: &str,
    ) -> Result<Option<Conflict>, StoreError> {
        let (tasks, kind, key) = (tasks.clone(), kind.to_string(), key.to_string());
        self.with_conn(move |c| sync::open_conflict(c, &tasks, &kind, &key))
            .await
    }

    pub async fn get_conflict(&self, id: &str) -> Result<Option<Conflict>, StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::get_conflict(c, &id)).await
    }

    pub async fn list_conflicts(&self) -> Result<Vec<Conflict>, StoreError> {
        self.with_conn(sync::list_conflicts).await
    }

    /// Marks an open conflict resolved; `Conflict` error if it already is.
    pub async fn resolve_conflict(&self, id: &str, resolution: &str) -> Result<(), StoreError> {
        let (id, resolution) = (id.to_string(), resolution.to_string());
        self.with_conn(move |c| sync::resolve_conflict(c, &id, &resolution))
            .await
    }

    // ---------- rate limits / usage ----------

    pub async fn upsert_rate_limit(&self, rl: RateLimit) -> Result<(), StoreError> {
        self.with_conn(move |c| sync::upsert_rate_limit(c, &rl))
            .await
    }

    /// A window's readings, oldest first (`GET /v1/usage/history`).
    pub async fn rate_limit_history(
        &self,
        window: String,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RateLimitPoint>, StoreError> {
        self.with_conn(move |c| sync::rate_limit_history(c, &window, since))
            .await
    }

    pub async fn rate_limits(&self) -> Result<Vec<RateLimit>, StoreError> {
        self.with_conn(sync::rate_limits).await
    }

    pub async fn usage(&self) -> Result<Usage, StoreError> {
        self.with_conn(sync::usage).await
    }

    pub async fn usage_breakdown(
        &self,
        since: Option<DateTime<Utc>>,
        by: UsageGroupBy,
    ) -> Result<UsageBreakdown, StoreError> {
        self.with_conn(move |c| sync::usage_breakdown(c, since, by))
            .await
    }

    /// One agent's turns bucketed by start time, from the ledger alone (removed agents too).
    pub async fn usage_series(
        &self,
        agent_id: String,
        since: Option<DateTime<Utc>>,
        bucket: UsageBucket,
    ) -> Result<UsageSeries, StoreError> {
        self.with_conn(move |c| sync::usage_series(c, &agent_id, since, bucket))
            .await
    }

    /// One row per session with usage in `[since, until)`, hosted and external (br-zn6s).
    pub async fn usage_sessions(
        &self,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Vec<SessionUsage>, StoreError> {
        self.with_conn(move |c| sync::usage_sessions(c, since, until))
            .await
    }

    /// Agent instances with a turn in the period, from the ledger alone.
    pub async fn usage_instances(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<UsageInstance>, StoreError> {
        self.with_conn(move |c| sync::usage_instances(c, since))
            .await
    }

    /// One `bridle statusline` snapshot: `observed_at` is set here, not
    /// trusted from the client, same as `upsert_rate_limit`.
    pub async fn record_interactive_usage(
        &self,
        row: InteractiveUsageRow,
    ) -> Result<(), StoreError> {
        self.with_conn(move |c| sync::record_interactive_usage(c, &row))
            .await
    }

    pub async fn agents_by_state(&self) -> Result<BTreeMap<String, u32>, StoreError> {
        self.with_conn(sync::agents_by_state).await
    }

    pub async fn unread_count(&self, to: &str) -> Result<u32, StoreError> {
        let to = to.to_string();
        self.with_conn(move |c| sync::unread_count(c, &to)).await
    }

    pub async fn get_meta(&self, key: &str) -> Result<Option<String>, StoreError> {
        let key = key.to_string();
        self.with_conn(move |c| sync::get_meta(c, &key)).await
    }

    /// Sets `key`, returning the previous value.
    pub async fn swap_meta(&self, key: &str, value: &str) -> Result<Option<String>, StoreError> {
        let (key, value) = (key.to_string(), value.to_string());
        self.with_conn(move |c| sync::swap_meta(c, &key, &value))
            .await
    }
}

// =====================================================================
// Sync layer: plain functions over `&Connection`. No async, no locking.
// =====================================================================

mod sync {
    use rusqlite::{OptionalExtension, Row, params, params_from_iter};
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    use super::*;

    pub(super) const SCHEMA_V1: &str = r#"
        CREATE TABLE principals (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            token_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            revoked_at TEXT
        );

        CREATE TABLE agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL,
            state TEXT NOT NULL,
            model TEXT NOT NULL,
            session_id TEXT NOT NULL,
            pid INTEGER,
            pid_start TEXT,
            workdir_kind TEXT NOT NULL,
            cwd TEXT NOT NULL,
            worktree TEXT,
            branch TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            turns INTEGER NOT NULL DEFAULT 0,
            cost_usd_total REAL NOT NULL DEFAULT 0,
            last_event_at TEXT,
            turn_started_at TEXT,
            exit_code INTEGER,
            exit_signal INTEGER,
            exit_reason TEXT,
            created_by TEXT NOT NULL
        );

        CREATE TABLE turns (
            agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            n INTEGER NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            subtype TEXT,
            is_error INTEGER,
            terminal_reason TEXT,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read INTEGER NOT NULL DEFAULT 0,
            cache_write INTEGER NOT NULL DEFAULT 0,
            cost_total REAL NOT NULL DEFAULT 0,
            PRIMARY KEY (agent_id, n)
        );

        -- `agent_id` deliberately has no foreign key: events must survive
        -- `delete_agent` (agents.md: "its events … are kept").
        CREATE TABLE events (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            ts TEXT NOT NULL,
            kind TEXT NOT NULL,
            actor TEXT NOT NULL,
            agent_id TEXT,
            data TEXT NOT NULL
        );
        CREATE INDEX events_kind ON events(kind);
        CREATE INDEX events_agent ON events(agent_id);

        -- `id` is derived from `seq` (the rowid) right after insert: "m-"
        -- plus the zero-padded rowid.
        CREATE TABLE messages (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            id TEXT NOT NULL UNIQUE,
            from_principal TEXT NOT NULL,
            to_kind TEXT NOT NULL,
            to_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            body TEXT NOT NULL,
            reply_to TEXT,
            when_mode TEXT NOT NULL,
            state TEXT NOT NULL,
            created_at TEXT NOT NULL,
            written_at TEXT,
            delivered_at TEXT,
            read_at TEXT
        );
        CREATE INDEX messages_to ON messages(to_kind, to_id, state);

        CREATE TABLE rate_limits (
            window TEXT PRIMARY KEY,
            status TEXT,
            utilization REAL,
            resets_at TEXT,
            observed_at TEXT NOT NULL
        );
    "#;

    // Turns outlive their agent, carrying its name, role and model, so
    // `rm` doesn't erase usage history. SQLite can't drop a foreign key in
    // place, hence the rebuild.
    pub(super) const SCHEMA_V2: &str = r#"
        CREATE TABLE turns_v2 (
            agent_id TEXT NOT NULL,
            agent_name TEXT NOT NULL,
            role TEXT NOT NULL,
            model TEXT NOT NULL,
            n INTEGER NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            subtype TEXT,
            is_error INTEGER,
            terminal_reason TEXT,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read INTEGER NOT NULL DEFAULT 0,
            cache_write INTEGER NOT NULL DEFAULT 0,
            cost_total REAL NOT NULL DEFAULT 0,
            PRIMARY KEY (agent_id, n)
        );
        INSERT INTO turns_v2
            SELECT t.agent_id, a.name, a.role, a.model, t.n, t.started_at, t.ended_at,
                   t.subtype, t.is_error, t.terminal_reason, t.input_tokens,
                   t.output_tokens, t.cache_read, t.cache_write, t.cost_total
            FROM turns t JOIN agents a ON a.id = t.agent_id;
        DROP TABLE turns;
        ALTER TABLE turns_v2 RENAME TO turns;

        -- Small daemon facts that outlive a restart, e.g. `claude_version`.
        CREATE TABLE meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    "#;

    // `bridle statusline` snapshots from interactive sessions bridle doesn't
    // host: no agent id to attach them to, so this is its own table rather
    // than a row in `turns`.
    pub(super) const SCHEMA_V3: &str = r#"
        CREATE TABLE interactive_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            observed_at TEXT NOT NULL,
            session_id TEXT,
            model TEXT,
            cost_usd REAL,
            context_used_tokens INTEGER,
            context_max_tokens INTEGER
        );
        CREATE INDEX interactive_usage_observed_at ON interactive_usage(observed_at);
    "#;

    // A `parse_get_usage` bug (fixed alongside this migration) recorded every
    // key of the real `get_usage` response's `rate_limits` object, including
    // null-valued internal codenames; this sweeps out any daemon's existing junk.
    pub(super) const SCHEMA_V4: &str = r#"
        DELETE FROM rate_limits WHERE utilization IS NULL AND resets_at IS NULL;
    "#;

    // The fast index over task records; the state branch (storage.md) holds
    // the body and thread, so this table is deliberately narrow. `id` is
    // `<project prefix>-<4 hex chars>` (storage.md), generated here with a
    // collision retry, not by the caller.
    pub(super) const SCHEMA_V5: &str = r#"
        CREATE TABLE tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            kind TEXT NOT NULL,
            state TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX tasks_state ON tasks(state);
    "#;

    // Context governor, part 1 (measure): the agent's latest known context
    // size, from its most recent turn's `result` event. Not cumulative, so
    // it's a plain column on `agents`, not folded into `turns` history.
    pub(super) const SCHEMA_V6: &str = r#"
        ALTER TABLE agents ADD COLUMN context_tokens INTEGER;
    "#;

    // Coordination edges between tasks (coordination.md, Edges). `(from_task,
    // to_task, kind)` is the natural key: there's no separate edge id, and
    // `dep rm` identifies the row to delete by that same triple. Named
    // `from_task`/`to_task`, not `from`/`to`, since both are SQL keywords.
    pub(super) const SCHEMA_V7: &str = r#"
        CREATE TABLE edges (
            from_task TEXT NOT NULL,
            to_task TEXT NOT NULL,
            kind TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY (from_task, to_task, kind)
        );
        CREATE INDEX edges_to ON edges(to_task);
        CREATE INDEX edges_from ON edges(from_task);
    "#;

    // The fast index over open questions (storage.md, "Questions aren't a
    // separate `questions/…` folder"): `task_id` is the primary key, since a
    // task has at most one open question at a time, and `message_id` points
    // at the `messages` row that carries the body rather than duplicating
    // it. `is_ready` (tasks.rs) treats a task's presence in this table as
    // "has an open question".
    pub(super) const SCHEMA_V8: &str = r#"
        CREATE TABLE open_questions (
            task_id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL,
            asked_by TEXT NOT NULL,
            asked_at TEXT NOT NULL
        );
    "#;

    // The fast index over claims (storage.md, "claims"), mirrored to the
    // state branch's `claims.toml` the same way `open_questions` mirrors its
    // thread entry. `task_id` is the primary key, since a task has at most
    // one claimant at a time.
    pub(super) const SCHEMA_V9: &str = r#"
        CREATE TABLE claims (
            task_id TEXT PRIMARY KEY,
            claimed_by TEXT NOT NULL,
            claimed_at TEXT NOT NULL
        );
    "#;

    // Claude Code's real statusline schema has `context_window.used_percentage`
    // precomputed (docs/tickets/open/statusline-real-context-and-a-tighter-layout-s8kn.md):
    // shown directly instead of recomputed from context_used_tokens/context_max_tokens,
    // since that recomputation is wrong on extended-context (1M) models.
    pub(super) const SCHEMA_V10: &str = r#"
        ALTER TABLE interactive_usage ADD COLUMN context_used_percentage REAL;
    "#;

    // Per-spawn `--allow-tool`/`--env` overrides (roles-and-config.md), JSON
    // arrays. Stored on the agent's own row so `renew` (context handoff) and
    // `resume` (daemon restart) can reapply them: both rebuild `cmd` from
    // the role alone otherwise, silently dropping the overrides mid-task.
    pub(super) const SCHEMA_V11: &str = r#"
        ALTER TABLE agents ADD COLUMN extra_allowed_tools TEXT NOT NULL DEFAULT '[]';
        ALTER TABLE agents ADD COLUMN extra_env TEXT NOT NULL DEFAULT '[]';
    "#;

    // Component ids the agent is scoped to (docs/design/components.md), a JSON
    // array. Kept so `resume`/`renew` re-pass `BRIDLE_COMPONENTS`.
    pub(super) const SCHEMA_V12: &str = r#"
        ALTER TABLE agents ADD COLUMN components TEXT NOT NULL DEFAULT '[]';
    "#;

    // Whether the agent's current `session_id` has had a turn start, i.e. claude
    // has written the session to disk. A fresh session that never got that far
    // (renewed, then the daemon restarted) can't be `--resume`d. Existing rows
    // predate the tracking and are assumed started.
    pub(super) const SCHEMA_V13: &str = r#"
        ALTER TABLE agents ADD COLUMN session_started INTEGER NOT NULL DEFAULT 1;
    "#;

    // A question to the human closed by a delegate's reply (`[messages]
    // answer_for_human`): who answered, and the reply's message id.
    pub(super) const SCHEMA_V14: &str = r#"
        ALTER TABLE messages ADD COLUMN answered_by TEXT;
        ALTER TABLE messages ADD COLUMN answered_reply TEXT;
        ALTER TABLE messages ADD COLUMN answered_line TEXT;
    "#;

    // Conflicts opened by `impact check` (impact-and-conflicts.md). The unique
    // key makes reopening the same overlap a no-op, resolved or not. The id
    // shown to users is `C<rowid>`.
    pub(super) const SCHEMA_V15: &str = r#"
        CREATE TABLE conflicts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_a TEXT NOT NULL,
            task_b TEXT NOT NULL,
            kind TEXT NOT NULL,
            key TEXT NOT NULL,
            state TEXT NOT NULL DEFAULT 'open',
            resolution TEXT,
            opened_at TEXT NOT NULL,
            resolved_at TEXT,
            UNIQUE (task_a, task_b, kind, key)
        );
    "#;

    // Ports handed out by `bridle port alloc`. Runtime state: not on the state
    // branch and not rebuilt (worktrees-and-ports.md).
    pub(super) const SCHEMA_V16: &str = r#"
        CREATE TABLE ports (
            port INTEGER PRIMARY KEY,
            agent TEXT NOT NULL,
            task TEXT,
            pid INTEGER,
            label TEXT,
            allocated_at TEXT NOT NULL
        );
    "#;

    // Orchestrator handover notes: also written to the state branch and restored by rebuild
    // (orchestrator-supervision.md, section 7). `id` is `h-<seq>`, filled in after the insert like messages'.
    pub(super) const SCHEMA_V17: &str = r#"
        CREATE TABLE handovers (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            id TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL,
            project TEXT NOT NULL,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL,
            created_by TEXT NOT NULL
        );
    "#;

    // The incident task a broadcast notice announces (incidents.md); NULL for every other message.
    pub(super) const SCHEMA_V18: &str = r#"
        ALTER TABLE messages ADD COLUMN incident_task TEXT;
        CREATE INDEX messages_incident ON messages(incident_task) WHERE incident_task IS NOT NULL;
    "#;

    // Who created the task (task watchers, xxxq). Nullable: NULL means unknown, and
    // `backfill_task_creators` fills what it can from the `task.created` events.
    pub(super) const SCHEMA_V19: &str = r#"
        ALTER TABLE tasks ADD COLUMN created_by TEXT;
    "#;

    // Mail between daemons (3haz): this daemon's outbox, one queue per destination project, and
    // the origins it has already accepted from other daemons (the dedup record).
    pub(super) const SCHEMA_V20: &str = r#"
        CREATE TABLE outbox (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            id TEXT NOT NULL DEFAULT '',
            project TEXT NOT NULL,
            from_principal TEXT NOT NULL,
            to_principal TEXT NOT NULL,
            kind TEXT NOT NULL,
            body TEXT NOT NULL,
            reply_to TEXT,
            when_mode TEXT NOT NULL,
            state TEXT NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            remote_ids TEXT,
            created_at TEXT NOT NULL,
            delivered_at TEXT
        );
        CREATE INDEX outbox_queue ON outbox(project, state, seq);
        CREATE TABLE forwarded_in (
            origin_machine TEXT NOT NULL,
            origin_daemon TEXT NOT NULL,
            origin_id TEXT NOT NULL,
            message_ids TEXT NOT NULL,
            received_at TEXT NOT NULL,
            PRIMARY KEY (origin_machine, origin_daemon, origin_id)
        );
    "#;

    // The account's rate-limit readings over time (xxw9): one row per change of a window's
    // utilization or resets_at, kept `RATE_LIMIT_HISTORY_DAYS` days.
    pub(super) const SCHEMA_V21: &str = r#"
        CREATE TABLE rate_limit_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            window TEXT NOT NULL,
            utilization REAL,
            resets_at TEXT,
            observed_at TEXT NOT NULL
        );
        CREATE INDEX rate_limit_history_window ON rate_limit_history(window, observed_at);
    "#;

    // Mail between daemons, slice 3 (br-fvkq): a stuck entry's one-time notice to its sender.
    pub(super) const SCHEMA_V22: &str = r#"
        ALTER TABLE outbox ADD COLUMN stuck_notified_at TEXT;
    "#;

    // Scheduled messages (hrcn, br-9xze).
    pub(super) const SCHEMA_V23: &str = r#"
        CREATE TABLE schedules (
            id TEXT PRIMARY KEY,
            created_by TEXT NOT NULL,
            target TEXT NOT NULL,
            body TEXT NOT NULL,
            kind TEXT NOT NULL,
            at_utc TEXT,
            cron TEXT,
            tz TEXT NOT NULL,
            next_fire_at TEXT,
            last_fired_at TEXT,
            state TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX schedules_due ON schedules(state, next_fire_at);
    "#;

    // Mail between daemons, slice 2 (br-n7cg): where a visitor's mail is forwarded. NULL (every
    // token minted before) keeps the visitor's inbox on this daemon.
    pub(super) const SCHEMA_V24: &str = r#"
        ALTER TABLE principals ADD COLUMN home TEXT;
    "#;

    // Mail between daemons, slice 4 (br-cufw): the one-time report of a long-undelivered message
    // to the human through the aide.
    pub(super) const SCHEMA_V25: &str = r#"
        ALTER TABLE outbox ADD COLUMN aide_notified_at TEXT;
    "#;

    // Code rename, slice 1 (br-95t9, 22ab step 9): the `tasks` table is the tickets' tracking
    // rows, so it is called `tickets`. SQLite rewrites references to it in the same step.
    pub(super) const SCHEMA_V26: &str = r#"
        ALTER TABLE tasks RENAME TO tickets;
        DROP INDEX IF EXISTS tasks_state;
        CREATE INDEX tickets_state ON tickets(state);
    "#;

    // Negative costs (br-9y7v): a respawned agent's new claude session counts its cost from 0,
    // but the delta baseline was the agent's lifetime total, so the first turn after a renew
    // recorded a large negative cost. `session_cost_usd` is the baseline per session now. The
    // repair zeroes each negative turn (its true delta is not recoverable) and re-sums the
    // lifetime total of every agent that had one. Existing agents start with their lifetime total
    // as the session baseline (right unless renewed; then the next delta clamps to 0).
    pub(super) const SCHEMA_V27: &str = r#"
        ALTER TABLE agents ADD COLUMN session_cost_usd REAL NOT NULL DEFAULT 0;
        UPDATE agents SET cost_usd_total = (
            SELECT COALESCE(SUM(MAX(t.cost_total, 0)), 0) FROM turns t WHERE t.agent_id = agents.id)
        WHERE id IN (SELECT agent_id FROM turns WHERE cost_total < 0);
        UPDATE turns SET cost_total = 0 WHERE cost_total < 0;
        UPDATE agents SET session_cost_usd = cost_usd_total;
    "#;

    // External sessions in `bridle usage` (hd28): who reported, their running token totals, and
    // the deltas against the session's previous report (what the usage report sums).
    pub(super) const SCHEMA_V28: &str = r#"
        ALTER TABLE interactive_usage ADD COLUMN identity TEXT;
        ALTER TABLE interactive_usage ADD COLUMN total_input_tokens INTEGER;
        ALTER TABLE interactive_usage ADD COLUMN total_output_tokens INTEGER;
        ALTER TABLE interactive_usage ADD COLUMN cost_delta REAL;
        ALTER TABLE interactive_usage ADD COLUMN input_delta INTEGER;
        ALTER TABLE interactive_usage ADD COLUMN output_delta INTEGER;
    "#;

    // Which Claude session each turn ran in, and the context size it ended at (br-pac3).
    // NULL on old rows: unknown, never zero.
    pub(super) const SCHEMA_V29: &str = r#"
        ALTER TABLE turns ADD COLUMN session_id TEXT;
        ALTER TABLE turns ADD COLUMN context_tokens INTEGER;
    "#;

    pub(super) const SCHEMA_V30: &str = r#"
        CREATE TABLE agent_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            agent_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            started_at TEXT NOT NULL,
            start_reason TEXT NOT NULL,
            ended_at TEXT,
            end_reason TEXT,
            end_tokens INTEGER,
            renewal_limit REAL
        );
        CREATE INDEX agent_sessions_agent ON agent_sessions(agent_id, id);
    "#;

    pub(super) const RATE_LIMIT_HISTORY_DAYS: i64 = 90;

    pub(super) const MIGRATIONS: &[&str] = &[
        SCHEMA_V1, SCHEMA_V2, SCHEMA_V3, SCHEMA_V4, SCHEMA_V5, SCHEMA_V6, SCHEMA_V7, SCHEMA_V8,
        SCHEMA_V9, SCHEMA_V10, SCHEMA_V11, SCHEMA_V12, SCHEMA_V13, SCHEMA_V14, SCHEMA_V15,
        SCHEMA_V16, SCHEMA_V17, SCHEMA_V18, SCHEMA_V19, SCHEMA_V20, SCHEMA_V21, SCHEMA_V22,
        SCHEMA_V23, SCHEMA_V24, SCHEMA_V25, SCHEMA_V26, SCHEMA_V27, SCHEMA_V28, SCHEMA_V29,
        SCHEMA_V30,
    ];

    pub(super) fn open(path: &Path) -> Result<Connection, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;
        migrate(&conn)?;
        // Best effort: an unfilled creator stays unknown, and must never stop start-up.
        if let Err(e) = backfill_task_creators(&conn) {
            tracing::warn!("backfilling tasks.created_by from events failed: {e}");
        }
        Ok(conn)
    }

    /// Fills `tasks.created_by` from each unfilled task's earliest `task.created` event's
    /// actor. Idempotent (only touches NULLs) and one pass over the kind index, not a scan
    /// per task. A task with no such event (or a bad row) stays NULL.
    pub(super) fn backfill_task_creators(conn: &Connection) -> Result<(), StoreError> {
        let mut stmt = conn.prepare(
            "SELECT json_extract(data, '$.task'), actor FROM events
             WHERE kind = 'task.created' AND json_valid(data) ORDER BY seq DESC",
        )?;
        // Newest first so the earliest event's actor wins the insert-overwrite.
        let mut creators: BTreeMap<String, String> = BTreeMap::new();
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, Option<String>>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows {
            match row {
                Ok((Some(task), actor)) => {
                    creators.insert(task, actor);
                }
                Ok((None, _)) => {}
                Err(e) => tracing::warn!("skipping a task.created event: {e}"),
            }
        }
        for (task, actor) in creators {
            if let Err(e) = conn.execute(
                "UPDATE tickets SET created_by = ?2 WHERE id = ?1 AND created_by IS NULL",
                params![task, actor],
            ) {
                tracing::warn!(task, "backfilling created_by failed: {e}");
            }
        }
        Ok(())
    }

    pub(super) fn migrate(conn: &Connection) -> Result<(), StoreError> {
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let version = usize::try_from(version).unwrap_or(0);
        if version > MIGRATIONS.len() {
            return Err(StoreError::SchemaTooNew {
                found: version,
                known: MIGRATIONS.len(),
            });
        }
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(version) {
            conn.execute_batch(sql)?;
            conn.pragma_update(None, "user_version", (i + 1) as i64)?;
        }
        Ok(())
    }

    // ---------- time / id helpers ----------

    fn fmt_dt(dt: DateTime<Utc>) -> String {
        dt.to_rfc3339_opts(SecondsFormat::Millis, true)
    }

    fn parse_dt(s: &str) -> Result<DateTime<Utc>, rusqlite::Error> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
    }

    fn parse_dt_opt(s: Option<String>) -> Result<Option<DateTime<Utc>>, rusqlite::Error> {
        s.map(|s| parse_dt(&s)).transpose()
    }

    const BASE36: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    /// `len` base36 characters, drawn from two v4 UUIDs' worth of OS
    /// randomness (there's no `rand` dependency in this workspace; `uuid`
    /// already pulls in a CSPRNG for `new_v4`).
    fn random_base36(len: usize) -> String {
        let mut bytes = Uuid::new_v4().into_bytes().to_vec();
        bytes.extend_from_slice(&Uuid::new_v4().into_bytes());
        let mut n: u128 = 0;
        for b in &bytes[..16] {
            n = (n << 8) | u128::from(*b);
        }
        let mut out = String::with_capacity(len);
        for _ in 0..len {
            out.push(BASE36[(n % 36) as usize] as char);
            n /= 36;
        }
        out
    }

    fn new_agent_id() -> String {
        format!("a-{}", random_base36(5))
    }

    /// The id alphabet shared with tickets (`bridle ticket new`): no `i`, `l`, `o`, `0`, `1`.
    const ID_ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";

    /// `len` characters of [`ID_ALPHABET`], drawn from a fresh v4 UUID's bytes (a CSPRNG draw;
    /// see `random_base36` above for why there's no `rand` dependency here). The modulo bias
    /// (256 % 31) is irrelevant for ids.
    fn random_id_chars(len: usize) -> String {
        Uuid::new_v4()
            .as_bytes()
            .iter()
            .take(len)
            .map(|b| ID_ALPHABET[*b as usize % ID_ALPHABET.len()] as char)
            .collect()
    }

    fn new_task_id(prefix: &str) -> String {
        format!("{prefix}-{}", random_id_chars(4))
    }

    /// 64 hex characters from two v4 UUIDs, per principals.md.
    fn random_token() -> String {
        format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn is_unique_violation(e: &rusqlite::Error) -> bool {
        matches!(
            e,
            rusqlite::Error::SqliteFailure(f, _) if f.code == rusqlite::ErrorCode::ConstraintViolation
        )
    }

    // ---------- small enum <-> TEXT mappings ----------

    fn kind_str(k: MessageKind) -> &'static str {
        match k {
            MessageKind::Note => "note",
            MessageKind::Question => "question",
            MessageKind::Answer => "answer",
            MessageKind::TaskUpdate => "task_update",
            MessageKind::System => "system",
        }
    }

    fn kind_from_str(s: &str) -> MessageKind {
        match s {
            "question" => MessageKind::Question,
            "answer" => MessageKind::Answer,
            "task_update" => MessageKind::TaskUpdate,
            "system" => MessageKind::System,
            _ => MessageKind::Note,
        }
    }

    fn when_str(w: When) -> &'static str {
        match w {
            When::Now => "now",
            When::Idle => "idle",
        }
    }

    fn when_from_str(s: &str) -> When {
        match s {
            "idle" => When::Idle,
            _ => When::Now,
        }
    }

    fn msg_state_str(s: MessageState) -> &'static str {
        match s {
            MessageState::Pending => "pending",
            MessageState::Held => "held",
            MessageState::Written => "written",
            MessageState::Delivered => "delivered",
            MessageState::Read => "read",
            MessageState::Dropped => "dropped",
        }
    }

    fn msg_state_from_str(s: &str) -> MessageState {
        match s {
            "held" => MessageState::Held,
            "written" => MessageState::Written,
            "delivered" => MessageState::Delivered,
            "read" => MessageState::Read,
            "dropped" => MessageState::Dropped,
            _ => MessageState::Pending,
        }
    }

    fn principal_kind_str(k: PrincipalKind) -> &'static str {
        match k {
            PrincipalKind::Human => "human",
            PrincipalKind::Agent => "agent",
            PrincipalKind::External => "external",
            PrincipalKind::Peer => "peer",
            PrincipalKind::System => "system",
            PrincipalKind::Local => {
                unreachable!("Local principals are synthesized per-request, never stored")
            }
        }
    }

    fn principal_kind_from_str(s: &str) -> PrincipalKind {
        match s {
            "agent" => PrincipalKind::Agent,
            "external" => PrincipalKind::External,
            "peer" => PrincipalKind::Peer,
            "system" => PrincipalKind::System,
            _ => PrincipalKind::Human,
        }
    }

    // ---------- principals / tokens ----------

    pub(super) fn ensure_human_token(conn: &Connection) -> Result<Option<String>, StoreError> {
        create_principal_if_absent(conn, "human", PrincipalKind::Human, "human")
    }

    /// Creates a principal with a fresh token, unless one already exists
    /// and is active, in which case returns `Ok(None)` (used only for the
    /// human principal, which is created at most once).
    fn create_principal_if_absent(
        conn: &Connection,
        id: &str,
        kind: PrincipalKind,
        name: &str,
    ) -> Result<Option<String>, StoreError> {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM principals WHERE id = ?1 AND revoked_at IS NULL",
                params![id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if exists {
            return Ok(None);
        }
        let token = random_token();
        conn.execute(
            "INSERT INTO principals(id, kind, name, token_hash, created_at, revoked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL)
             ON CONFLICT(id) DO UPDATE SET kind=excluded.kind, name=excluded.name,
                 token_hash=excluded.token_hash, created_at=excluded.created_at, revoked_at=NULL",
            params![
                id,
                principal_kind_str(kind),
                name,
                hash_token(&token),
                fmt_dt(Utc::now())
            ],
        )?;
        Ok(Some(token))
    }

    /// Creates a principal, or replaces the token of an active one.
    fn create_principal_replacing(
        conn: &Connection,
        id: &str,
        kind: PrincipalKind,
        name: &str,
    ) -> Result<String, StoreError> {
        let token = random_token();
        conn.execute(
            "INSERT INTO principals(id, kind, name, token_hash, created_at, revoked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL)
             ON CONFLICT(id) DO UPDATE SET kind=excluded.kind, name=excluded.name,
                 token_hash=excluded.token_hash, created_at=excluded.created_at, revoked_at=NULL",
            params![
                id,
                principal_kind_str(kind),
                name,
                hash_token(&token),
                fmt_dt(Utc::now())
            ],
        )?;
        Ok(token)
    }

    /// Creates a principal, failing with `Conflict` if one with this id is
    /// already active (409 if the name is taken).
    fn create_principal_active_only(
        conn: &Connection,
        id: &str,
        kind: PrincipalKind,
        name: &str,
    ) -> Result<String, StoreError> {
        let active: bool = conn
            .query_row(
                "SELECT 1 FROM principals WHERE id = ?1 AND revoked_at IS NULL",
                params![id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if active {
            // The usual cause is the receiver's own name given to `--peer` (gdf3).
            return Err(StoreError::Conflict(if kind == PrincipalKind::Peer {
                format!(
                    "{id} already exists on this daemon: --peer takes the machine that SENDS \
                     to this project (the receiver is this machine); to replace its token, \
                     revoke it first with `bridle token revoke {id}`"
                )
            } else {
                format!("principal {id:?} already exists")
            }));
        }
        let token = random_token();
        conn.execute(
            "INSERT INTO principals(id, kind, name, token_hash, created_at, revoked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL)
             ON CONFLICT(id) DO UPDATE SET kind=excluded.kind, name=excluded.name,
                 token_hash=excluded.token_hash, created_at=excluded.created_at, revoked_at=NULL",
            params![
                id,
                principal_kind_str(kind),
                name,
                hash_token(&token),
                fmt_dt(Utc::now())
            ],
        )?;
        Ok(token)
    }

    /// `human@<machine>`: the human's token on another machine's daemon (3ehu).
    fn is_human_visitor(name: &str) -> bool {
        name.strip_prefix("human@").is_some_and(|m| !m.is_empty())
    }

    /// The principal id behind a token name: `external:<name>`, except `human@<machine>`.
    fn token_principal_id(name: &str) -> String {
        if is_human_visitor(name) {
            name.to_string()
        } else {
            format!("external:{name}")
        }
    }

    pub(super) fn create_external_token(
        conn: &Connection,
        name: &str,
        home: Option<&str>,
    ) -> Result<TokenCreated, StoreError> {
        // `human@<machine>` is the human on another machine: human authority, its own id.
        let kind = if is_human_visitor(name) {
            PrincipalKind::Human
        } else {
            PrincipalKind::External
        };
        let id = token_principal_id(name);
        let token = create_principal_active_only(conn, &id, kind, name)?;
        conn.execute(
            "UPDATE principals SET home = ?1 WHERE id = ?2",
            params![home, id],
        )?;
        Ok(TokenCreated {
            principal: id,
            token,
        })
    }

    pub(super) fn create_peer_token(
        conn: &Connection,
        machine: &str,
        replace: bool,
    ) -> Result<TokenCreated, StoreError> {
        let id = format!("peer:{machine}");
        let token = if replace {
            // The upsert swaps the hash in one statement: the old token stops working
            // exactly when the new one exists.
            create_principal_replacing(conn, &id, PrincipalKind::Peer, machine)?
        } else {
            create_principal_active_only(conn, &id, PrincipalKind::Peer, machine)?
        };
        Ok(TokenCreated {
            principal: id,
            token,
        })
    }

    const SCHEDULE_COLS: &str = "id, created_by, target, body, kind, cron, tz, next_fire_at,
        last_fired_at, state, created_at";

    fn schedule_from(r: &Row<'_>) -> rusqlite::Result<Schedule> {
        Ok(Schedule {
            id: r.get(0)?,
            created_by: r.get(1)?,
            target: r.get(2)?,
            body: r.get(3)?,
            kind: r.get(4)?,
            cron: r.get(5)?,
            tz: r.get(6)?,
            next_fire_at: parse_dt_opt(r.get(7)?)?,
            last_fired_at: parse_dt_opt(r.get(8)?)?,
            state: r.get(9)?,
            created_at: parse_dt(&r.get::<_, String>(10)?)?,
            relation: None,
        })
    }

    pub(super) fn schedule_add(conn: &Connection, mut s: Schedule) -> Result<Schedule, StoreError> {
        s.id = new_task_id("sc");
        s.created_at = Utc::now().trunc_subsecs(3);
        conn.execute(
            "INSERT INTO schedules(id, created_by, target, body, kind, at_utc, cron, tz,
                next_fire_at, state, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'active', ?10)",
            params![
                s.id,
                s.created_by,
                s.target,
                s.body,
                s.kind,
                (s.kind == "once")
                    .then(|| s.next_fire_at.map(fmt_dt))
                    .flatten(),
                s.cron,
                s.tz,
                s.next_fire_at.map(fmt_dt),
                fmt_dt(s.created_at),
            ],
        )?;
        s.state = "active".to_string();
        Ok(s)
    }

    pub(super) fn schedules_list(
        conn: &Connection,
        involving: Option<&str>,
        all: bool,
    ) -> Result<Vec<Schedule>, StoreError> {
        // Targets are stored as `agent:<name>` for agents, so match the bare name either way.
        let bare = involving.map(|p| p.strip_prefix("agent:").unwrap_or(p).to_string());
        let prefixed = bare.as_ref().map(|b| format!("agent:{b}"));
        let mut stmt = conn.prepare(&format!(
            "SELECT {SCHEDULE_COLS} FROM schedules
             WHERE (?1 IS NULL OR created_by = ?1 OR target = ?1 OR target = ?3 OR target = ?4)
               AND (?2 OR state = 'active')
             ORDER BY next_fire_at IS NULL, next_fire_at, created_at"
        ))?;
        Ok(stmt
            .query_map(params![involving, all, bare, prefixed], schedule_from)?
            .collect::<Result<_, _>>()?)
    }

    pub(super) fn schedule_get(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<Schedule>, StoreError> {
        Ok(conn
            .query_row(
                &format!("SELECT {SCHEDULE_COLS} FROM schedules WHERE id = ?1"),
                params![id],
                schedule_from,
            )
            .optional()?)
    }

    pub(super) fn schedule_remove(conn: &Connection, id: &str) -> Result<bool, StoreError> {
        Ok(conn.execute("DELETE FROM schedules WHERE id = ?1", params![id])? > 0)
    }

    pub(super) fn schedules_due(
        conn: &Connection,
        now: DateTime<Utc>,
    ) -> Result<Vec<Schedule>, StoreError> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {SCHEDULE_COLS} FROM schedules
             WHERE state = 'active' AND next_fire_at IS NOT NULL AND next_fire_at <= ?1
             ORDER BY next_fire_at"
        ))?;
        Ok(stmt
            .query_map(params![fmt_dt(now)], schedule_from)?
            .collect::<Result<_, _>>()?)
    }

    pub(super) fn schedule_fired(
        conn: &Connection,
        id: &str,
        at: DateTime<Utc>,
        next: Option<DateTime<Utc>>,
    ) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE schedules SET last_fired_at = ?2, next_fire_at = ?3,
                state = CASE WHEN ?3 IS NULL THEN 'done' ELSE 'active' END
             WHERE id = ?1",
            params![id, fmt_dt(at), next.map(fmt_dt)],
        )?;
        Ok(())
    }

    pub(super) fn outbox_enqueue(conn: &Connection, row: &OutboxRow) -> Result<String, StoreError> {
        conn.execute(
            "INSERT INTO outbox(project, from_principal, to_principal, kind, body, reply_to,
                when_mode, state, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'queued', ?8)",
            params![
                row.project,
                row.from,
                row.to,
                kind_str(row.kind),
                row.body,
                row.reply_to,
                when_str(row.when),
                fmt_dt(Utc::now()),
            ],
        )?;
        let seq = conn.last_insert_rowid();
        let id = format!("o-{seq:04}");
        conn.execute("UPDATE outbox SET id = ?1 WHERE seq = ?2", params![id, seq])?;
        Ok(id)
    }

    fn outbox_row_from(r: &rusqlite::Row<'_>) -> rusqlite::Result<OutboxRow> {
        let kind: String = r.get(4)?;
        let when: String = r.get(7)?;
        Ok(OutboxRow {
            id: r.get(0)?,
            project: r.get(1)?,
            from: r.get(2)?,
            to: r.get(3)?,
            kind: kind_from_str(&kind),
            body: r.get(5)?,
            reply_to: r.get(6)?,
            when: when_from_str(&when),
            attempts: r.get(8)?,
            last_error: r.get(9)?,
        })
    }

    pub(super) fn outbox_next(
        conn: &Connection,
        project: &str,
    ) -> Result<Option<OutboxRow>, StoreError> {
        Ok(conn
            .query_row(
                "SELECT id, project, from_principal, to_principal, kind, body, reply_to,
                        when_mode, attempts, last_error
                 FROM outbox WHERE project = ?1 AND state = 'queued' ORDER BY seq ASC LIMIT 1",
                params![project],
                outbox_row_from,
            )
            .optional()?)
    }

    pub(super) fn outbox_finish(
        conn: &Connection,
        id: &str,
        outcome: &Result<Vec<String>, (String, bool)>,
    ) -> Result<(), StoreError> {
        match outcome {
            Ok(ids) => conn.execute(
                "UPDATE outbox SET state = 'delivered', attempts = attempts + 1, last_error = NULL,
                    remote_ids = ?2, delivered_at = ?3 WHERE id = ?1",
                params![id, ids.join(","), fmt_dt(Utc::now())],
            )?,
            Err((error, permanent)) => conn.execute(
                "UPDATE outbox SET state = CASE WHEN ?3 THEN 'failed' ELSE state END,
                    attempts = attempts + 1, last_error = ?2 WHERE id = ?1",
                params![id, error, permanent],
            )?,
        };
        Ok(())
    }

    pub(super) fn outbox_state(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<(String, Vec<String>)>, StoreError> {
        Ok(conn
            .query_row(
                "SELECT state, remote_ids FROM outbox WHERE id = ?1",
                params![id],
                |r| {
                    let ids: Option<String> = r.get(1)?;
                    Ok((
                        r.get(0)?,
                        ids.unwrap_or_default()
                            .split(',')
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                            .collect(),
                    ))
                },
            )
            .optional()?)
    }

    pub(super) fn outbox_queued_projects(conn: &Connection) -> Result<Vec<String>, StoreError> {
        let mut stmt =
            conn.prepare("SELECT DISTINCT project FROM outbox WHERE state = 'queued'")?;
        Ok(stmt
            .query_map([], |r| r.get(0))?
            .collect::<Result<_, _>>()?)
    }

    pub(super) fn outbox_info(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<(String, Option<String>)>, StoreError> {
        Ok(conn
            .query_row(
                "SELECT state, last_error FROM outbox WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    }

    /// Entries queued since before `cutoff` that have not yet had their stuck notice.
    pub(super) fn outbox_stuck(
        conn: &Connection,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<OutboxRow>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT id FROM outbox WHERE state = 'queued' AND stuck_notified_at IS NULL
                AND created_at < ?1 ORDER BY seq",
        )?;
        let ids: Vec<String> = stmt
            .query_map(params![fmt_dt(cutoff)], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        ids.iter()
            .filter_map(|id| outbox_row(conn, id).transpose())
            .collect()
    }

    pub(super) fn outbox_row(conn: &Connection, id: &str) -> Result<Option<OutboxRow>, StoreError> {
        Ok(conn
            .query_row(
                "SELECT id, project, from_principal, to_principal, kind, body, reply_to,
                        when_mode, attempts, last_error
                 FROM outbox WHERE id = ?1",
                params![id],
                outbox_row_from,
            )
            .optional()?)
    }

    pub(super) fn outbox_aide_due(
        conn: &Connection,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<OutboxRow>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT id FROM outbox WHERE state = 'queued' AND aide_notified_at IS NULL
                AND created_at < ?1 ORDER BY seq",
        )?;
        let ids: Vec<String> = stmt
            .query_map(params![fmt_dt(cutoff)], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        ids.iter()
            .filter_map(|id| outbox_row(conn, id).transpose())
            .collect()
    }

    pub(super) fn outbox_mark_aide(conn: &Connection, id: &str) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE outbox SET aide_notified_at = ?2 WHERE id = ?1",
            params![id, fmt_dt(Utc::now())],
        )?;
        Ok(())
    }

    pub(super) fn outbox_peers(conn: &Connection) -> Result<Vec<OutboxPeer>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT project, COUNT(*), MIN(created_at),
                    (SELECT last_error FROM outbox o2 WHERE o2.project = o.project
                        AND o2.state = 'queued' ORDER BY o2.seq LIMIT 1)
             FROM outbox o WHERE state = 'queued' GROUP BY project ORDER BY project",
        )?;
        let rows = stmt.query_map([], |r| {
            let oldest: String = r.get(2)?;
            Ok(OutboxPeer {
                project: r.get(0)?,
                queued: r.get(1)?,
                oldest_queued_at: parse_dt(&oldest)?,
                last_error: r.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn outbox_entry(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<OutboxEntry>, StoreError> {
        Ok(conn
            .query_row(
                "SELECT id, project, to_principal, state, attempts, last_error, created_at,
                        delivered_at, remote_ids FROM outbox WHERE id = ?1",
                params![id],
                |r| {
                    let queued: String = r.get(6)?;
                    let arrived: Option<String> = r.get(7)?;
                    let ids: Option<String> = r.get(8)?;
                    Ok(OutboxEntry {
                        id: r.get(0)?,
                        project: r.get(1)?,
                        to: r.get(2)?,
                        // The table says `delivered` once the destination accepted it.
                        state: r.get(3)?,
                        attempts: r.get(4)?,
                        last_error: r.get(5)?,
                        queued_at: parse_dt(&queued)?,
                        arrived_at: arrived.as_deref().map(parse_dt).transpose()?,
                        remote_ids: ids
                            .unwrap_or_default()
                            .split(',')
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                            .collect(),
                    })
                },
            )
            .optional()?)
    }

    pub(super) fn outbox_mark_stuck(conn: &Connection, id: &str) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE outbox SET stuck_notified_at = ?2 WHERE id = ?1",
            params![id, fmt_dt(Utc::now())],
        )?;
        Ok(())
    }

    pub(super) fn forwarded_seen(
        conn: &Connection,
        origin: &(String, String, String),
    ) -> Result<Option<Vec<String>>, StoreError> {
        let ids: Option<String> = conn
            .query_row(
                "SELECT message_ids FROM forwarded_in
                 WHERE origin_machine = ?1 AND origin_daemon = ?2 AND origin_id = ?3",
                params![origin.0, origin.1, origin.2],
                |r| r.get(0),
            )
            .optional()?;
        Ok(ids.map(|s| {
            s.split(',')
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        }))
    }

    pub(super) fn forwarded_record(
        conn: &Connection,
        origin: &(String, String, String),
        ids: &[String],
    ) -> Result<(), StoreError> {
        conn.execute(
            "INSERT OR IGNORE INTO forwarded_in(origin_machine, origin_daemon, origin_id,
                message_ids, received_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                origin.0,
                origin.1,
                origin.2,
                ids.join(","),
                fmt_dt(Utc::now())
            ],
        )?;
        Ok(())
    }

    pub(super) fn create_agent_token(
        conn: &Connection,
        agent_name: &str,
    ) -> Result<String, StoreError> {
        let id = format!("agent:{agent_name}");
        create_principal_active_only(conn, &id, PrincipalKind::Agent, agent_name)
    }

    pub(super) fn authenticate(
        conn: &Connection,
        token: &str,
    ) -> Result<Option<Principal>, StoreError> {
        let hash = hash_token(token);
        let row = conn
            .query_row(
                "SELECT id, kind FROM principals WHERE token_hash = ?1 AND revoked_at IS NULL",
                params![hash],
                |row| {
                    let id: String = row.get(0)?;
                    let kind: String = row.get(1)?;
                    Ok(Principal {
                        id,
                        kind: principal_kind_from_str(&kind),
                    })
                },
            )
            .optional()?;
        Ok(row)
    }

    pub(super) fn revoke_principal(conn: &Connection, id: &str) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE principals SET revoked_at = ?1 WHERE id = ?2",
            params![fmt_dt(Utc::now()), id],
        )?;
        Ok(())
    }

    pub(super) fn list_external_tokens(conn: &Connection) -> Result<Vec<TokenInfo>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, revoked_at FROM principals
             WHERE kind IN ('external', 'peer') OR id LIKE 'human@%' ORDER BY created_at ASC, rowid ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let created_at: String = row.get(2)?;
            let revoked_at: Option<String> = row.get(3)?;
            Ok(TokenInfo {
                principal: row.get(0)?,
                name: row.get(1)?,
                created_at: parse_dt(&created_at)?,
                revoked: revoked_at.is_some(),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn visitor_home(conn: &Connection, id: &str) -> Result<Option<String>, StoreError> {
        Ok(conn
            .query_row(
                "SELECT home FROM principals WHERE id = ?1 AND revoked_at IS NULL",
                params![id],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten())
    }

    pub(super) fn visitor_homes_like(
        conn: &Connection,
        like: &str,
    ) -> Result<Vec<String>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT home FROM principals
             WHERE id LIKE ?1 AND home IS NOT NULL AND revoked_at IS NULL",
        )?;
        let rows = stmt.query_map(params![like], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub(super) fn external_exists(conn: &Connection, name: &str) -> Result<bool, StoreError> {
        let id = token_principal_id(name);
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM principals WHERE id = ?1 AND kind IN ('external', 'human') AND revoked_at IS NULL",
                params![id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        Ok(exists)
    }

    pub(super) fn revoke_external_token(conn: &Connection, name: &str) -> Result<(), StoreError> {
        // `peer:<machine>` is named in full; anything else is an external principal's name.
        let id = if name.starts_with("peer:") {
            name.to_string()
        } else {
            token_principal_id(name)
        };
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM principals WHERE id = ?1 AND kind IN ('external', 'peer', 'human')",
                params![id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !exists {
            return Err(StoreError::NotFound(format!(
                "no such external token: {name}"
            )));
        }
        revoke_principal(conn, &id)
    }

    // ---------- agents ----------

    const AGENT_SELECT: &str = "
        SELECT a.id, a.name, a.role, a.state, a.model, a.session_id, a.pid, a.pid_start,
               a.workdir_kind, a.cwd, a.worktree, a.branch, a.created_at, a.updated_at,
               a.turns, a.cost_usd_total, a.last_event_at, a.turn_started_at,
               a.exit_code, a.exit_signal, a.exit_reason, a.created_by,
               (SELECT COUNT(*) FROM messages m WHERE m.to_kind='agent' AND m.to_id=a.id AND m.state='held') AS held_messages,
               (SELECT COUNT(*) FROM messages m WHERE m.to_kind='agent' AND m.to_id=a.id AND m.state='written') AS unacked_messages,
               a.context_tokens, a.components
        FROM agents a";

    fn row_to_agent(row: &Row<'_>) -> rusqlite::Result<Agent> {
        let state_str: String = row.get(3)?;
        let exit_code: Option<i32> = row.get(18)?;
        let exit_signal: Option<i32> = row.get(19)?;
        let exit_reason: Option<String> = row.get(20)?;
        let exit = exit_reason.map(|reason| ExitInfo {
            code: exit_code,
            signal: exit_signal,
            reason,
        });
        Ok(Agent {
            id: row.get(0)?,
            name: row.get(1)?,
            role: crate::config::canonical_role(&row.get::<_, String>(2)?),
            state: state_str.parse().map_err(|_| {
                rusqlite::Error::InvalidColumnType(3, "state".into(), rusqlite::types::Type::Text)
            })?,
            model: row.get(4)?,
            session_id: row.get(5)?,
            pid: row.get(6)?,
            cwd: row.get(9)?,
            worktree: row.get(10)?,
            branch: row.get(11)?,
            created_at: parse_dt(&row.get::<_, String>(12)?)?,
            updated_at: parse_dt(&row.get::<_, String>(13)?)?,
            turns: row.get::<_, i64>(14)? as u32,
            cost_usd_total: row.get(15)?,
            last_event_at: parse_dt_opt(row.get(16)?)?,
            turn_started_at: parse_dt_opt(row.get(17)?)?,
            exit,
            created_by: row.get(21)?,
            held_messages: row.get::<_, i64>(22)? as u32,
            unacked_messages: row.get::<_, i64>(23)? as u32,
            context_tokens: row.get::<_, Option<i64>>(24)?.map(|n| n as u64),
            components: serde_json::from_str(&row.get::<_, String>(25)?).unwrap_or_default(),
        })
    }

    pub(super) fn insert_agent(conn: &Connection, new: &NewAgent) -> Result<Agent, StoreError> {
        let id = new_agent_id();
        let now = Utc::now();
        let extra_allowed_tools =
            serde_json::to_string(&new.extra_allowed_tools).expect("Vec<String> always serializes");
        let extra_env =
            serde_json::to_string(&new.extra_env).expect("Vec<(String,String)> always serializes");
        let result = conn.execute(
            "INSERT INTO agents(id, name, role, state, model, session_id, pid, pid_start,
                workdir_kind, cwd, worktree, branch, created_at, updated_at, turns,
                cost_usd_total, last_event_at, turn_started_at, exit_code, exit_signal,
                exit_reason, created_by, extra_allowed_tools, extra_env, components,
                session_started)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7, ?8, ?9, ?10, ?11, ?11, 0, 0,
                NULL, NULL, NULL, NULL, NULL, ?12, ?13, ?14, ?15, 0)",
            params![
                id,
                new.name,
                new.role,
                AgentState::Starting.as_str(),
                new.model,
                new.session_id,
                new.workdir_kind,
                new.cwd,
                new.worktree,
                new.branch,
                fmt_dt(now),
                new.created_by,
                extra_allowed_tools,
                extra_env,
                serde_json::to_string(&new.components).expect("Vec<String> always serializes"),
            ],
        );
        match result {
            Ok(_) => {}
            Err(e) if is_unique_violation(&e) => {
                return Err(StoreError::Conflict(format!(
                    "agent name {:?} already exists",
                    new.name
                )));
            }
            Err(e) => return Err(e.into()),
        }
        Ok(Agent {
            id,
            name: new.name.clone(),
            role: new.role.clone(),
            state: AgentState::Starting,
            model: new.model.clone(),
            session_id: new.session_id.clone(),
            pid: None,
            cwd: new.cwd.clone(),
            worktree: new.worktree.clone(),
            branch: new.branch.clone(),
            created_at: now,
            updated_at: now,
            last_event_at: None,
            turns: 0,
            turn_started_at: None,
            cost_usd_total: 0.0,
            exit: None,
            created_by: new.created_by.clone(),
            held_messages: 0,
            unacked_messages: 0,
            context_tokens: None,
            components: new.components.clone(),
        })
    }

    pub(super) fn get_agent(
        conn: &Connection,
        id_or_name: &str,
    ) -> Result<Option<Agent>, StoreError> {
        let sql = format!("{AGENT_SELECT} WHERE a.id = ?1 OR a.name = ?1");
        Ok(conn
            .query_row(&sql, params![id_or_name], row_to_agent)
            .optional()?)
    }

    /// This agent's persisted `--allow-tool`/`--env` overrides from spawn
    /// (`NewAgent::extra_allowed_tools`/`extra_env`), for `renew` and
    /// `resume` to reapply. Not part of the wire `Agent` type: it's an
    /// internal detail of how the daemon rebuilds `cmd`.
    pub(super) fn get_agent_overrides(
        conn: &Connection,
        id: &str,
    ) -> Result<AgentOverrides, StoreError> {
        let (tools_json, env_json): (String, String) = conn
            .query_row(
                "SELECT extra_allowed_tools, extra_env FROM agents WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| StoreError::NotFound(id.to_string()))?;
        let tools = serde_json::from_str(&tools_json).unwrap_or_default();
        let env = serde_json::from_str(&env_json).unwrap_or_default();
        Ok((tools, env))
    }

    pub(super) fn list_agents(
        conn: &Connection,
        include_terminal: bool,
    ) -> Result<Vec<Agent>, StoreError> {
        let sql = if include_terminal {
            format!("{AGENT_SELECT} ORDER BY a.created_at ASC, a.rowid ASC")
        } else {
            format!(
                "{AGENT_SELECT} WHERE a.state NOT IN ('stopped','exited','crashed','lost') ORDER BY a.created_at ASC, a.rowid ASC"
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], row_to_agent)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn set_agent_state(
        conn: &Connection,
        id: &str,
        state: AgentState,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE agents SET state = ?1, updated_at = ?2 WHERE id = ?3",
            params![state.as_str(), fmt_dt(Utc::now()), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub(super) fn set_agent_process(
        conn: &Connection,
        id: &str,
        pid: Option<i32>,
        pid_start: Option<&str>,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE agents SET pid = ?1, pid_start = ?2, updated_at = ?3 WHERE id = ?4",
            params![pid, pid_start, fmt_dt(Utc::now()), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub(super) fn mark_session_started(conn: &Connection, id: &str) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE agents SET session_started = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub(super) fn session_started(conn: &Connection, id: &str) -> Result<bool, StoreError> {
        conn.query_row(
            "SELECT session_started FROM agents WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| StoreError::NotFound(id.to_string()))
    }

    /// Also clears `context_tokens`: a fresh session has no completed turn
    /// yet, so the old session's context size no longer applies (renew's
    /// only caller — htp6b's context governor relies on this to stop
    /// re-notifying a just-renewed agent on the very next check).
    /// Also marks the session not yet started (see `SCHEMA_V13`).
    pub(super) fn set_agent_session(
        conn: &Connection,
        id: &str,
        session_id: &str,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE agents SET session_id = ?1, session_started = 0, context_tokens = NULL,
                session_cost_usd = 0, updated_at = ?2 WHERE id = ?3",
            params![session_id, fmt_dt(Utc::now()), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub(super) fn session_start(
        conn: &Connection,
        agent_id: &str,
        session_id: &str,
        reason: &str,
        renewal_limit: f64,
    ) -> Result<(), StoreError> {
        conn.execute(
            "INSERT INTO agent_sessions (agent_id, session_id, started_at, start_reason, renewal_limit)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![agent_id, session_id, fmt_dt(Utc::now()), reason, renewal_limit],
        )?;
        Ok(())
    }

    /// Closes the agent's open session row, if any: the first ender wins, so a renewal that
    /// ends it before the process exits isn't overwritten by the exit's own reason.
    pub(super) fn session_end(
        conn: &Connection,
        agent_id: &str,
        reason: &str,
        tokens: Option<i64>,
    ) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE agent_sessions SET ended_at = ?2, end_reason = ?3, end_tokens = ?4
             WHERE agent_id = ?1 AND ended_at IS NULL",
            params![agent_id, fmt_dt(Utc::now()), reason, tokens],
        )?;
        Ok(())
    }

    pub(super) fn session_cost(conn: &Connection, id: &str) -> Result<f64, StoreError> {
        conn.query_row(
            "SELECT session_cost_usd FROM agents WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| StoreError::NotFound(id.to_string()))
    }

    pub(super) fn set_session_cost(
        conn: &Connection,
        id: &str,
        cost: f64,
    ) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE agents SET session_cost_usd = ?1 WHERE id = ?2",
            params![cost, id],
        )?;
        Ok(())
    }

    pub(super) fn set_agent_exit(
        conn: &Connection,
        id: &str,
        exit: &ExitInfo,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE agents SET exit_code = ?1, exit_signal = ?2, exit_reason = ?3, updated_at = ?4 WHERE id = ?5",
            params![exit.code, exit.signal, exit.reason, fmt_dt(Utc::now()), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub(super) fn touch_agent(
        conn: &Connection,
        id: &str,
        last_event_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE agents SET last_event_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![fmt_dt(last_event_at), fmt_dt(Utc::now()), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub(super) fn add_turn_start(
        conn: &Connection,
        id: &str,
        n: u32,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        conn.execute(
            "INSERT INTO turns(agent_id, agent_name, role, model, n, started_at, session_id)
             SELECT id, name, role, model, ?2, ?3, session_id FROM agents WHERE id = ?1",
            params![id, n, fmt_dt(at)],
        )?;
        let updated = conn.execute(
            "UPDATE agents SET turn_started_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![fmt_dt(at), fmt_dt(Utc::now()), id],
        )?;
        if updated == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub(super) fn end_turn(
        conn: &Connection,
        id: &str,
        n: u32,
        turn: &TurnEnd,
    ) -> Result<(), StoreError> {
        let now = Utc::now();
        let updated_turn = conn.execute(
            "UPDATE turns SET ended_at = ?1, subtype = ?2, is_error = ?3, terminal_reason = ?4,
                input_tokens = ?5, output_tokens = ?6, cache_read = ?7, cache_write = ?8, cost_total = ?9,
                context_tokens = ?10
             WHERE agent_id = ?11 AND n = ?12",
            params![
                fmt_dt(now),
                turn.subtype,
                turn.is_error,
                turn.terminal_reason,
                turn.input_tokens as i64,
                turn.output_tokens as i64,
                turn.cache_read as i64,
                turn.cache_write as i64,
                turn.cost_total,
                turn.context_tokens as i64,
                id,
                n,
            ],
        )?;
        if updated_turn == 0 {
            return Err(StoreError::NotFound(format!("turn {id}/{n}")));
        }
        let updated_agent = conn.execute(
            "UPDATE agents SET turns = turns + 1, cost_usd_total = cost_usd_total + ?1,
                context_tokens = ?2, turn_started_at = NULL, updated_at = ?3 WHERE id = ?4",
            params![turn.cost_total, turn.context_tokens as i64, fmt_dt(now), id],
        )?;
        if updated_agent == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub(super) fn delete_agent(conn: &Connection, id: &str) -> Result<(), StoreError> {
        let n = conn.execute("DELETE FROM agents WHERE id = ?1", params![id])?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub(super) fn running_agents(conn: &Connection) -> Result<Vec<RunningAgent>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT id, pid, pid_start, state FROM agents
             WHERE state IN ('starting', 'idle', 'working', 'stopping')",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(RunningAgent {
                id: row.get(0)?,
                pid: row.get(1)?,
                pid_start: row.get(2)?,
                state: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ---------- messages ----------

    const MESSAGE_SELECT: &str = "
        SELECT id, from_principal, to_kind, to_id, kind, body, reply_to, when_mode, state,
               created_at, written_at, delivered_at, read_at, answered_by, answered_reply,
               answered_line, incident_task
        FROM messages";

    fn row_to_message(row: &Row<'_>) -> rusqlite::Result<Message> {
        let to_kind: String = row.get(2)?;
        let to_id: String = row.get(3)?;
        let to = if to_kind == "human" {
            "human".to_string()
        } else {
            to_id
        };
        Ok(Message {
            id: row.get(0)?,
            from: row.get(1)?,
            to,
            kind: kind_from_str(&row.get::<_, String>(4)?),
            body: row.get(5)?,
            reply_to: row.get(6)?,
            when: when_from_str(&row.get::<_, String>(7)?),
            state: msg_state_from_str(&row.get::<_, String>(8)?),
            created_at: parse_dt(&row.get::<_, String>(9)?)?,
            written_at: parse_dt_opt(row.get(10)?)?,
            delivered_at: parse_dt_opt(row.get(11)?)?,
            read_at: parse_dt_opt(row.get(12)?)?,
            answered_by: row.get(13)?,
            answered_reply: row.get(14)?,
            answered_line: row.get(15)?,
            incident_task: row.get(16)?,
            recipient_note: None,
        })
    }

    pub(super) fn insert_message(
        conn: &Connection,
        new: &NewMessage,
    ) -> Result<Message, StoreError> {
        let now = Utc::now();
        conn.execute(
            "INSERT INTO messages(id, from_principal, to_kind, to_id, kind, body, reply_to,
                when_mode, state, created_at, written_at, delivered_at, read_at)
             VALUES ('', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, NULL, NULL)",
            params![
                new.from,
                new.to_kind.as_str(),
                new.to,
                kind_str(new.kind),
                new.body,
                new.reply_to,
                when_str(new.when),
                msg_state_str(new.state),
                fmt_dt(now),
            ],
        )?;
        let seq = conn.last_insert_rowid();
        let id = format!("m-{seq:04}");
        conn.execute(
            "UPDATE messages SET id = ?1 WHERE seq = ?2",
            params![id, seq],
        )?;
        Ok(Message {
            id,
            from: new.from.clone(),
            to: new.to.clone(),
            kind: new.kind,
            body: new.body.clone(),
            reply_to: new.reply_to.clone(),
            when: new.when,
            state: new.state,
            created_at: now,
            written_at: None,
            delivered_at: None,
            read_at: None,
            answered_by: None,
            answered_reply: None,
            answered_line: None,
            incident_task: None,
            recipient_note: None,
        })
    }

    pub(super) fn get_message(conn: &Connection, id: &str) -> Result<Option<Message>, StoreError> {
        let sql = format!("{MESSAGE_SELECT} WHERE id = ?1");
        Ok(conn
            .query_row(&sql, params![id], row_to_message)
            .optional()?)
    }

    pub(super) fn list_messages(
        conn: &Connection,
        q: &ListMessages,
    ) -> Result<Vec<Message>, StoreError> {
        let sql = format!(
            "{MESSAGE_SELECT}
             WHERE (?1 IS NULL OR to_id = ?1)
               AND (?2 IS NULL OR from_principal = ?2)
               AND (?3 = 0 OR state NOT IN ('read', 'dropped'))
             ORDER BY seq DESC
             LIMIT ?4"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            params![q.to, q.from, q.unread, q.limit.unwrap_or(u32::MAX)],
            row_to_message,
        )?;
        let mut msgs = rows.collect::<Result<Vec<_>, _>>()?;
        msgs.reverse(); // chronological order, oldest first
        Ok(msgs)
    }

    pub(super) fn set_message_state(
        conn: &Connection,
        id: &str,
        state: MessageState,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE messages SET state = ?1,
                written_at = CASE WHEN ?1 = 'written' THEN ?2 ELSE written_at END,
                delivered_at = CASE WHEN ?1 = 'delivered' THEN ?2 ELSE delivered_at END,
                read_at = CASE WHEN ?1 = 'read' THEN ?2 WHEN ?1 = 'delivered' THEN NULL ELSE read_at END
             WHERE id = ?3",
            params![msg_state_str(state), fmt_dt(at), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Closes a still-open message as answered; a no-op if it's already read.
    pub(super) fn answer_message(
        conn: &Connection,
        id: &str,
        by: &str,
        reply: &str,
        line: &str,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE messages SET state = 'read', read_at = ?1, answered_by = ?2, answered_reply = ?3,
                answered_line = ?4
             WHERE id = ?5 AND state NOT IN ('read', 'dropped')",
            params![fmt_dt(at), by, reply, line, id],
        )?;
        Ok(())
    }

    pub(super) fn mark_message_written(
        conn: &Connection,
        id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        conn.execute(
            "UPDATE messages SET state = 'written', written_at = ?1
             WHERE id = ?2 AND state = 'pending'",
            params![fmt_dt(at), id],
        )?;
        Ok(())
    }

    pub(super) fn messages_for_agent(
        conn: &Connection,
        agent_id: &str,
        states: &[MessageState],
    ) -> Result<Vec<Message>, StoreError> {
        if states.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = states.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "{MESSAGE_SELECT} WHERE to_kind = 'agent' AND to_id = ? AND state IN ({placeholders}) ORDER BY seq ASC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut all_params: Vec<String> = vec![agent_id.to_string()];
        all_params.extend(states.iter().map(|s| msg_state_str(*s).to_string()));
        let rows = stmt.query_map(params_from_iter(all_params.iter()), row_to_message)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Every message announcing `task`, oldest first.
    pub(super) fn messages_for_incident(
        conn: &Connection,
        task: &str,
    ) -> Result<Vec<Message>, StoreError> {
        let sql = format!("{MESSAGE_SELECT} WHERE incident_task = ?1 ORDER BY seq ASC");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![task], row_to_message)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ---------- events ----------

    pub(super) fn append_event(
        conn: &Connection,
        kind: &str,
        actor: &str,
        agent: Option<&str>,
        data: &serde_json::Value,
    ) -> Result<Event, StoreError> {
        let ts = Utc::now();
        conn.execute(
            "INSERT INTO events(ts, kind, actor, agent_id, data) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![fmt_dt(ts), kind, actor, agent, data.to_string()],
        )?;
        let seq = conn.last_insert_rowid();
        Ok(Event {
            seq,
            ts,
            kind: kind.to_string(),
            actor: actor.to_string(),
            agent: agent.map(str::to_string),
            data: data.clone(),
        })
    }

    fn row_to_event(row: &Row<'_>) -> rusqlite::Result<Event> {
        let data_text: String = row.get(4)?;
        let data = serde_json::from_str(&data_text).unwrap_or(serde_json::Value::Null);
        Ok(Event {
            seq: row.get(0)?,
            ts: parse_dt(&row.get::<_, String>(1)?)?,
            kind: row.get(2)?,
            actor: row.get(3)?,
            agent: row.get(5)?,
            data,
        })
    }

    pub(super) fn list_events(conn: &Connection, q: &EventQuery) -> Result<Vec<Event>, StoreError> {
        let limit = q.limit.unwrap_or(500);
        // No cursor: "recent", not "oldest" — take the newest `limit` rows
        // and put them back in ascending order so the shape is unchanged.
        // A cursor is forward pagination and keeps the ascending scan.
        if q.since.is_none() {
            let sql = "SELECT seq, ts, kind, actor, data, agent_id FROM events
                 WHERE (?1 IS NULL OR agent_id = ?1)
                   AND (?2 IS NULL OR kind LIKE ?2 || '%')
                   AND (?3 IS NULL OR json_extract(data, '$.message') = ?3)
                   AND (?4 IS NULL OR json_extract(data, '$.message') IN
                        (SELECT id FROM messages WHERE to_id = ?4))
                 ORDER BY seq DESC
                 LIMIT ?5";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(
                params![q.agent, q.kind, q.message, q.to, limit],
                row_to_event,
            )?;
            let mut events = rows.collect::<Result<Vec<_>, _>>()?;
            events.reverse();
            return Ok(events);
        }
        let sql = "SELECT seq, ts, kind, actor, data, agent_id FROM events
             WHERE seq > ?1
               AND (?2 IS NULL OR agent_id = ?2)
               AND (?3 IS NULL OR kind LIKE ?3 || '%')
               AND (?4 IS NULL OR json_extract(data, '$.message') = ?4)
               AND (?5 IS NULL OR json_extract(data, '$.message') IN
                    (SELECT id FROM messages WHERE to_id = ?5))
             ORDER BY seq ASC
             LIMIT ?6";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![q.since, q.agent, q.kind, q.message, q.to, limit],
            row_to_event,
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn prune_events(
        conn: &Connection,
        older_than: DateTime<Utc>,
    ) -> Result<u64, StoreError> {
        let n = conn.execute(
            "DELETE FROM events WHERE ts < ?1",
            params![fmt_dt(older_than)],
        )?;
        Ok(n as u64)
    }

    // ---------- tasks ----------

    fn row_to_task(row: &Row<'_>) -> rusqlite::Result<TicketRow> {
        let kind: String = row.get(2)?;
        let state: String = row.get(3)?;
        Ok(TicketRow {
            id: row.get(0)?,
            title: row.get(1)?,
            kind: kind.parse().map_err(|_| {
                rusqlite::Error::InvalidColumnType(2, "kind".into(), rusqlite::types::Type::Text)
            })?,
            state: state.parse().map_err(|_| {
                rusqlite::Error::InvalidColumnType(3, "state".into(), rusqlite::types::Type::Text)
            })?,
            created_at: parse_dt(&row.get::<_, String>(4)?)?,
            updated_at: parse_dt(&row.get::<_, String>(5)?)?,
            created_by: row.get(6)?,
        })
    }

    /// Retries the id up to this many times on a unique-constraint
    /// collision (31^4 is ~920k values, so this is generous) before
    /// giving up.
    const TASK_ID_ATTEMPTS: u32 = 8;

    pub(super) fn insert_task(
        conn: &Connection,
        prefix: &str,
        title: &str,
        kind: TicketKind,
        created_by: &str,
        wanted: Option<&str>,
    ) -> Result<TicketRow, StoreError> {
        // Truncate to millisecond precision (matching fmt_dt's on-disk format) so the
        // returned in-memory value can't be strictly less than a later floor-truncated
        // read of the same row.
        let now = Utc::now().trunc_subsecs(3);
        for attempt in 0..TASK_ID_ATTEMPTS {
            // Only the first attempt tries the ticket's id: a unique violation means a task
            // already has it.
            let id = match wanted {
                Some(w) if attempt == 0 => w.to_string(),
                _ => new_task_id(prefix),
            };
            let result = conn.execute(
                "INSERT INTO tickets(id, title, kind, state, created_at, updated_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
                params![
                    id,
                    title,
                    kind.as_str(),
                    TicketState::Pending.as_str(),
                    fmt_dt(now),
                    created_by
                ],
            );
            match result {
                Ok(_) => {
                    return Ok(TicketRow {
                        id,
                        title: title.to_string(),
                        kind,
                        state: TicketState::Pending,
                        created_at: now,
                        updated_at: now,
                        created_by: Some(created_by.to_string()),
                    });
                }
                Err(e) if is_unique_violation(&e) => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Err(StoreError::Conflict(format!(
            "could not generate a unique task id with prefix {prefix:?} after {TASK_ID_ATTEMPTS} attempts"
        )))
    }

    pub(super) fn insert_task_row(conn: &Connection, row: &TicketRow) -> Result<(), StoreError> {
        let result = conn.execute(
            "INSERT INTO tickets(id, title, kind, state, created_at, updated_at, created_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                row.id,
                row.title,
                row.kind.as_str(),
                row.state.as_str(),
                fmt_dt(row.created_at),
                fmt_dt(row.updated_at),
                row.created_by,
            ],
        );
        match result {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => Err(StoreError::Conflict(format!(
                "task {:?} already exists",
                row.id
            ))),
            Err(e) => Err(e.into()),
        }
    }

    pub(super) fn get_task(conn: &Connection, id: &str) -> Result<Option<TicketRow>, StoreError> {
        Ok(conn
            .query_row(
                "SELECT id, title, kind, state, created_at, updated_at, created_by FROM tickets WHERE id = ?1",
                params![id],
                row_to_task,
            )
            .optional()?)
    }

    pub(super) fn list_tasks(conn: &Connection) -> Result<Vec<TicketRow>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT id, title, kind, state, created_at, updated_at, created_by FROM tickets ORDER BY created_at ASC, rowid ASC",
        )?;
        let rows = stmt.query_map([], row_to_task)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn set_task_title(
        conn: &Connection,
        id: &str,
        title: &str,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE tickets SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, fmt_dt(Utc::now()), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(format!("no such task: {id}")));
        }
        Ok(())
    }

    pub(super) fn set_task_kind(
        conn: &Connection,
        id: &str,
        kind: TicketKind,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE tickets SET kind = ?1, updated_at = ?2 WHERE id = ?3",
            params![kind.as_str(), fmt_dt(Utc::now()), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(format!("no such task: {id}")));
        }
        Ok(())
    }

    pub(super) fn set_task_state(
        conn: &Connection,
        id: &str,
        state: TicketState,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE tickets SET state = ?1, updated_at = ?2 WHERE id = ?3",
            params![state.as_str(), fmt_dt(Utc::now()), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(format!("no such task: {id}")));
        }
        Ok(())
    }

    // ---------- edges ----------

    fn row_to_edge(row: &Row<'_>) -> rusqlite::Result<Edge> {
        let kind: String = row.get(2)?;
        Ok(Edge {
            from: row.get(0)?,
            to: row.get(1)?,
            kind: kind.parse().map_err(|_| {
                rusqlite::Error::InvalidColumnType(2, "kind".into(), rusqlite::types::Type::Text)
            })?,
            created_at: parse_dt(&row.get::<_, String>(3)?)?,
        })
    }

    pub(super) fn insert_edge(
        conn: &Connection,
        from: &str,
        to: &str,
        kind: EdgeKind,
    ) -> Result<Edge, StoreError> {
        // Truncated to millisecond precision (matching fmt_dt's on-disk
        // format, and insert_task's own fix above) so the returned in-memory
        // value matches a later `list_edges` read of the same row exactly.
        let now = Utc::now().trunc_subsecs(3);
        let result = conn.execute(
            "INSERT INTO edges(from_task, to_task, kind, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![from, to, kind.as_str(), fmt_dt(now)],
        );
        match result {
            Ok(_) => Ok(Edge {
                from: from.to_string(),
                to: to.to_string(),
                kind,
                created_at: now,
            }),
            Err(e) if is_unique_violation(&e) => Err(StoreError::Conflict(format!(
                "edge already exists: {from} -{}-> {to}",
                kind.as_str()
            ))),
            Err(e) => Err(e.into()),
        }
    }

    pub(super) fn insert_edge_row(conn: &Connection, edge: &Edge) -> Result<(), StoreError> {
        let result = conn.execute(
            "INSERT INTO edges(from_task, to_task, kind, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                edge.from,
                edge.to,
                edge.kind.as_str(),
                fmt_dt(edge.created_at)
            ],
        );
        match result {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => Err(StoreError::Conflict(format!(
                "edge already exists: {} -{}-> {}",
                edge.from,
                edge.kind.as_str(),
                edge.to
            ))),
            Err(e) => Err(e.into()),
        }
    }

    pub(super) fn delete_edge(
        conn: &Connection,
        from: &str,
        to: &str,
        kind: EdgeKind,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "DELETE FROM edges WHERE from_task = ?1 AND to_task = ?2 AND kind = ?3",
            params![from, to, kind.as_str()],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(format!(
                "no such edge: {from} -{}-> {to}",
                kind.as_str()
            )));
        }
        Ok(())
    }

    pub(super) fn list_edges(conn: &Connection) -> Result<Vec<Edge>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT from_task, to_task, kind, created_at FROM edges ORDER BY created_at ASC, rowid ASC",
        )?;
        let rows = stmt.query_map([], row_to_edge)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ---------- ports ----------

    const PORT_COLS: &str = "port, agent, task, pid, label, allocated_at";

    fn row_to_port(row: &Row<'_>) -> rusqlite::Result<PortAllocation> {
        Ok(PortAllocation {
            port: row.get(0)?,
            agent: row.get(1)?,
            task: row.get(2)?,
            pid: row.get(3)?,
            label: row.get(4)?,
            allocated_at: parse_dt(&row.get::<_, String>(5)?)?,
        })
    }

    pub(super) fn insert_port(conn: &Connection, p: &PortAllocation) -> Result<bool, StoreError> {
        let n = conn.execute(
            "INSERT OR IGNORE INTO ports(port, agent, task, pid, label, allocated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                p.port,
                p.agent,
                p.task,
                p.pid,
                p.label,
                fmt_dt(p.allocated_at)
            ],
        )?;
        Ok(n > 0)
    }

    pub(super) fn list_ports(conn: &Connection) -> Result<Vec<PortAllocation>, StoreError> {
        let mut stmt = conn.prepare(&format!("SELECT {PORT_COLS} FROM ports ORDER BY port ASC"))?;
        let rows = stmt.query_map([], row_to_port)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn release_port(
        conn: &Connection,
        port: u16,
    ) -> Result<Option<PortAllocation>, StoreError> {
        let found = conn
            .query_row(
                &format!("SELECT {PORT_COLS} FROM ports WHERE port = ?1"),
                [port],
                row_to_port,
            )
            .optional()?;
        conn.execute("DELETE FROM ports WHERE port = ?1", [port])?;
        Ok(found)
    }

    // ---------- handovers ----------

    const HANDOVER_COLS: &str = "id, role, project, body, created_at, created_by";

    fn row_to_handover(row: &Row<'_>) -> rusqlite::Result<Handover> {
        Ok(Handover {
            id: row.get(0)?,
            role: row.get(1)?,
            project: row.get(2)?,
            body: row.get(3)?,
            created_at: parse_dt(&row.get::<_, String>(4)?)?,
            created_by: row.get(5)?,
        })
    }

    pub(super) fn insert_handover(
        conn: &Connection,
        role: &str,
        project: &str,
        body: &str,
        created_by: &str,
    ) -> Result<Handover, StoreError> {
        let now = Utc::now();
        conn.execute(
            "INSERT INTO handovers(id, role, project, body, created_at, created_by)
             VALUES ('', ?1, ?2, ?3, ?4, ?5)",
            params![role, project, body, fmt_dt(now), created_by],
        )?;
        let seq = conn.last_insert_rowid();
        let id = format!("h-{seq:04}");
        conn.execute(
            "UPDATE handovers SET id = ?1 WHERE seq = ?2",
            params![id, seq],
        )?;
        Ok(Handover {
            id,
            role: role.to_string(),
            project: project.to_string(),
            body: body.to_string(),
            created_at: now,
            created_by: created_by.to_string(),
        })
    }

    pub(super) fn restore_handover(conn: &Connection, h: &Handover) -> Result<(), StoreError> {
        conn.execute(
            "INSERT OR IGNORE INTO handovers(seq, id, role, project, body, created_at, created_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                crate::state_branch::handover_seq(&h.id),
                h.id,
                h.role,
                h.project,
                h.body,
                fmt_dt(h.created_at),
                h.created_by
            ],
        )?;
        Ok(())
    }

    pub(super) fn list_handovers(conn: &Connection) -> Result<Vec<Handover>, StoreError> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {HANDOVER_COLS} FROM handovers ORDER BY seq DESC"
        ))?;
        let rows = stmt.query_map([], row_to_handover)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn get_handover(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<Handover>, StoreError> {
        Ok(conn
            .query_row(
                &format!("SELECT {HANDOVER_COLS} FROM handovers WHERE id = ?1"),
                [id],
                row_to_handover,
            )
            .optional()?)
    }

    pub(super) fn prune_handovers(
        conn: &Connection,
        older_than: DateTime<Utc>,
    ) -> Result<u64, StoreError> {
        let n = conn.execute(
            "DELETE FROM handovers WHERE created_at < ?1
               AND seq NOT IN (SELECT MAX(seq) FROM handovers GROUP BY role)",
            params![fmt_dt(older_than)],
        )?;
        Ok(n as u64)
    }

    // ---------- conflicts ----------

    const CONFLICT_COLS: &str =
        "id, task_a, task_b, kind, key, state, resolution, opened_at, resolved_at";

    fn row_to_conflict(row: &Row<'_>) -> rusqlite::Result<Conflict> {
        let resolved_at: Option<String> = row.get(8)?;
        Ok(Conflict {
            id: format!("C{}", row.get::<_, i64>(0)?),
            tasks: [row.get(1)?, row.get(2)?],
            kind: row.get(3)?,
            key: row.get(4)?,
            state: row.get(5)?,
            resolution: row.get(6)?,
            opened_at: parse_dt(&row.get::<_, String>(7)?)?,
            resolved_at: resolved_at.as_deref().map(parse_dt).transpose()?,
        })
    }

    pub(super) fn open_conflict(
        conn: &Connection,
        tasks: &[String; 2],
        kind: &str,
        key: &str,
    ) -> Result<Option<Conflict>, StoreError> {
        let n = conn.execute(
            "INSERT OR IGNORE INTO conflicts(task_a, task_b, kind, key, opened_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![tasks[0], tasks[1], kind, key, fmt_dt(Utc::now())],
        )?;
        if n == 0 {
            return Ok(None);
        }
        let id = format!("C{}", conn.last_insert_rowid());
        get_conflict(conn, &id)
    }

    pub(super) fn get_conflict(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<Conflict>, StoreError> {
        let Some(n) = id.strip_prefix('C').and_then(|n| n.parse::<i64>().ok()) else {
            return Ok(None);
        };
        Ok(conn
            .query_row(
                &format!("SELECT {CONFLICT_COLS} FROM conflicts WHERE id = ?1"),
                [n],
                row_to_conflict,
            )
            .optional()?)
    }

    pub(super) fn list_conflicts(conn: &Connection) -> Result<Vec<Conflict>, StoreError> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {CONFLICT_COLS} FROM conflicts ORDER BY id ASC"
        ))?;
        let rows = stmt.query_map([], row_to_conflict)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn resolve_conflict(
        conn: &Connection,
        id: &str,
        resolution: &str,
    ) -> Result<(), StoreError> {
        let n = id
            .strip_prefix('C')
            .and_then(|n| n.parse::<i64>().ok())
            .ok_or_else(|| StoreError::NotFound(format!("no such conflict: {id}")))?;
        let changed = conn.execute(
            "UPDATE conflicts SET state = 'resolved', resolution = ?1, resolved_at = ?2 \
             WHERE id = ?3 AND state = 'open'",
            params![resolution, fmt_dt(Utc::now()), n],
        )?;
        if changed == 0 {
            return Err(match get_conflict(conn, id)? {
                Some(_) => StoreError::Conflict(format!("conflict {id} is already resolved")),
                None => StoreError::NotFound(format!("no such conflict: {id}")),
            });
        }
        Ok(())
    }

    // ---------- open questions ----------

    fn row_to_open_question(row: &Row<'_>) -> rusqlite::Result<OpenQuestion> {
        Ok(OpenQuestion {
            task_id: row.get(0)?,
            message_id: row.get(1)?,
            asked_by: row.get(2)?,
            asked_at: parse_dt(&row.get::<_, String>(3)?)?,
        })
    }

    pub(super) fn insert_open_question(
        conn: &Connection,
        task_id: &str,
        message_id: &str,
        asked_by: &str,
        asked_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let result = conn.execute(
            "INSERT INTO open_questions(task_id, message_id, asked_by, asked_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![task_id, message_id, asked_by, fmt_dt(asked_at)],
        );
        match result {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => Err(StoreError::Conflict(format!(
                "task {task_id} already has an open question"
            ))),
            Err(e) => Err(e.into()),
        }
    }

    pub(super) fn delete_open_question(conn: &Connection, task_id: &str) -> Result<(), StoreError> {
        let n = conn.execute(
            "DELETE FROM open_questions WHERE task_id = ?1",
            params![task_id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(format!(
                "task {task_id} has no open question"
            )));
        }
        Ok(())
    }

    pub(super) fn list_open_questions(conn: &Connection) -> Result<Vec<OpenQuestion>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT task_id, message_id, asked_by, asked_at FROM open_questions ORDER BY asked_at ASC, rowid ASC",
        )?;
        let rows = stmt.query_map([], row_to_open_question)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ---------- claims ----------

    fn row_to_claim(row: &Row<'_>) -> rusqlite::Result<Claim> {
        Ok(Claim {
            task_id: row.get(0)?,
            claimed_by: row.get(1)?,
            claimed_at: parse_dt(&row.get::<_, String>(2)?)?,
        })
    }

    pub(super) fn insert_claim(
        conn: &Connection,
        task_id: &str,
        claimed_by: &str,
        claimed_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        let result = conn.execute(
            "INSERT INTO claims(task_id, claimed_by, claimed_at) VALUES (?1, ?2, ?3)",
            params![task_id, claimed_by, fmt_dt(claimed_at)],
        );
        match result {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => Err(StoreError::Conflict(format!(
                "task {task_id} is already claimed"
            ))),
            Err(e) => Err(e.into()),
        }
    }

    pub(super) fn delete_claim(conn: &Connection, task_id: &str) -> Result<(), StoreError> {
        let n = conn.execute("DELETE FROM claims WHERE task_id = ?1", params![task_id])?;
        if n == 0 {
            return Err(StoreError::NotFound(format!(
                "task {task_id} is not claimed"
            )));
        }
        Ok(())
    }

    pub(super) fn end_claim(
        conn: &Connection,
        task_id: &str,
        state: TicketState,
    ) -> Result<(), StoreError> {
        let tx = conn.unchecked_transaction()?;
        tx.execute("DELETE FROM claims WHERE task_id = ?1", params![task_id])?;
        set_task_state(&tx, task_id, state)?;
        tx.commit()?;
        Ok(())
    }

    pub(super) fn list_claims(conn: &Connection) -> Result<Vec<Claim>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT task_id, claimed_by, claimed_at FROM claims ORDER BY claimed_at ASC, rowid ASC",
        )?;
        let rows = stmt.query_map([], row_to_claim)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ---------- rate limits / usage ----------

    pub(super) fn upsert_rate_limit(conn: &Connection, rl: &RateLimit) -> Result<(), StoreError> {
        conn.execute(
            "INSERT INTO rate_limits(window, status, utilization, resets_at, observed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(window) DO UPDATE SET
                status = excluded.status, utilization = excluded.utilization,
                resets_at = excluded.resets_at, observed_at = excluded.observed_at",
            params![
                rl.window,
                rl.status,
                rl.utilization,
                rl.resets_at.map(fmt_dt),
                fmt_dt(rl.observed_at)
            ],
        )?;
        append_rate_limit_history(conn, rl)
    }

    /// Appends the reading only when it differs from the window's last one, so a poll every
    /// 30 s that sees the same numbers adds nothing; prunes past the retention at the same time.
    fn append_rate_limit_history(conn: &Connection, rl: &RateLimit) -> Result<(), StoreError> {
        let last: Option<(Option<f64>, Option<String>)> = conn
            .query_row(
                "SELECT utilization, resets_at FROM rate_limit_history
                 WHERE window = ?1 ORDER BY id DESC LIMIT 1",
                params![rl.window],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let resets_at = rl.resets_at.map(fmt_dt);
        if last.is_some_and(|(u, r)| u == rl.utilization && r == resets_at) {
            return Ok(());
        }
        conn.execute(
            "INSERT INTO rate_limit_history(window, utilization, resets_at, observed_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![rl.window, rl.utilization, resets_at, fmt_dt(rl.observed_at)],
        )?;
        conn.execute(
            "DELETE FROM rate_limit_history WHERE observed_at < ?1",
            params![fmt_dt(
                rl.observed_at - chrono::Duration::days(RATE_LIMIT_HISTORY_DAYS)
            )],
        )?;
        Ok(())
    }

    pub(super) fn rate_limit_history(
        conn: &Connection,
        window: &str,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RateLimitPoint>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT window, utilization, resets_at, observed_at FROM rate_limit_history
             WHERE window = ?1 AND observed_at >= ?2 ORDER BY observed_at, id",
        )?;
        let rows = stmt.query_map(
            params![window, since.map(fmt_dt).unwrap_or_default()],
            |row| {
                Ok(RateLimitPoint {
                    window: row.get(0)?,
                    utilization: row.get(1)?,
                    resets_at: parse_dt_opt(row.get(2)?)?,
                    observed_at: parse_dt(&row.get::<_, String>(3)?)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn rate_limits(conn: &Connection) -> Result<Vec<RateLimit>, StoreError> {
        let mut stmt = conn.prepare("SELECT window, status, utilization, resets_at, observed_at FROM rate_limits ORDER BY window")?;
        let rows = stmt.query_map([], |row| {
            Ok(RateLimit {
                window: row.get(0)?,
                status: row.get(1)?,
                utilization: row.get(2)?,
                resets_at: parse_dt_opt(row.get(3)?)?,
                observed_at: parse_dt(&row.get::<_, String>(4)?)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    #[derive(Default)]
    struct TurnTimeStats {
        busy_seconds: i64,
        min_started: Option<DateTime<Utc>>,
        max_ended: Option<DateTime<Utc>>,
    }

    impl TurnTimeStats {
        fn wall_seconds(&self) -> Option<u64> {
            let (s, e) = (self.min_started?, self.max_ended?);
            Some((e - s).num_seconds().max(0) as u64)
        }
    }

    /// Busy and wall time per `group_col` value (`agent_id`, `role`, or
    /// `model`), from the turns ledger directly rather than SQL date
    /// arithmetic, since `chrono` already parses the RFC3339 timestamps
    /// `fmt_dt`/`parse_dt` write and reads them back exactly.
    fn turn_time_stats(
        conn: &Connection,
        group_col: &str,
        since: Option<&str>,
    ) -> Result<BTreeMap<String, TurnTimeStats>, StoreError> {
        let sql = format!(
            "SELECT {group_col} AS key, started_at, ended_at FROM turns
             WHERE (?1 IS NULL OR started_at >= ?1)"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![since], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        let mut acc: BTreeMap<String, TurnTimeStats> = BTreeMap::new();
        for row in rows {
            let (key, started, ended) = row?;
            let started_dt = parse_dt(&started)?;
            let ended_dt = parse_dt_opt(ended)?;
            let entry = acc.entry(key).or_default();
            if let Some(e) = ended_dt {
                entry.busy_seconds += (e - started_dt).num_seconds().max(0);
                entry.max_ended = Some(entry.max_ended.map_or(e, |m| m.max(e)));
            }
            entry.min_started = Some(entry.min_started.map_or(started_dt, |m| m.min(started_dt)));
        }
        Ok(acc)
    }

    pub(super) fn usage(conn: &Connection) -> Result<Usage, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, role, model, turns, cost, tin, tout, cr, cw, removed FROM (
                 SELECT a.id, a.name, a.role, a.model, a.turns, a.cost_usd_total AS cost,
                        COALESCE(SUM(t.input_tokens), 0) AS tin, COALESCE(SUM(t.output_tokens), 0) AS tout,
                        COALESCE(SUM(t.cache_read), 0) AS cr, COALESCE(SUM(t.cache_write), 0) AS cw,
                        0 AS removed
                 FROM agents a LEFT JOIN turns t ON t.agent_id = a.id
                 GROUP BY a.id
                 UNION ALL
                 SELECT t.agent_id, MAX(t.agent_name), MAX(t.role), MAX(t.model),
                        COUNT(t.ended_at), SUM(t.cost_total),
                        SUM(t.input_tokens), SUM(t.output_tokens), SUM(t.cache_read), SUM(t.cache_write),
                        1
                 FROM turns t WHERE t.agent_id NOT IN (SELECT id FROM agents)
                 GROUP BY t.agent_id
             ) ORDER BY removed, name",
        )?;
        let time_stats = turn_time_stats(conn, "agent_id", None)?;
        let rows = stmt.query_map([], |row| {
            let agent: String = row.get(0)?;
            let stats = time_stats.get(&agent);
            Ok(AgentUsage {
                agent: agent.clone(),
                name: row.get(1)?,
                role: row.get(2)?,
                model: row.get(3)?,
                turns: row.get::<_, i64>(4)? as u32,
                cost_usd_total: row.get(5)?,
                tokens: TokenTotals {
                    input: row.get::<_, i64>(6)? as u64,
                    output: row.get::<_, i64>(7)? as u64,
                    cache_read: row.get::<_, i64>(8)? as u64,
                    cache_write: row.get::<_, i64>(9)? as u64,
                },
                busy_seconds: stats.map(|s| s.busy_seconds as u64).unwrap_or(0),
                wall_seconds: stats.and_then(TurnTimeStats::wall_seconds),
                removed: row.get::<_, i64>(10)? != 0,
            })
        })?;
        let agents = rows.collect::<Result<Vec<_>, _>>()?;

        let mut total_tokens = TokenTotals::default();
        let mut total_turns = 0u32;
        let mut total_cost = 0.0f64;
        for a in &agents {
            total_tokens.input += a.tokens.input;
            total_tokens.output += a.tokens.output;
            total_tokens.cache_read += a.tokens.cache_read;
            total_tokens.cache_write += a.tokens.cache_write;
            total_turns += a.turns;
            total_cost += a.cost_usd_total;
        }
        let denom = total_tokens.input + total_tokens.cache_read + total_tokens.cache_write;
        let cache_hit_ratio = if denom > 0 {
            Some(total_tokens.cache_read as f64 / denom as f64)
        } else {
            None
        };

        Ok(Usage {
            agents,
            total_turns,
            total_tokens,
            cache_hit_ratio,
            total_cost_usd: total_cost,
            rate_limits: rate_limits(conn)?,
            interactive_today: interactive_usage_today(conn)?,
            external: external_usage(conn, None, UsageGroupBy::Agent)?,
        })
    }

    /// Aggregates the turns ledger by role, model or agent, since the turns
    /// table already carries all three per row (docs/design/usage-and-budget.md,
    /// "Tracking token use over time"). Unlike `usage()`, this reads turns
    /// directly rather than `agents.cost_usd_total`, so `since` filtering
    /// applies uniformly, including to the `agent` grouping.
    pub(super) fn usage_breakdown(
        conn: &Connection,
        since: Option<DateTime<Utc>>,
        by: UsageGroupBy,
    ) -> Result<UsageBreakdown, StoreError> {
        let group_col = match by {
            UsageGroupBy::Role => "role",
            UsageGroupBy::Model => "model",
            UsageGroupBy::Agent => "agent_id",
        };
        let sql = format!(
            "SELECT {group_col} AS key, COUNT(*),
                    COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0),
                    COALESCE(SUM(cache_read), 0), COALESCE(SUM(cache_write), 0),
                    COALESCE(SUM(cost_total), 0)
             FROM turns
             WHERE (?1 IS NULL OR started_at >= ?1)
             GROUP BY {group_col}
             ORDER BY key"
        );
        let since_str = since.map(fmt_dt);
        let time_stats = turn_time_stats(conn, group_col, since_str.as_deref())?;
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![since_str], |row| {
            let tokens = TokenTotals {
                input: row.get::<_, i64>(2)? as u64,
                output: row.get::<_, i64>(3)? as u64,
                cache_read: row.get::<_, i64>(4)? as u64,
                cache_write: row.get::<_, i64>(5)? as u64,
            };
            let denom = tokens.input + tokens.cache_read + tokens.cache_write;
            let cache_hit_ratio = (denom > 0).then(|| tokens.cache_read as f64 / denom as f64);
            let key: String = row.get(0)?;
            let stats = time_stats.get(&key);
            let wall_seconds = (by == UsageGroupBy::Agent)
                .then(|| stats.and_then(TurnTimeStats::wall_seconds))
                .flatten();
            Ok(UsageGroup {
                key,
                turns: row.get::<_, i64>(1)? as u32,
                tokens,
                cost_usd_total: row.get(6)?,
                cache_hit_ratio,
                busy_seconds: stats.map(|s| s.busy_seconds as u64).unwrap_or(0),
                wall_seconds,
            })
        })?;
        let groups = rows.collect::<Result<Vec<_>, _>>()?;

        let mut total_tokens = TokenTotals::default();
        let mut total_turns = 0u32;
        let mut total_cost = 0.0f64;
        for g in &groups {
            total_tokens.input += g.tokens.input;
            total_tokens.output += g.tokens.output;
            total_tokens.cache_read += g.tokens.cache_read;
            total_tokens.cache_write += g.tokens.cache_write;
            total_turns += g.turns;
            total_cost += g.cost_usd_total;
        }
        let denom = total_tokens.input + total_tokens.cache_read + total_tokens.cache_write;
        let cache_hit_ratio = (denom > 0).then(|| total_tokens.cache_read as f64 / denom as f64);

        Ok(UsageBreakdown {
            by,
            since,
            groups,
            total_turns,
            total_tokens,
            cache_hit_ratio,
            total_cost_usd: total_cost,
            external: external_usage(conn, since, by)?,
        })
    }

    fn bucket_secs(b: UsageBucket) -> i64 {
        match b {
            UsageBucket::Hour => 3600,
            UsageBucket::Day => 86400,
        }
    }

    const EXTERNAL_PREFIX: &str = "external:";
    const UNBRIDLED: &str = "external:unbridled";

    pub(super) fn usage_series(
        conn: &Connection,
        agent_id: &str,
        since: Option<DateTime<Utc>>,
        bucket: UsageBucket,
    ) -> Result<UsageSeries, StoreError> {
        if agent_id.starts_with(EXTERNAL_PREFIX) {
            return external_series(conn, agent_id, since, bucket);
        }
        let since_str = since.map(fmt_dt);
        let mut stmt = conn.prepare(
            "SELECT started_at, ended_at, input_tokens, output_tokens, cache_read, cache_write,
                    cost_total
             FROM turns WHERE agent_id = ?1 AND (?2 IS NULL OR started_at >= ?2)
             ORDER BY started_at",
        )?;
        let rows = stmt.query_map(params![agent_id, since_str], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                [2, 3, 4, 5].map(|i| row.get::<_, i64>(i).unwrap_or(0) as u64),
                row.get::<_, f64>(6)?,
            ))
        })?;
        let mut points: BTreeMap<DateTime<Utc>, UsageSeriesPoint> = BTreeMap::new();
        for row in rows {
            let (started, ended, t, cost) = row?;
            let started = parse_dt(&started)?;
            let secs = bucket_secs(bucket);
            let start = DateTime::from_timestamp(started.timestamp().div_euclid(secs) * secs, 0)
                .unwrap_or(started);
            let p = points.entry(start).or_insert_with(|| UsageSeriesPoint {
                start,
                turns: Some(0),
                tokens: TokenTotals::default(),
                cost_usd: 0.0,
                busy_seconds: Some(0),
            });
            *p.turns.get_or_insert(0) += 1;
            p.tokens.input += t[0];
            p.tokens.output += t[1];
            p.tokens.cache_read += t[2];
            p.tokens.cache_write += t[3];
            p.cost_usd += cost;
            if let Some(e) = parse_dt_opt(ended)? {
                *p.busy_seconds.get_or_insert(0) += (e - started).num_seconds().max(0) as u64;
            }
        }
        Ok(UsageSeries {
            agent_id: agent_id.to_string(),
            bucket,
            since,
            points: points.into_values().collect(),
        })
    }

    /// An external identity's series: the per-report deltas `record_interactive_usage` stored,
    /// bucketed by report time, so the points add up to its group in the `by=agent` breakdown.
    /// Turns and busy time aren't reported, so they stay `None`; a row with no delta (no
    /// session id) has nothing to bucket.
    fn external_series(
        conn: &Connection,
        identity: &str,
        since: Option<DateTime<Utc>>,
        bucket: UsageBucket,
    ) -> Result<UsageSeries, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT observed_at, cost_delta, input_delta, output_delta
             FROM interactive_usage
             WHERE COALESCE(identity, ?3) = ?1 AND (?2 IS NULL OR observed_at >= ?2)
               AND (cost_delta IS NOT NULL OR input_delta IS NOT NULL OR output_delta IS NOT NULL)
             ORDER BY observed_at",
        )?;
        let rows = stmt.query_map(params![identity, since.map(fmt_dt), UNBRIDLED], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<f64>>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, Option<i64>>(3)?,
            ))
        })?;
        let secs = bucket_secs(bucket);
        let mut points: BTreeMap<DateTime<Utc>, UsageSeriesPoint> = BTreeMap::new();
        for row in rows {
            let (at, cost, input, output) = row?;
            let at = parse_dt(&at)?;
            let start =
                DateTime::from_timestamp(at.timestamp().div_euclid(secs) * secs, 0).unwrap_or(at);
            let p = points.entry(start).or_insert_with(|| UsageSeriesPoint {
                start,
                turns: None,
                tokens: TokenTotals::default(),
                cost_usd: 0.0,
                busy_seconds: None,
            });
            p.cost_usd += cost.unwrap_or(0.0);
            p.tokens.input += input.unwrap_or(0).max(0) as u64;
            p.tokens.output += output.unwrap_or(0).max(0) as u64;
        }
        Ok(UsageSeries {
            agent_id: identity.to_string(),
            bucket,
            since,
            points: points.into_values().collect(),
        })
    }

    /// External identities with reports in the period, as instances: the identity is the id and
    /// name, the first path part of it the role.
    fn external_instances(
        conn: &Connection,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<UsageInstance>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(identity, ?2), model, cost_delta, observed_at
             FROM interactive_usage
             WHERE (?1 IS NULL OR observed_at >= ?1)
               AND (cost_delta IS NOT NULL OR input_delta IS NOT NULL OR output_delta IS NOT NULL)
             ORDER BY observed_at",
        )?;
        let rows = stmt.query_map(params![since.map(fmt_dt), UNBRIDLED], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<f64>>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        let mut by_id: BTreeMap<String, UsageInstance> = BTreeMap::new();
        for row in rows {
            let (id, model, cost, at) = row?;
            let at = parse_dt(&at)?;
            let inst = by_id.entry(id.clone()).or_insert_with(|| UsageInstance {
                role: id
                    .strip_prefix(EXTERNAL_PREFIX)
                    .unwrap_or(&id)
                    .split('/')
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                agent_id: id.clone(),
                name: id.clone(),
                model: "unknown".to_string(),
                removed: false,
                external: true,
                turns: None,
                cost_usd: 0.0,
                first_turn_at: at,
                last_turn_at: at,
            });
            // Rows come oldest first, so the newest model wins.
            if let Some(m) = model {
                inst.model = m;
            }
            inst.cost_usd += cost.unwrap_or(0.0);
            inst.last_turn_at = at;
        }
        Ok(by_id.into_values().collect())
    }

    pub(super) fn usage_sessions(
        conn: &Connection,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Vec<SessionUsage>, StoreError> {
        let (since, until) = (since.map(fmt_dt), until.map(fmt_dt));
        let mut out: Vec<SessionUsage> = Vec::new();
        // Oldest first, so a session's first and last context size are the first and last rows
        // that carry one.
        let mut stmt = conn.prepare(
            "SELECT agent_id, agent_name, role, COALESCE(session_id, ''),
                    input_tokens, output_tokens, cache_read, cache_write, cost_total, context_tokens
             FROM turns
             WHERE (?1 IS NULL OR started_at >= ?1) AND (?2 IS NULL OR started_at < ?2)
             ORDER BY started_at",
        )?;
        let rows = stmt.query_map(params![since, until], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                [4, 5, 6, 7].map(|i| r.get::<_, Option<i64>>(i).ok().flatten().unwrap_or(0) as u64),
                r.get::<_, Option<f64>>(8)?.unwrap_or(0.0),
                r.get::<_, Option<i64>>(9)?,
            ))
        })?;
        for row in rows {
            let (agent_id, name, role, session_id, t, cost, ctx) = row?;
            let at = match out.iter().position(|s| {
                s.agent_id.as_deref() == Some(&agent_id) && s.session_id == session_id
            }) {
                Some(i) => i,
                None => {
                    out.push(SessionUsage {
                        role,
                        name,
                        session_id,
                        agent_id: Some(agent_id),
                        turns: Some(0),
                        tokens: TokenTotals::default(),
                        cost_usd: 0.0,
                        first_context: None,
                        last_context: None,
                    });
                    out.len() - 1
                }
            };
            let s = &mut out[at];
            *s.turns.get_or_insert(0) += 1;
            s.tokens.input += t[0];
            s.tokens.output += t[1];
            s.tokens.cache_read += t[2];
            s.tokens.cache_write += t[3];
            s.cost_usd += cost;
            if let Some(c) = ctx {
                s.first_context.get_or_insert(c);
                s.last_context = Some(c);
            }
        }
        let mut stmt = conn.prepare(
            "SELECT COALESCE(identity, ?3), session_id, cost_delta, input_delta, output_delta,
                    context_used_tokens
             FROM interactive_usage
             WHERE (?1 IS NULL OR observed_at >= ?1) AND (?2 IS NULL OR observed_at < ?2)
               AND (cost_delta IS NOT NULL OR input_delta IS NOT NULL OR output_delta IS NOT NULL)
             ORDER BY observed_at, id",
        )?;
        let rows = stmt.query_map(params![since, until, UNBRIDLED], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<f64>>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<i64>>(5)?,
            ))
        })?;
        let first_ext = out.len();
        for row in rows {
            let (identity, session, cost, input, output, ctx) = row?;
            let session_id = session.unwrap_or_default();
            let at = match out[first_ext..]
                .iter()
                .position(|s| s.name == identity && s.session_id == session_id)
            {
                Some(i) => first_ext + i,
                None => {
                    out.push(SessionUsage {
                        role: identity
                            .strip_prefix(EXTERNAL_PREFIX)
                            .unwrap_or(&identity)
                            .split('/')
                            .next()
                            .unwrap_or_default()
                            .to_string(),
                        name: identity,
                        session_id,
                        agent_id: None,
                        turns: None,
                        tokens: TokenTotals::default(),
                        cost_usd: 0.0,
                        first_context: None,
                        last_context: None,
                    });
                    out.len() - 1
                }
            };
            let s = &mut out[at];
            s.cost_usd += cost.unwrap_or(0.0);
            s.tokens.input += input.unwrap_or(0).max(0) as u64;
            s.tokens.output += output.unwrap_or(0).max(0) as u64;
            if let Some(c) = ctx {
                s.first_context.get_or_insert(c);
                s.last_context = Some(c);
            }
        }
        Ok(out)
    }

    pub(super) fn usage_instances(
        conn: &Connection,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<UsageInstance>, StoreError> {
        let since_str = since.map(fmt_dt);
        // The newest turn's name/role/model win: an agent keeps its identity, but a role or
        // model change mid-life shows the current one.
        let mut stmt = conn.prepare(
            "SELECT t.agent_id, t.agent_name, t.role, t.model,
                    EXISTS (SELECT 1 FROM agents a WHERE a.id = t.agent_id),
                    COUNT(*), SUM(t.cost_total), MIN(t.started_at), MAX(t.started_at)
             FROM turns t WHERE (?1 IS NULL OR t.started_at >= ?1)
             GROUP BY t.agent_id
             ORDER BY MAX(t.started_at) DESC",
        )?;
        let rows = stmt.query_map(params![since_str], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, bool>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, f64>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (agent_id, name, role, model, live, turns, cost_usd, first, last) = row?;
            out.push(UsageInstance {
                agent_id,
                name,
                role,
                model,
                removed: !live,
                external: false,
                turns: Some(turns as u32),
                cost_usd,
                first_turn_at: parse_dt(&first)?,
                last_turn_at: parse_dt(&last)?,
            });
        }
        out.extend(external_instances(conn, since)?);
        out.sort_by_key(|a| std::cmp::Reverse(a.last_turn_at));
        Ok(out)
    }

    /// The delta of a running total against the session's previous report: a total that went
    /// down means the session restarted its counter, so the new total is all new. The first
    /// report of a session is its baseline (its earlier spend can't be dated), so 0, unless
    /// `fresh`: the total is small enough that the session is new and the report is all new
    /// spend. A figure the report lacks stays `None`: not measured.
    fn running_delta(prev: Option<f64>, cur: Option<f64>, fresh: bool) -> Option<f64> {
        let cur = cur?;
        Some(match prev {
            Some(p) if cur >= p => cur - p,
            Some(_) => cur,
            None if fresh => cur,
            None => 0.0,
        })
    }

    /// A never-seen session whose first running totals are all below these is new, not resumed:
    /// its first report counts in full (startup cost is real spend). A resumed session's first
    /// report carries its earlier spend and stays a baseline.
    const FRESH_SESSION_MAX_COST_USD: f64 = 1.0;
    const FRESH_SESSION_MAX_TOKENS: i64 = 100_000;

    pub(super) fn record_interactive_usage(
        conn: &Connection,
        row: &InteractiveUsageRow,
    ) -> Result<(), StoreError> {
        let prev: Option<(Option<f64>, Option<i64>, Option<i64>)> = match &row.session_id {
            Some(sid) => conn
                .query_row(
                    "SELECT cost_usd, total_input_tokens, total_output_tokens
                     FROM interactive_usage WHERE session_id = ?1 ORDER BY id DESC LIMIT 1",
                    params![sid],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()?,
            None => None,
        };
        let cur = (
            row.cost_usd,
            row.total_input_tokens.map(|n| n as i64),
            row.total_output_tokens.map(|n| n as i64),
        );
        // The statusline fires on every update; only a change in the totals is a new row.
        if prev == Some(cur) {
            return Ok(());
        }
        let fresh = prev.is_none()
            && row.cost_usd.is_none_or(|c| c < FRESH_SESSION_MAX_COST_USD)
            && cur.1.is_none_or(|n| n < FRESH_SESSION_MAX_TOKENS)
            && cur.2.is_none_or(|n| n < FRESH_SESSION_MAX_TOKENS);
        let (pc, pi, po) = prev.unwrap_or((None, None, None));
        // A session with no id can't be told apart from another: nothing to subtract from.
        let measurable = row.session_id.is_some();
        let cost_delta = measurable
            .then(|| running_delta(pc, row.cost_usd, fresh))
            .flatten();
        let input_delta = measurable
            .then(|| running_delta(pi.map(|n| n as f64), cur.1.map(|n| n as f64), fresh))
            .flatten();
        let output_delta = measurable
            .then(|| running_delta(po.map(|n| n as f64), cur.2.map(|n| n as f64), fresh))
            .flatten();
        conn.execute(
            "INSERT INTO interactive_usage
                (observed_at, session_id, model, cost_usd, context_used_percentage,
                 context_used_tokens, context_max_tokens, identity,
                 total_input_tokens, total_output_tokens,
                 cost_delta, input_delta, output_delta)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                fmt_dt(row.observed_at),
                row.session_id,
                row.model,
                row.cost_usd,
                row.context_used_percentage,
                row.context_used_tokens.map(|n| n as i64),
                row.context_max_tokens.map(|n| n as i64),
                row.identity,
                cur.1,
                cur.2,
                cost_delta,
                input_delta.map(|d| d as i64),
                output_delta.map(|d| d as i64),
            ],
        )?;
        Ok(())
    }

    /// External sessions' deltas since `since`, grouped like the hosted breakdown: `agent` is
    /// the identity, `role` its first path part, `model` the model. A group's figure is
    /// `None` when none of its rows had one (not measured).
    fn external_usage(
        conn: &Connection,
        since: Option<DateTime<Utc>>,
        by: UsageGroupBy,
    ) -> Result<Vec<ExternalUsage>, StoreError> {
        let mut stmt = conn.prepare(
            "SELECT identity, model, session_id, cost_delta, input_delta, output_delta
             FROM interactive_usage WHERE (?1 IS NULL OR observed_at >= ?1)",
        )?;
        let rows = stmt.query_map(params![since.map(fmt_dt)], |r| {
            Ok((
                r.get::<_, Option<String>>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<f64>>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<i64>>(5)?,
            ))
        })?;
        let mut groups: BTreeMap<String, (BTreeSet<Option<String>>, ExternalUsage)> =
            BTreeMap::new();
        for row in rows {
            let (identity, model, session, cost, input, output) = row?;
            let identity = identity.unwrap_or_else(|| "external:unbridled".to_string());
            let key = match by {
                UsageGroupBy::Agent => identity,
                UsageGroupBy::Role => identity
                    .strip_prefix("external:")
                    .unwrap_or(&identity)
                    .split('/')
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                UsageGroupBy::Model => model.unwrap_or_else(|| "unknown".to_string()),
            };
            let (sessions, g) = groups.entry(key.clone()).or_insert_with(|| {
                (
                    BTreeSet::new(),
                    ExternalUsage {
                        key,
                        sessions: 0,
                        cost_usd: None,
                        input_tokens: None,
                        output_tokens: None,
                    },
                )
            });
            sessions.insert(session);
            g.cost_usd = add_opt(g.cost_usd, cost);
            g.input_tokens = add_opt(g.input_tokens.map(|n| n as f64), input.map(|n| n as f64))
                .map(|n| n as u64);
            g.output_tokens = add_opt(g.output_tokens.map(|n| n as f64), output.map(|n| n as f64))
                .map(|n| n as u64);
        }
        Ok(groups
            .into_values()
            .map(|(sessions, mut g)| {
                g.sessions = sessions.len() as u32;
                g
            })
            .collect())
    }

    fn add_opt(a: Option<f64>, b: Option<f64>) -> Option<f64> {
        match (a, b) {
            (None, None) => None,
            (a, b) => Some(a.unwrap_or(0.0) + b.unwrap_or(0.0)),
        }
    }

    /// Since UTC midnight, newest first: `bridle usage` shows it alongside
    /// hosted-agent usage as "today's" non-hosted usage.
    fn interactive_usage_today(conn: &Connection) -> Result<Vec<InteractiveUsageRow>, StoreError> {
        let today_start = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .expect("midnight is a valid time")
            .and_utc();
        let mut stmt = conn.prepare(
            "SELECT observed_at, session_id, model, cost_usd, context_used_percentage,
                    context_used_tokens, context_max_tokens
             FROM interactive_usage WHERE observed_at >= ?1 ORDER BY observed_at DESC",
        )?;
        let rows = stmt.query_map(params![fmt_dt(today_start)], |row| {
            Ok(InteractiveUsageRow {
                observed_at: parse_dt(&row.get::<_, String>(0)?)?,
                session_id: row.get(1)?,
                model: row.get(2)?,
                cost_usd: row.get(3)?,
                context_used_percentage: row.get(4)?,
                context_used_tokens: row.get::<_, Option<i64>>(5)?.map(|n| n as u64),
                context_max_tokens: row.get::<_, Option<i64>>(6)?.map(|n| n as u64),
                identity: None,
                total_input_tokens: None,
                total_output_tokens: None,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub(super) fn agents_by_state(conn: &Connection) -> Result<BTreeMap<String, u32>, StoreError> {
        let mut stmt = conn.prepare("SELECT state, COUNT(*) FROM agents GROUP BY state")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
        })?;
        let mut out = BTreeMap::new();
        for row in rows {
            let (state, count) = row?;
            out.insert(state, count);
        }
        Ok(out)
    }

    pub(super) fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>, StoreError> {
        conn.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| {
            r.get(0)
        })
        .optional()
        .map_err(Into::into)
    }

    pub(super) fn swap_meta(
        conn: &Connection,
        key: &str,
        value: &str,
    ) -> Result<Option<String>, StoreError> {
        let previous = get_meta(conn, key)?;
        conn.execute(
            "INSERT INTO meta(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(previous)
    }

    pub(super) fn move_marked_unread_messages(
        conn: &Connection,
        from: &str,
        to: &str,
        mark: &str,
    ) -> Result<usize, StoreError> {
        Ok(conn.execute(
            "UPDATE messages SET to_id = ?2, body = substr(body, length(?3) + 1)
             WHERE to_id = ?1 AND state NOT IN ('read', 'dropped')
               AND substr(body, 1, length(?3)) = ?3",
            params![from, to, mark],
        )?)
    }

    pub(super) fn close_handover_requests(
        conn: &Connection,
        to: &str,
    ) -> Result<usize, StoreError> {
        Ok(conn.execute(
            "UPDATE messages SET state = 'read', read_at = ?2
             WHERE to_kind = 'external' AND to_id = ?1 AND state NOT IN ('read', 'dropped')
               AND (instr(body, 'Say nothing more once it is written') > 0
                    OR (from_principal = 'system' AND body LIKE 'Your context is %k tokens%'))",
            params![to, fmt_dt(Utc::now())],
        )?)
    }

    pub(super) fn unread_count(conn: &Connection, to: &str) -> Result<u32, StoreError> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE to_id = ?1 AND state NOT IN ('read', 'dropped')",
            params![to],
            |row| row.get(0),
        )?;
        Ok(n as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_database_newer_than_this_binary_is_refused() {
        let conn = Connection::open_in_memory().expect("open");
        let newer = (sync::MIGRATIONS.len() + 1) as i64;
        conn.pragma_update(None, "user_version", newer)
            .expect("set version");
        let err = sync::migrate(&conn).unwrap_err();
        assert!(matches!(err, StoreError::SchemaTooNew { .. }), "{err}");
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .expect("version");
        assert_eq!(version, newer, "a refused database is left untouched");
    }

    #[test]
    fn left_behind_counts_what_is_queued_and_active() {
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("bridle.db");
        let conn = Connection::open(&path).expect("open");
        sync::migrate(&conn).expect("migrate");
        conn.execute(
            "INSERT INTO outbox (project, from_principal, to_principal, kind, body, when_mode, \
             state, created_at) VALUES ('p', 'a', 'b', 'note', 'x', 'now', 'queued', 't'), \
             ('p', 'a', 'b', 'note', 'x', 'now', 'delivered', 't')",
            [],
        )
        .expect("outbox");
        drop(conn);
        let left = left_behind(&path).expect("counts");
        assert_eq!(left.undelivered_mail, 1);
        assert_eq!(left.active_schedules, 0);
        assert!(left_behind(&dir.path().join("none.db")).is_none());
    }

    #[test]
    fn v2_migration_keeps_turns_and_gives_them_the_agents_identity() {
        let conn = Connection::open_in_memory().expect("open");
        conn.execute_batch(sync::SCHEMA_V1).expect("v1 schema");
        conn.pragma_update(None, "user_version", 1)
            .expect("set version");
        conn.execute_batch(
            "INSERT INTO agents(id, name, role, state, model, session_id, workdir_kind, cwd,
                                created_at, updated_at, created_by)
             VALUES ('a-1', 'w1', 'worker', 'idle', 'sonnet', 's', 'repo', '/r', 't', 't', 'human');
             INSERT INTO turns(agent_id, n, started_at, input_tokens) VALUES ('a-1', 1, 't', 42);",
        )
        .expect("v1 rows");

        sync::migrate(&conn).expect("migrate");

        let (name, role, model, input): (String, String, String, i64) = conn
            .query_row(
                "SELECT agent_name, role, model, input_tokens FROM turns WHERE agent_id = 'a-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .expect("migrated turn");
        assert_eq!(
            (name.as_str(), role.as_str(), model.as_str(), input),
            ("w1", "worker", "sonnet", 42)
        );
        // No cascade any more: the turn outlives its agent.
        conn.execute("DELETE FROM agents WHERE id = 'a-1'", [])
            .expect("delete agent");
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM turns", [], |r| r.get(0))
            .expect("count");
        assert_eq!(n, 1);
    }

    /// br-9y7v: negative turn costs are zeroed and the agent's total re-summed; an agent
    /// without one keeps its total; a second pass changes nothing.
    #[test]
    fn v27_migration_repairs_negative_turn_costs() {
        let conn = Connection::open_in_memory().expect("open");
        for sql in &sync::MIGRATIONS[..26] {
            conn.execute_batch(sql).expect("earlier schema");
        }
        conn.pragma_update(None, "user_version", 26)
            .expect("set version");
        conn.execute_batch(
            "INSERT INTO agents(id, name, role, state, model, session_id, workdir_kind, cwd,
                                created_at, updated_at, created_by, cost_usd_total)
             VALUES ('a-1', 'w1', 'worker', 'idle', 'sonnet', 's', 'repo', '/r', 't', 't', 'human', -1.0),
                    ('a-2', 'w2', 'worker', 'idle', 'sonnet', 's', 'repo', '/r', 't', 't', 'human', 7.0);
             INSERT INTO turns(agent_id, agent_name, role, model, n, started_at, cost_total) VALUES
                ('a-1', 'w1', 'worker', 'sonnet', 1, 't', 1.5),
                ('a-1', 'w1', 'worker', 'sonnet', 2, 't', -3.0),
                ('a-1', 'w1', 'worker', 'sonnet', 3, 't', 0.5),
                ('a-2', 'w2', 'worker', 'sonnet', 1, 't', 7.0);",
        )
        .expect("rows");

        sync::migrate(&conn).expect("migrate");

        let total = |id: &str| -> f64 {
            conn.query_row(
                "SELECT cost_usd_total FROM agents WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .expect("total")
        };
        assert_eq!(total("a-1"), 2.0);
        assert_eq!(total("a-2"), 7.0);
        let negative: i64 = conn
            .query_row("SELECT COUNT(*) FROM turns WHERE cost_total < 0", [], |r| {
                r.get(0)
            })
            .expect("count");
        assert_eq!(negative, 0);
    }

    #[test]
    fn v4_migration_deletes_windowless_rate_limit_rows() {
        let conn = Connection::open_in_memory().expect("open");
        for sql in [sync::SCHEMA_V1, sync::SCHEMA_V2, sync::SCHEMA_V3] {
            conn.execute_batch(sql).expect("earlier schema");
        }
        conn.pragma_update(None, "user_version", 3)
            .expect("set version");
        conn.execute_batch(
            "INSERT INTO rate_limits(window, utilization, resets_at, observed_at)
                VALUES ('five_hour', 0.42, '2026-01-01T00:00:00Z', 't');
             INSERT INTO rate_limits(window, utilization, resets_at, observed_at)
                VALUES ('project_zephyr_beta', NULL, NULL, 't');",
        )
        .expect("v3 rows");

        sync::migrate(&conn).expect("migrate");

        let windows: Vec<String> = conn
            .prepare("SELECT window FROM rate_limits")
            .expect("prepare")
            .query_map([], |r| r.get(0))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows");
        assert_eq!(windows, vec!["five_hour".to_string()]);
    }

    #[tokio::test]
    async fn new_task_records_its_creator() {
        let (store, _tmp) = store().await;
        let row = store
            .insert_task("tw", "A", TicketKind::Chore, "external:advisor")
            .await
            .expect("insert");
        assert_eq!(row.created_by.as_deref(), Some("external:advisor"));
        let got = store.get_task(&row.id).await.expect("get").expect("row");
        assert_eq!(got.created_by.as_deref(), Some("external:advisor"));
    }

    /// An un-migrated (v18) database with old tasks opens fine; the backfill fills the task
    /// that has a `task.created` event (the earliest one) and leaves the other unknown.
    #[test]
    fn opening_an_unmigrated_db_backfills_creators_and_leaves_the_rest_unknown() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("bridle.db");
        {
            let conn = Connection::open(&path).expect("open");
            for sql in &sync::MIGRATIONS[..18] {
                conn.execute_batch(sql).expect("schema");
            }
            conn.pragma_update(None, "user_version", 18)
                .expect("set version");
            conn.execute_batch(
                "INSERT INTO tasks(id, title, kind, state, created_at, updated_at) VALUES
                    ('t-a', 'A', 'chore', 'open', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z'),
                    ('t-b', 'B', 'chore', 'open', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z');
                 INSERT INTO events(ts, kind, actor, agent_id, data) VALUES
                    ('t', 'task.created', 'agent-1', NULL, '{\"task\": \"t-a\"}'),
                    ('t', 'task.created', 'agent-2', NULL, '{\"task\": \"t-a\"}'),
                    ('t', 'task.created', 'agent-3', NULL, 'not json');",
            )
            .expect("old rows");
        }
        let conn = sync::open(&path).expect("start-up succeeds");
        let by = |id: &str| {
            sync::get_task(&conn, id)
                .expect("get")
                .expect("row")
                .created_by
        };
        assert_eq!(by("t-a").as_deref(), Some("agent-1"));
        assert_eq!(by("t-b"), None);
        // Idempotent: a second pass changes nothing.
        sync::backfill_task_creators(&conn).expect("again");
        assert_eq!(by("t-a").as_deref(), Some("agent-1"));
    }

    /// The `tasks` table becomes `tickets` (br-95t9) with every row intact.
    #[test]
    fn the_tasks_table_is_renamed_with_its_rows() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("bridle.db");
        {
            let conn = Connection::open(&path).expect("open");
            for sql in &sync::MIGRATIONS[..25] {
                conn.execute_batch(sql).expect("schema");
            }
            conn.pragma_update(None, "user_version", 25)
                .expect("set version");
            conn.execute_batch(
                "INSERT INTO tasks(id, title, kind, state, created_at, updated_at) VALUES
                    ('t-a', 'A', 'chore', 'open', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z'),
                    ('t-b', 'B', 'chore', 'open', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z');",
            )
            .expect("old rows");
        }
        let conn = sync::open(&path).expect("migrates");
        let n: i64 = conn
            .query_row("SELECT count(*) FROM tickets", [], |r| r.get(0))
            .expect("count");
        assert_eq!(n, 2);
        assert!(conn.prepare("SELECT 1 FROM tasks").is_err());
        assert_eq!(sync::list_tasks(&conn).expect("list").len(), 2);
    }

    /// Reproduces the real `JoinError::Cancelled` that `Store::open` and
    /// `with_conn` await: caps the blocking pool at one thread, occupies
    /// it, queues a second blocking task behind it, then cancels that
    /// second task before it can start (exactly what the runtime does to
    /// queued-but-not-yet-started blocking work during shutdown, per the
    /// `JoinHandle::abort` docs). Before the fix, `join_error` didn't
    /// exist and the call sites did
    /// `spawn_blocking(...).await.expect("... task panicked")`, which
    /// turned this ordinary cancellation into a panic.
    ///
    /// The runtime is built by hand: `#[tokio::test(worker_threads = 1)]` leaves
    /// `max_blocking_threads` at 512, so the second task got its own thread and
    /// could finish before the abort (br-vabu).
    #[test]
    fn cancelled_blocking_task_returns_shutting_down_not_a_panic() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .max_blocking_threads(1)
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let occupy = tokio::task::spawn_blocking(|| {
                std::thread::sleep(std::time::Duration::from_millis(200))
            });
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;

            // The only blocking thread is busy, so this is queued, not started.
            let handle = tokio::task::spawn_blocking(|| 1);
            handle.abort();
            let err = handle.await.expect_err("aborted task should error");
            assert!(err.is_cancelled());

            match join_error(err) {
                StoreError::ShuttingDown => {}
                other => panic!("expected StoreError::ShuttingDown, got {other:?}"),
            }

            occupy.await.expect("occupying task should finish normally");
        });
    }

    /// A genuine panic inside the blocking closure must still surface as a
    /// panic, not be swallowed as `ShuttingDown`.
    #[tokio::test]
    async fn real_panic_in_blocking_task_still_panics() {
        let handle = tokio::task::spawn_blocking(|| panic!("boom"));
        let err = handle.await.expect_err("panicking task should error");
        assert!(err.is_panic());

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| join_error(err)));
        assert!(result.is_err(), "join_error should propagate the panic");
    }

    async fn store() -> (Store, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        (store, tmp)
    }

    #[tokio::test]
    async fn handovers_latest_first_and_prune_keeps_the_newest() {
        let (store, _tmp) = store().await;
        let a = store
            .insert_handover("orchestrator", "bridle", "one", "human")
            .await
            .expect("a");
        let b = store
            .insert_handover("orchestrator", "bridle", "two", "human")
            .await
            .expect("b");
        assert_eq!((a.id.as_str(), b.id.as_str()), ("h-0001", "h-0002"));
        let list = store.list_handovers().await.expect("list");
        assert_eq!(list[0].id, "h-0002");
        assert_eq!(list.len(), 2);

        // Everything is "old" against a future cutoff; the newest still stays.
        let cutoff = Utc::now() + chrono::Duration::days(1);
        assert_eq!(store.prune_handovers(cutoff).await.expect("prune"), 1);
        let list = store.list_handovers().await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "h-0002");
        assert_eq!(store.prune_handovers(cutoff).await.expect("prune"), 0);
        assert!(store.get_handover("h-0001").await.expect("get").is_none());
    }

    #[tokio::test]
    async fn list_tasks_breaks_created_at_ties_by_insertion_order() {
        let (store, _tmp) = store().await;
        let at = Utc::now();
        // Ids descend, so neither an id sort nor an unspecified tie order passes.
        for id in ["t-c", "t-b", "t-a", "t-d"] {
            store
                .insert_task_row(&TicketRow {
                    id: id.to_string(),
                    title: id.to_string(),
                    kind: TicketKind::Feature,
                    state: TicketState::Open,
                    created_at: at,
                    updated_at: at,
                    created_by: None,
                })
                .await
                .expect("insert task");
        }
        let ids: Vec<String> = store
            .list_tasks()
            .await
            .expect("list")
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, ["t-c", "t-b", "t-a", "t-d"]);
    }

    fn new_agent(name: &str) -> NewAgent {
        NewAgent {
            name: name.to_string(),
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
        }
    }

    #[tokio::test]
    async fn stored_product_manager_agent_reads_as_project_manager() {
        let (store, _tmp) = store().await;
        let mut new = new_agent("pm-1");
        new.role = "product-manager".to_string();
        store.insert_agent(new).await.expect("insert");
        // The queue gate, stop_check and the config role lookup all read this.
        let got = store.get_agent("pm-1").await.expect("get").expect("some");
        assert_eq!(got.role, "project-manager");
        let all = store.list_agents(false).await.expect("list");
        assert_eq!(all[0].role, "project-manager");
    }

    #[tokio::test]
    async fn agent_full_lifecycle() {
        let (store, _tmp) = store().await;

        let agent = store
            .insert_agent(new_agent("w1"))
            .await
            .expect("insert agent");
        assert_eq!(agent.state, AgentState::Starting);
        assert_eq!(agent.turns, 0);
        assert_eq!(agent.cost_usd_total, 0.0);

        store
            .set_agent_process(
                &agent.id,
                Some(4242),
                Some("Wed Jan 1 00:00:00 2026".to_string()),
            )
            .await
            .expect("set process");
        store
            .set_agent_state(&agent.id, AgentState::Idle)
            .await
            .expect("set state");
        store
            .touch_agent(&agent.id, Utc::now())
            .await
            .expect("touch");

        let started = Utc::now();
        store
            .add_turn_start(&agent.id, 1, started)
            .await
            .expect("turn start");
        let mid = store
            .get_agent(&agent.id)
            .await
            .expect("get")
            .expect("found");
        assert_eq!(mid.state, AgentState::Idle); // turn bookkeeping doesn't itself change state
        assert!(mid.turn_started_at.is_some());
        assert_eq!(mid.context_tokens, None); // no turn has ended yet

        store
            .end_turn(
                &agent.id,
                1,
                TurnEnd {
                    subtype: "success".to_string(),
                    is_error: false,
                    terminal_reason: None,
                    input_tokens: 100,
                    output_tokens: 50,
                    cache_read: 10,
                    cache_write: 5,
                    cost_total: 0.05,
                    context_tokens: 115,
                },
            )
            .await
            .expect("end turn");

        let after = store
            .get_agent(&agent.id)
            .await
            .expect("get")
            .expect("found");
        assert_eq!(after.turns, 1);
        assert!((after.cost_usd_total - 0.05).abs() < 1e-9);
        assert!(after.turn_started_at.is_none());
        assert_eq!(after.pid, Some(4242));
        assert_eq!(after.session_id, "sess-1");
        assert_eq!(after.context_tokens, Some(115));

        // Lookup by name works too.
        let by_name = store
            .get_agent("w1")
            .await
            .expect("get by name")
            .expect("found");
        assert_eq!(by_name.id, agent.id);

        store
            .set_agent_exit(
                &agent.id,
                ExitInfo {
                    code: Some(0),
                    signal: None,
                    reason: "stdin_closed".to_string(),
                },
            )
            .await
            .expect("set exit");
        store
            .set_agent_state(&agent.id, AgentState::Exited)
            .await
            .expect("set exited");

        let exited = store
            .get_agent(&agent.id)
            .await
            .expect("get")
            .expect("found");
        assert_eq!(exited.state, AgentState::Exited);
        assert_eq!(exited.exit.expect("exit info").reason, "stdin_closed");

        // Not included when excluding terminal agents.
        let running = store.list_agents(false).await.expect("list running");
        assert!(running.is_empty());
        let all = store.list_agents(true).await.expect("list all");
        assert_eq!(all.len(), 1);

        store.delete_agent(&agent.id).await.expect("delete agent");
        assert!(
            store
                .get_agent(&agent.id)
                .await
                .expect("get after delete")
                .is_none()
        );
    }

    #[tokio::test]
    async fn agent_name_conflict() {
        let (store, _tmp) = store().await;
        store
            .insert_agent(new_agent("dup"))
            .await
            .expect("first insert");
        let err = store
            .insert_agent(new_agent("dup"))
            .await
            .expect_err("second insert should conflict");
        assert!(
            matches!(err, StoreError::Conflict(_)),
            "expected Conflict, got {err:?}"
        );
    }

    #[tokio::test]
    async fn running_agents_reconciliation() {
        let (store, _tmp) = store().await;
        let a = store.insert_agent(new_agent("w1")).await.expect("insert");
        store
            .set_agent_process(&a.id, Some(111), Some("start-a".to_string()))
            .await
            .expect("set process");
        store
            .set_agent_state(&a.id, AgentState::Working)
            .await
            .expect("set working");

        let b = store.insert_agent(new_agent("w2")).await.expect("insert");
        store
            .set_agent_state(&b.id, AgentState::Exited)
            .await
            .expect("set exited");

        let running = store.running_agents().await.expect("running agents");
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].id, a.id);
        assert_eq!(running[0].pid, Some(111));
        assert_eq!(running[0].pid_start.as_deref(), Some("start-a"));
    }

    #[tokio::test]
    async fn duplicate_peer_token_error_names_the_sending_machine() {
        let (store, _tmp) = store().await;
        store
            .create_peer_token("dalek", false)
            .await
            .expect("first");
        let err = store
            .create_peer_token("dalek", false)
            .await
            .expect_err("duplicate");
        let msg = err.to_string();
        assert!(msg.contains("peer:dalek already exists"), "{msg}");
        assert!(msg.contains("SENDS"), "{msg}");
        assert!(msg.contains("bridle token revoke peer:dalek"), "{msg}");
    }

    #[tokio::test]
    async fn replacing_a_peer_token_swaps_it_in_one_step() {
        let (store, _tmp) = store().await;
        let old = store
            .create_peer_token("dalek", false)
            .await
            .expect("first");
        let new = store
            .create_peer_token("dalek", true)
            .await
            .expect("replace");
        assert_ne!(old.token, new.token);
        assert!(
            store
                .authenticate(&old.token)
                .await
                .expect("auth")
                .is_none()
        );
        assert!(
            store
                .authenticate(&new.token)
                .await
                .expect("auth")
                .is_some()
        );
    }

    #[tokio::test]
    async fn token_auth_round_trip() {
        let (store, _tmp) = store().await;

        let human_token = store
            .ensure_human_token()
            .await
            .expect("ensure human token")
            .expect("first call creates it");
        assert_eq!(human_token.len(), 64);
        assert!(human_token.chars().all(|c| c.is_ascii_hexdigit()));

        // Second call is a no-op: the human principal already exists.
        assert!(
            store
                .ensure_human_token()
                .await
                .expect("ensure again")
                .is_none()
        );

        let principal = store
            .authenticate(&human_token)
            .await
            .expect("authenticate")
            .expect("valid token");
        assert_eq!(principal.id, "human");
        assert_eq!(principal.kind, PrincipalKind::Human);

        let external = store
            .create_external_token("orchestrator", None)
            .await
            .expect("create external token");
        assert_eq!(external.principal, "external:orchestrator");
        let ext_principal = store
            .authenticate(&external.token)
            .await
            .expect("auth")
            .expect("valid");
        assert_eq!(ext_principal.kind, PrincipalKind::External);

        // A second, still-active external token of the same name conflicts.
        let conflict = store
            .create_external_token("orchestrator", None)
            .await
            .expect_err("should conflict");
        assert!(matches!(conflict, StoreError::Conflict(_)));

        let agent_token = store
            .create_agent_token("a-abcde", "w1")
            .await
            .expect("create agent token");
        let agent_principal = store
            .authenticate(&agent_token)
            .await
            .expect("auth")
            .expect("valid");
        assert_eq!(agent_principal.id, "agent:w1");
        assert_eq!(agent_principal.kind, PrincipalKind::Agent);

        // Revoking invalidates the token.
        store
            .revoke_principal(&ext_principal.id)
            .await
            .expect("revoke");
        assert!(
            store
                .authenticate(&external.token)
                .await
                .expect("auth after revoke")
                .is_none()
        );

        // An unknown token authenticates to nothing.
        assert!(
            store
                .authenticate("not-a-real-token")
                .await
                .expect("auth bogus")
                .is_none()
        );
    }

    #[tokio::test]
    async fn list_and_revoke_external_tokens() {
        let (store, _tmp) = store().await;
        store
            .create_external_token("orchestrator", None)
            .await
            .expect("create external token");
        store
            .create_external_token("tui", None)
            .await
            .expect("create external token");

        let tokens = store.list_external_tokens().await.expect("list tokens");
        assert_eq!(tokens.len(), 2);
        // Never carries the secret itself.
        let json = serde_json::to_string(&tokens).expect("serialize");
        assert!(!json.contains("token"));
        let orchestrator = tokens
            .iter()
            .find(|t| t.name == "orchestrator")
            .expect("orchestrator listed");
        assert_eq!(orchestrator.principal, "external:orchestrator");
        assert!(!orchestrator.revoked);

        store
            .revoke_external_token("orchestrator")
            .await
            .expect("revoke");
        let tokens = store.list_external_tokens().await.expect("list again");
        let orchestrator = tokens
            .iter()
            .find(|t| t.name == "orchestrator")
            .expect("still listed after revoke");
        assert!(orchestrator.revoked);
        let tui = tokens.iter().find(|t| t.name == "tui").expect("tui listed");
        assert!(!tui.revoked);

        let err = store
            .revoke_external_token("no-such-name")
            .await
            .expect_err("revoking an unknown name fails");
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    #[tokio::test]
    async fn message_state_transitions() {
        let (store, _tmp) = store().await;
        let agent = store
            .insert_agent(new_agent("w1"))
            .await
            .expect("insert agent");

        let msg = store
            .insert_message(NewMessage {
                from: "human".to_string(),
                to: agent.id.clone(),
                to_kind: RecipientKind::Agent,
                kind: MessageKind::Question,
                body: "are you there?".to_string(),
                reply_to: None,
                when: When::Now,
                state: MessageState::Pending,
            })
            .await
            .expect("insert message");
        assert!(msg.id.starts_with("m-"));
        assert_eq!(msg.state, MessageState::Pending);
        assert!(msg.written_at.is_none());

        let t1 = Utc::now();
        store
            .set_message_state(&msg.id, MessageState::Written, t1)
            .await
            .expect("set written");
        let after_written = store
            .get_message(&msg.id)
            .await
            .expect("get")
            .expect("found");
        assert_eq!(after_written.state, MessageState::Written);
        assert!(after_written.written_at.is_some());
        assert!(after_written.delivered_at.is_none());

        let t2 = Utc::now();
        store
            .set_message_state(&msg.id, MessageState::Delivered, t2)
            .await
            .expect("set delivered");
        let after_delivered = store
            .get_message(&msg.id)
            .await
            .expect("get")
            .expect("found");
        assert_eq!(after_delivered.state, MessageState::Delivered);
        assert!(after_delivered.delivered_at.is_some());
        // written_at is untouched by the later transition.
        assert!(after_delivered.written_at.is_some());

        let t3 = Utc::now();
        store
            .set_message_state(&msg.id, MessageState::Read, t3)
            .await
            .expect("set read");
        let after_read = store
            .get_message(&msg.id)
            .await
            .expect("get")
            .expect("found");
        assert_eq!(after_read.state, MessageState::Read);
        assert!(after_read.read_at.is_some());

        // held/unacked counters on the agent reflect message state.
        let msg2 = store
            .insert_message(NewMessage {
                from: "human".to_string(),
                to: agent.id.clone(),
                to_kind: RecipientKind::Agent,
                kind: MessageKind::Note,
                body: "fyi".to_string(),
                reply_to: None,
                when: When::Idle,
                state: MessageState::Held,
            })
            .await
            .expect("insert held message");
        let with_held = store
            .get_agent(&agent.id)
            .await
            .expect("get")
            .expect("found");
        assert_eq!(with_held.held_messages, 1);
        assert_eq!(with_held.unacked_messages, 0);

        let for_agent = store
            .messages_for_agent(&agent.id, &[MessageState::Held])
            .await
            .expect("messages for agent");
        assert_eq!(for_agent.len(), 1);
        assert_eq!(for_agent[0].id, msg2.id);

        let unread = store
            .list_messages(ListMessages {
                to: Some(agent.id.clone()),
                unread: true,
                ..Default::default()
            })
            .await
            .expect("list unread");
        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].id, msg2.id);
    }

    #[tokio::test]
    async fn event_paging() {
        let (store, _tmp) = store().await;
        for i in 0..5 {
            store
                .append_event(
                    "agent.text",
                    "human".to_string(),
                    Some("a-1".to_string()),
                    serde_json::json!({"i": i}),
                )
                .await
                .expect("append event");
        }

        let all = store
            .list_events(EventQuery::default())
            .await
            .expect("list all");
        assert_eq!(all.len(), 5);
        assert_eq!(all[0].seq, 1);
        assert_eq!(all[4].seq, 5);

        let page = store
            .list_events(EventQuery {
                since: Some(2),
                ..Default::default()
            })
            .await
            .expect("list since 2");
        assert_eq!(page.len(), 3);
        assert_eq!(page[0].seq, 3);

        // No cursor: "recent", so a limit of 2 gets the newest two, not the
        // oldest, still returned in ascending order.
        let limited = store
            .list_events(EventQuery {
                limit: Some(2),
                ..Default::default()
            })
            .await
            .expect("list limited");
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].seq, 4);
        assert_eq!(limited[1].seq, 5);

        store
            .append_event(
                "daemon.started",
                "system".to_string(),
                None,
                serde_json::json!({}),
            )
            .await
            .expect("append daemon event");
        let prefix = store
            .list_events(EventQuery {
                kind: Some("agent.".to_string()),
                ..Default::default()
            })
            .await
            .expect("list prefix");
        assert_eq!(prefix.len(), 5);
    }

    #[tokio::test]
    async fn list_events_with_no_cursor_returns_the_recent_end() {
        let (store, _tmp) = store().await;
        for i in 0..10 {
            store
                .append_event(
                    "agent.text",
                    "human".to_string(),
                    Some("a-1".to_string()),
                    serde_json::json!({"i": i}),
                )
                .await
                .expect("append event");
        }

        // since=None: the most recent `limit`, still in ascending order.
        let recent = store
            .list_events(EventQuery {
                since: None,
                limit: Some(3),
                ..Default::default()
            })
            .await
            .expect("list recent");
        assert_eq!(
            recent.iter().map(|e| e.seq).collect::<Vec<_>>(),
            vec![8, 9, 10]
        );

        // since=Some(_): forward pagination from the cursor, unchanged.
        let page = store
            .list_events(EventQuery {
                since: Some(3),
                limit: Some(3),
                ..Default::default()
            })
            .await
            .expect("list page");
        assert_eq!(
            page.iter().map(|e| e.seq).collect::<Vec<_>>(),
            vec![4, 5, 6]
        );
    }

    #[tokio::test]
    async fn usage_aggregate() {
        let (store, _tmp) = store().await;
        let a = store.insert_agent(new_agent("w1")).await.expect("insert a");
        let b = store.insert_agent(new_agent("w2")).await.expect("insert b");

        for (agent, cost, input, cache_read) in
            [(&a, 0.10, 100u64, 20u64), (&b, 0.20, 200u64, 0u64)]
        {
            store
                .add_turn_start(&agent.id, 1, Utc::now())
                .await
                .expect("turn start");
            store
                .end_turn(
                    &agent.id,
                    1,
                    TurnEnd {
                        subtype: "success".to_string(),
                        is_error: false,
                        terminal_reason: None,
                        input_tokens: input,
                        output_tokens: 10,
                        cache_read,
                        cache_write: 0,
                        cost_total: cost,
                        context_tokens: input + cache_read,
                    },
                )
                .await
                .expect("end turn");
        }

        let usage = store.usage().await.expect("usage");
        assert_eq!(usage.agents.len(), 2);
        assert_eq!(usage.total_turns, 2);
        assert!((usage.total_cost_usd - 0.30).abs() < 1e-9);
        assert_eq!(usage.total_tokens.input, 300);
        assert_eq!(usage.total_tokens.cache_read, 20);
        // cache_hit_ratio = cache_read / (input + cache_read + cache_write) = 20 / 320
        assert!((usage.cache_hit_ratio.expect("ratio") - (20.0 / 320.0)).abs() < 1e-9);

        let by_state = store.agents_by_state().await.expect("agents by state");
        assert_eq!(by_state.get("starting").copied(), Some(2));
    }

    #[tokio::test]
    async fn usage_breakdown_groups_by_role_and_model() {
        let (store, _tmp) = store().await;
        let a = store
            .insert_agent(NewAgent {
                role: "worker".to_string(),
                model: "sonnet".to_string(),
                ..new_agent("w1")
            })
            .await
            .expect("insert a");
        let b = store
            .insert_agent(NewAgent {
                role: "worker".to_string(),
                model: "haiku".to_string(),
                ..new_agent("w2")
            })
            .await
            .expect("insert b");
        let c = store
            .insert_agent(NewAgent {
                role: "manager".to_string(),
                model: "sonnet".to_string(),
                ..new_agent("m1")
            })
            .await
            .expect("insert c");

        for (agent, cost, input, cache_read) in [
            (&a, 0.10, 100u64, 20u64),
            (&b, 0.20, 200u64, 0u64),
            (&c, 0.05, 50u64, 10u64),
        ] {
            store
                .add_turn_start(&agent.id, 1, Utc::now())
                .await
                .expect("turn start");
            store
                .end_turn(
                    &agent.id,
                    1,
                    TurnEnd {
                        subtype: "success".to_string(),
                        is_error: false,
                        terminal_reason: None,
                        input_tokens: input,
                        output_tokens: 10,
                        cache_read,
                        cache_write: 0,
                        cost_total: cost,
                        context_tokens: input + cache_read,
                    },
                )
                .await
                .expect("end turn");
        }

        let by_role = store
            .usage_breakdown(None, UsageGroupBy::Role)
            .await
            .expect("by role");
        assert_eq!(by_role.groups.len(), 2);
        let worker = by_role
            .groups
            .iter()
            .find(|g| g.key == "worker")
            .expect("worker group");
        assert_eq!(worker.turns, 2);
        assert_eq!(worker.tokens.input, 300);
        assert!((worker.cost_usd_total - 0.30).abs() < 1e-9);
        assert!((worker.cache_hit_ratio.expect("ratio") - (20.0 / 320.0)).abs() < 1e-9);
        assert_eq!(by_role.total_turns, 3);
        assert!((by_role.total_cost_usd - 0.35).abs() < 1e-9);

        let by_model = store
            .usage_breakdown(None, UsageGroupBy::Model)
            .await
            .expect("by model");
        assert_eq!(by_model.groups.len(), 2);
        let sonnet = by_model
            .groups
            .iter()
            .find(|g| g.key == "sonnet")
            .expect("sonnet group");
        assert_eq!(sonnet.turns, 2);
        assert!((sonnet.cost_usd_total - 0.15).abs() < 1e-9);
    }

    /// A renewed agent's turns carry the session they ran in, and each its context size;
    /// rows from before the migration stay NULL.
    #[tokio::test]
    async fn turns_record_session_id_and_context_tokens() {
        let (store, _tmp) = store().await;
        let a = store.insert_agent(new_agent("w1")).await.expect("insert");
        for (n, session) in [(1, None), (2, Some("s-new"))] {
            if let Some(s) = session {
                store.set_agent_session(&a.id, s).await.expect("renew");
            }
            store
                .add_turn_start(&a.id, n, Utc::now())
                .await
                .expect("start");
            store
                .end_turn(
                    &a.id,
                    n,
                    TurnEnd {
                        subtype: "success".to_string(),
                        is_error: false,
                        terminal_reason: None,
                        input_tokens: 10,
                        output_tokens: 1,
                        cache_read: 5,
                        cache_write: 2,
                        cost_total: 0.1,
                        context_tokens: 100 * n as u64,
                    },
                )
                .await
                .expect("end");
        }
        let conn = store.conn.lock().expect("lock");
        let rows: Vec<(String, i64)> = conn
            .prepare("SELECT session_id, context_tokens FROM turns WHERE agent_id = ?1 ORDER BY n")
            .expect("prep")
            .query_map([&a.id], |r| Ok((r.get(0)?, r.get(1)?)))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows");
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0].0, rows[1].0);
        assert_eq!(rows[1], ("s-new".to_string(), 200));
        assert_eq!(rows[0].1, 100);
    }

    /// A hosted agent with two sessions and an external session: the sums are right at each
    /// level and the shares add up to the week's seven_day percentage (br-zn6s).
    #[tokio::test]
    async fn usage_groups_sum_at_each_level_and_shares_add_up() {
        use crate::usage_groups::{build, seven_day_points};
        let (store, _tmp) = store().await;
        let a = store.insert_agent(new_agent("w1")).await.expect("insert");
        let t0 = DateTime::parse_from_rfc3339("2026-10-10T10:00:00Z")
            .expect("ts")
            .with_timezone(&Utc);
        let m = chrono::Duration::minutes;
        for (n, session, cost, ctx) in [
            (1, None, 1.0, 100),
            (2, None, 2.0, 400),
            (3, Some("s-new"), 4.0, 50),
        ] {
            if let Some(s) = session {
                store.set_agent_session(&a.id, s).await.expect("renew");
            }
            store
                .add_turn_start(&a.id, n, t0 + m(n as i64))
                .await
                .expect("start");
            store
                .end_turn(
                    &a.id,
                    n,
                    TurnEnd {
                        subtype: "success".to_string(),
                        is_error: false,
                        terminal_reason: None,
                        input_tokens: 10,
                        output_tokens: 1,
                        cache_read: 5,
                        cache_write: 2,
                        cost_total: cost,
                        context_tokens: ctx,
                    },
                )
                .await
                .expect("end");
        }
        // The first report is a baseline (its totals are above the fresh limits); the second
        // adds $3 and 1000 input tokens.
        for (cost, input, at) in [(50.0, 200_000, t0 + m(10)), (53.0, 201_000, t0 + m(20))] {
            store
                .record_interactive_usage(InteractiveUsageRow {
                    observed_at: at,
                    session_id: Some("ext-1".to_string()),
                    model: None,
                    cost_usd: Some(cost),
                    context_used_percentage: None,
                    context_used_tokens: Some(input),
                    context_max_tokens: None,
                    identity: Some("external:advisor/pm".to_string()),
                    total_input_tokens: Some(input),
                    total_output_tokens: Some(0),
                })
                .await
                .expect("record");
        }
        let mk = |u, at| RateLimit {
            window: "seven_day".to_string(),
            status: None,
            utilization: Some(u),
            resets_at: Some(t0 + chrono::Duration::days(3)),
            observed_at: at,
        };
        store
            .upsert_rate_limit(mk(0.10, t0 - m(30)))
            .await
            .expect("rl");
        store
            .upsert_rate_limit(mk(0.35, t0 + m(30)))
            .await
            .expect("rl");

        let sessions = store
            .usage_sessions(Some(t0), None)
            .await
            .expect("sessions");
        let hist = store
            .rate_limit_history("seven_day".to_string(), None)
            .await
            .expect("history");
        let points = seven_day_points(&hist, Some(t0), None);
        assert!(
            (points - 25.0).abs() < 1e-9,
            "35% - 10% before the period: {points}"
        );
        let g = build(sessions, points, "p", "m", Some(t0), None);

        assert!((g.total_cost_usd - 10.0).abs() < 1e-9);
        let roles: Vec<(&str, f64)> = g
            .roles
            .iter()
            .map(|r| (r.key.as_str(), r.cost_usd))
            .collect();
        assert_eq!(roles, [("advisor", 3.0), ("worker", 7.0)]);
        let worker = &g.roles[1];
        assert_eq!(worker.turns, Some(3));
        assert_eq!(worker.tokens.input, 30);
        assert_eq!(worker.children.len(), 1);
        let name = &worker.children[0];
        assert_eq!((name.key.as_str(), name.children.len()), ("w1", 2));
        let (s1, s2) = (&name.children[0], &name.children[1]);
        assert!((s1.cost_usd - 3.0).abs() < 1e-9 && (s2.cost_usd - 4.0).abs() < 1e-9);
        assert_eq!((s1.context_growth, s2.context_growth), (300, 0));
        assert_eq!(s2.key, "s-new");
        let ext = &g.roles[0].children[0].children[0];
        assert!(ext.external && ext.agent_id.is_none());
        assert_eq!((ext.tokens.input, ext.context_growth), (1000, 1000));
        assert!(g.roles.iter().all(|r| r.project == "p" && r.machine == "m"));
        let sum: f64 = g.roles.iter().map(|r| r.share_points).sum();
        assert!((sum - 25.0).abs() < 1e-9);
        let leaves: f64 = g
            .roles
            .iter()
            .flat_map(|r| &r.children)
            .flat_map(|n| &n.children)
            .map(|s| s.share_points)
            .sum();
        assert!((leaves - 25.0).abs() < 1e-9);
        assert!((g.points_per_dollar.expect("ppd") - 2.5).abs() < 1e-9);
    }

    /// A removed agent is still listed and charted, and the buckets add up to the breakdown.
    #[tokio::test]
    async fn usage_series_and_instances_cover_removed_agents() {
        let (store, _tmp) = store().await;
        let a = store.insert_agent(new_agent("w1")).await.expect("insert");
        let base = DateTime::parse_from_rfc3339("2026-10-10T10:30:00Z")
            .expect("ts")
            .with_timezone(&Utc);
        for (n, at, cost) in [
            (1, base, 1.0),
            (2, base + chrono::Duration::minutes(10), 2.0),
            (3, base + chrono::Duration::hours(2), 4.0),
            (4, base + chrono::Duration::days(1), 8.0),
        ] {
            store.add_turn_start(&a.id, n, at).await.expect("start");
            store
                .end_turn(
                    &a.id,
                    n,
                    TurnEnd {
                        subtype: "success".to_string(),
                        is_error: false,
                        terminal_reason: None,
                        input_tokens: 10,
                        output_tokens: 1,
                        cache_read: 5,
                        cache_write: 2,
                        cost_total: cost,
                        context_tokens: 10,
                    },
                )
                .await
                .expect("end");
        }
        store.delete_agent(&a.id).await.expect("remove");

        let inst = store.usage_instances(None).await.expect("instances");
        assert_eq!(inst.len(), 1);
        assert!(inst[0].removed);
        assert_eq!(inst[0].name, "w1");
        assert_eq!(inst[0].turns, Some(4));
        assert_eq!(inst[0].first_turn_at, base);

        let hourly = store
            .usage_series(a.id.clone(), None, UsageBucket::Hour)
            .await
            .expect("hourly");
        assert_eq!(hourly.points.len(), 3);
        assert_eq!(hourly.points[0].turns, Some(2));
        assert!((hourly.points[0].cost_usd - 3.0).abs() < 1e-9);

        let daily = store
            .usage_series(a.id.clone(), None, UsageBucket::Day)
            .await
            .expect("daily");
        assert_eq!(daily.points.len(), 2);
        let bd = store
            .usage_breakdown(None, UsageGroupBy::Agent)
            .await
            .expect("breakdown");
        for series in [&hourly, &daily] {
            let turns: u32 = series.points.iter().filter_map(|p| p.turns).sum();
            let cost: f64 = series.points.iter().map(|p| p.cost_usd).sum();
            let input: u64 = series.points.iter().map(|p| p.tokens.input).sum();
            assert_eq!(turns, bd.total_turns);
            assert!((cost - bd.total_cost_usd).abs() < 1e-9);
            assert_eq!(input, bd.total_tokens.input);
        }
    }

    #[tokio::test]
    async fn usage_breakdown_since_filters_turns() {
        let (store, _tmp) = store().await;
        let a = store.insert_agent(new_agent("w1")).await.expect("insert a");

        let old_turn_at = Utc::now() - chrono::Duration::days(10);
        store
            .add_turn_start(&a.id, 1, old_turn_at)
            .await
            .expect("old turn start");
        store
            .end_turn(
                &a.id,
                1,
                TurnEnd {
                    subtype: "success".to_string(),
                    is_error: false,
                    terminal_reason: None,
                    input_tokens: 1000,
                    output_tokens: 10,
                    cache_read: 0,
                    cache_write: 0,
                    cost_total: 9.99,
                    context_tokens: 1000,
                },
            )
            .await
            .expect("end old turn");

        store
            .add_turn_start(&a.id, 2, Utc::now())
            .await
            .expect("recent turn start");
        store
            .end_turn(
                &a.id,
                2,
                TurnEnd {
                    subtype: "success".to_string(),
                    is_error: false,
                    terminal_reason: None,
                    input_tokens: 50,
                    output_tokens: 5,
                    cache_read: 0,
                    cache_write: 0,
                    cost_total: 0.01,
                    context_tokens: 50,
                },
            )
            .await
            .expect("end recent turn");

        let since = Utc::now() - chrono::Duration::days(1);
        let recent = store
            .usage_breakdown(Some(since), UsageGroupBy::Agent)
            .await
            .expect("recent breakdown");
        assert_eq!(recent.total_turns, 1);
        assert!((recent.total_cost_usd - 0.01).abs() < 1e-9);

        let all = store
            .usage_breakdown(None, UsageGroupBy::Agent)
            .await
            .expect("unfiltered breakdown");
        assert_eq!(all.total_turns, 2);
    }

    /// Two turns with a gap between them: wall time spans the gap, busy
    /// time doesn't, so wall must come out strictly greater than busy.
    #[tokio::test]
    async fn agent_wall_time_spans_the_gap_between_turns() {
        let (store, _tmp) = store().await;
        let a = store.insert_agent(new_agent("w1")).await.expect("insert a");

        let t0 = Utc::now() - chrono::Duration::seconds(300);
        let t1 = t0 + chrono::Duration::seconds(10); // turn 1: 10s busy
        let t2 = t1 + chrono::Duration::seconds(50); // 50s gap, idle
        let t3 = t2 + chrono::Duration::seconds(20); // turn 2: 20s busy

        let agent_id = a.id.clone();
        store
            .with_conn(move |conn| {
                for (n, started, ended) in [(1, t0, t1), (2, t2, t3)] {
                    conn.execute(
                        "INSERT INTO turns(agent_id, agent_name, role, model, n, started_at, ended_at)
                         SELECT id, name, role, model, ?2, ?3, ?4 FROM agents WHERE id = ?1",
                        rusqlite::params![agent_id, n, started.to_rfc3339(), ended.to_rfc3339()],
                    )?;
                }
                Ok(())
            })
            .await
            .expect("insert turns");

        let usage = store.usage().await.expect("usage");
        let row = usage
            .agents
            .iter()
            .find(|u| u.agent == a.id)
            .expect("agent row");
        assert_eq!(row.busy_seconds, 30);
        assert_eq!(row.wall_seconds, Some(80));
        assert!(row.wall_seconds.expect("wall") > row.busy_seconds);
    }

    #[tokio::test]
    async fn handover_closes_the_stale_restart_request_and_context_note() {
        let (store, _tmp) = store().await;
        let send = |from: &str, to: &str, body: &str| {
            store.insert_message(NewMessage {
                from: from.to_string(),
                to: to.to_string(),
                to_kind: RecipientKind::External,
                kind: MessageKind::Note,
                body: body.to_string(),
                reply_to: None,
                when: When::Now,
                state: MessageState::Written,
            })
        };
        let request = send(
            "external:aide",
            "external:aide",
            "The human is restarting.\n\nSay nothing more once it is written; go.",
        )
        .await
        .expect("request");
        let ctx = send(
            "system",
            "external:aide",
            "Your context is 200k tokens. Hand over.",
        )
        .await
        .expect("ctx");
        let other = send("human", "external:aide", "unrelated")
            .await
            .expect("other");
        let elsewhere = send("system", "external:advisor", "Your context is 200k tokens.")
            .await
            .expect("else");

        let n = store
            .close_handover_requests("external:aide")
            .await
            .expect("close");
        assert_eq!(n, 2);
        let state = |id: String| {
            let store = &store;
            async move {
                store
                    .get_message(&id)
                    .await
                    .expect("get")
                    .expect("some")
                    .state
            }
        };
        assert_eq!(state(request.id).await, MessageState::Read);
        assert_eq!(state(ctx.id).await, MessageState::Read);
        assert_eq!(state(other.id).await, MessageState::Written);
        assert_eq!(state(elsewhere.id).await, MessageState::Written);
    }

    #[tokio::test]
    async fn unread_count_for_human_inbox() {
        let (store, _tmp) = store().await;
        assert_eq!(store.unread_count("human").await.expect("unread"), 0);

        store
            .insert_message(NewMessage {
                from: "agent:w1".to_string(),
                to: "human".to_string(),
                to_kind: RecipientKind::Human,
                kind: MessageKind::Note,
                body: "done".to_string(),
                reply_to: None,
                when: When::Now,
                state: MessageState::Written,
            })
            .await
            .expect("insert");
        assert_eq!(store.unread_count("human").await.expect("unread"), 1);
    }

    #[tokio::test]
    async fn rate_limits_upsert() {
        let (store, _tmp) = store().await;
        store
            .upsert_rate_limit(RateLimit {
                window: "five_hour".to_string(),
                status: Some("allowed".to_string()),
                utilization: Some(0.5),
                resets_at: None,
                observed_at: Utc::now(),
            })
            .await
            .expect("upsert");
        store
            .upsert_rate_limit(RateLimit {
                window: "five_hour".to_string(),
                status: Some("allowed_warning".to_string()),
                utilization: Some(0.9),
                resets_at: None,
                observed_at: Utc::now(),
            })
            .await
            .expect("upsert again");

        let limits = store.rate_limits().await.expect("rate limits");
        assert_eq!(limits.len(), 1);
        assert_eq!(limits[0].status.as_deref(), Some("allowed_warning"));
    }

    fn reading(u: f64, resets: Option<DateTime<Utc>>, at: DateTime<Utc>) -> RateLimit {
        RateLimit {
            window: "five_hour".to_string(),
            status: None,
            utilization: Some(u),
            resets_at: resets,
            observed_at: at,
        }
    }

    #[tokio::test]
    async fn rate_limit_history_appends_changes_only_in_time_order() {
        let (store, _tmp) = store().await;
        let t0 = Utc::now() - chrono::Duration::hours(3);
        let m = chrono::Duration::minutes;
        for (u, at) in [
            (0.1, t0),
            (0.1, t0 + m(1)),
            (0.2, t0 + m(2)),
            (0.2, t0 + m(3)),
        ] {
            store
                .upsert_rate_limit(reading(u, None, at))
                .await
                .expect("upsert");
        }
        // Another window and a changed resets_at are their own points.
        let mut other = reading(0.5, None, t0 + m(4));
        other.window = "seven_day".to_string();
        store.upsert_rate_limit(other).await.expect("other");
        store
            .upsert_rate_limit(reading(0.2, Some(t0 + m(300)), t0 + m(5)))
            .await
            .expect("reset");

        let h = store
            .rate_limit_history("five_hour".to_string(), None)
            .await
            .expect("history");
        let us: Vec<_> = h.iter().map(|p| p.utilization).collect();
        assert_eq!(us, vec![Some(0.1), Some(0.2), Some(0.2)]);
        assert!(h.windows(2).all(|w| w[0].observed_at <= w[1].observed_at));
        let h = store
            .rate_limit_history("five_hour".to_string(), Some(t0 + m(2)))
            .await
            .expect("since");
        assert_eq!(h.len(), 2);
    }

    #[tokio::test]
    async fn rate_limit_history_prunes_past_retention() {
        let (store, _tmp) = store().await;
        let now = Utc::now();
        let old = now - chrono::Duration::days(sync::RATE_LIMIT_HISTORY_DAYS + 1);
        store
            .upsert_rate_limit(reading(0.1, None, old))
            .await
            .expect("old");
        store
            .upsert_rate_limit(reading(0.2, None, now))
            .await
            .expect("new");
        let h = store
            .rate_limit_history("five_hour".to_string(), None)
            .await
            .expect("history");
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].utilization, Some(0.2));
    }

    #[tokio::test]
    async fn interactive_usage_shows_up_in_usage_today() {
        let (store, _tmp) = store().await;
        store
            .record_interactive_usage(InteractiveUsageRow {
                observed_at: Utc::now(),
                session_id: Some("sess-1".to_string()),
                model: Some("opus".to_string()),
                cost_usd: Some(0.42),
                context_used_percentage: Some(0.25),
                context_used_tokens: Some(50_000),
                context_max_tokens: Some(200_000),
                identity: None,
                total_input_tokens: None,
                total_output_tokens: None,
            })
            .await
            .expect("record");

        let usage = store.usage().await.expect("usage");
        assert_eq!(usage.interactive_today.len(), 1);
        let row = &usage.interactive_today[0];
        assert_eq!(row.session_id.as_deref(), Some("sess-1"));
        assert_eq!(row.model.as_deref(), Some("opus"));
        assert_eq!(row.cost_usd, Some(0.42));
        assert_eq!(row.context_used_percentage, Some(0.25));
        assert_eq!(row.context_used_tokens, Some(50_000));
        assert_eq!(row.context_max_tokens, Some(200_000));
    }

    #[tokio::test]
    async fn interactive_usage_before_today_is_excluded() {
        let (store, _tmp) = store().await;
        store
            .record_interactive_usage(InteractiveUsageRow {
                observed_at: Utc::now() - chrono::Duration::days(2),
                session_id: None,
                model: None,
                cost_usd: Some(1.0),
                context_used_percentage: None,
                context_used_tokens: None,
                context_max_tokens: None,
                identity: None,
                total_input_tokens: None,
                total_output_tokens: None,
            })
            .await
            .expect("record");

        let usage = store.usage().await.expect("usage");
        assert!(usage.interactive_today.is_empty());
    }

    fn external_row(
        session: &str,
        identity: &str,
        cost: Option<f64>,
        input: Option<u64>,
    ) -> InteractiveUsageRow {
        InteractiveUsageRow {
            observed_at: Utc::now(),
            session_id: Some(session.to_string()),
            model: Some("opus".to_string()),
            cost_usd: cost,
            context_used_percentage: None,
            context_used_tokens: None,
            context_max_tokens: None,
            identity: Some(identity.to_string()),
            total_input_tokens: input,
            total_output_tokens: None,
        }
    }

    #[tokio::test]
    async fn a_new_sessions_first_report_counts_but_a_resumed_ones_does_not() {
        let (store, _tmp) = store().await;
        let rec = |r| store.record_interactive_usage(r);
        rec(external_row(
            "new",
            "external:advisor/n",
            Some(0.2),
            Some(5_000),
        ))
        .await
        .expect("new");
        rec(external_row(
            "old",
            "external:advisor/o",
            Some(30.0),
            Some(900_000),
        ))
        .await
        .expect("resumed");
        let by_agent = store.usage().await.expect("usage").external;
        let get = |k: &str| by_agent.iter().find(|r| r.key == k).expect("row").clone();
        assert_eq!(get("external:advisor/n").cost_usd, Some(0.2));
        assert_eq!(get("external:advisor/n").input_tokens, Some(5_000));
        assert_eq!(get("external:advisor/o").cost_usd, Some(0.0));
        assert_eq!(get("external:advisor/o").input_tokens, Some(0));
    }

    #[tokio::test]
    async fn external_usage_sums_deltas_per_session_and_survives_a_counter_reset() {
        let (store, _tmp) = store().await;
        let rec = |r| store.record_interactive_usage(r);
        // First report is the baseline; then +1.0, an unchanged repeat (no row), a reset to
        // 0.5 (all new), and a second session of the same role.
        rec(external_row(
            "s1",
            "external:advisor/a",
            Some(10.0),
            Some(100),
        ))
        .await
        .expect("1");
        rec(external_row(
            "s1",
            "external:advisor/a",
            Some(11.0),
            Some(150),
        ))
        .await
        .expect("2");
        rec(external_row(
            "s1",
            "external:advisor/a",
            Some(11.0),
            Some(150),
        ))
        .await
        .expect("3");
        rec(external_row(
            "s1",
            "external:advisor/a",
            Some(0.5),
            Some(20),
        ))
        .await
        .expect("4");
        rec(external_row("s2", "external:advisor/b", Some(2.0), None))
            .await
            .expect("5");
        rec(external_row("s2", "external:advisor/b", Some(3.0), None))
            .await
            .expect("6");

        let by_agent = store.usage().await.expect("usage").external;
        assert_eq!(by_agent.len(), 2);
        assert_eq!(by_agent[0].key, "external:advisor/a");
        assert_eq!(by_agent[0].cost_usd, Some(1.5));
        assert_eq!(by_agent[0].input_tokens, Some(70));
        // No token totals ever reported: not measured, not zero.
        assert_eq!(by_agent[1].cost_usd, Some(1.0));
        assert_eq!(by_agent[1].input_tokens, None);

        let by_role = store
            .usage_breakdown(None, UsageGroupBy::Role)
            .await
            .expect("breakdown")
            .external;
        assert_eq!(by_role.len(), 1);
        assert_eq!(by_role[0].key, "advisor");
        assert_eq!(by_role[0].sessions, 2);
        assert_eq!(by_role[0].cost_usd, Some(2.5));
    }

    /// An external session is listed and charted like an instance, and its buckets add up to
    /// the `by=agent` external totals; turns and busy time stay unmeasured.
    #[tokio::test]
    async fn external_sessions_list_and_chart_like_instances() {
        let (store, _tmp) = store().await;
        let mut r1 = external_row("s1", "external:advisor/a", Some(10.0), Some(100));
        let mut r2 = external_row("s1", "external:advisor/a", Some(12.5), Some(180));
        r1.observed_at = DateTime::parse_from_rfc3339("2026-10-10T10:00:00Z")
            .expect("ts")
            .with_timezone(&Utc);
        r2.observed_at = r1.observed_at + chrono::Duration::hours(3);
        store.record_interactive_usage(r1).await.expect("1");
        store.record_interactive_usage(r2).await.expect("2");

        let inst = store.usage_instances(None).await.expect("instances");
        assert_eq!(inst.len(), 1);
        assert!(inst[0].external && !inst[0].removed);
        assert_eq!(inst[0].agent_id, "external:advisor/a");
        assert_eq!(inst[0].role, "advisor");
        assert_eq!(inst[0].turns, None);
        assert!((inst[0].cost_usd - 2.5).abs() < 1e-9);

        let series = store
            .usage_series("external:advisor/a".to_string(), None, UsageBucket::Hour)
            .await
            .expect("series");
        // The baseline report adds a zero bucket; the second report carries the delta.
        assert_eq!(series.points.len(), 2);
        assert!(
            series
                .points
                .iter()
                .all(|p| p.turns.is_none() && p.busy_seconds.is_none())
        );
        let cost: f64 = series.points.iter().map(|p| p.cost_usd).sum();
        let input: u64 = series.points.iter().map(|p| p.tokens.input).sum();
        let ext = store.usage().await.expect("usage").external;
        assert!((cost - ext[0].cost_usd.expect("cost")).abs() < 1e-9);
        assert_eq!(Some(input), ext[0].input_tokens);
    }

    #[tokio::test]
    async fn external_usage_without_a_session_id_is_not_measured() {
        let (store, _tmp) = store().await;
        let mut row = external_row("x", "external:aide", Some(5.0), None);
        row.session_id = None;
        store.record_interactive_usage(row).await.expect("record");
        let ext = store.usage().await.expect("usage").external;
        assert_eq!(ext.len(), 1);
        assert_eq!(ext[0].cost_usd, None);
    }

    #[tokio::test]
    async fn insert_task_uses_the_prefix_and_four_ticket_alphabet_chars() {
        let (store, _tmp) = store().await;
        let task = store
            .insert_task("tw", "Add foo", TicketKind::Feature, "human")
            .await
            .expect("insert");
        assert!(task.id.starts_with("tw-"));
        let suffix = task.id.strip_prefix("tw-").expect("prefix");
        assert_eq!(suffix.len(), 4);
        assert!(
            suffix
                .bytes()
                .all(|b| b"abcdefghjkmnpqrstuvwxyz23456789".contains(&b))
        );
        assert_eq!(task.title, "Add foo");
        assert_eq!(task.kind, TicketKind::Feature);
        assert_eq!(task.state, TicketState::Pending);
        assert_eq!(task.created_at, task.updated_at);
    }

    /// Not a forced collision (that would need to control the RNG), but
    /// exercises the id generator enough times to give the retry loop's
    /// unique-violation path a real chance to fire, and pins down that
    /// every id actually is unique.
    #[tokio::test]
    async fn insert_task_ids_are_unique_under_repeated_use() {
        let (store, _tmp) = store().await;
        let mut ids = std::collections::HashSet::new();
        for i in 0..50 {
            let t = store
                .insert_task("tw", &format!("task {i}"), TicketKind::Chore, "human")
                .await
                .expect("insert");
            assert!(ids.insert(t.id), "duplicate task id generated");
        }
    }

    #[tokio::test]
    async fn get_list_edit_and_transition_a_task() {
        let (store, _tmp) = store().await;
        let task = store
            .insert_task("tw", "Add foo", TicketKind::Bug, "human")
            .await
            .expect("insert");

        let fetched = store.get_task(&task.id).await.expect("get").expect("some");
        assert_eq!(fetched.id, task.id);
        assert_eq!(fetched.title, task.title);
        assert_eq!(fetched.kind, task.kind);
        assert_eq!(fetched.state, task.state);

        assert!(store.get_task("tw-nope").await.expect("get").is_none());

        store
            .set_task_title(&task.id, "Add foo, better")
            .await
            .expect("retitle");
        let renamed = store.get_task(&task.id).await.expect("get").expect("some");
        assert_eq!(renamed.title, "Add foo, better");
        assert!(renamed.updated_at >= task.updated_at);

        store
            .set_task_state(&task.id, TicketState::Dropped)
            .await
            .expect("drop");
        store
            .set_task_state(&task.id, TicketState::Reopened)
            .await
            .expect("reopen");
        let reopened = store.get_task(&task.id).await.expect("get").expect("some");
        assert_eq!(reopened.state, TicketState::Reopened);

        let err = store
            .set_task_title("tw-nope", "x")
            .await
            .expect_err("no such task");
        assert!(matches!(err, StoreError::NotFound(_)));

        let list = store.list_tasks().await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, task.id);
    }

    #[tokio::test]
    async fn insert_list_and_delete_edges() {
        let (store, _tmp) = store().await;
        let a = store
            .insert_task("tw", "A", TicketKind::Chore, "human")
            .await
            .expect("insert a");
        let b = store
            .insert_task("tw", "B", TicketKind::Chore, "human")
            .await
            .expect("insert b");

        let edge = store
            .insert_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("insert edge");
        assert_eq!(edge.from, a.id);
        assert_eq!(edge.to, b.id);
        assert_eq!(edge.kind, EdgeKind::Blocks);

        let list = store.list_edges().await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], edge);

        let err = store
            .insert_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect_err("duplicate edge");
        assert!(matches!(err, StoreError::Conflict(_)));

        store
            .delete_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect("delete edge");
        assert!(store.list_edges().await.expect("list").is_empty());

        let err = store
            .delete_edge(&a.id, &b.id, EdgeKind::Blocks)
            .await
            .expect_err("already deleted");
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    #[tokio::test]
    async fn insert_list_and_delete_open_questions() {
        let (store, _tmp) = store().await;
        let task = store
            .insert_task("tw", "Add foo", TicketKind::Feature, "human")
            .await
            .expect("insert task");
        let msg = store
            .insert_message(NewMessage {
                from: "agent:w1".to_string(),
                to: task.id.clone(),
                to_kind: RecipientKind::Ticket,
                kind: MessageKind::Question,
                body: "which endpoint?".to_string(),
                reply_to: None,
                when: When::Now,
                state: MessageState::Delivered,
            })
            .await
            .expect("insert question message");

        let asked_at = Utc::now();
        store
            .insert_open_question(&task.id, &msg.id, &"agent:w1".to_string(), asked_at)
            .await
            .expect("insert open question");

        let list = store.list_open_questions().await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].task_id, task.id);
        assert_eq!(list[0].message_id, msg.id);
        assert_eq!(list[0].asked_by, "agent:w1");

        // One open question per task at a time: a second insert for the
        // same task conflicts on the `task_id` primary key.
        let err = store
            .insert_open_question(&task.id, &msg.id, &"human".to_string(), Utc::now())
            .await
            .expect_err("already has an open question");
        assert!(matches!(err, StoreError::Conflict(_)));

        store
            .delete_open_question(&task.id)
            .await
            .expect("delete open question");
        assert!(store.list_open_questions().await.expect("list").is_empty());

        let err = store
            .delete_open_question(&task.id)
            .await
            .expect_err("already answered");
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    #[tokio::test]
    async fn insert_list_and_delete_claims() {
        let (store, _tmp) = store().await;
        let task = store
            .insert_task("tw", "Add foo", TicketKind::Feature, "human")
            .await
            .expect("insert task");

        let claimed_at = Utc::now();
        store
            .insert_claim(&task.id, &"agent:w1".to_string(), claimed_at)
            .await
            .expect("insert claim");

        let list = store.list_claims().await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].task_id, task.id);
        assert_eq!(list[0].claimed_by, "agent:w1");

        // One claimant per task at a time: a second insert for the same
        // task conflicts on the `task_id` primary key.
        let err = store
            .insert_claim(&task.id, &"agent:w2".to_string(), Utc::now())
            .await
            .expect_err("already claimed");
        assert!(matches!(err, StoreError::Conflict(_)));

        store.delete_claim(&task.id).await.expect("delete claim");
        assert!(store.list_claims().await.expect("list").is_empty());

        let err = store
            .delete_claim(&task.id)
            .await
            .expect_err("already released");
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    /// br-w6ww: the claim row and the task state change together or not at all.
    #[tokio::test]
    async fn end_claim_changes_both_or_neither() {
        let (store, _tmp) = store().await;
        let task = store
            .insert_task("tw", "Add foo", TicketKind::Feature, "human")
            .await
            .expect("insert task");
        store
            .insert_claim(&task.id, &"agent:w1".to_string(), Utc::now())
            .await
            .expect("insert claim");

        // An unknown task fails the state change; the claim delete rolls back.
        store
            .end_claim("tw-nope", TicketState::Integrated)
            .await
            .expect_err("no such task");
        assert_eq!(store.list_claims().await.expect("list").len(), 1);

        store
            .end_claim(&task.id, TicketState::Integrated)
            .await
            .expect("end claim");
        assert!(store.list_claims().await.expect("list").is_empty());
        let row = store.get_task(&task.id).await.expect("get").expect("row");
        assert_eq!(row.state, TicketState::Integrated);

        // No claims row is fine (a task stuck by an older build).
        store
            .end_claim(&task.id, TicketState::Integrated)
            .await
            .expect("no row");
    }
}
