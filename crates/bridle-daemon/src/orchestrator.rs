//! The orchestrator supervisor: keeps the human's interactive `claude` running in its tmux
//! pane by relaunching `bridle session orchestrator` when it dies, with crash-loop backoff
//! (docs/design/agent-host/orchestrator-supervision.md, sections 3 and 4; spike 07).
//!
//! The pane is the human's: the supervisor types into it only when the recorded process is
//! dead and the pane's foreground command is a shell on two checks 5 s apart. tmux, process
//! liveness and the incident sink sit behind traits so tests drive a fake clock and fakes.

use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use bridle_api::types::{MessageKind, WakeReason, When, event_kind};
use chrono::{DateTime, Utc};
use serde_json::json;
use tokio::sync::Mutex;

use crate::config::OrchestratorConfig;
use crate::events::Emitter;
use crate::supervisor::{AgentManager, ToTarget};
use crate::wake::{Waiters, Wakes};

pub const TICK_INTERVAL: Duration = Duration::from_secs(10);
/// The tag the human sets on the orchestrator's pane: `tmux set -p @bridle orchestrator`.
const PANE_TAG: &str = "orchestrator";
const SHELLS: &[&str] = &["zsh", "bash", "sh", "fish"];
/// The pane must show a shell on two checks at least this far apart.
const SHELL_RECHECK: Duration = Duration::from_secs(5);
/// A launch is confirmed when the pid file changes to a live pid within this long.
const CONFIRM_WITHIN: Duration = Duration::from_secs(60);
/// After SIGTERM, a process still alive this long gets SIGKILL.
const KILL_AFTER: Duration = Duration::from_secs(15);

pub trait Tmux: Send + Sync {
    /// `(pane id, @bridle tag)` for every pane; the tag is empty when unset.
    async fn panes(&self) -> Result<Vec<(String, String)>, String>;
    async fn pane_command(&self, pane: &str) -> Result<String, String>;
    /// Types `text` literally, then a separate Enter (spike 07 #7).
    async fn type_line(&self, pane: &str, text: &str) -> Result<(), String>;
}

pub trait Procs: Send + Sync {
    /// The pid exists and its start time matches: a reused pid is not the session.
    async fn is_alive(&self, pid: i32, start: &str) -> bool;
    /// SIGTERM, or SIGKILL when `kill`, to the pid if its start time still matches.
    async fn signal(&self, pid: i32, start: &str, kill: bool);
}

/// Where the supervisor's context and uptime notes go: the wake queue.
pub trait WakeSink: Send + Sync {
    async fn push(&self, wake: WakeReason);
}

/// The event emitter for the supervisor's context tracking.
pub trait EventEmitter: Send + Sync {
    async fn emit(
        &self,
        kind: &str,
        actor: String,
        agent: Option<String>,
        data: serde_json::Value,
    ) -> Result<(), String>;
}

impl WakeSink for Arc<Wakes> {
    async fn push(&self, wake: WakeReason) {
        Wakes::push(self, wake).await
    }
}

impl EventEmitter for Emitter {
    async fn emit(
        &self,
        kind: &str,
        actor: String,
        agent: Option<String>,
        data: serde_json::Value,
    ) -> Result<(), String> {
        Emitter::emit(self, kind, actor, agent, data)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

/// `bridle handover done`: when the orchestrator said it has written its state. Only a mark
/// made after the current session launched counts, so a stale one never stops the next session.
#[derive(Default)]
pub struct Handover(StdMutex<Option<DateTime<Utc>>>);

impl Handover {
    pub fn mark(&self, now: DateTime<Utc>) {
        *self.0.lock().expect("handover lock") = Some(now);
    }

    fn done_since(&self, launched: i64) -> bool {
        self.0
            .lock()
            .expect("handover lock")
            .is_some_and(|t| t.timestamp() >= launched)
    }
}

/// The interim "record an incident" (section 8): one function nc7r replaces.
pub trait Incidents: Send + Sync {
    async fn record(&self, text: &str);
}

/// `<pid> <process start time> <launch epoch>`; the start time has spaces (`ps lstart`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct PidRecord {
    pid: i32,
    start: String,
    launched: i64,
}

fn parse_pid_file(text: &str) -> Option<PidRecord> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    Some(PidRecord {
        pid: parts[0].parse().ok()?,
        launched: parts[parts.len() - 1].parse().ok()?,
        start: parts[1..parts.len() - 1].join(" "),
    })
}

/// One session's context notes: the last reading and which thresholds have been announced.
#[derive(Default)]
struct ContextNotes {
    session: String,
    last: u64,
    fired: [bool; 3],
    /// When the last context event was emitted.
    last_event: Option<DateTime<Utc>>,
}

