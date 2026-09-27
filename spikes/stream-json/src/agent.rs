//! `AgentProcess`: one headless `claude -p` with stream-json on stdin and
//! stdout, in its own process group. Separate tasks own stdout, stderr and
//! stdin; the caller only ever awaits channels.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, anyhow, bail};
use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::events::{Event, EventKind, ResultEvent};
use crate::log::Transcript;

pub struct SpawnConfig {
    pub cwd: PathBuf,
    /// `--session-id`; None when resuming (see S7).
    pub session_id: Option<Uuid>,
    pub model: String,
    pub extra_args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub transcript: Transcript,
}

type Pending = Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>;

pub struct AgentProcess {
    child: Child,
    pub pid: i32,
    stdin_tx: Option<mpsc::UnboundedSender<String>>,
    stdin_task: Option<JoinHandle<()>>,
    events_rx: mpsc::UnboundedReceiver<Event>,
    pending: Pending,
    next_request: u64,
    pub log: Transcript,
    /// Every event consumed through `next_event`, in order.
    pub history: Vec<Event>,
}

impl AgentProcess {
    pub async fn spawn(cfg: SpawnConfig) -> anyhow::Result<Self> {
        let mut args: Vec<String> = [
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--verbose",
            "--model",
            &cfg.model,
            "--effort",
            "low",
            "--max-budget-usd",
            "0.50",
            "--strict-mcp-config",
            "--permission-mode",
            "dontAsk",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        if let Some(id) = cfg.session_id {
            args.push("--session-id".into());
            args.push(id.to_string());
        }
        args.extend(cfg.extra_args);

        let mut cmd = Command::new("claude");
        cmd.args(&args)
            .current_dir(&cfg.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);
        // This harness runs inside a Claude Code session; a bridle daemon
        // won't. Strip the inherited session vars so the child isn't a
        // "nested" session.
        for (k, _) in std::env::vars() {
            if k.starts_with("CLAUDE") {
                cmd.env_remove(k);
            }
        }
        for (k, v) in &cfg.env {
            cmd.env(k, v);
        }

        let log = cfg.transcript;
        log.note(&format!("spawn: claude {}", args.join(" ")));
        let mut child = cmd.spawn().context("spawning claude")?;
        let pid = child.id().ok_or_else(|| anyhow!("no pid"))? as i32;
        log.note(&format!("pid {pid}"));

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let mut stdin = child.stdin.take().unwrap();

        let pending: Pending = Arc::default();
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        // stdout: line → transcript → parse → resolve pending control request → channel.
        {
            let log = log.clone();
            let pending = pending.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    log.record("out", &line);
                    let ev = Event::parse(log.elapsed_ms(), line);
                    if let EventKind::ControlResponse(cr) = &ev.kind
                        && let Some(tx) = pending.lock().unwrap().remove(&cr.response.request_id)
                    {
                        let _ = tx.send(ev.value.clone());
                    }
                    if events_tx.send(ev).is_err() {
                        break;
                    }
                }
                log.note("stdout EOF");
            });
        }
        // stderr: transcript only.
        {
            let log = log.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    log.record("err", &line);
                }
            });
        }
        // stdin: channel → pipe. Dropping the sender closes the pipe.
        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<String>();
        let stdin_task = {
            let log = log.clone();
            tokio::spawn(async move {
                while let Some(line) = stdin_rx.recv().await {
                    log.record("in", &line);
                    let write = async {
                        stdin.write_all(line.as_bytes()).await?;
                        stdin.write_all(b"\n").await?;
                        stdin.flush().await
                    };
                    if let Err(e) = write.await {
                        log.note(&format!("stdin write failed: {e}"));
                        break;
                    }
                }
                drop(stdin);
                log.note("stdin closed");
            })
        };

        Ok(Self {
            child,
            pid,
            stdin_tx: Some(stdin_tx),
            stdin_task: Some(stdin_task),
            events_rx,
            pending,
            next_request: 0,
            log,
            history: Vec::new(),
        })
    }

    fn send_line(&self, v: Value) -> anyhow::Result<()> {
        let tx = self.stdin_tx.as_ref().ok_or_else(|| anyhow!("stdin already closed"))?;
        tx.send(v.to_string()).map_err(|_| anyhow!("stdin writer gone"))
    }

    pub fn send_user(&self, text: &str) -> anyhow::Result<()> {
        self.send_line(json!({
            "type": "user",
            "message": { "role": "user", "content": text },
            "parent_tool_use_id": null,
        }))
    }

    /// Sends a control request and returns a receiver for its correlated
    /// response. Not awaited here: the caller keeps draining events meanwhile.
    pub fn control(&mut self, request: Value) -> anyhow::Result<oneshot::Receiver<Value>> {
        self.next_request += 1;
        let id = format!("bridle-{}", self.next_request);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id.clone(), tx);
        self.send_line(json!({ "type": "control_request", "request_id": id, "request": request }))?;
        Ok(rx)
    }

    /// Interrupt and wait for the receipt, buffering any events that arrive first.
    pub async fn interrupt(&mut self, cancel_queued: bool) -> anyhow::Result<Value> {
        let mut req = json!({ "subtype": "interrupt" });
        if cancel_queued {
            req["cancel_queued"] = json!(true);
        }
        let rx = self.control(req)?;
        tokio::time::timeout(Duration::from_secs(15), rx)
            .await
            .context("interrupt receipt timed out")?
            .map_err(|_| anyhow!("reader dropped before receipt"))
    }

    pub async fn next_event(&mut self) -> Option<Event> {
        let ev = self.events_rx.recv().await?;
        self.history.push(ev.clone());
        Some(ev)
    }

    pub async fn next_event_timeout(&mut self, d: Duration) -> Option<Event> {
        tokio::time::timeout(d, self.next_event()).await.ok().flatten()
    }

    /// Waits for an event matching `pred`. Errors on timeout or stream end.
    pub async fn wait_for(
        &mut self,
        timeout: Duration,
        mut pred: impl FnMut(&Event) -> bool,
    ) -> anyhow::Result<Event> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            match tokio::time::timeout_at(deadline, self.next_event()).await {
                Err(_) => bail!("timed out after {timeout:?}"),
                Ok(None) => bail!("event stream ended"),
                Ok(Some(ev)) if pred(&ev) => return Ok(ev),
                Ok(Some(_)) => {}
            }
        }
    }

    pub async fn wait_for_result(&mut self, timeout: Duration) -> anyhow::Result<ResultEvent> {
        let ev = self.wait_for(timeout, |e| matches!(e.kind, EventKind::Result(_))).await?;
        match ev.kind {
            EventKind::Result(r) => Ok(r),
            _ => unreachable!(),
        }
    }

    /// Close stdin, drain the remaining events into `history`, reap.
    pub async fn close_stdin(mut self, timeout: Duration) -> anyhow::Result<(ExitStatus, Vec<Event>)> {
        self.log.note("closing stdin");
        self.stdin_tx.take();
        if let Some(t) = self.stdin_task.take() {
            let _ = t.await;
        }
        let before = self.history.len();
        let status = tokio::time::timeout(timeout, async {
            while self.next_event().await.is_some() {}
            self.child.wait().await
        })
        .await
        .context("exit after stdin close timed out")??;
        self.log.note(&format!("exit status {status:?}"));
        Ok((status, self.history[before..].to_vec()))
    }

    pub fn terminate_group(&self, sig: Signal) -> anyhow::Result<()> {
        self.log.note(&format!("killpg({}, {sig:?})", self.pid));
        killpg(Pid::from_raw(self.pid), sig)?;
        Ok(())
    }

    pub fn try_wait(&mut self) -> anyhow::Result<Option<ExitStatus>> {
        Ok(self.child.try_wait()?)
    }

    pub async fn wait(&mut self) -> anyhow::Result<ExitStatus> {
        Ok(self.child.wait().await?)
    }

    /// Drain whatever is already buffered or arrives within `d`.
    pub async fn drain(&mut self, d: Duration) -> Vec<Event> {
        let mut out = Vec::new();
        let deadline = tokio::time::Instant::now() + d;
        while let Ok(Some(ev)) = tokio::time::timeout_at(deadline, self.next_event()).await {
            out.push(ev);
        }
        out
    }
}
