//! Integration tests against `tests/fake-claude.py` (see its module
//! docstring for the magic words). These exercise `process::spawn` and
//! `AgentHandle` without spending any tokens on a real `claude`.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bridle_claude::events::EventKind;
use bridle_claude::{ClaudeCommand, Session, Transcript, spawn};
use nix::sys::signal::Signal;
use tokio::time::timeout;
use uuid::Uuid;

/// Hang guard timeout for waiting on the fake claude process to exit or close stdout.
/// A passing test should never be slowed by this; only a genuinely hung one waits longer.
const HANG_GUARD_TIMEOUT: Duration = Duration::from_secs(60);

fn fake_claude_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fake-claude.py")
}

/// A `ClaudeCommand` pointed at the fake, in a fresh tempdir cwd.
fn fake_command(cwd: &Path, session: Session) -> ClaudeCommand {
    let mut cmd = ClaudeCommand::new(cwd, session);
    cmd.program = fake_claude_path().to_string_lossy().into_owned();
    cmd
}

fn open_transcript(dir: &Path) -> Transcript {
    Transcript::open(&dir.join("transcript.jsonl"), Instant::now()).expect("open transcript")
}

#[tokio::test]
async fn one_turn_produces_init_assistant_text_and_result() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let mut spawned = spawn(&cmd, transcript).await.expect("spawn");

    spawned.handle.send_user("hello").expect("send_user");

    let mut saw_init = false;
    let mut saw_text = false;
    timeout(HANG_GUARD_TIMEOUT, async {
        loop {
            let ev = spawned
                .events
                .recv()
                .await
                .expect("events channel ended before result");
            match &ev.kind {
                EventKind::SystemInit(_) => saw_init = true,
                EventKind::Assistant(m) if m.text().contains("echo: hello") => saw_text = true,
                EventKind::Result(r) => {
                    assert_eq!(r.subtype, "success");
                    assert!(!r.is_error);
                    assert_eq!(r.result.as_deref(), Some("echo: hello"));
                    break;
                }
                _ => {}
            }
        }
    })
    .await
    .expect("timed out waiting for the turn to complete");
    assert!(saw_init, "expected a system/init event");
    assert!(saw_text, "expected the assistant's echoed text");

    spawned.handle.close_stdin();
    let outcome = timeout(HANG_GUARD_TIMEOUT, spawned.exit)
        .await
        .expect("timed out waiting for exit")
        .expect("exit sender dropped");
    assert_eq!(outcome.code, Some(0));
    assert_eq!(outcome.signal, None);
}

#[tokio::test]
async fn mid_turn_message_is_folded_into_the_running_turn() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let mut spawned = spawn(&cmd, transcript).await.expect("spawn");

    spawned.handle.send_user("SLEEP 2").expect("send_user");
    tokio::time::sleep(Duration::from_millis(300)).await;
    spawned
        .handle
        .send_user("banana")
        .expect("send_user fold-in");

    let result = timeout(HANG_GUARD_TIMEOUT, async {
        loop {
            let ev = spawned
                .events
                .recv()
                .await
                .expect("events channel ended before result");
            if let EventKind::Result(r) = ev.kind {
                return r;
            }
        }
    })
    .await
    .expect("timed out waiting for the folded turn to complete");

    assert_eq!(result.subtype, "success");
    let text = result.result.expect("result text");
    assert!(text.contains("echo: SLEEP 2"), "{text}");
    assert!(text.contains("echo: banana"), "{text}");

    spawned.handle.close_stdin();
    let outcome = timeout(HANG_GUARD_TIMEOUT, spawned.exit)
        .await
        .expect("exit timeout")
        .expect("exit sender dropped");
    assert_eq!(outcome.code, Some(0));
}

