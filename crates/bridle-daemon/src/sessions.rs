//! Interactive sessions (advisors) registered by `bridle session advisor` (ticket jttf): the
//! daemon knows each running one, reads its context file and reports tokens per session.
//! In memory only: a session re-registers when its launcher restarts, and the daemon
//! forgetting across its own restart costs one missing status line until the next hook.
//!
//! The steps are `[sessions] warn` (150k, 200k, 250k, 300k): warn, plan a handover, the normal
//! ceiling, the hard limit. Reaching a step emits `session.context` and messages the session and
//! the human (through `external:aide`), once until the reading drops (/compact, /clear); a
//! reading that jumps several steps announces only the highest. The human's override
//! ([`Sessions::keep`]) is recorded and the next step asks again; the hard limit has none: it
//! runs `bridle session restart` (a handover first, a fresh start if the note never comes).

use std::path::PathBuf;
use std::sync::Mutex;

use bridle_api::types::{
    MessageKind, MessageState, SessionEnd, SessionInfo, SessionRegister, When, event_kind,
};
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::config::{SessionSteps, SessionsConfig};
use crate::events::Emitter;
use crate::store::{NewMessage, RecipientKind, Store};

/// The principal the human's warnings go to.
const AIDE: &str = "external:aide";

/// The shared advisor principal: where a named advisor's mail goes when it isn't running.
pub const ADVISOR: &str = "external:advisor";

/// `body` marked as meant for the named advisor, for the shared inbox.
pub fn originally_for(name: &str, body: &str) -> String {
    format!("(originally for advisor/{name})\n{body}")
}

struct Entry {
    info: SessionInfo,
    pid_start: String,
    fired: [bool; 4],
    /// The human said to carry on past the last step that fired.
    kept: bool,
}

/// How the hard limit restarts a session: this program with `[--project p] session restart ...`,
/// run in `cwd` (the daemon's own binary, in tests a stub).
struct Restarter {
    program: PathBuf,
    cwd: PathBuf,
}

pub struct Sessions {
    home: PathBuf,
    config: SessionsConfig,
    emitter: Emitter,
    store: Store,
    restarter: Option<std::sync::Arc<Restarter>>,
    entries: Mutex<Vec<Entry>>,
}

/// A step a session just reached.
struct Reached {
    identity: String,
    project: Option<String>,
    session: String,
    tokens: u64,
    threshold: u64,
    step: usize,
    kept: bool,
}

