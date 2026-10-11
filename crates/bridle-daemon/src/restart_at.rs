//! Scheduled restart of an interactive session at a clock time (ticket cbbn). Each tick reads the
//! machine config's `[[restart]]` entries for this project and, once a day at `at` on its
//! `days`, runs the normal handover and restart ([`Sessions::scheduled_restart`]): forced, with
//! no deferring when the human is active; no note by the handover deadline restarts anyway.
//! Every firing writes `session.restart.completed` or `.skipped`. A daemon that was down at
//! the time does not catch up: an entry fires only within [`WINDOW`] after `at`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bridle_api::types::event_kind;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use serde_json::json;

use crate::config::{RestartEntry, restart_entries};
use crate::events::Emitter;
use crate::sessions::{AIDE, Sessions, insert_note, session_identity};
use crate::store::Store;

/// How long after `at` an entry may still fire (a tick late, a slow predecessor).
const WINDOW: chrono::Duration = chrono::Duration::minutes(10);

pub struct RestartAt {
    home: PathBuf,
    project: String,
    sessions: Arc<Sessions>,
    emitter: Emitter,
    store: Store,
    /// The day each entry (by project, session, time) last fired.
    fired: Mutex<HashMap<String, NaiveDate>>,
}

impl RestartAt {
    pub fn new(
        home: PathBuf,
        project: String,
        sessions: Arc<Sessions>,
        emitter: Emitter,
        store: Store,
    ) -> Self {
        RestartAt {
            home,
            project,
            sessions,
            emitter,
            store,
            fired: Mutex::default(),
        }
    }

    /// Fires what is due at host-local `now`. Runs the restarts inline, so a slow handover
    /// delays the next tick, not anything else.
    pub async fn tick(&self, now: DateTime<Local>) {
        let entries = match restart_entries(&self.home) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("[[restart]] ignored: {e}");
                return;
            }
        };
        for entry in entries.iter().filter(|e| e.project == self.project) {
            if self.due(entry, now) {
                self.fire(entry).await;
            }
        }
    }

    /// Whether `entry` is due, and marks it fired for today if so.
    fn due(&self, entry: &RestartEntry, now: DateTime<Local>) -> bool {
        let today = now.date_naive();
        let since = now.time().signed_duration_since(entry.at);
        if !entry.days.contains(&now.weekday())
            || since < chrono::Duration::zero()
            || since >= WINDOW
        {
            return false;
        }
        let key = format!("{}/{}@{}", entry.project, entry.session, entry.at);
        self.fired
            .lock()
            .expect("restart lock")
            .insert(key, today)
            .is_none_or(|d| d != today)
    }

    async fn fire(&self, entry: &RestartEntry) {
        let identity = session_identity(&entry.session);
        let at = entry.at.format("%H:%M").to_string();
        let data = |extra: serde_json::Value| {
            let mut d = json!({ "project": entry.project, "session": entry.session, "at": at });
            d.as_object_mut()
                .expect("object")
                .extend(extra.as_object().cloned().unwrap_or_default());
            d
        };
        let running = self.sessions.list().iter().any(|s| s.identity == identity);
        if !running {
            tracing::info!("scheduled restart of {identity} at {at}: not running, skipped");
            self.emit(
                event_kind::SESSION_RESTART_SKIPPED,
                data(json!({ "reason": "not running" })),
            )
            .await;
            return;
        }
        let reason = request_reason(entry);
        let Some((note, out)) = self.sessions.scheduled_restart(&identity, &reason).await else {
            tracing::warn!("scheduled restart of {identity}: nothing to restart it with");
            return;
        };
        if note {
            tracing::info!("scheduled restart of {identity} at {at}: {out}");
        } else {
            let body = format!(
                "Scheduled restart of {identity} at {at}: no handover note by the deadline, \
                 restarted without one. {out}"
            );
            tracing::warn!("{body}");
            insert_note(&self.store, &self.emitter, AIDE, body).await;
        }
        self.emit(
            event_kind::SESSION_RESTART_COMPLETED,
            data(json!({ "note": note })),
        )
        .await;
    }

    async fn emit(&self, kind: &str, data: serde_json::Value) {
        let _ = self
            .emitter
            .emit(kind, "system".to_string(), None, data)
            .await;
    }
}

