//! The Claude Code contract: the few undocumented behaviours bridle depends
//! on, checked against the real `claude` (Haiku, tiny prompts, roughly $0.10
//! a run). Claude Code updates itself; bridle doesn't pin it. After an
//! update, run `just test-contract`: if it passes, the new version is
//! accepted and recorded; if not, fix bridle forward.
//!
//! Each test cites the spike 01 scenario (docs/spikes/01-stream-json-findings.md)
//! or later findings it guards. Ignored unless `BRIDLE_LIVE_TESTS=1`.

use std::time::{Duration, Instant};

use bridle_claude::events::{Event, EventKind, ResultEvent};
use bridle_claude::{ClaudeCommand, Session, Spawned, Transcript, spawn};
use tokio::time::timeout;
use uuid::Uuid;

const WAIT: Duration = Duration::from_secs(90);

fn live() -> bool {
    let on = std::env::var("BRIDLE_LIVE_TESTS").as_deref() == Ok("1");
    if !on {
        eprintln!("skipping: set BRIDLE_LIVE_TESTS=1 to run the Claude Code contract tests");
    }
    on
}

fn command(dir: &std::path::Path, session: Session) -> ClaudeCommand {
    let mut cmd = ClaudeCommand::new(dir, session);
    cmd.model = Some("haiku".to_string());
    cmd.permission_mode = Some("dontAsk".to_string());
    cmd.allowed_tools = vec!["Bash".to_string()];
    cmd
}

async fn start(dir: &std::path::Path, cmd: &ClaudeCommand) -> Spawned {
    let transcript =
        Transcript::open(&dir.join("transcript.jsonl"), Instant::now()).expect("open transcript");
    spawn(cmd, transcript).await.expect("spawn real claude")
}

async fn next(s: &mut Spawned) -> Event {
    timeout(WAIT, s.events.recv())
        .await
        .expect("timed out waiting for claude")
        .expect("claude's output ended early")
}

/// Every event up to and including the turn's `result`.
async fn until_result(s: &mut Spawned) -> (Vec<Event>, ResultEvent) {
    let mut seen = Vec::new();
    loop {
        let ev = next(s).await;
        if let EventKind::Result(r) = &ev.kind {
            let r = r.clone();
            seen.push(ev);
            return (seen, r);
        }
        seen.push(ev);
    }
}

async fn until_tool_use(s: &mut Spawned) {
    loop {
        if let EventKind::Assistant(m) = next(s).await.kind
            && !m.tool_uses().is_empty()
        {
            return;
        }
    }
}

async fn exit_code(s: Spawned) -> Option<i32> {
    s.handle.close_stdin();
    timeout(Duration::from_secs(30), s.exit)
        .await
        .expect("claude didn't exit after stdin closed")
        .expect("exit sender dropped")
        .code
}

fn replayed(events: &[Event], text: &str) -> Option<usize> {
    events.iter().position(|e| match &e.kind {
        EventKind::User(m) => m.is_replay() && m.replay_text() == Some(text),
        _ => false,
    })
}

/// Turn boundaries, delivery acks, exit codes and resume: S2, S3, S5, S7, S8.
#[tokio::test]
#[ignore = "spends real tokens against the real claude binary"]
async fn turns_acks_exit_codes_and_resume() {
    if !live() {
        return;
    }
    let dir = tempfile::tempdir().expect("tempdir");
    let session = Uuid::new_v4();

    let mut s = start(dir.path(), &command(dir.path(), Session::New(session))).await;
    let first = "Reply with exactly: PONG";
    s.handle.send_user(first).expect("send");
    let (events, result) = until_result(&mut s).await;

    let init = events
        .iter()
        .find_map(|e| match &e.kind {
            EventKind::SystemInit(i) => Some(i.clone()),
            _ => None,
        })
        .expect("system/init starts every turn (S2)");
    let version = init
        .claude_code_version
        .expect("init reports claude_code_version");
    eprintln!("contract: Claude Code {version}");
    assert!(
        replayed(&events, first).is_some(),
        "--replay-user-messages echoes the text verbatim (S3): acks match on it"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::RateLimit(_))),
        "a rate_limit_event arrives once per process (S8)"
    );
    assert_eq!(result.subtype, "success");
    assert!(!result.is_error);
    let cost_first = result.total_cost_usd.expect("result has total_cost_usd");
    assert_eq!(
        exit_code(s).await,
        Some(0),
        "exit 0 after a successful last turn (S5)"
    );

    let mut s = start(dir.path(), &command(dir.path(), Session::Resume(session))).await;
    s.handle
        .send_user("Reply with exactly: PONG AGAIN")
        .expect("send");
    let (_, result) = until_result(&mut s).await;
    assert_eq!(result.subtype, "success", "--resume keeps the session (S7)");
    let cost_resumed = result.total_cost_usd.expect("result has total_cost_usd");
    assert!(
        cost_resumed > cost_first,
        "total_cost_usd is cumulative across --resume (S7): {cost_first} then {cost_resumed}"
    );
    exit_code(s).await;
}

