//! Interactive sessions (advisors) registered by `bridle session advisor` (ticket jttf): the
//! daemon knows each running one, reads its context file and reports tokens per session.
//! In memory only: a session re-registers when its launcher restarts, and the daemon
//! forgetting across its own restart costs one missing status line until the next hook.
//!
//! The thresholds are the orchestrator's (`[orchestrator] note_tokens, plan_tokens,
//! handover_tokens`); crossing one emits `session.context`, once until the reading drops
//! (/compact, /clear).

use std::path::PathBuf;
use std::sync::Mutex;

use bridle_api::types::{SessionEnd, SessionInfo, SessionRegister, event_kind};
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::events::Emitter;

struct Entry {
    info: SessionInfo,
    pid_start: String,
    fired: [bool; 3],
}

pub struct Sessions {
    home: PathBuf,
    tokens: [u64; 3],
    emitter: Emitter,
    entries: Mutex<Vec<Entry>>,
}

impl Sessions {
    pub fn new(home: PathBuf, tokens: [u64; 3], emitter: Emitter) -> Self {
        Sessions {
            home,
            tokens,
            emitter,
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Registers the session, or fills in what the later call knows for the same pid.
    pub fn register(&self, req: SessionRegister, now: DateTime<Utc>) -> SessionInfo {
        let mut entries = self.entries.lock().expect("sessions lock");
        // A different start time is a reused pid: the old entry is stale.
        entries.retain(|e| e.info.pid != req.pid || e.pid_start == req.pid_start);
        if let Some(e) = entries.iter_mut().find(|e| e.info.pid == req.pid) {
            e.info.pane = req.pane.or(e.info.pane.take());
            if req.claude_session_id.is_some() && req.claude_session_id != e.info.claude_session_id
            {
                e.info.claude_session_id = req.claude_session_id;
                e.info.tokens = None;
                e.fired = [false; 3];
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
        };
        entries.push(Entry {
            info: info.clone(),
            pid_start: req.pid_start,
            fired: [false; 3],
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
        let mut crossed = Vec::new();
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
                let Some(tokens) = std::fs::read_to_string(self.home.join("context").join(id))
                    .ok()
                    .and_then(|t| t.trim().parse::<u64>().ok())
                else {
                    continue;
                };
                if e.info.tokens.is_some_and(|last| tokens < last) {
                    e.fired = [false; 3]; // /compact
                }
                e.info.tokens = Some(tokens);
                for i in 0..3 {
                    if tokens >= self.tokens[i] && !e.fired[i] {
                        e.fired[i] = true;
                        crossed.push((
                            e.info.identity.clone(),
                            id.to_string(),
                            tokens,
                            self.tokens[i],
                        ));
                    }
                }
            }
        }
        for (identity, session, tokens, threshold) in crossed {
            let _ = self
                .emitter
                .emit(
                    event_kind::SESSION_CONTEXT,
                    "system".to_string(),
                    None,
                    json!({
                        "identity": identity,
                        "session": session,
                        "tokens": tokens,
                        "threshold": threshold,
                    }),
                )
                .await;
        }
    }

    async fn emit_ended(&self, e: &Entry) {
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
            [100, 200, 300],
            Emitter::new(store.clone()),
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

    #[tokio::test]
    async fn context_events_fire_once_per_threshold_and_reset_on_a_lower_reading() {
        let (s, dir, store) = fixture().await;
        s.register(
            reg(std::process::id() as i32, &real_start(), Some("sess-1")),
            Utc::now(),
        );
        let count = || async {
            let q = bridle_api::types::EventQuery {
                kind: Some(event_kind::SESSION_CONTEXT.into()),
                ..Default::default()
            };
            store.list_events(q).await.expect("events").len()
        };
        write_context(&dir, "sess-1", 50);
        s.tick().await;
        assert_eq!(count().await, 0);
        write_context(&dir, "sess-1", 250);
        s.tick().await;
        s.tick().await;
        assert_eq!(count().await, 2);
        assert_eq!(s.list()[0].tokens, Some(250));
        write_context(&dir, "sess-1", 10);
        s.tick().await;
        write_context(&dir, "sess-1", 150);
        s.tick().await;
        assert_eq!(count().await, 3);
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