#[derive(Default)]
struct State {
    warned_no_file: bool,
    /// Relaunches since the session was last stable.
    attempts: u32,
    last_attempt: Option<DateTime<Utc>>,
    /// The pid file as it was when we typed, and when: confirmation is it changing to a live pid.
    pending: Option<DateTime<Utc>>,
    given_up: bool,
    /// When the pane first showed a shell for the current relaunch.
    shell_seen: Option<DateTime<Utc>>,
    /// The death of this session has been reported.
    dead_noted: bool,
    /// No wake command has been running for longer than `waiter_grace`; reported once.
    waiter_noted: bool,
    /// The last incident text, so a condition that persists across ticks is reported once.
    last_incident: Option<String>,
    /// The launch epoch the per-session fields below belong to.
    session_key: i64,
    context: ContextNotes,
    uptime_fired: bool,
    /// When the session is stopped without a handover, once a "now" or uptime note went out.
    deadline: Option<DateTime<Utc>>,
    /// SIGTERM sent at, and SIGKILL sent.
    term_sent: Option<DateTime<Utc>>,
    kill_sent: bool,
    /// The stop was ours: the next relaunch is free (not a crash). `forced` = no handover.
    deliberate: Option<Stop>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stop {
    HandoverDone,
    Deadline,
}

pub struct Supervisor<T, P, I, W, E> {
    home: PathBuf,
    launcher: String,
    backoff: Vec<Duration>,
    stable_after: Duration,
    waiter_grace: Duration,
    tokens: [u64; 3],
    handover_deadline: Duration,
    max_uptime: Duration,
    waiters: Arc<Waiters>,
    handover: Arc<Handover>,
    tmux: T,
    procs: P,
    incidents: I,
    wakes: W,
    emitter: E,
    state: Mutex<State>,
}

impl<T: Tmux, P: Procs, I: Incidents, W: WakeSink, E: EventEmitter> Supervisor<T, P, I, W, E> {
    /// `launcher` is the command line to type into the pane (see `launch_line`).
    #[allow(clippy::too_many_arguments)] // the fakes in tests are why each is a parameter
    pub fn new(
        home: PathBuf,
        launcher: String,
        cfg: &OrchestratorConfig,
        waiters: Arc<Waiters>,
        handover: Arc<Handover>,
        tmux: T,
        procs: P,
        incidents: I,
        wakes: W,
        emitter: E,
    ) -> Self {
        Supervisor {
            home,
            launcher,
            backoff: cfg.relaunch_backoff.clone(),
            stable_after: cfg.stable_after,
            waiter_grace: cfg.waiter_grace,
            tokens: [cfg.note_tokens, cfg.plan_tokens, cfg.handover_tokens],
            handover_deadline: cfg.handover_deadline,
            max_uptime: cfg.max_uptime,
            waiters,
            handover,
            tmux,
            procs,
            incidents,
            wakes,
            emitter,
            state: Mutex::new(State::default()),
        }
    }

    pub async fn tick(&self, now: DateTime<Utc>) {
        let mut st = self.state.lock().await;
        let Some(rec) = std::fs::read_to_string(self.home.join("orchestrator.pid"))
            .ok()
            .and_then(|t| parse_pid_file(&t))
        else {
            // Never launch what the supervisor hasn't seen run.
            if !std::mem::replace(&mut st.warned_no_file, true) {
                tracing::info!("no orchestrator.pid: the orchestrator supervisor does nothing");
            }
            return;
        };
        st.warned_no_file = false;

        if self.procs.is_alive(rec.pid, &rec.start).await {
            let up = now.timestamp() - rec.launched;
            if up >= self.stable_after.as_secs() as i64 {
                st.attempts = 0;
            }
            st.given_up = false;
            st.pending = None;
            st.shell_seen = None;
            st.dead_noted = false;
            st.last_incident = None;
            self.check_waiter(&mut st, now, rec.launched).await;
            self.check_thresholds(&mut st, now, &rec).await;
            return;
        }
        st.waiter_noted = false;

        if !std::mem::replace(&mut st.dead_noted, true) {
            let text = match st.deliberate {
                Some(Stop::Deadline) => format!(
                    "The orchestrator was stopped at {}: the handover deadline passed with no `bridle handover done`. The new session starts from the previous handover or state file, which is stale. {}",
                    now.format("%H:%M:%S UTC"),
                    self.transcript_activity()
                ),
                Some(Stop::HandoverDone) => {
                    tracing::info!("orchestrator stopped after its handover; relaunching");
                    String::new()
                }
                None => format!(
                    "The orchestrator session (pid {}) is not running; found dead at {}. {}",
                    rec.pid,
                    now.format("%H:%M:%S UTC"),
                    self.transcript_activity()
                ),
            };
            if !text.is_empty() {
                self.report(&mut st, text).await;
            }
        }
        if let Some(at) = st.pending {
            if now - at < chrono_dur(CONFIRM_WITHIN) {
                return;
            }
            // Never confirmed (the trust prompt, a broken launcher): a failed relaunch.
            st.pending = None;
        }
        // A restart we caused is not a crash: no backoff, no count, and it can't give up.
        let free = st.deliberate.is_some();
        if st.given_up && !free {
            return;
        }
        if st.attempts as usize > self.backoff.len() && !free {
            st.given_up = true;
            let text = format!(
                "The orchestrator is not staying up: {} relaunches, last attempt {}. Not trying again until it is running.",
                st.attempts,
                st.last_attempt
                    .map(|t| t.format("%H:%M:%S UTC").to_string())
                    .unwrap_or_default()
            );
            self.report(&mut st, text).await;
            return;
        }
        // The first relaunch goes at once, then `backoff[0]`, `backoff[1]`, ...
        if !free
            && let (Some(last), Some(wait)) = (
                st.last_attempt,
                st.attempts.checked_sub(1).map(|i| self.backoff[i as usize]),
            )
            && now - last < chrono_dur(wait)
        {
            return;
        }

        let pane = match self.tmux.panes().await {
            Err(e) => {
                return self
                    .report(
                        &mut st,
                        format!("The orchestrator is down and tmux is unavailable: {e}"),
                    )
                    .await;
            }
            Ok(panes) => panes.into_iter().find(|(_, tag)| tag == PANE_TAG),
        };
        let Some((pane, _)) = pane else {
            let text = "The orchestrator is down and no pane is tagged @bridle=orchestrator (`tmux set -p @bridle orchestrator` in its pane).".to_string();
            return self.report(&mut st, text).await;
        };
        match self.tmux.pane_command(&pane).await {
            Err(e) => {
                return self
                    .report(
                        &mut st,
                        format!("The orchestrator is down and tmux failed: {e}"),
                    )
                    .await;
            }
            Ok(cmd) if !SHELLS.contains(&cmd.trim()) => {
                st.shell_seen = None;
                let text = format!(
                    "The orchestrator is down but pane {pane} is running {:?}, not a shell; not typing into it.",
                    cmd.trim()
                );
                return self.report(&mut st, text).await;
            }
            Ok(_) => {}
        }
        match st.shell_seen {
            None => {
                st.shell_seen = Some(now);
                return;
            }
            Some(t) if now - t < chrono_dur(SHELL_RECHECK) => return,
            Some(_) => {}
        }

        if free {
            // Only this first relaunch is free; if it fails, the ordinary rules apply.
            st.deliberate = None;
        } else {
            st.attempts += 1;
            st.last_attempt = Some(now);
        }
        st.shell_seen = None;
        tracing::info!(pane = %pane, attempt = st.attempts, free, "relaunching the orchestrator");
        match self.tmux.type_line(&pane, &self.launcher).await {
            Ok(()) => st.pending = Some(now),
            Err(e) => {
                self.report(&mut st, format!("Relaunching the orchestrator failed: {e}"))
                    .await
            }
        }
    }