#[tokio::test]
async fn interrupt_during_sleep_gets_a_fast_receipt_and_aborts_the_turn() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let mut spawned = spawn(&cmd, transcript).await.expect("spawn");

    spawned.handle.send_user("SLEEP 5").expect("send_user");

    // Wait for the tool_use event before timing the interrupt, so the
    // 1-second bound below measures interrupt latency once the process is
    // up and mid-turn, not interpreter/process startup (which can itself
    // take a while, e.g. behind a slow `python3` shim).
    timeout(HANG_GUARD_TIMEOUT, async {
        loop {
            let ev = spawned
                .events
                .recv()
                .await
                .expect("events channel ended before tool_use");
            if let EventKind::Assistant(m) = &ev.kind
                && m.tool_uses().iter().any(|(_, name, _)| name == "Bash")
            {
                return;
            }
        }
    })
    .await
    .expect("timed out waiting for the sleep tool_use");

    let started = Instant::now();
    let receipt = spawned
        .handle
        .interrupt(Duration::from_secs(1))
        .await
        .expect("interrupt");
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "receipt took {:?}",
        started.elapsed()
    );
    // The receipt is the whole control_response envelope:
    // {"type":"control_response","response":{"subtype":"success","request_id":…,"response":{"still_queued":[]}}}.
    let still_queued = receipt
        .get("response")
        .and_then(|r| r.get("response"))
        .and_then(|r| r.get("still_queued"))
        .and_then(|v| v.as_array())
        .map(|a| a.len());
    assert_eq!(still_queued, Some(0));

    let result = timeout(HANG_GUARD_TIMEOUT, async {
        loop {
            let ev = spawned
                .events
                .recv()
                .await
                .expect("events channel ended before result");
            if let EventKind::Result(r) = ev.kind {
                return r;
            }
        }
    })
    .await
    .expect("timed out waiting for the aborted result");
    assert_eq!(result.subtype, "error_during_execution");
    assert!(result.is_error);
    assert_eq!(result.terminal_reason.as_deref(), Some("aborted_tools"));

    spawned.handle.close_stdin();
    let outcome = timeout(HANG_GUARD_TIMEOUT, spawned.exit)
        .await
        .expect("exit timeout")
        .expect("exit sender dropped");
    // The last turn's result was an error, so the exit code is 1 (spike S5b).
    assert_eq!(outcome.code, Some(1));
}

#[tokio::test]
async fn close_stdin_with_no_turn_exits_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let spawned = spawn(&cmd, transcript).await.expect("spawn");

    spawned.handle.close_stdin();
    assert!(!spawned.handle.stdin_open());

    let outcome = timeout(HANG_GUARD_TIMEOUT, spawned.exit)
        .await
        .expect("exit timeout")
        .expect("exit sender dropped");
    assert_eq!(outcome.code, Some(0));
}

#[tokio::test]
async fn close_stdin_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let spawned = spawn(&cmd, transcript).await.expect("spawn");

    assert!(spawned.handle.stdin_open());
    spawned.handle.close_stdin();
    spawned.handle.close_stdin();
    assert!(!spawned.handle.stdin_open());
    assert!(spawned.handle.send_user("too late").is_err());

    let _ = timeout(HANG_GUARD_TIMEOUT, spawned.exit)
        .await
        .expect("exit timeout");
}

#[tokio::test]
async fn sigterm_via_signal_group_exits_143() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let spawned = spawn(&cmd, transcript).await.expect("spawn");

    // Prove the fake has actually started (interpreter up, SIGTERM handler
    // installed) before signaling it: an unrelated process launched behind
    // a slow `python3` shim can still be in its exec chain a moment after
    // `spawn` returns, in which case the signal would just kill it outright
    // instead of exercising the handler this test is about.
    let rx = spawned
        .handle
        .control(serde_json::json!({"subtype": "get_usage"}))
        .expect("control");
    timeout(HANG_GUARD_TIMEOUT, rx)
        .await
        .expect("control response timed out")
        .expect("control sender dropped");

    spawned
        .handle
        .signal_group(Signal::SIGTERM)
        .expect("signal_group");

    let outcome = timeout(HANG_GUARD_TIMEOUT, spawned.exit)
        .await
        .expect("exit timeout")
        .expect("exit sender dropped");
    assert_eq!(outcome.code, Some(143));
    assert_eq!(outcome.signal, None);
}

