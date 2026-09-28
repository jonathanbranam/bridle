//! Spawns one `claude` process and drives it over stream-json. Separate
//! tasks own stdout, stderr and stdin; the caller only ever touches the
//! channels in [`Spawned`] and the cheap, cloneable [`AgentHandle`]. This
//! module has no daemon knowledge: it doesn't know what an "agent" is to
//! bridle, only how to run one `claude` process.

use std::collections::{HashMap, VecDeque};
use std::os::unix::process::ExitStatusExt;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot};

use crate::command::{ClaudeCommand, env_removal_keys};
use crate::events::{Event, EventKind};
use crate::transcript::Transcript;

/// How many trailing stderr lines [`ExitOutcome::stderr_tail`] keeps.
const STDERR_TAIL_LINES: usize = 50;

type Pending = Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>;

#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("spawning {program:?}: {source}")]
    Io {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("spawned child process has no pid")]
    NoPid,
}

/// The stdin pipe (or the process it belonged to) is gone.
#[derive(Debug, thiserror::Error)]
#[error("stdin is closed")]
pub struct SendError;

#[derive(Debug, thiserror::Error)]
pub enum ControlError {
    #[error(transparent)]
    Send(#[from] SendError),
    #[error("timed out waiting for a control response")]
    Timeout,
    #[error("process exited before responding")]
    Dropped,
}

/// What spawning hands back: a handle to drive the process, its event
/// stream, and a one-shot for how it eventually exited.
pub struct Spawned {
    pub handle: AgentHandle,
    pub events: mpsc::UnboundedReceiver<Event>,
    pub exit: oneshot::Receiver<ExitOutcome>,
}

/// How the process ended. `code`/`signal` mirror `std::process::ExitStatus`
/// on unix: a normal exit sets `code`, death by signal sets `signal`, never
/// both.
#[derive(Debug, Clone)]
pub struct ExitOutcome {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    /// The last ~50 stderr lines, for diagnosing a crash.
    pub stderr_tail: Vec<String>,
}

struct HandleInner {
    pid: i32,
    stdin_tx: Mutex<Option<mpsc::UnboundedSender<String>>>,
    pending: Pending,
    next_request: AtomicU64,
}

/// A cheap-to-clone handle to a running (or just-exited) `claude` process.
/// Cloning shares the same underlying stdin pipe and pending-control-request
/// table; it does not spawn or duplicate anything.
#[derive(Clone)]
pub struct AgentHandle(Arc<HandleInner>);

impl AgentHandle {
    pub fn pid(&self) -> i32 {
        self.0.pid
    }

    /// Writes a stdin user message:
    /// `{"type":"user","message":{"role":"user","content":text},"parent_tool_use_id":null}`.
    /// Non-async: it hands the line to the writer task and returns.
    pub fn send_user(&self, text: &str) -> Result<(), SendError> {
        self.send_line(json!({
            "type": "user",
            "message": { "role": "user", "content": text },
            "parent_tool_use_id": null,
        }))
    }

    /// Sends a `control_request` with a freshly generated `request_id` and
    /// returns a receiver correlated to its `control_response`. Generic:
    /// callers can use this for `get_usage` and friends, not just interrupt.
    pub fn control(&self, request: Value) -> Result<oneshot::Receiver<Value>, SendError> {
        let n = self.0.next_request.fetch_add(1, Ordering::Relaxed) + 1;
        let id = format!("bridle-{n}");
        let (tx, rx) = oneshot::channel();
        self.0
            .pending
            .lock()
            .expect("pending mutex poisoned")
            .insert(id.clone(), tx);
        self.send_line(json!({ "type": "control_request", "request_id": id, "request": request }))?;
        Ok(rx)
    }

    /// Sends `{"subtype":"interrupt"}` and waits for the correlated receipt.
    /// Never sets `cancel_queued`: it silently drops pending messages
    /// (docs/design/agent-host/messages.md / spike surprise 2).
    pub async fn interrupt(&self, timeout: Duration) -> Result<Value, ControlError> {
        let rx = self.control(json!({ "subtype": "interrupt" }))?;
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(_)) => Err(ControlError::Dropped),
            Err(_) => Err(ControlError::Timeout),
        }
    }

    /// Sends `{"subtype":"get_usage"}` and waits for the correlated receipt.
    /// Undocumented (docs/spikes/01-stream-json-findings.md): works on an
    /// idle process with no prior model call.
    pub async fn get_usage(&self, timeout: Duration) -> Result<Value, ControlError> {
        let rx = self.control(json!({ "subtype": "get_usage" }))?;
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(_)) => Err(ControlError::Dropped),
            Err(_) => Err(ControlError::Timeout),
        }
    }

    /// Sends `{"subtype":"get_context_usage"}` and waits for the correlated
    /// receipt. Undocumented (docs/spikes/01-stream-json-findings.md §11):
    /// gives a per-category token breakdown plus a single `totalTokens` for
    /// the whole context window, and the auto-compact threshold.
    pub async fn get_context_usage(&self, timeout: Duration) -> Result<Value, ControlError> {
        let rx = self.control(json!({ "subtype": "get_context_usage" }))?;
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(_)) => Err(ControlError::Dropped),
            Err(_) => Err(ControlError::Timeout),
        }
    }

    /// Closes stdin. Idempotent: closing an already-closed handle is a
    /// no-op. This is what makes claude finish its current turn and exit
    /// (docs/design/agent-host/agents.md, stop step 1).
    pub fn close_stdin(&self) {
        self.0.stdin_tx.lock().expect("stdin mutex poisoned").take();
    }

    pub fn stdin_open(&self) -> bool {
        self.0
            .stdin_tx
            .lock()
            .expect("stdin mutex poisoned")
            .is_some()
    }

    /// `killpg` on the process's group. Note this does not reach Bash tool
    /// subprocesses, which run in their own group (spike surprise 3); a
    /// containment sweep is still needed after this.
    pub fn signal_group(&self, sig: Signal) -> nix::Result<()> {
        killpg(Pid::from_raw(self.0.pid), sig)
    }

    fn send_line(&self, v: Value) -> Result<(), SendError> {
        let guard = self.0.stdin_tx.lock().expect("stdin mutex poisoned");
        let tx = guard.as_ref().ok_or(SendError)?;
        tx.send(v.to_string()).map_err(|_| SendError)
    }
}