    /// Emit a context event on the first reading, on a lower reading (compact), or after 10
    /// minutes if the reading changed.
    async fn emit_context_event(
        &self,
        notes: &mut ContextNotes,
        now: DateTime<Utc>,
        tokens: u64,
        launched: i64,
    ) {
        let should_emit = match notes.last_event {
            None => true, // first reading
            Some(t) => {
                if tokens < notes.last {
                    true // lower reading (compact)
                } else {
                    tokens != notes.last && now - t >= chrono_dur(Duration::from_secs(600))
                }
            }
        };
        if should_emit {
            notes.last_event = Some(now);
            let uptime = (now.timestamp() - launched).max(0) as u64;
            // The token thresholds (note_tokens, plan_tokens, handover_tokens) in the config
            // assume a 1M context window (orchestrator-supervision.md, section 2). The window
            // size is available per-session in Claude Code's statusline JSON, but we don't
            // currently extract it per-session; capturing it would require reading the transcript
            // or adding a separate window_size file. For now, report the assumed 1M.
            let window_size = 1_000_000u64;
            let _ = self
                .emitter
                .emit(
                    event_kind::ORCHESTRATOR_CONTEXT,
                    "system".to_string(),
                    None,
                    json!({
                        "session": notes.session,
                        "tokens": tokens,
                        "window_size": window_size,
                        "uptime_secs": uptime,
                    }),
                )
                .await;
        }
    }

    /// Context and uptime notes, and the restart (section 6).
    async fn check_thresholds(&self, st: &mut State, now: DateTime<Utc>, rec: &PidRecord) {
        if st.session_key != rec.launched {
            // A new process: nothing announced yet, no deadline, no stop under way.
            st.session_key = rec.launched;
            st.context = ContextNotes::default();
            st.uptime_fired = false;
            st.deadline = None;
            st.term_sent = None;
            st.kill_sent = false;
            st.deliberate = None;
        }

        if let Some(tokens) = self.context_tokens(st) {
            self.emit_context_event(&mut st.context, now, tokens, rec.launched)
                .await;
            let notes = &mut st.context;
            if tokens < notes.last {
                notes.fired = [false; 3]; // /compact
            }
            notes.last = tokens;
            let due: Vec<usize> = (0..3)
                .filter(|&i| tokens >= self.tokens[i] && !notes.fired[i])
                .collect();
            for i in due {
                st.context.fired[i] = true;
                if i == 2 && st.deadline.is_none() {
                    st.deadline = Some(now + chrono_dur(self.handover_deadline));
                }
                let text = match i {
                    0 => format!("context at {}K of the window", tokens / 1000),
                    1 => "plan a handover at the next quiet point".to_string(),
                    _ => format!(
                        "hand over now; the session is stopped at {}",
                        self.deadline_text(st)
                    ),
                };
                self.wake(
                    &text,
                    json!({ "tokens": tokens, "threshold": self.tokens[i] }),
                )
                .await;
            }
        }

        let up = Duration::from_secs((now.timestamp() - rec.launched).max(0) as u64);
        if up >= self.max_uptime && !std::mem::replace(&mut st.uptime_fired, true) {
            st.deadline
                .get_or_insert(now + chrono_dur(self.handover_deadline));
            let text = format!(
                "uptime {}h: plan a handover at the next quiet point; the session is stopped at {}",
                up.as_secs() / 3600,
                self.deadline_text(st)
            );
            self.wake(&text, json!({ "uptime_secs": up.as_secs() }))
                .await;
        }

        self.check_stop(st, now, rec).await;
    }

    fn deadline_text(&self, st: &State) -> String {
        st.deadline
            .map(|d| d.format("%H:%M:%S UTC").to_string())
            .unwrap_or_default()
    }

    async fn wake(&self, text: &str, detail: serde_json::Value) {
        self.wakes
            .push(WakeReason {
                reason: "context".to_string(),
                text: text.to_string(),
                detail,
            })
            .await;
    }

    /// The marker or the deadline stops the session: SIGTERM, then SIGKILL after 15 s. The
    /// relaunch is the dead-session path, flagged deliberate.
    async fn check_stop(&self, st: &mut State, now: DateTime<Utc>, rec: &PidRecord) {
        if st.deliberate.is_none() {
            let stop = if self.handover.done_since(rec.launched) {
                Stop::HandoverDone
            } else if st.deadline.is_some_and(|d| now >= d) {
                Stop::Deadline
            } else {
                return;
            };
            tracing::info!(
                pid = rec.pid,
                forced = stop == Stop::Deadline,
                "stopping the orchestrator"
            );
            st.deliberate = Some(stop);
            st.term_sent = Some(now);
            self.procs.signal(rec.pid, &rec.start, false).await;
        } else if let Some(t) = st.term_sent
            && !st.kill_sent
            && now - t >= chrono_dur(KILL_AFTER)
        {
            st.kill_sent = true;
            self.procs.signal(rec.pid, &rec.start, true).await;
        }
    }

    /// Context tokens of the current session: the statusline's file, else the transcript's last
    /// assistant usage when the file is missing or older than the transcript.
    fn context_tokens(&self, st: &mut State) -> Option<u64> {
        let text = std::fs::read_to_string(self.home.join("orchestrator.session")).ok()?;
        let (id, transcript) = match text.trim().split_once(' ') {
            Some((id, p)) => (id, Some(p.trim())),
            None => (text.trim(), None),
        };
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return None;
        }
        if st.context.session != id {
            // /clear: a new session id starts its own notes.
            st.context = ContextNotes {
                session: id.to_string(),
                ..Default::default()
            };
        }
        let file = self.home.join("context").join(id);
        let file_mtime = std::fs::metadata(&file).and_then(|m| m.modified()).ok();
        let transcript_mtime = transcript
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        let stale = match (file_mtime, transcript_mtime) {
            (None, _) => true,
            (Some(f), Some(t)) => f < t,
            _ => false,
        };
        let from_file = std::fs::read_to_string(&file)
            .ok()
            .and_then(|t| t.trim().parse().ok());
        if !stale && from_file.is_some() {
            return from_file;
        }
        transcript.and_then(transcript_tokens).or(from_file)
    }