#[tokio::test]
async fn crash_reports_exit_code_and_stderr_tail() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let mut spawned = spawn(&cmd, transcript).await.expect("spawn");

    spawned.handle.send_user("CRASH").expect("send_user");

    // No result is ever emitted for a crash; drain events until the channel
    // closes (stdout EOF) rather than waiting for a Result.
    let _ = timeout(HANG_GUARD_TIMEOUT, async {
        while spawned.events.recv().await.is_some() {}
    })
    .await;

    let outcome = timeout(HANG_GUARD_TIMEOUT, spawned.exit)
        .await
        .expect("exit timeout")
        .expect("exit sender dropped");
    assert_eq!(outcome.code, Some(3));
    assert!(
        !outcome.stderr_tail.is_empty(),
        "expected a non-empty stderr tail"
    );
    assert!(
        outcome.stderr_tail.iter().any(|l| l.contains("CRASH")),
        "{:?}",
        outcome.stderr_tail
    );
}

#[tokio::test]
async fn events_channel_closes_at_stdout_eof() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let mut spawned = spawn(&cmd, transcript).await.expect("spawn");

    spawned.handle.send_user("EXIT 0").expect("send_user");

    let drained = timeout(HANG_GUARD_TIMEOUT, async {
        let mut n = 0;
        while spawned.events.recv().await.is_some() {
            n += 1;
        }
        n
    })
    .await
    .expect("events channel never closed");
    // EXIT n exits at once: at most the replay echo and system/init are
    // emitted before the process terminates, never a result.
    assert!(
        drained <= 2,
        "expected at most the replay echo and system/init, got {drained}"
    );
}

#[tokio::test]
async fn pending_control_request_is_dropped_on_exit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let spawned = spawn(&cmd, transcript).await.expect("spawn");

    // Register a pending control request, then kill the process outright
    // (SIGKILL, not caught) so it never gets a chance to answer.
    let rx = spawned
        .handle
        .control(serde_json::json!({"subtype": "get_usage"}))
        .expect("control");
    spawned
        .handle
        .signal_group(Signal::SIGKILL)
        .expect("signal_group");

    let resolved = timeout(HANG_GUARD_TIMEOUT, rx)
        .await
        .expect("did not resolve promptly");
    assert!(
        resolved.is_err(),
        "expected the pending control request's sender to be dropped, not resolved"
    );
}

/// `get_context_usage` reports a single `totalTokens` for the whole context
/// window, separate from a turn's `usage` sum (kc4v: the daemon uses this,
/// not `result.usage`, for `context_tokens`, since that sum is per-API-call
/// and inflates across a turn's tool round-trips).
#[tokio::test]
async fn get_context_usage_returns_total_tokens() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join(".fake-claude-context-usage"),
        serde_json::json!({"totalTokens": 42_000, "maxTokens": 200_000}).to_string(),
    )
    .expect("write .fake-claude-context-usage");
    let cmd = fake_command(dir.path(), Session::New(Uuid::new_v4()));
    let transcript = open_transcript(dir.path());
    let spawned = spawn(&cmd, transcript).await.expect("spawn");

    let response = timeout(
        HANG_GUARD_TIMEOUT,
        spawned.handle.get_context_usage(HANG_GUARD_TIMEOUT),
    )
    .await
    .expect("get_context_usage timed out")
    .expect("get_context_usage control error");

    let total = response
        .pointer("/response/response/totalTokens")
        .and_then(serde_json::Value::as_u64);
    assert_eq!(total, Some(42_000));
}