impl Sessions {
    pub fn new(home: PathBuf, config: SessionsConfig, emitter: Emitter, store: Store) -> Self {
        Sessions {
            home,
            config,
            emitter,
            store,
            restarter: None,
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Lets the hard limit restart sessions with `program` (run in `cwd`).
    pub fn with_restart(mut self, program: PathBuf, cwd: PathBuf) -> Self {
        self.restarter = Some(std::sync::Arc::new(Restarter { program, cwd }));
        self
    }

    fn steps(&self, identity: &str) -> SessionSteps {
        self.config.steps_for(identity)
    }

    fn handover_note(&self, identity: &str) -> PathBuf {
        self.home
            .join("handover")
            .join(format!("{}.md", identity.replace('/', "-")))
    }

    /// Registers the session, or fills in what the later call knows for the same pid.
    pub fn register(&self, req: SessionRegister, now: DateTime<Utc>) -> SessionInfo {
        let mut entries = self.entries.lock().expect("sessions lock");
        // A different start time is a reused pid: the old entry is stale.
        entries.retain(|e| e.info.pid != req.pid || e.pid_start == req.pid_start);
        if let Some(e) = entries.iter_mut().find(|e| e.info.pid == req.pid) {
            e.info.pane = req.pane.or(e.info.pane.take());
            e.info.project = req.project.or(e.info.project.take());
            e.info.machine = req.machine.or(e.info.machine.take());
            if req.claude_session_id.is_some() && req.claude_session_id != e.info.claude_session_id
            {
                e.info.claude_session_id = req.claude_session_id;
                e.info.tokens = None;
                e.fired = [false; 4];
                e.kept = false;
            }
            return e.info.clone();
        }
        let info = SessionInfo {
            identity: req.identity,
            pid: req.pid,
            pane: req.pane,
            claude_session_id: req.claude_session_id,
            started_at: now,
            tokens: None,
            project: req.project,
            machine: req.machine,
            last_activity: None,
        };
        entries.push(Entry {
            info: info.clone(),
            pid_start: req.pid_start,
            fired: [false; 4],
            kept: false,
        });
        info
    }

    pub async fn end(&self, req: SessionEnd) {
        let gone = {
            let mut entries = self.entries.lock().expect("sessions lock");
            let at = entries.iter().position(|e| e.info.pid == req.pid);
            at.map(|i| entries.remove(i))
        };
        if let Some(e) = gone {
            self.emit_ended(&e).await;
        }
    }

    /// Whether a session named `advisor/<name>` is registered.
    pub fn is_running(&self, name: &str) -> bool {
        let identity = format!("advisor/{name}");
        let entries = self.entries.lock().expect("sessions lock");
        entries.iter().any(|e| e.info.identity == identity)
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        let entries = self.entries.lock().expect("sessions lock");
        entries.iter().map(|e| e.info.clone()).collect()
    }

    /// Ends sessions whose pid is gone, reads the others' context files and announces
    /// thresholds crossed.
    pub async fn tick(&self) {
        let snapshot: Vec<(i32, String)> = {
            let entries = self.entries.lock().expect("sessions lock");
            entries
                .iter()
                .map(|e| (e.info.pid, e.pid_start.clone()))
                .collect()
        };
        for (pid, start) in snapshot {
            let alive = tokio::task::spawn_blocking(move || {
                crate::containment::is_same_process(pid, &start)
            })
            .await
            .unwrap_or(true);
            if !alive {
                self.end(SessionEnd { pid }).await;
            }
        }
        let mut reached = Vec::new();
        {
            let mut entries = self.entries.lock().expect("sessions lock");
            for e in entries.iter_mut() {
                let Some(id) = e.info.claude_session_id.as_deref() else {
                    continue;
                };
                // Ids come from a hook's JSON: refuse anything that could leave the directory.
                if id.is_empty()
                    || !id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    continue;
                }
                let file = self.home.join("context").join(id);
                let Some(tokens) = std::fs::read_to_string(&file)
                    .ok()
                    .and_then(|t| t.trim().parse::<u64>().ok())
                else {
                    continue;
                };
                e.info.last_activity = std::fs::metadata(&file)
                    .and_then(|m| m.modified())
                    .ok()
                    .map(DateTime::<Utc>::from);
                if e.info.tokens.is_some_and(|last| tokens < last) {
                    e.fired = [false; 4]; // /compact
                    e.kept = false;
                }
                e.info.tokens = Some(tokens);
                let steps = self.steps(&e.info.identity);
                let Some(step) = (0..4).rev().find(|&i| tokens >= steps[i]) else {
                    continue;
                };
                if e.fired[step] {
                    continue;
                }
                for f in &mut e.fired[..=step] {
                    *f = true;
                }
                reached.push(Reached {
                    identity: e.info.identity.clone(),
                    project: e.info.project.clone(),
                    session: id.to_string(),
                    tokens,
                    threshold: steps[step],
                    step,
                    kept: std::mem::replace(&mut e.kept, false),
                });
            }
        }
        for r in reached {
            let _ = self
                .emitter
                .emit(
                    event_kind::SESSION_CONTEXT,
                    "system".to_string(),
                    None,
                    json!({
                        "identity": r.identity,
                        "session": r.session,
                        "tokens": r.tokens,
                        "threshold": r.threshold,
                        "step": r.step,
                    }),
                )
                .await;
            self.warn(&r).await;
        }
    }

    /// The human's override: carry on past the step the session is at. Recorded as an event and
    /// told to the session; the next step asks again. There is none at the hard limit.
    pub async fn keep(&self, identity: &str) -> Result<(), String> {
        let (tokens, step) = {
            let mut entries = self.entries.lock().expect("sessions lock");
            let e = entries
                .iter_mut()
                .find(|e| e.info.identity == identity)
                .ok_or_else(|| format!("no running session {identity:?} (see `bridle status`)"))?;
            if e.fired[3] {
                return Err(
                    "at the hard limit the handover is forced; there is no override".into(),
                );
            }
            let step = e.fired.iter().rposition(|f| *f).ok_or_else(|| {
                format!("{identity} hasn't reached a warning step yet; nothing to override")
            })?;
            e.kept = true;
            (e.info.tokens.unwrap_or_default(), step)
        };
        let _ = self
            .emitter
            .emit(
                event_kind::SESSION_OVERRIDE,
                "system".to_string(),
                None,
                json!({ "identity": identity, "tokens": tokens, "step": step }),
            )
            .await;
        let hard = self.steps(identity)[3] / 1000;
        self.tell(
            &format!("external:{identity}"),
            format!(
                "The human says to carry on: no handover now. You'll be asked again at the next \
                 step; at {hard}k the handover is forced."
            ),
        )
        .await;
        Ok(())
    }

    /// The message to the session and to the human for a step reached; the hard limit then
    /// restarts the session.
    async fn warn(&self, r: &Reached) {
        let k = r.tokens / 1000;
        let steps = self.steps(&r.identity);
        let (next, hard) = (steps.get(r.step + 1).map(|t| t / 1000), steps[3] / 1000);
        let id = &r.identity;
        let note = self.handover_note(id).display().to_string();
        let kept = if r.kept {
            " (You carried on at the last step; asking again.)"
        } else {
            ""
        };
        let (session, human) = match r.step {
            0 => (
                format!(
                    "Your context is {k}k tokens. Nothing to do yet; the human has been told, and \
                     you'll be told again at {}k.",
                    next.unwrap_or(hard)
                ),
                format!(
                    "{id} is at {k}k tokens of context. `bridle session restart {id} --fresh` \
                     restarts it clean; `--handover` asks for a note first."
                ),
            ),
            1 => (
                format!(
                    "Your context is {k}k tokens. Plan a handover at the next quiet point: write \
                     a short note (what you were doing, open threads, what the next session \
                     needs) to {note}. The human may override this.{kept}"
                ),
                format!(
                    "{id} is at {k}k tokens and will plan a handover. Carry on instead with \
                     `bridle session keep {id}`; restart now with `bridle session restart {id}` \
                     (`--fresh` for no handover).{kept}"
                ),
            ),
            2 => (
                format!(
                    "Your context is {k}k tokens: the normal ceiling. Hand over (write {note}) or \
                     shut down, unless the human overrides. At {hard}k the handover is forced.{kept}"
                ),
                format!(
                    "{id} is at {k}k tokens, the normal ceiling: it will hand over or shut down. \
                     Override with `bridle session keep {id}` (forced at {hard}k), or restart now \
                     with `bridle session restart {id}` (`--fresh` for no handover).{kept}"
                ),
            ),
            _ => (
                format!(
                    "Your context is {k}k tokens: the hard limit. Write your handover note to \
                     {note} now; the session is restarted when it appears, and in any case \
                     within minutes. No override."
                ),
                format!("{id} is at {k}k tokens, the hard limit: it is being restarted."),
            ),
        };
        // Aide's own session is the human's channel: one message, not two.
        if id == "aide" {
            self.tell(AIDE, format!("{session} {human}")).await;
        } else {
            self.tell(&format!("external:{id}"), session).await;
            self.tell(AIDE, human).await;
        }
        if r.step == 3 {
            self.force_restart(r);
        }
    }

    /// The hard limit: `bridle session restart <id>` (a handover first); if the note never comes,
    /// again with `--fresh`. The outcome goes to the human.
    fn force_restart(&self, r: &Reached) {
        let Some(rs) = self.restarter.clone() else {
            return;
        };
        let (store, emitter) = (self.store.clone(), self.emitter.clone());
        let (identity, project) = (r.identity.clone(), r.project.clone());
        tokio::spawn(async move {
            let mut out = String::new();
            for fresh in [false, true] {
                let mut cmd = tokio::process::Command::new(&rs.program);
                if let Some(p) = &project {
                    cmd.args(["--project", p]);
                }
                cmd.args(["session", "restart", &identity])
                    .current_dir(&rs.cwd)
                    .env_remove("BRIDLE_AGENT_ID")
                    .env_remove("BRIDLE_AS");
                if fresh {
                    cmd.arg("--fresh");
                }
                match cmd.output().await {
                    Ok(o) if o.status.success() => {
                        out = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        break;
                    }
                    Ok(o) => out = String::from_utf8_lossy(&o.stderr).trim().to_string(),
                    Err(e) => out = e.to_string(),
                }
            }
            let body = format!("Hard-limit restart of {identity}: {out}");
            insert_note(&store, &emitter, AIDE, body).await;
        });
    }

    async fn tell(&self, to: &str, body: String) {
        insert_note(&self.store, &self.emitter, to, body).await;
    }

    async fn emit_ended(&self, e: &Entry) {
        // Mail it hadn't read goes to the shared inbox, marked, so someone sees it.
        if let Some(name) = e.info.identity.strip_prefix("advisor/") {
            let from = format!("{ADVISOR}/{name}");
            let mark = originally_for(name, "");
            if let Err(err) = self.store.move_unread_messages(&from, ADVISOR, &mark).await {
                tracing::warn!("moving {from}'s unread messages: {err}");
            }
        }
        let _ = self
            .emitter
            .emit(
                event_kind::SESSION_ENDED,
                "system".to_string(),
                None,
                json!({ "identity": e.info.identity, "pid": e.info.pid }),
            )
            .await;
    }
}

/// A note from the system to an external principal, delivered at once.
async fn insert_note(store: &Store, emitter: &Emitter, to: &str, body: String) {
    let m = store
        .insert_message(NewMessage {
            from: "system".into(),
            to: to.into(),
            to_kind: RecipientKind::External,
            kind: MessageKind::Note,
            body,
            reply_to: None,
            when: When::Now,
            state: MessageState::Written,
        })
        .await;
    match m {
        Ok(m) => {
            let _ = emitter
                .emit(
                    event_kind::MESSAGE_SENT,
                    "system".to_string(),
                    None,
                    json!({ "message": m.id, "to": to }),
                )
                .await;
        }
        Err(e) => tracing::warn!("session warning to {to}: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    async fn fixture() -> (Sessions, tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(dir.path().join("bridle.db"))
            .await
            .expect("store");
        let s = Sessions::new(
            dir.path().to_path_buf(),
            SessionsConfig {
                warn: [100, 200, 300, 400],
                ..Default::default()
            },
            Emitter::new(store.clone()),
            store.clone(),
        );
        (s, dir, store)
    }

    fn reg(pid: i32, start: &str, sid: Option<&str>) -> SessionRegister {
        SessionRegister {
            identity: "advisor/alice".into(),
            pid,
            pid_start: start.into(),
            pane: Some("%3".into()),
            claude_session_id: sid.map(Into::into),
            project: Some("bridle".into()),
            machine: Some("nuc".into()),
        }
    }

    fn write_context(dir: &tempfile::TempDir, id: &str, tokens: u64) {
        let d = dir.path().join("context");
        std::fs::create_dir_all(&d).expect("mkdir");
        std::fs::write(d.join(id), format!("{tokens}\n")).expect("write");
    }

    #[tokio::test]
    async fn register_update_and_end_round_trip() {
        let (s, _dir, _store) = fixture().await;
        s.register(reg(42, "t0", None), Utc::now());
        let info = s.register(reg(42, "t0", Some("sess-1")), Utc::now());
        assert_eq!(info.claude_session_id.as_deref(), Some("sess-1"));
        assert_eq!(info.pane.as_deref(), Some("%3"));
        assert_eq!(s.list().len(), 1);
        s.end(SessionEnd { pid: 42 }).await;
        assert!(s.list().is_empty());
    }

    async fn events(store: &Store, kind: &str) -> usize {
        let q = bridle_api::types::EventQuery {
            kind: Some(kind.into()),
            ..Default::default()
        };
        store.list_events(q).await.expect("events").len()
    }

    async fn notes_to(store: &Store, to: &str) -> Vec<String> {
        let q = crate::store::ListMessages {
            to: Some(to.into()),
            ..Default::default()
        };
        let m = store.list_messages(q).await.expect("messages");
        m.into_iter().map(|m| m.body).collect()
    }

    fn register_self(s: &Sessions) {
        s.register(
            reg(std::process::id() as i32, &real_start(), Some("sess-1")),
            Utc::now(),
        );
    }

    #[tokio::test]
    async fn each_step_fires_once_warns_both_and_resets_on_a_lower_reading() {
        let (s, dir, store) = fixture().await;
        register_self(&s);
        let count = || events(&store, event_kind::SESSION_CONTEXT);
        write_context(&dir, "sess-1", 50);
        s.tick().await;
        assert_eq!(count().await, 0);
        // A jump over two steps announces only the highest.
        write_context(&dir, "sess-1", 300);
        s.tick().await;
        s.tick().await;
        assert_eq!(count().await, 1);
        assert_eq!(s.list()[0].tokens, Some(300));
        assert!(s.list()[0].last_activity.is_some());
        let to_session = notes_to(&store, "external:advisor/alice").await;
        let to_human = notes_to(&store, AIDE).await;
        assert_eq!((to_session.len(), to_human.len()), (1, 1));
        assert!(to_session[0].contains("normal ceiling"), "{to_session:?}");
        assert!(to_human[0].contains("bridle session keep advisor/alice"));
        // The next step asks again; /compact resets.
        write_context(&dir, "sess-1", 10);
        s.tick().await;
        write_context(&dir, "sess-1", 100);
        s.tick().await;
        assert_eq!(count().await, 2);
        assert!(notes_to(&store, AIDE).await[1].contains("restart"));
    }

    #[tokio::test]
    async fn an_override_is_recorded_and_the_next_step_asks_again() {
        let (s, dir, store) = fixture().await;
        register_self(&s);
        assert!(
            s.keep("advisor/alice").await.is_err(),
            "no step reached yet"
        );
        assert!(s.keep("advisor/bob").await.is_err(), "no such session");
        write_context(&dir, "sess-1", 200);
        s.tick().await;
        s.keep("advisor/alice").await.expect("keep");
        assert_eq!(events(&store, event_kind::SESSION_OVERRIDE).await, 1);
        let to_session = notes_to(&store, "external:advisor/alice").await;
        assert!(to_session[1].contains("carry on"), "{to_session:?}");
        write_context(&dir, "sess-1", 300);
        s.tick().await;
        let to_session = notes_to(&store, "external:advisor/alice").await;
        assert!(to_session[2].contains("You carried on"), "{to_session:?}");
    }

    #[tokio::test]
    async fn the_hard_limit_has_no_override_and_forces_a_restart() {
        let (s, dir, store) = fixture().await;
        // A stub `bridle`: records its arguments; the handover attempt fails, so --fresh follows.
        let log = dir.path().join("calls");
        let stub = dir.path().join("stub.sh");
        std::fs::write(
            &stub,
            format!(
                "#!/bin/sh\necho \"$@\" >> {}\ncase \"$*\" in *--fresh*) echo restarted;; \
                 *) echo no note >&2; exit 1;; esac\n",
                log.display()
            ),
        )
        .expect("stub");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        let s = s.with_restart(stub, dir.path().to_path_buf());
        register_self(&s);
        write_context(&dir, "sess-1", 400);
        s.tick().await;
        assert!(s.keep("advisor/alice").await.is_err());
        let to_session = notes_to(&store, "external:advisor/alice").await;
        assert!(to_session[0].contains("hard limit"), "{to_session:?}");
        for _ in 0..50 {
            if notes_to(&store, AIDE).await.len() == 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let calls = std::fs::read_to_string(&log).expect("calls");
        assert_eq!(
            calls.lines().collect::<Vec<_>>(),
            [
                "--project bridle session restart advisor/alice",
                "--project bridle session restart advisor/alice --fresh"
            ]
        );
        assert!(
            notes_to(&store, AIDE).await[1].contains("restarted"),
            "{:?}",
            notes_to(&store, AIDE).await
        );
    }

    #[tokio::test]
    async fn a_session_whose_pid_is_gone_is_ended() {
        let (s, _dir, _store) = fixture().await;
        s.register(
            reg(std::process::id() as i32, "not its start time", None),
            Utc::now(),
        );
        s.tick().await;
        assert!(s.list().is_empty());
    }

    fn real_start() -> String {
        crate::containment::start_time(std::process::id() as i32).expect("own start time")
    }
}