/// A mid-turn stdin message is folded into the running turn (S3), and an
/// interrupt gets a receipt and ends the turn as aborted (S4).
#[tokio::test]
#[ignore = "spends real tokens against the real claude binary; includes wall-clock timing assertions"]
async fn mid_turn_fold_and_interrupt() {
    if !live() {
        return;
    }
    let dir = tempfile::tempdir().expect("tempdir");
    let mut s = start(
        dir.path(),
        &command(dir.path(), Session::New(Uuid::new_v4())),
    )
    .await;

    s.handle
        .send_user("Use the Bash tool to run `sleep 5`, then reply with exactly: DONE")
        .expect("send");
    until_tool_use(&mut s).await;
    let extra = "Also include the word BANANA in your final reply.";
    s.handle.send_user(extra).expect("send mid-turn");
    let (events, result) = until_result(&mut s).await;
    assert!(
        replayed(&events, extra).is_some(),
        "the mid-turn message is acked before the turn's result: folded, not queued (S3)"
    );
    assert!(
        result.result.as_deref().unwrap_or("").contains("BANANA"),
        "the folded message shaped this turn's reply: {:?}",
        result.result
    );

    s.handle
        .send_user("Use the Bash tool to run `sleep 30`, then reply with exactly: DONE")
        .expect("send");
    until_tool_use(&mut s).await;
    let asked = Instant::now();
    s.handle
        .interrupt(Duration::from_secs(10))
        .await
        .expect("interrupt gets a correlated control_response receipt (S4)");
    let (_, result) = until_result(&mut s).await;
    assert!(
        asked.elapsed() < Duration::from_secs(15),
        "interrupt ends the turn promptly"
    );
    assert_eq!(result.subtype, "error_during_execution");
    // `aborted_tools` once the tool is running, `aborted_streaming` if the
    // interrupt lands while the tool call is still streaming. Bridle only
    // records it.
    let reason = result.terminal_reason.as_deref().unwrap_or("");
    assert!(reason.starts_with("aborted_"), "terminal_reason {reason:?}");
    assert_eq!(
        exit_code(s).await,
        Some(1),
        "exit 1 after an errored last turn (S5)"
    );
}

/// `--max-budget-usd` ends turns with its own result subtype, and the process
/// stays up (docs/spikes/02-budget-cap-findings.md).
#[tokio::test]
#[ignore = "spends real tokens against the real claude binary"]
async fn budget_cap_has_its_own_result_subtype() {
    if !live() {
        return;
    }
    let dir = tempfile::tempdir().expect("tempdir");
    let mut cmd = command(dir.path(), Session::New(Uuid::new_v4()));
    cmd.max_budget_usd = Some(0.0001);
    let mut s = start(dir.path(), &cmd).await;

    s.handle.send_user("Reply with exactly: OK").expect("send");
    let (_, result) = until_result(&mut s).await;
    assert_eq!(result.subtype, "error_max_budget_usd");
    assert_eq!(result.terminal_reason.as_deref(), Some("budget_exhausted"));

    s.handle.send_user("Reply with exactly: OK").expect("send");
    let (_, again) = until_result(&mut s).await;
    assert_eq!(
        again.subtype, "error_max_budget_usd",
        "later turns fail the same way"
    );
    assert_eq!(
        again.total_cost_usd, result.total_cost_usd,
        "without calling the model"
    );
    exit_code(s).await;
}