    /// A live session with no wake command running (section 5): the human is told, the session
    /// is left alone. Closed when a request arrives.
    async fn check_waiter(&self, st: &mut State, now: DateTime<Utc>, launched: i64) {
        let floor = DateTime::from_timestamp(launched, 0).unwrap_or(now);
        match self.waiters.absent_since(now, self.waiter_grace, floor) {
            None => st.waiter_noted = false,
            Some(since) if !std::mem::replace(&mut st.waiter_noted, true) => {
                let text = format!(
                    "The orchestrator has no wake command running (`bridle wait-for-wake`) since {}.",
                    since.format("%H:%M:%S UTC")
                );
                self.incidents.record(&text).await;
            }
            Some(_) => {}
        }
    }

    async fn report(&self, st: &mut State, text: String) {
        if st.last_incident.as_deref() == Some(&text) {
            return;
        }
        self.incidents.record(&text).await;
        st.last_incident = Some(text);
    }

    /// When the transcript was last written, from `orchestrator.session` (`<id> <path>`).
    fn transcript_activity(&self) -> String {
        let path = std::fs::read_to_string(self.home.join("orchestrator.session"))
            .ok()
            .and_then(|t| t.split_once(' ').map(|(_, p)| p.trim().to_string()))
            .filter(|p| !p.is_empty());
        let mtime = path
            .as_deref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        match mtime {
            Some(t) => format!(
                "Its transcript was last written {}.",
                DateTime::<Utc>::from(t).format("%H:%M:%S UTC")
            ),
            None => "Its transcript's last activity is unknown.".to_string(),
        }
    }
}

fn chrono_dur(d: Duration) -> chrono::Duration {
    chrono::Duration::from_std(d).unwrap_or(chrono::Duration::MAX)
}

/// The line typed into the pane to relaunch. The pane's shell finds `bridle` on its PATH, as the
/// advisor relaunch (`bridle session advisor`) already assumes. A configured launcher is typed
/// verbatim.
pub fn launch_line(launcher: Option<&str>, project: &str) -> String {
    match launcher {
        Some(l) => l.to_string(),
        None => format!(
            "bridle session orchestrator --project {}",
            shell_word(project)
        ),
    }
}