/// The first sentence of the handover request: why, and the configured time.
fn request_reason(entry: &RestartEntry) -> String {
    let when = if entry.days.len() == 7 {
        "every day".to_string()
    } else {
        let days: Vec<String> = entry
            .days
            .iter()
            .map(|d| d.to_string().to_lowercase())
            .collect();
        format!("on {}", days.join(", "))
    };
    format!(
        "The human has configured your session to hand over at {} {when}; it is time to hand over.",
        entry.at.format("%-I:%M %p")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    struct Fixture {
        dir: tempfile::TempDir,
        store: Store,
        sessions: Arc<Sessions>,
        sched: RestartAt,
    }

    /// `body` is the stub `bridle`'s shell: it sees the arguments as `$*`.
    async fn fixture(config: &str, stub_body: &str, register: bool) -> Fixture {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("config.toml"), config).expect("config");
        let store = Store::open(dir.path().join("bridle.db"))
            .await
            .expect("store");
        let emitter = Emitter::new(store.clone());
        let stub = dir.path().join("stub.sh");
        std::fs::write(
            &stub,
            format!(
                "#!/bin/sh\necho \"$@\" >> {}\n{stub_body}\n",
                dir.path().join("calls").display()
            ),
        )
        .expect("stub");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        let sessions = Sessions::new(
            dir.path().to_path_buf(),
            dir.path().join("sessions.json"),
            Default::default(),
            emitter.clone(),
            store.clone(),
        )
        .with_restart(stub, dir.path().to_path_buf());
        let sessions = Arc::new(sessions);
        if register {
            sessions.register(
                bridle_api::types::SessionRegister {
                    pid: std::process::id() as i32,
                    pid_start: crate::containment::start_time(std::process::id() as i32)
                        .expect("own start time"),
                    identity: "advisor/alice".into(),
                    project: Some("notes".into()),
                    pane: None,
                    claude_session_id: None,
                    machine: None,
                },
                chrono::Utc::now(),
            );
        }
        let sched = RestartAt::new(
            dir.path().to_path_buf(),
            "notes".into(),
            sessions.clone(),
            emitter,
            store.clone(),
        );
        Fixture {
            dir,
            store,
            sessions,
            sched,
        }
    }

    const CONFIG: &str = r#"
        [[restart]]
        project = "notes"
        session = "advisor/alice"
        at = "04:00"
        days = ["mon", "tue"]
        [[restart]]
        project = "other"
        session = "advisor/alice"
        at = "04:00"
    "#;
    const NOTE_OK: &str = "echo restarted";
    const NO_NOTE: &str =
        "case \"$*\" in *--fresh*) echo restarted;; *) echo no note >&2; exit 1;; esac";

    /// 2026-10-12 is a Monday; 2026-10-14 a Wednesday.
    fn at(day: u32, h: u32, m: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 10, day, h, m, 0)
            .earliest()
            .expect("time")
    }

    async fn kinds(f: &Fixture, kind: &str) -> Vec<serde_json::Value> {
        let q = bridle_api::types::EventQuery {
            kind: Some(kind.into()),
            ..Default::default()
        };
        f.store
            .list_events(q)
            .await
            .expect("events")
            .into_iter()
            .map(|e| e.data)
            .collect()
    }

    fn calls(f: &Fixture) -> Vec<String> {
        std::fs::read_to_string(f.dir.path().join("calls"))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    #[tokio::test]
    async fn fires_once_at_the_time_with_the_reason_and_not_again_that_day() {
        let f = fixture(CONFIG, NOTE_OK, true).await;
        f.sched.tick(at(12, 3, 59)).await;
        assert!(calls(&f).is_empty(), "before the time");
        f.sched.tick(at(12, 4, 0)).await;
        f.sched.tick(at(12, 4, 5)).await;
        let c = calls(&f);
        assert_eq!(c.len(), 1, "{c:?}");
        assert!(c[0].starts_with("--project notes session restart advisor/alice --reason "));
        assert!(c[0].contains("hand over at 4:00 AM on mon, tue; it is time to hand over."));
        let done = kinds(&f, event_kind::SESSION_RESTART_COMPLETED).await;
        assert_eq!(done.len(), 1);
        assert_eq!(done[0]["note"], true);
        assert_eq!(done[0]["at"], "04:00");
        // The next scheduled day fires again.
        f.sched.tick(at(13, 4, 0)).await;
        assert_eq!(calls(&f).len(), 2);
    }

    #[tokio::test]
    async fn respects_days_and_does_not_catch_up_after_downtime() {
        let f = fixture(CONFIG, NOTE_OK, true).await;
        f.sched.tick(at(14, 4, 0)).await; // Wednesday
        f.sched.tick(at(12, 4, 30)).await; // Monday, but the daemon was down at 04:00
        assert!(calls(&f).is_empty(), "{:?}", calls(&f));
    }

    #[tokio::test]
    async fn restarts_anyway_when_no_note_comes_and_says_so() {
        let f = fixture(CONFIG, NO_NOTE, true).await;
        f.sched.tick(at(12, 4, 0)).await;
        let c = calls(&f);
        assert_eq!(c.len(), 2, "{c:?}");
        assert!(c[1].ends_with("--fresh"));
        let done = kinds(&f, event_kind::SESSION_RESTART_COMPLETED).await;
        assert_eq!(done[0]["note"], false);
        let q = crate::store::ListMessages {
            to: Some(AIDE.into()),
            ..Default::default()
        };
        let notes = f.store.list_messages(q).await.expect("notes");
        assert!(notes.iter().any(|m| m.body.contains("no handover note")));
    }

    #[tokio::test]
    async fn skips_a_session_that_is_not_running_and_says_so() {
        let f = fixture(CONFIG, NOTE_OK, false).await;
        f.sched.tick(at(12, 4, 0)).await;
        assert!(calls(&f).is_empty());
        let skipped = kinds(&f, event_kind::SESSION_RESTART_SKIPPED).await;
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0]["reason"], "not running");
        assert!(
            kinds(&f, event_kind::SESSION_RESTART_COMPLETED)
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    async fn no_entry_does_nothing() {
        let f = fixture("", NOTE_OK, true).await;
        f.sched.tick(at(12, 4, 0)).await;
        assert!(calls(&f).is_empty());
        assert!(f.sessions.has_restarter());
    }

    #[test]
    fn a_bad_time_is_a_config_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("config.toml"),
            "[[restart]]\nproject = \"notes\"\nsession = \"advisor\"\nat = \"4am\"\n",
        )
        .expect("config");
        let err = restart_entries(dir.path()).expect_err("bad time");
        assert!(err.to_string().contains("HH:MM"), "{err}");
    }
}
