//! SQLite-backed store for principals, agents, turns, messages, events and
//! rate limits. See docs/design/storage.md for the schema this implements
//! and docs/design/agent-host/principals.md for principals/tokens.
//!
//! `Store` is a thin, cloneable async handle: every public method runs its
//! SQL on a single connection inside `spawn_blocking`, guarded by a mutex
//! (CLAUDE.md: never block the runtime). The actual SQL lives in the
//! private `sync` module, which is plain, synchronous and independently
//! testable without an async runtime.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bridle_api::types::{
    Agent, AgentState, AgentUsage, Event, EventQuery, ExitInfo, Message, MessageKind, MessageState,
    PrincipalId, PrincipalKind, RateLimit, TokenCreated, TokenTotals, Usage, When,
};
use chrono::{DateTime, SecondsFormat, Utc};
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
}

/// An authenticated caller. Not a wire type (see `bridle_api::types`):
/// tokens never leave the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub id: PrincipalId,
    pub kind: PrincipalKind,
}

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningAgent {
    pub id: String,
    /// As stored, e.g. `working`.
    pub state: String,
    pub pid: Option<i32>,
    pub pid_start: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewMessage {
    pub from: PrincipalId,
    /// Concrete recipient: `"human"` or an agent id. Name/`"me"` resolution
    /// happens above the store.
    pub to: String,
    pub kind: MessageKind,
    pub body: String,
    pub reply_to: Option<String>,
    pub when: When,
    /// The message's initial state (`Pending`, `Held` or `Written`,
    /// depending on whether it was delivered immediately). Timestamps for
    /// later transitions are set through `set_message_state`, not here.
    pub state: MessageState,
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
            .expect("store open task panicked")?;
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
        .expect("store blocking task panicked")
    }

    // ---------- principals / tokens ----------

    pub async fn ensure_human_token(&self) -> Result<Option<String>, StoreError> {
        self.with_conn(sync::ensure_human_token).await
    }

    pub async fn create_external_token(&self, name: &str) -> Result<TokenCreated, StoreError> {
        let name = name.to_string();
        self.with_conn(move |c| sync::create_external_token(c, &name))
            .await
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

    pub async fn revoke_principal(&self, id: &str) -> Result<(), StoreError> {
        let id = id.to_string();
        self.with_conn(move |c| sync::revoke_principal(c, &id))
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

    pub async fn set_session(&self, id: &str, session_id: &str) -> Result<(), StoreError> {
        let id = id.to_string();
        let session_id = session_id.to_string();
        self.with_conn(move |c| sync::set_session(c, &id, &session_id))
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

    // ---------- rate limits / usage ----------

    pub async fn upsert_rate_limit(&self, rl: RateLimit) -> Result<(), StoreError> {
        self.with_conn(move |c| sync::upsert_rate_limit(c, &rl))
            .await
    }

    pub async fn rate_limits(&self) -> Result<Vec<RateLimit>, StoreError> {
        self.with_conn(sync::rate_limits).await
    }

    pub async fn usage(&self) -> Result<Usage, StoreError> {
        self.with_conn(sync::usage).await
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
    const SCHEMA_V2: &str = r#"
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

    const MIGRATIONS: &[&str] = &[SCHEMA_V1, SCHEMA_V2];

    pub(super) fn open(path: &Path) -> Result<Connection, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;
        migrate(&conn)?;
        Ok(conn)
    }

    pub(super) fn migrate(conn: &Connection) -> Result<(), StoreError> {
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let version = usize::try_from(version).unwrap_or(0);
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
        }
    }

    fn kind_from_str(s: &str) -> MessageKind {
        match s {
            "question" => MessageKind::Question,
            "answer" => MessageKind::Answer,
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
            PrincipalKind::System => "system",
        }
    }

    fn principal_kind_from_str(s: &str) -> PrincipalKind {
        match s {
            "agent" => PrincipalKind::Agent,
            "external" => PrincipalKind::External,
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
            return Err(StoreError::Conflict(format!(
                "principal {id:?} already exists"
            )));
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

    pub(super) fn create_external_token(
        conn: &Connection,
        name: &str,
    ) -> Result<TokenCreated, StoreError> {
        let id = format!("external:{name}");
        let token = create_principal_active_only(conn, &id, PrincipalKind::External, name)?;
        Ok(TokenCreated {
            principal: id,
            token,
        })
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

    // ---------- agents ----------

    const AGENT_SELECT: &str = "
        SELECT a.id, a.name, a.role, a.state, a.model, a.session_id, a.pid, a.pid_start,
               a.workdir_kind, a.cwd, a.worktree, a.branch, a.created_at, a.updated_at,
               a.turns, a.cost_usd_total, a.last_event_at, a.turn_started_at,
               a.exit_code, a.exit_signal, a.exit_reason, a.created_by,
               (SELECT COUNT(*) FROM messages m WHERE m.to_kind='agent' AND m.to_id=a.id AND m.state='held') AS held_messages,
               (SELECT COUNT(*) FROM messages m WHERE m.to_kind='agent' AND m.to_id=a.id AND m.state='written') AS unacked_messages
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
            role: row.get(2)?,
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
        })
    }

    pub(super) fn insert_agent(conn: &Connection, new: &NewAgent) -> Result<Agent, StoreError> {
        let id = new_agent_id();
        let now = Utc::now();
        let result = conn.execute(
            "INSERT INTO agents(id, name, role, state, model, session_id, pid, pid_start,
                workdir_kind, cwd, worktree, branch, created_at, updated_at, turns,
                cost_usd_total, last_event_at, turn_started_at, exit_code, exit_signal,
                exit_reason, created_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7, ?8, ?9, ?10, ?11, ?11, 0, 0,
                NULL, NULL, NULL, NULL, NULL, ?12)",
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

    pub(super) fn list_agents(
        conn: &Connection,
        include_terminal: bool,
    ) -> Result<Vec<Agent>, StoreError> {
        let sql = if include_terminal {
            format!("{AGENT_SELECT} ORDER BY a.created_at ASC")
        } else {
            format!(
                "{AGENT_SELECT} WHERE a.state NOT IN ('stopped','exited','crashed','lost') ORDER BY a.created_at ASC"
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

    pub(super) fn set_session(
        conn: &Connection,
        id: &str,
        session_id: &str,
    ) -> Result<(), StoreError> {
        let n = conn.execute(
            "UPDATE agents SET session_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![session_id, fmt_dt(Utc::now()), id],
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
            "INSERT INTO turns(agent_id, agent_name, role, model, n, started_at)
             SELECT id, name, role, model, ?2, ?3 FROM agents WHERE id = ?1",
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
                input_tokens = ?5, output_tokens = ?6, cache_read = ?7, cache_write = ?8, cost_total = ?9
             WHERE agent_id = ?10 AND n = ?11",
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
                id,
                n,
            ],
        )?;
        if updated_turn == 0 {
            return Err(StoreError::NotFound(format!("turn {id}/{n}")));
        }
        let updated_agent = conn.execute(
            "UPDATE agents SET turns = turns + 1, cost_usd_total = cost_usd_total + ?1,
                turn_started_at = NULL, updated_at = ?2 WHERE id = ?3",
            params![turn.cost_total, fmt_dt(now), id],
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
               created_at, written_at, delivered_at, read_at
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
        })
    }

    pub(super) fn insert_message(
        conn: &Connection,
        new: &NewMessage,
    ) -> Result<Message, StoreError> {
        let to_kind = if new.to == "human" { "human" } else { "agent" };
        let now = Utc::now();
        conn.execute(
            "INSERT INTO messages(id, from_principal, to_kind, to_id, kind, body, reply_to,
                when_mode, state, created_at, written_at, delivered_at, read_at)
             VALUES ('', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, NULL, NULL)",
            params![
                new.from,
                to_kind,
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
                read_at = CASE WHEN ?1 = 'read' THEN ?2 ELSE read_at END
             WHERE id = ?3",
            params![msg_state_str(state), fmt_dt(at), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
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
        let sql = "SELECT seq, ts, kind, actor, data, agent_id FROM events
             WHERE (?1 IS NULL OR seq > ?1)
               AND (?2 IS NULL OR agent_id = ?2)
               AND (?3 IS NULL OR kind LIKE ?3 || '%')
             ORDER BY seq ASC
             LIMIT ?4";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![q.since, q.agent, q.kind, q.limit.unwrap_or(500)],
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
        Ok(())
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
        let rows = stmt.query_map([], |row| {
            Ok(AgentUsage {
                agent: row.get(0)?,
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
        })
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

    async fn store() -> (Store, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        (store, tmp)
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
        }
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
            .set_session(&agent.id, "sess-2")
            .await
            .expect("set session");
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
        assert_eq!(after.session_id, "sess-2");

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
            .create_external_token("orchestrator")
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
            .create_external_token("orchestrator")
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

        let limited = store
            .list_events(EventQuery {
                limit: Some(2),
                ..Default::default()
            })
            .await
            .expect("list limited");
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].seq, 1);

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
    async fn unread_count_for_human_inbox() {
        let (store, _tmp) = store().await;
        assert_eq!(store.unread_count("human").await.expect("unread"), 0);

        store
            .insert_message(NewMessage {
                from: "agent:w1".to_string(),
                to: "human".to_string(),
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
}