/// A word as typed into the shell: itself unless it needs quoting.
fn shell_word(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "_./-+@:".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

pub struct RealTmux;

impl RealTmux {
    async fn run(args: &[&str]) -> Result<String, String> {
        let out = tokio::process::Command::new("tmux")
            .args(args)
            .output()
            .await
            .map_err(|e| format!("running tmux: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "tmux {} exited {}: {}",
                args[0],
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

impl Tmux for RealTmux {
    async fn panes(&self) -> Result<Vec<(String, String)>, String> {
        let out = Self::run(&["list-panes", "-a", "-F", "#{pane_id} #{@bridle}"]).await?;
        Ok(out
            .lines()
            .map(|l| match l.split_once(' ') {
                Some((id, tag)) => (id.to_string(), tag.trim().to_string()),
                None => (l.to_string(), String::new()),
            })
            .collect())
    }

    async fn pane_command(&self, pane: &str) -> Result<String, String> {
        Self::run(&[
            "display-message",
            "-p",
            "-t",
            pane,
            "#{pane_current_command}",
        ])
        .await
        .map(|s| s.trim().to_string())
    }

    async fn type_line(&self, pane: &str, text: &str) -> Result<(), String> {
        Self::run(&["send-keys", "-t", pane, "-l", text]).await?;
        Self::run(&["send-keys", "-t", pane, "Enter"]).await?;
        Ok(())
    }
}

/// The last `assistant` entry's usage in a transcript: input + cache creation + cache read
/// (spike 07 #6). Only the tail is read; a transcript can be many megabytes.
fn transcript_tokens(path: &str) -> Option<u64> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    let from = len.saturating_sub(1 << 20);
    f.seek(SeekFrom::Start(from)).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    String::from_utf8_lossy(&buf)
        .lines()
        .rev()
        .find_map(|line| {
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            if v.get("type")?.as_str()? != "assistant" {
                return None;
            }
            let u = v.get("message")?.get("usage")?;
            Some(
                [
                    "input_tokens",
                    "cache_creation_input_tokens",
                    "cache_read_input_tokens",
                ]
                .iter()
                .filter_map(|k| u.get(k).and_then(serde_json::Value::as_u64))
                .sum(),
            )
        })
}

pub struct RealProcs;

impl Procs for RealProcs {
    async fn is_alive(&self, pid: i32, start: &str) -> bool {
        let start = start.to_string();
        tokio::task::spawn_blocking(move || crate::containment::is_same_process(pid, &start))
            .await
            .unwrap_or(false)
    }

    async fn signal(&self, pid: i32, start: &str, kill: bool) {
        let start = start.to_string();
        let _ = tokio::task::spawn_blocking(move || {
            // Never signal a reused pid.
            if !crate::containment::is_same_process(pid, &start) {
                return;
            }
            let sig = if kill {
                nix::sys::signal::Signal::SIGKILL
            } else {
                nix::sys::signal::Signal::SIGTERM
            };
            // The pid is the launcher script's; SIGTERM to it alone kills the script and orphans
            // `claude`, which keeps the pane's tty while the shell and the relaunch take it back
            // (csfe: focus reports typed into the input). Signal the children so the script
            // sees claude end, records it and exits; SIGKILL also takes the script.
            let mut targets: Vec<i32> = crate::containment::snapshot()
                .map(|snap| crate::containment::descendants(pid, &snap))
                .unwrap_or_default()
                .into_iter()
                .map(|p| p.pid)
                .collect();
            if kill || targets.is_empty() {
                targets.push(pid);
            }
            for t in targets {
                if let Err(e) = nix::sys::signal::kill(nix::unistd::Pid::from_raw(t), sig) {
                    tracing::warn!(pid = t, error = %e, "signalling the orchestrator failed");
                }
            }
        })
        .await;
    }
}

/// A `system` note to the human plus an `orchestrator.incident` event.
pub struct RealIncidents {
    pub manager: AgentManager,
    pub emitter: Emitter,
}

impl Incidents for RealIncidents {
    async fn record(&self, text: &str) {
        tracing::warn!(incident = %text, "orchestrator incident");
        let _ = self
            .emitter
            .emit(
                event_kind::ORCHESTRATOR_INCIDENT,
                "system".to_string(),
                None,
                json!({ "text": text }),
            )
            .await;
        let _ = self
            .manager
            .send(
                "system".to_string(),
                ToTarget::Human,
                MessageKind::Note,
                text.to_string(),
                When::Now,
                None,
            )
            .await;
    }
}

pub type RealSupervisor = Supervisor<RealTmux, RealProcs, RealIncidents, Arc<Wakes>, Emitter>;

#[allow(clippy::too_many_arguments)]
pub fn real(
    home: PathBuf,
    launcher: String,
    cfg: &OrchestratorConfig,
    waiters: Arc<Waiters>,
    handover: Arc<Handover>,
    wakes: Arc<Wakes>,
    manager: AgentManager,
    emitter: Emitter,
) -> Arc<RealSupervisor> {
    let emitter_clone = emitter.clone();
    Arc::new(Supervisor::new(
        home,
        launcher,
        cfg,
        waiters,
        handover,
        RealTmux,
        RealProcs,
        RealIncidents { manager, emitter },
        wakes,
        emitter_clone,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Mutex as StdMutex;

    use super::*;

    #[derive(Default)]
    struct FakeTmux {
        panes: StdMutex<Vec<(String, String)>>,
        command: StdMutex<String>,
        typed: StdMutex<Vec<(String, String)>>,
    }
    impl Tmux for Arc<FakeTmux> {
        async fn panes(&self) -> Result<Vec<(String, String)>, String> {
            Ok(self.panes.lock().unwrap().clone())
        }
        async fn pane_command(&self, _pane: &str) -> Result<String, String> {
            Ok(self.command.lock().unwrap().clone())
        }
        async fn type_line(&self, pane: &str, text: &str) -> Result<(), String> {
            self.typed.lock().unwrap().push((pane.into(), text.into()));
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeProcs {
        alive: StdMutex<HashSet<i32>>,
        /// `(pid, kill)` for every signal sent.
        signals: StdMutex<Vec<(i32, bool)>>,
    }
    impl Procs for Arc<FakeProcs> {
        async fn is_alive(&self, pid: i32, _start: &str) -> bool {
            self.alive.lock().unwrap().contains(&pid)
        }
        async fn signal(&self, pid: i32, _start: &str, kill: bool) {
            self.signals.lock().unwrap().push((pid, kill));
        }
    }

    #[derive(Default)]
    struct FakeWakes(StdMutex<Vec<String>>);
    impl WakeSink for Arc<FakeWakes> {
        async fn push(&self, wake: WakeReason) {
            self.0.lock().unwrap().push(wake.text);
        }
    }

    #[derive(Default)]
    struct FakeIncidents(StdMutex<Vec<String>>);
    impl Incidents for Arc<FakeIncidents> {
        async fn record(&self, text: &str) {
            self.0.lock().unwrap().push(text.into());
        }
    }

    #[derive(Default)]
    struct FakeEmitter(StdMutex<Vec<(String, serde_json::Value)>>);
    impl EventEmitter for Arc<FakeEmitter> {
        async fn emit(
            &self,
            kind: &str,
            _actor: String,
            _agent: Option<String>,
            data: serde_json::Value,
        ) -> Result<(), String> {
            self.0.lock().unwrap().push((kind.to_string(), data));
            Ok(())
        }
    }

    type TestSupervisor = Supervisor<
        Arc<FakeTmux>,
        Arc<FakeProcs>,
        Arc<FakeIncidents>,
        Arc<FakeWakes>,
        Arc<FakeEmitter>,
    >;

    struct Rig {
        dir: tempfile::TempDir,
        tmux: Arc<FakeTmux>,
        procs: Arc<FakeProcs>,
        incidents: Arc<FakeIncidents>,
        emitter: Arc<FakeEmitter>,
        sup: TestSupervisor,
        waiters: Arc<Waiters>,
        wakes: Arc<FakeWakes>,
        handover: Arc<Handover>,
        t0: DateTime<Utc>,
    }

    impl Rig {
        fn new() -> Rig {
            let dir = tempfile::tempdir().unwrap();
            let tmux = Arc::new(FakeTmux::default());
            *tmux.panes.lock().unwrap() = vec![
                ("%1".into(), String::new()),
                ("%2".into(), "orchestrator".into()),
            ];
            *tmux.command.lock().unwrap() = "zsh".into();
            let procs = Arc::new(FakeProcs::default());
            let incidents = Arc::new(FakeIncidents::default());
            let cfg = OrchestratorConfig {
                relaunch_backoff: vec![Duration::from_secs(30), Duration::from_secs(120)],
                stable_after: Duration::from_secs(600),
                note_tokens: 150,
                plan_tokens: 210,
                handover_tokens: 255,
                handover_deadline: Duration::from_secs(1800),
                max_uptime: Duration::from_secs(12 * 3600),
                ..Default::default()
            };
            let t0 = DateTime::from_timestamp(1_800_000_000, 0).unwrap();
            let waiters = Waiters::new(t0);
            let wakes = Arc::new(FakeWakes::default());
            let handover = Arc::new(Handover::default());
            let emitter = Arc::new(FakeEmitter::default());
            let sup = Supervisor::new(
                dir.path().to_path_buf(),
                launch_line(None, "proj"),
                &cfg,
                waiters.clone(),
                handover.clone(),
                tmux.clone(),
                procs.clone(),
                incidents.clone(),
                wakes.clone(),
                emitter.clone(),
            );
            Rig {
                dir,
                tmux,
                procs,
                incidents,
                emitter,
                sup,
                waiters,
                wakes,
                handover,
                t0,
            }
        }

        fn write_pid(&self, pid: i32, launched_at: i64) {
            std::fs::write(
                self.dir.path().join("orchestrator.pid"),
                format!("{pid} Mon Sep 29 10:00:00 2026 {launched_at}\n"),
            )
            .unwrap();
        }

        /// A live session (pid 10, launched at t0) whose statusline says `tokens`.
        fn set_context(&self, tokens: u64) {
            std::fs::write(self.dir.path().join("orchestrator.session"), "sess-1\n").unwrap();
            std::fs::create_dir_all(self.dir.path().join("context")).unwrap();
            std::fs::write(self.dir.path().join("context/sess-1"), tokens.to_string()).unwrap();
        }

        fn live(&self) {
            self.write_pid(10, self.t0.timestamp());
            self.procs.alive.lock().unwrap().insert(10);
            // The waiter incident isn't under test here.
            std::mem::forget(self.waiters.opened());
        }

        fn wakes(&self) -> Vec<String> {
            self.wakes.0.lock().unwrap().clone()
        }

        fn signals(&self) -> Vec<(i32, bool)> {
            self.procs.signals.lock().unwrap().clone()
        }

        fn typed(&self) -> usize {
            self.tmux.typed.lock().unwrap().len()
        }

        fn incidents(&self) -> Vec<String> {
            self.incidents.0.lock().unwrap().clone()
        }

        fn events(&self) -> Vec<(String, serde_json::Value)> {
            self.emitter.0.lock().unwrap().clone()
        }

        /// A tick `secs` after t0.
        async fn tick(&self, secs: i64) {
            self.sup
                .tick(self.t0 + chrono::Duration::seconds(secs))
                .await;
        }

        /// Ticks at `secs` and `secs + 10` (the two shell checks), so a due relaunch types.
        async fn relaunch(&self, secs: i64) {
            self.tick(secs).await;
            self.tick(secs + 10).await;
        }
    }

    #[test]
    fn pid_file_with_a_spaced_start_time() {
        let r = parse_pid_file("123 Mon Sep 29 10:00:00 2026 1800000000\n").unwrap();
        assert_eq!(r.pid, 123);
        assert_eq!(r.start, "Mon Sep 29 10:00:00 2026");
        assert_eq!(r.launched, 1_800_000_000);
        assert!(parse_pid_file("123 x").is_none());
        assert!(parse_pid_file("abc def 5").is_none());
    }

    #[test]
    fn shell_word_quotes_only_when_needed() {
        assert_eq!(shell_word("/a/b-c/launch"), "/a/b-c/launch");
        assert_eq!(shell_word("/a b/it's"), "'/a b/it'\\''s'");
    }

    #[test]
    fn launch_line_defaults_to_the_session_command_and_types_a_custom_one_verbatim() {
        assert_eq!(
            launch_line(None, "bridle"),
            "bridle session orchestrator --project bridle"
        );
        assert_eq!(
            launch_line(Some("my launch --x 'y'"), "bridle"),
            "my launch --x 'y'"
        );
    }

    #[tokio::test]
    async fn no_pid_file_does_nothing() {
        let r = Rig::new();
        r.relaunch(0).await;
        assert_eq!(r.typed(), 0);
        assert!(r.incidents().is_empty());
    }

    #[tokio::test]
    async fn alive_session_is_left_alone() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp());
        r.procs.alive.lock().unwrap().insert(10);
        r.relaunch(0).await;
        assert_eq!(r.typed(), 0);
        assert!(r.incidents().is_empty());
    }

    #[tokio::test]
    async fn live_session_without_a_waiter_is_one_incident_closed_by_a_request() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp());
        r.procs.alive.lock().unwrap().insert(10);
        // Within the grace of the launch: nothing yet.
        r.tick(60).await;
        assert!(r.incidents().is_empty());
        r.tick(1000).await;
        r.tick(1010).await;
        assert_eq!(r.incidents().len(), 1, "reported once");
        assert!(r.incidents()[0].contains("wait-for-wake"));
        // A request arrives: the incident closes, and doesn't repeat while it is open.
        let guard = r.waiters.opened();
        r.tick(5000).await;
        assert_eq!(r.incidents().len(), 1);
        drop(guard);
    }

    #[tokio::test]
    async fn dead_session_relaunches_after_two_shell_checks() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        r.tick(0).await;
        assert_eq!(r.typed(), 0, "one shell check is not enough");
        r.tick(3).await;
        assert_eq!(r.typed(), 0, "the second check must be 5 s later");
        r.tick(10).await;
        assert_eq!(
            *r.tmux.typed.lock().unwrap(),
            vec![(
                "%2".to_string(),
                "bridle session orchestrator --project proj".to_string()
            )]
        );
        assert!(r.incidents()[0].contains("not running"));
    }

    #[tokio::test]
    async fn backoff_sequence_then_give_up() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        // Relaunch 1 at once (types at t=10 after the second check).
        r.relaunch(0).await;
        assert_eq!(r.typed(), 1);
        // Unconfirmed: nothing more inside the 60 s window or the 30 s backoff.
        r.tick(50).await;
        assert_eq!(r.typed(), 1);
        // Relaunch 2 waits backoff[0] = 30 s after the previous attempt (t=10).
        r.relaunch(70).await;
        assert_eq!(r.typed(), 2, "attempt at t=80");
        // Relaunch 3 waits backoff[1] = 120 s after t=80.
        r.relaunch(150).await;
        assert_eq!(r.typed(), 2, "too soon");
        r.relaunch(210).await;
        assert_eq!(r.typed(), 3, "attempt at t=220");
        // All failed: give up with one incident, then stay quiet.
        r.tick(300).await;
        r.tick(10_000).await;
        r.relaunch(20_000).await;
        assert_eq!(r.typed(), 3);
        let gave_up: Vec<_> = r
            .incidents()
            .into_iter()
            .filter(|t| t.contains("not staying up"))
            .collect();
        assert_eq!(gave_up.len(), 1);
        assert!(gave_up[0].contains("3 relaunches"));

        // A live pid (the human launched it) clears the give-up.
        r.write_pid(11, r.t0.timestamp() + 30_000);
        r.procs.alive.lock().unwrap().insert(11);
        r.tick(30_001).await;
        r.tick(30_700).await;
        r.procs.alive.lock().unwrap().clear();
        r.relaunch(31_000).await;
        assert_eq!(r.typed(), 4);
    }

    #[tokio::test]
    async fn stable_session_resets_the_count() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        r.relaunch(0).await;
        assert_eq!(r.typed(), 1);
        // The relaunched session (pid 11, launched at t=15) comes up and stays up.
        r.write_pid(11, r.t0.timestamp() + 15);
        r.procs.alive.lock().unwrap().insert(11);
        r.tick(20).await;
        r.tick(15 + 601).await;
        // It dies: the next relaunch is the first again, at once, not after backoff[0].
        r.procs.alive.lock().unwrap().clear();
        r.relaunch(700).await;
        assert_eq!(r.typed(), 2);
    }

    #[tokio::test]
    async fn short_lived_session_keeps_the_count() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        r.relaunch(0).await; // attempt 1 at t=10
        r.write_pid(11, r.t0.timestamp() + 15);
        r.procs.alive.lock().unwrap().insert(11);
        r.tick(20).await;
        r.procs.alive.lock().unwrap().clear();
        // Died young: attempt 2 still waits backoff[0] after t=10.
        r.tick(25).await;
        assert_eq!(r.typed(), 1);
        r.relaunch(45).await;
        assert_eq!(r.typed(), 2);
    }

    #[tokio::test]
    async fn non_shell_foreground_is_never_typed_into() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        *r.tmux.command.lock().unwrap() = "vim".into();
        r.relaunch(0).await;
        r.relaunch(20).await;
        assert_eq!(r.typed(), 0);
        let inc = r.incidents();
        assert_eq!(inc.iter().filter(|t| t.contains("not a shell")).count(), 1);
        // The human leaves vim: back at a shell, it relaunches.
        *r.tmux.command.lock().unwrap() = "bash".into();
        r.relaunch(40).await;
        assert_eq!(r.typed(), 1);
    }

    #[tokio::test]
    async fn no_tagged_pane_reports_then_retries_once_tagged() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        *r.tmux.panes.lock().unwrap() = vec![("%1".into(), String::new())];
        r.relaunch(0).await;
        r.relaunch(20).await;
        assert_eq!(r.typed(), 0);
        let inc = r.incidents();
        assert_eq!(
            inc.iter()
                .filter(|t| t.contains("no pane is tagged"))
                .count(),
            1
        );
        *r.tmux.panes.lock().unwrap() = vec![("%1".into(), "orchestrator".into())];
        r.relaunch(40).await;
        assert_eq!(r.typed(), 1);
    }

    #[tokio::test]
    async fn each_context_threshold_fires_once_and_resets_on_compact() {
        let r = Rig::new();
        r.live();
        r.set_context(100);
        r.tick(10).await;
        assert!(r.wakes().is_empty());
        r.set_context(160);
        r.tick(20).await;
        r.tick(30).await;
        assert_eq!(r.wakes(), vec!["context at 0K of the window"]);
        r.set_context(220);
        r.tick(40).await;
        r.tick(50).await;
        assert_eq!(r.wakes().len(), 2);
        assert!(r.wakes()[1].contains("plan a handover"));
        r.set_context(260);
        r.tick(60).await;
        r.tick(70).await;
        assert_eq!(r.wakes().len(), 3);
        assert!(r.wakes()[2].starts_with("hand over now; the session is stopped at"));
        // /compact: a lower reading resets the notes, and they fire again on the next climb.
        r.set_context(50);
        r.tick(80).await;
        assert_eq!(r.wakes().len(), 3);
        r.set_context(160);
        r.tick(90).await;
        assert_eq!(r.wakes().len(), 4);
    }

    #[tokio::test]
    async fn a_session_already_past_the_notes_gets_them_on_the_first_tick() {
        let r = Rig::new();
        r.live();
        r.set_context(220);
        r.tick(10).await;
        assert_eq!(r.wakes().len(), 2);
    }

    #[tokio::test]
    async fn a_new_session_id_starts_its_own_notes() {
        let r = Rig::new();
        r.live();
        r.set_context(160);
        r.tick(10).await;
        std::fs::write(r.dir.path().join("orchestrator.session"), "sess-2\n").unwrap();
        std::fs::write(r.dir.path().join("context/sess-2"), "160").unwrap();
        r.tick(20).await;
        assert_eq!(
            r.wakes().len(),
            2,
            "/clear gave a new id, so the note repeats"
        );
    }

    #[tokio::test]
    async fn context_falls_back_to_the_transcript_when_the_file_is_missing() {
        let r = Rig::new();
        r.live();
        let tp = r.dir.path().join("t.jsonl");
        std::fs::write(
            &tp,
            "{\"type\":\"assistant\",\"message\":{\"usage\":{\"input_tokens\":1,\"cache_creation_input_tokens\":9,\"cache_read_input_tokens\":150}}}\n\
             {\"type\":\"user\"}\n",
        )
        .unwrap();
        std::fs::write(
            r.dir.path().join("orchestrator.session"),
            format!("sess-1 {}\n", tp.display()),
        )
        .unwrap();
        r.tick(10).await;
        assert_eq!(r.wakes().len(), 1);
    }

    #[tokio::test]
    async fn uptime_notes_once_and_starts_the_deadline() {
        let r = Rig::new();
        r.live();
        r.tick(12 * 3600 - 10).await;
        assert!(r.wakes().is_empty());
        r.tick(12 * 3600 + 10).await;
        r.tick(12 * 3600 + 20).await;
        assert_eq!(r.wakes().len(), 1);
        assert!(r.wakes()[0].starts_with("uptime 12h"));
        assert!(r.signals().is_empty());
        // The deadline follows: 30 minutes after the note.
        r.tick(12 * 3600 + 10 + 1800).await;
        assert_eq!(r.signals(), vec![(10, false)]);
    }

    #[tokio::test]
    async fn the_deadline_stops_the_session_and_the_relaunch_is_free() {
        let r = Rig::new();
        r.live();
        r.set_context(260);
        r.tick(10).await; // the "now" note, the deadline at t=1810
        r.tick(1800).await;
        assert!(r.signals().is_empty());
        r.tick(1810).await;
        assert_eq!(r.signals(), vec![(10, false)]);
        // Still alive 15 s later: SIGKILL, once.
        r.tick(1820).await;
        assert_eq!(r.signals().len(), 1);
        r.tick(1825).await;
        r.tick(1835).await;
        assert_eq!(r.signals(), vec![(10, false), (10, true)]);
        // It dies; the relaunch goes at once with no crash backoff, and says the stop was forced.
        r.procs.alive.lock().unwrap().clear();
        r.relaunch(1840).await;
        assert_eq!(r.typed(), 1);
        assert!(
            r.incidents()
                .iter()
                .any(|t| t.contains("no `bridle handover done`"))
        );
        assert!(!r.incidents().iter().any(|t| t.contains("not running")));
        assert_eq!(
            r.sup.state.lock().await.attempts,
            0,
            "not counted as a crash"
        );
    }

    #[tokio::test]
    async fn handover_done_stops_at_once_and_relaunches_uncounted() {
        let r = Rig::new();
        r.live();
        r.tick(10).await;
        assert!(r.signals().is_empty());
        r.handover.mark(r.t0 + chrono::Duration::seconds(20));
        r.tick(20).await;
        assert_eq!(r.signals(), vec![(10, false)]);
        r.procs.alive.lock().unwrap().clear();
        r.relaunch(30).await;
        assert_eq!(r.typed(), 1);
        assert!(r.incidents().is_empty(), "a handover is not an incident");
        let st = r.sup.state.lock().await;
        assert_eq!((st.attempts, st.last_attempt), (0, None));
    }

    #[tokio::test]
    async fn a_marker_from_before_the_launch_does_not_stop_the_new_session() {
        let r = Rig::new();
        r.handover.mark(r.t0 - chrono::Duration::seconds(5));
        r.live();
        r.tick(10).await;
        assert!(r.signals().is_empty());
    }

    #[tokio::test]
    async fn only_the_first_deliberate_relaunch_is_free() {
        let r = Rig::new();
        r.live();
        r.handover.mark(r.t0 + chrono::Duration::seconds(20));
        r.tick(20).await;
        r.procs.alive.lock().unwrap().clear();
        r.relaunch(30).await; // free
        assert_eq!(r.typed(), 1);
        // Never confirmed and dead again: this one counts as a crash.
        r.relaunch(100).await;
        assert_eq!(r.typed(), 2);
        assert_eq!(r.sup.state.lock().await.attempts, 1);
    }

    #[tokio::test]
    async fn context_event_emitted_on_first_reading() {
        let r = Rig::new();
        r.live();
        r.set_context(100);
        r.tick(10).await;
        let events = r.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "orchestrator.context");
        let data = &events[0].1;
        assert_eq!(data["session"], "sess-1");
        assert_eq!(data["tokens"], 100);
        assert_eq!(data["window_size"], 1_000_000);
        assert_eq!(data["uptime_secs"], 10);
    }

    #[tokio::test]
    async fn context_event_not_emitted_within_10_minutes_if_unchanged() {
        let r = Rig::new();
        r.live();
        r.set_context(100);
        r.tick(10).await;
        assert_eq!(r.events().len(), 1);
        // Same reading within 10 minutes: no event.
        r.tick(100).await;
        assert_eq!(r.events().len(), 1);
        r.tick(500).await;
        assert_eq!(r.events().len(), 1);
        // After 10 minutes with a different reading: new event.
        r.set_context(150);
        r.tick(10 + 600).await;
        assert_eq!(r.events().len(), 2);
        assert_eq!(r.events()[1].1["tokens"], 150);
    }

    #[tokio::test]
    async fn context_event_emitted_on_lower_reading_compact() {
        let r = Rig::new();
        r.live();
        r.set_context(100);
        r.tick(10).await;
        assert_eq!(r.events().len(), 1);
        // Lower reading resets (compact); this is tracked by the threshold logic.
        // Emit an event on the first tick after a lower reading.
        r.set_context(50);
        r.tick(20).await;
        assert_eq!(r.events().len(), 2);
        assert_eq!(r.events()[1].1["tokens"], 50);
    }
    /// csfe: SIGTERM must reach the launcher's child (claude), not just the launcher, or claude
    /// is orphaned on the pane's tty.
    #[tokio::test]
    async fn real_sigterm_goes_to_the_child_and_spares_the_launcher() {
        let mut script = std::process::Command::new("sh")
            .args(["-c", "sleep 60; sleep 60"])
            .spawn()
            .unwrap();
        let pid = script.id() as i32;
        // The child has to exist before it can be found.
        let mut has_child = false;
        for _ in 0..50 {
            let snap = crate::containment::snapshot().unwrap();
            if !crate::containment::descendants(pid, &snap).is_empty() {
                has_child = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert!(has_child);
        let start = crate::containment::start_time(pid).unwrap();
        RealProcs.signal(pid, &start, false).await;
        // sh started the next sleep after the first died: it was the child that got the signal.
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(
            script.try_wait().unwrap().is_none(),
            "the launcher must outlive the TERM"
        );
        RealProcs.signal(pid, &start, true).await;
        script.wait().unwrap();
    }
}