/// Spawns `cmd` as a child of the current process, in its own process group,
/// with `kill_on_drop(false)`: the caller (a supervisor) decides when and
/// how to kill it, per docs/design/agent-host/agents.md. Every raw line in and out is
/// recorded to `transcript`.
pub async fn spawn(cmd: &ClaudeCommand, transcript: Transcript) -> Result<Spawned, SpawnError> {
    let args = cmd.args();

    let mut tcmd = Command::new(&cmd.program);
    tcmd.args(&args)
        .current_dir(&cmd.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(false);
    for key in env_removal_keys(std::env::vars()) {
        tcmd.env_remove(key);
    }
    for (k, v) in &cmd.env {
        tcmd.env(k, v);
    }

    transcript.note(&format!("spawn: {} {}", cmd.program, args.join(" ")));
    let mut child = tcmd.spawn().map_err(|source| SpawnError::Io {
        program: cmd.program.clone(),
        source,
    })?;
    let pid = child.id().ok_or(SpawnError::NoPid)? as i32;
    transcript.note(&format!("pid {pid}"));
    tracing::debug!(pid, program = %cmd.program, "spawned claude");

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");
    let stdin = child.stdin.take().expect("stdin was piped");

    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
    let (events_tx, events_rx) = mpsc::unbounded_channel::<Event>();
    let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<String>();
    let stderr_tail: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));

    // stdout: line -> transcript "out" -> parse -> resolve a pending control
    // request by request_id -> events channel. The channel closes (and
    // pending control requests get dropped senders) at EOF.
    let stdout_task = {
        let transcript = transcript.clone();
        let pending = pending.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        transcript.record("out", &line);
                        let ev = Event::parse(transcript.elapsed_ms(), line);
                        if let EventKind::ControlResponse(cr) = &ev.kind
                            && let Some(tx) = pending
                                .lock()
                                .expect("pending mutex poisoned")
                                .remove(&cr.response.request_id)
                        {
                            let _ = tx.send(ev.value.clone());
                        }
                        if events_tx.send(ev).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        transcript.note(&format!("stdout read error: {e}"));
                        break;
                    }
                }
            }
            transcript.note("stdout EOF");
            pending.lock().expect("pending mutex poisoned").clear();
        })
    };

    // stderr: transcript "err" + a rolling tail for ExitOutcome.
    let stderr_task = {
        let transcript = transcript.clone();
        let tail = stderr_tail.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                transcript.record("err", &line);
                let mut t = tail.lock().expect("stderr tail mutex poisoned");
                t.push_back(line);
                if t.len() > STDERR_TAIL_LINES {
                    t.pop_front();
                }
            }
        })
    };

    // stdin: channel -> pipe. Dropping (or closing) the sender ends the
    // channel, which ends this task, which drops the pipe.
    let stdin_task = {
        let transcript = transcript.clone();
        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(line) = stdin_rx.recv().await {
                transcript.record("in", &line);
                let write = async {
                    stdin.write_all(line.as_bytes()).await?;
                    stdin.write_all(b"\n").await?;
                    stdin.flush().await
                };
                if let Err(e) = write.await {
                    transcript.note(&format!("stdin write failed: {e}"));
                    break;
                }
            }
            drop(stdin);
            transcript.note("stdin closed");
        })
    };

    // The stdin writer task is deliberately not joined here: it only ends
    // when its sender is dropped (`close_stdin`, or every `AgentHandle`
    // dropping) or a write fails, neither of which is guaranteed by the
    // time the process exits (e.g. a caller that stops the process by
    // signal without ever closing stdin). Joining it here would risk the
    // reaper waiting forever on a pipe nobody is going to close.
    drop(stdin_task);

    // Reaper: waits for both readers to see EOF, then reaps the child, so no
    // events are lost to a race between "process exited" and "we're still
    // reading its last lines".
    let (exit_tx, exit_rx) = oneshot::channel();
    tokio::spawn(async move {
        let _ = stdout_task.await;
        let _ = stderr_task.await;
        let status = child.wait().await;
        let stderr_tail: Vec<String> = stderr_tail
            .lock()
            .expect("stderr tail mutex poisoned")
            .iter()
            .cloned()
            .collect();
        let outcome = match status {
            Ok(status) => ExitOutcome {
                code: status.code(),
                signal: status.signal(),
                stderr_tail,
            },
            Err(_) => ExitOutcome {
                code: None,
                signal: None,
                stderr_tail,
            },
        };
        tracing::debug!(code = ?outcome.code, signal = ?outcome.signal, "claude exited");
        let _ = exit_tx.send(outcome);
    });

    let handle = AgentHandle(Arc::new(HandleInner {
        pid,
        stdin_tx: Mutex::new(Some(stdin_tx)),
        pending,
        next_request: AtomicU64::new(0),
    }));

    Ok(Spawned {
        handle,
        events: events_rx,
        exit: exit_rx,
    })
}
