mod agent;
mod events;
mod log;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context;
use clap::{Parser, Subcommand};
use nix::sys::signal::Signal;
use serde_json::{Value, json};
use uuid::Uuid;

use agent::{AgentProcess, SpawnConfig};
use events::{Event, EventKind};
use log::Transcript;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Clone, Copy, PartialEq)]
enum Cmd {
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    /// Usage report over all fixtures (no claude calls).
    S8,
    /// Capabilities / init report from s1 (no claude calls).
    S9,
    /// S8 follow-up: query usage/limits on demand via control requests.
    S8probe,
    /// Parse every fixture: distinct type/subtype counts + any parse fallbacks.
    Catalogue,
    S10,
    S11,
    All,
}

const TURN: Duration = Duration::from_secs(90);
const SLEEP_PROMPT: &str = "Run `sleep 20` with the Bash tool, then reply DONE.";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let all = [Cmd::S1, Cmd::S2, Cmd::S3, Cmd::S4, Cmd::S5, Cmd::S6, Cmd::S7, Cmd::S10, Cmd::S11, Cmd::S9, Cmd::S8];
    let run: Vec<Cmd> = if cli.cmd == Cmd::All { all.to_vec() } else { vec![cli.cmd] };
    for c in run {
        let r = match c {
            Cmd::S1 => s1().await,
            Cmd::S2 => s2().await,
            Cmd::S3 => s3().await,
            Cmd::S4 => s4().await,
            Cmd::S5 => s5().await,
            Cmd::S6 => s6().await,
            Cmd::S7 => s7().await,
            Cmd::S8 => s8(),
            Cmd::S9 => s9(),
            Cmd::S8probe => s8probe().await,
            Cmd::Catalogue => catalogue(),
            Cmd::S10 => s10().await,
            Cmd::S11 => s11().await,
            Cmd::All => unreachable!(),
        };
        if let Err(e) = r {
            println!("!! scenario failed: {e:#}");
        }
    }
    Ok(())
}

// ---------- helpers ----------

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn scratch(name: &str) -> anyhow::Result<PathBuf> {
    let dir = std::env::temp_dir().join("bridle-spike").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

struct Scenario {
    name: &'static str,
    dir: PathBuf,
    log: Transcript,
    start: Instant,
}

impl Scenario {
    fn new(name: &'static str) -> anyhow::Result<Self> {
        println!("\n=== {name} ===");
        let dir = scratch(name)?;
        let path = fixtures_dir().join(format!("{name}.jsonl"));
        let _ = std::fs::remove_file(&path);
        let start = Instant::now();
        let log = Transcript::open(&path, start)?;
        Ok(Self { name, dir, log, start })
    }

    fn cfg(&self, session_id: Option<Uuid>, extra: &[&str]) -> SpawnConfig {
        SpawnConfig {
            cwd: self.dir.clone(),
            session_id,
            model: "haiku".into(),
            extra_args: extra.iter().map(|s| s.to_string()).collect(),
            env: vec![("BRIDLE_AGENT".into(), format!("spike-01-{}", self.name))],
            transcript: self.log.clone(),
        }
    }

    fn say(&self, msg: impl AsRef<str>) {
        let msg = msg.as_ref();
        println!("[{:>6}ms] {msg}", self.start.elapsed().as_millis());
        self.log.note(msg);
    }
}

fn short(ev: &Event) -> String {
    let detail = match &ev.kind {
        EventKind::SystemInit(i) => format!(
            "session={} model={:?} tools={} caps={}",
            i.session_id,
            i.model,
            i.tools.len(),
            i.capabilities.as_ref().map(|c| c.to_string()).unwrap_or_default()
        ),
        EventKind::Assistant(m) => {
            let tools: Vec<_> = m.tool_uses().into_iter().map(|(_, n, i)| format!("{n}({i})")).collect();
            format!("text={:?} tool_use={tools:?}", m.text())
        }
        EventKind::User(m) => trunc(&m.message.content.to_string(), 160),
        EventKind::Result(r) => format!(
            "subtype={} is_error={} turns={:?} result={:?} cost={:?} reason={:?} usage={:?}",
            r.subtype, r.is_error, r.num_turns, r.result, r.total_cost_usd, r.terminal_reason, r.usage
        ),
        EventKind::RateLimit(r) => r.rate_limit_info.to_string(),
        EventKind::ControlResponse(_) => trunc(&ev.raw, 200),
        EventKind::Unparsed { error, .. } => format!("UNPARSED: {error}"),
        _ => trunc(&ev.raw, 200),
    };
    format!("{:>6}ms {:<24} {detail}", ev.t_ms, ev.label())
}

fn trunc(s: &str, n: usize) -> String {
    if s.len() <= n { s.to_string() } else { format!("{}…", &s[..s.floor_char_boundary(n)]) }
}

fn print_events(evs: &[Event]) {
    for ev in evs {
        println!("    {}", short(ev));
    }
}

fn sh(cmd: &str) -> String {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|e| format!("<{e}>"))
}

/// Every process whose pgid is `pgid`, plus any `sleep 20` anywhere (with its pgid).
fn process_snapshot(pgid: i32) -> String {
    let group = sh(&format!("ps -axo pid=,pgid=,ppid=,command= | awk '$2=={pgid}'"));
    let sleeps = sh("ps -axo pid=,pgid=,ppid=,command= | grep -E 'sleep 20$' | grep -v grep");
    format!("group {pgid}:\n{group}\n  sleep 20 anywhere:\n{sleeps}")
}

/// Polls until a `sleep 20` process exists: the Bash tool can take seconds
/// to actually start it (first-call shell snapshot sources the login rc).
async fn wait_for_sleep(sc: &Scenario, timeout: Duration) -> bool {
    let t0 = Instant::now();
    while t0.elapsed() < timeout {
        if !sh("pgrep -f '^sleep 20$'").is_empty() {
            sc.say(format!("sleep 20 running {}ms after tool_use", t0.elapsed().as_millis()));
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    sc.say("sleep 20 never appeared");
    false
}

fn is_tool_use(ev: &Event) -> bool {
    matches!(&ev.kind, EventKind::Assistant(m) if !m.tool_uses().is_empty())
}

// ---------- scenarios ----------

async fn s1() -> anyhow::Result<()> {
    let sc = Scenario::new("s1")?;
    let mut a = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &["--name", "spike-s1"])).await?;
    a.send_user("Reply with exactly: OK")?;
    let r = a.wait_for_result(TURN).await?;
    let first = a.history.first().map(|e| e.t_ms);
    let result_t = a.history.last().map(|e| e.t_ms);
    print_events(&a.history);
    let (status, tail) = a.close_stdin(Duration::from_secs(30)).await?;
    print_events(&tail);
    sc.say(format!(
        "first event {first:?}ms, result {result_t:?}ms, exit {:?} at {}ms, result text {:?}",
        status.code(),
        sc.start.elapsed().as_millis(),
        r.result
    ));
    Ok(())
}

async fn s2() -> anyhow::Result<()> {
    let sc = Scenario::new("s2")?;
    let mut a = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &[])).await?;
    a.send_user("Reply with exactly: OK")?;
    let r1 = a.wait_for_result(TURN).await?;
    sc.say(format!("turn 1 result {:?} session {}", r1.result, r1.session_id));
    sc.say("idle 15s");
    let idle = a.drain(Duration::from_secs(15)).await;
    sc.say(format!("events during idle: {}", idle.len()));
    print_events(&idle);
    let mark = a.history.len();
    a.send_user("Reply with exactly: TWO")?;
    let r2 = a.wait_for_result(TURN).await?;
    print_events(&a.history[mark..]);
    let inits = a.history.iter().filter(|e| matches!(e.kind, EventKind::SystemInit(_))).count();
    sc.say(format!(
        "turn 2 result {:?} session {} (same={}); system/init count {inits}",
        r2.result,
        r2.session_id,
        r2.session_id == r1.session_id
    ));
    let (status, _) = a.close_stdin(Duration::from_secs(30)).await?;
    sc.say(format!("exit {:?}", status.code()));
    Ok(())
}

async fn s3() -> anyhow::Result<()> {
    let sc = Scenario::new("s3")?;
    let mut a = AgentProcess::spawn(sc.cfg(
        Some(Uuid::new_v4()),
        &["--allowedTools", "Bash(sleep *)", "--replay-user-messages"],
    ))
    .await?;
    a.send_user("Run `sleep 8` with the Bash tool, then reply DONE.")?;
    let early = a.drain(Duration::from_secs(2)).await;
    print_events(&early);
    sc.say("sending second message mid-turn");
    a.send_user("Also reply BANANA")?;
    let mut results = 0;
    while results < 2 {
        match a.next_event_timeout(Duration::from_secs(45)).await {
            Some(ev) => {
                println!("    {}", short(&ev));
                if matches!(ev.kind, EventKind::Result(_)) {
                    results += 1;
                }
            }
            None => {
                sc.say(format!("no more events after {results} result(s)"));
                break;
            }
        }
    }
    sc.say(format!("results seen: {results}"));
    let (status, tail) = a.close_stdin(Duration::from_secs(30)).await?;
    print_events(&tail);
    sc.say(format!("exit {:?}", status.code()));
    Ok(())
}

async fn s4() -> anyhow::Result<()> {
    let sc = Scenario::new("s4")?;
    let mut a = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &["--allowedTools", "Bash(sleep *)", "--replay-user-messages"])).await?;

    // (a) plain interrupt mid-tool
    a.send_user(SLEEP_PROMPT)?;
    let tu = a.wait_for(TURN, is_tool_use).await?;
    sc.say(format!("tool_use seen: {}", short(&tu)));
    wait_for_sleep(&sc, Duration::from_secs(20)).await;
    sc.say(format!("before interrupt: {}", process_snapshot(a.pid)));
    let mark = a.history.len();
    let receipt = a.interrupt(false).await?;
    sc.say(format!("receipt: {receipt}"));
    let r = a.wait_for_result(Duration::from_secs(30)).await;
    print_events(&a.history[mark..]);
    sc.say(format!("interrupted turn result: {:?}", r.as_ref().map(|r| (&r.subtype, &r.terminal_reason, r.is_error))));
    tokio::time::sleep(Duration::from_secs(1)).await;
    sc.say(format!("after interrupt: {}", process_snapshot(a.pid)));

    let mark = a.history.len();
    a.send_user("Reply with exactly: AFTER")?;
    let r = a.wait_for_result(TURN).await?;
    print_events(&a.history[mark..]);
    sc.say(format!("post-interrupt turn: {:?} {:?}", r.subtype, r.result));

    // (b) cancel_queued: queue a message behind a running tool call, then interrupt.
    sc.say("--- part b: cancel_queued ---");
    let mark = a.history.len();
    a.send_user(SLEEP_PROMPT)?;
    a.wait_for(TURN, is_tool_use).await?;
    wait_for_sleep(&sc, Duration::from_secs(20)).await;
    a.send_user("Reply with exactly: QUEUED")?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let receipt = a.interrupt(true).await?;
    sc.say(format!("receipt (cancel_queued): {receipt}"));
    a.drain(Duration::from_secs(20)).await;
    print_events(&a.history[mark..]);
    let ran_queued = a.history[mark..].iter().any(|e| match &e.kind {
        EventKind::Assistant(m) => m.text().contains("QUEUED"),
        _ => false,
    });
    sc.say(format!("queued message ran: {ran_queued}"));
    sc.say(format!("after b: {}", process_snapshot(a.pid)));

    // (c) control for (b): same queueing, interrupt WITHOUT cancel_queued.
    sc.say("--- part c: queued message + plain interrupt ---");
    let mark = a.history.len();
    a.send_user(SLEEP_PROMPT)?;
    a.wait_for(TURN, is_tool_use).await?;
    wait_for_sleep(&sc, Duration::from_secs(20)).await;
    a.send_user("Reply with exactly: QUEUED2")?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let receipt = a.interrupt(false).await?;
    sc.say(format!("receipt (plain): {receipt}"));
    a.drain(Duration::from_secs(20)).await;
    print_events(&a.history[mark..]);
    let ran_queued = a.history[mark..].iter().any(|e| match &e.kind {
        EventKind::Assistant(m) => m.text().contains("QUEUED2"),
        _ => false,
    });
    sc.say(format!("queued message ran: {ran_queued}"));
    let (status, _) = a.close_stdin(Duration::from_secs(30)).await?;
    sc.say(format!("exit {:?}", status.code()));
    Ok(())
}

async fn s5() -> anyhow::Result<()> {
    let sc = Scenario::new("s5")?;
    let mut a = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &[])).await?;
    a.send_user("Reply with exactly: OK")?;
    a.wait_for_result(TURN).await?;
    a.drain(Duration::from_secs(3)).await;
    let t0 = Instant::now();
    let (status, tail) = a.close_stdin(Duration::from_secs(30)).await?;
    sc.say(format!(
        "exit {:?} (signal {:?}) {}ms after close; {} events after close",
        status.code(),
        std::os::unix::process::ExitStatusExt::signal(&status),
        t0.elapsed().as_millis(),
        tail.len()
    ));
    print_events(&tail);

    // (b) exit code when the last turn was interrupted (S4 run 1 exited 1).
    sc.say("--- part b: close stdin after an interrupted turn ---");
    let mut b = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &["--allowedTools", "Bash(sleep *)"])).await?;
    b.send_user(SLEEP_PROMPT)?;
    b.wait_for(TURN, is_tool_use).await?;
    wait_for_sleep(&sc, Duration::from_secs(20)).await;
    b.interrupt(false).await?;
    let r = b.wait_for_result(Duration::from_secs(30)).await?;
    sc.say(format!("last result: {} is_error={}", r.subtype, r.is_error));
    let (status, _) = b.close_stdin(Duration::from_secs(30)).await?;
    sc.say(format!("exit {:?}", status.code()));
    Ok(())
}

async fn s6() -> anyhow::Result<()> {
    let sc = Scenario::new("s6")?;
    let mut a = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &["--allowedTools", "Bash(sleep *)"])).await?;
    a.send_user(SLEEP_PROMPT)?;
    a.wait_for(TURN, is_tool_use).await?;
    wait_for_sleep(&sc, Duration::from_secs(20)).await;
    sc.say(format!("before SIGTERM: {}", process_snapshot(a.pid)));
    let mark = a.history.len();
    let t0 = Instant::now();
    a.terminate_group(Signal::SIGTERM)?;
    let status = loop {
        if let Some(s) = a.try_wait()? {
            break Some(s);
        }
        if t0.elapsed() > Duration::from_secs(3) {
            break None;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    let status = match status {
        Some(s) => s,
        None => {
            sc.say("still alive after 3s, SIGKILL");
            a.terminate_group(Signal::SIGKILL)?;
            a.wait().await?
        }
    };
    let exit_ms = t0.elapsed().as_millis();
    a.drain(Duration::from_millis(500)).await;
    print_events(&a.history[mark..]);
    sc.say(format!(
        "exit code {:?} signal {:?} after {exit_ms}ms",
        status.code(),
        std::os::unix::process::ExitStatusExt::signal(&status)
    ));
    tokio::time::sleep(Duration::from_millis(500)).await;
    sc.say(format!("after: {}", process_snapshot(a.pid)));

    // (b) SIGKILL the group: claude gets no chance to clean up its tool
    // processes, which live in their own process group.
    sc.say("--- part b: killpg(SIGKILL) ---");
    let mut b = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &["--allowedTools", "Bash(sleep *)"])).await?;
    b.send_user(SLEEP_PROMPT)?;
    b.wait_for(TURN, is_tool_use).await?;
    wait_for_sleep(&sc, Duration::from_secs(20)).await;
    sc.say(format!("before SIGKILL: {}", process_snapshot(b.pid)));
    b.terminate_group(Signal::SIGKILL)?;
    let status = b.wait().await?;
    sc.say(format!("exit signal {:?}", std::os::unix::process::ExitStatusExt::signal(&status)));
    tokio::time::sleep(Duration::from_millis(500)).await;
    let orphans = sh("pgrep -f '^sleep 20$'");
    sc.say(format!("after SIGKILL: {} orphans: {orphans:?}", process_snapshot(b.pid)));
    if !orphans.is_empty() {
        sh("pkill -f '^sleep 20$'");
        sc.say("cleaned up orphaned sleep");
    }
    Ok(())
}

async fn s7() -> anyhow::Result<()> {
    let sc = Scenario::new("s7")?;
    let x = Uuid::new_v4();
    sc.say(format!("process A, session {x}"));
    let mut a = AgentProcess::spawn(sc.cfg(Some(x), &[])).await?;
    a.send_user("Remember the word PELICAN. Reply OK.")?;
    let r = a.wait_for_result(TURN).await?;
    sc.say(format!("A: {:?} session {}", r.result, r.session_id));
    a.close_stdin(Duration::from_secs(30)).await?;

    let q = "What word did I ask you to remember? One word.";
    sc.say("process B: --resume X");
    let mut b = AgentProcess::spawn(sc.cfg(None, &["--resume", &x.to_string()])).await?;
    b.send_user(q)?;
    let r = b.wait_for_result(TURN).await?;
    let init_sid = b.history.iter().find_map(|e| match &e.kind {
        EventKind::SystemInit(i) => Some(i.session_id.clone()),
        _ => None,
    });
    sc.say(format!("B: {:?} result session {} init session {init_sid:?} (same as X: {})", r.result, r.session_id, r.session_id == x.to_string()));
    b.close_stdin(Duration::from_secs(30)).await?;

    let y = Uuid::new_v4();
    sc.say(format!("process C: --resume X --session-id {y}"));
    let mut c = AgentProcess::spawn(sc.cfg(Some(y), &["--resume", &x.to_string()])).await?;
    c.send_user(q)?;
    match c.wait_for_result(Duration::from_secs(60)).await {
        Ok(r) => sc.say(format!("C: {:?} session {}", r.result, r.session_id)),
        Err(e) => sc.say(format!("C: no result: {e:#}")),
    }
    let (status, tail) = c.close_stdin(Duration::from_secs(30)).await?;
    print_events(&tail);
    sc.say(format!("C exit {:?}; stderr in fixture", status.code()));
    Ok(())
}

fn read_fixture(name: &str) -> anyhow::Result<Vec<(String, Event)>> {
    let text = std::fs::read_to_string(fixtures_dir().join(format!("{name}.jsonl")))?;
    let mut out = Vec::new();
    for l in text.lines() {
        let v: Value = serde_json::from_str(l)?;
        if v["dir"] == "out" {
            let line = v["line"].as_str().unwrap_or_default().to_string();
            out.push((name.to_string(), Event::parse(v["t_ms"].as_u64().unwrap_or(0), line)));
        }
    }
    Ok(out)
}

fn s8() -> anyhow::Result<()> {
    println!("\n=== s8 (usage report) ===");
    for name in ["s1", "s2", "s3", "s4", "s5", "s6", "s7", "s10", "s11"] {
        let Ok(evs) = read_fixture(name) else { continue };
        println!("  {name}:");
        for (_, ev) in &evs {
            match &ev.kind {
                EventKind::Result(r) => println!(
                    "    result {:<22} turns={:?} cost={:?} usage={:?}\n      modelUsage={}",
                    r.subtype,
                    r.num_turns,
                    r.total_cost_usd,
                    r.usage,
                    r.model_usage.as_ref().map(|v| v.to_string()).unwrap_or_default()
                ),
                EventKind::RateLimit(r) => println!("    rate_limit_event {}", r.rate_limit_info),
                _ => {}
            }
        }
    }
    Ok(())
}

/// Subtypes found by grepping the 2.1.283 binary; request shapes are guesses
/// (bare subtype), so an error reply is itself a finding.
async fn s8probe() -> anyhow::Result<()> {
    let sc = Scenario::new("s8probe")?;
    let mut a = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &[])).await?;
    for subtype in ["get_usage", "get_context_usage", "get_session_cost", "get_status"] {
        let rx = a.control(json!({ "subtype": subtype }))?;
        match tokio::time::timeout(Duration::from_secs(20), rx).await {
            Ok(Ok(v)) => sc.say(format!("{subtype}: {}", trunc(&v.to_string(), 1500))),
            _ => sc.say(format!("{subtype}: no response in 20s")),
        }
    }
    let (status, tail) = a.close_stdin(Duration::from_secs(30)).await?;
    sc.say(format!("exit {:?}, {} events total", status.code(), tail.len()));
    Ok(())
}

fn catalogue() -> anyhow::Result<()> {
    let mut counts: std::collections::BTreeMap<String, (usize, String)> = Default::default();
    let mut names: Vec<String> = std::fs::read_dir(fixtures_dir())?
        .filter_map(|e| e.ok()?.file_name().into_string().ok()?.strip_suffix(".jsonl").map(String::from))
        .collect();
    names.sort();
    for name in names {
        for (_, ev) in read_fixture(&name)? {
            match &ev.kind {
                EventKind::Unparsed { error, .. } => println!("  UNPARSED {name}: {error}"),
                EventKind::NotJson => println!("  NOT JSON {name}: {}", trunc(&ev.raw, 120)),
                _ => {}
            }
            let e = counts.entry(ev.label()).or_insert((0, name.clone()));
            e.0 += 1;
        }
    }
    for (label, (n, first)) in counts {
        println!("  {label:<32} {n:>4}  (first in {first})");
    }
    Ok(())
}

fn s9() -> anyhow::Result<()> {
    println!("\n=== s9 (init report from s1) ===");
    for (_, ev) in read_fixture("s1")? {
        if let EventKind::SystemInit(i) = &ev.kind {
            println!("  init bytes: {}", ev.raw.len());
            println!("  keys: {:?}", ev.value.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            println!("  model: {:?}", i.model);
            println!("  capabilities: {}", i.capabilities.as_ref().map(|c| c.to_string()).unwrap_or("<absent>".into()));
            println!("  tools ({}): {:?}", i.tools.len(), i.tools);
        }
    }
    Ok(())
}

async fn s10() -> anyhow::Result<()> {
    let sc = Scenario::new("s10")?;
    // ~1,500 tokens of filler plus one fact only the file states.
    let mut role = String::from(
        "# Role: spike worker\n\nYou are a worker agent in the bridle spike. \
         The project codename is MARZIPAN-7. If asked for the codename, answer with it exactly.\n\n## Background\n\n",
    );
    for i in 1..=60 {
        role.push_str(&format!(
            "{i}. Workers keep their changes small, run the tests before reporting, and describe what they changed in one short paragraph.\n"
        ));
    }
    let role_path = sc.dir.join("role.md");
    std::fs::write(&role_path, &role)?;
    sc.say(format!("role.md: {} bytes (~{} tokens)", role.len(), role.len() / 4));
    let rp = role_path.to_string_lossy().to_string();
    let args = ["--append-system-prompt-file", rp.as_str()];

    let mut a = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &args)).await?;
    a.send_user("What is the project codename? Reply with the codename only.")?;
    let r1 = a.wait_for_result(TURN).await?;
    sc.say(format!("A1: {:?} usage {:?}", r1.result, r1.usage));
    a.send_user("Reply with exactly: TWO")?;
    let r2 = a.wait_for_result(TURN).await?;
    sc.say(format!("A2: {:?} usage {:?}", r2.result, r2.usage));
    a.close_stdin(Duration::from_secs(30)).await?;

    let mut b = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &args)).await?;
    b.send_user("Reply with exactly: B")?;
    let r = b.wait_for_result(TURN).await?;
    sc.say(format!("B1: {:?} usage {:?}", r.result, r.usage));
    b.close_stdin(Duration::from_secs(30)).await?;

    // (b) same pair with --exclude-dynamic-system-prompt-sections: does the
    // cross-process cache miss shrink?
    let args_x = ["--append-system-prompt-file", rp.as_str(), "--exclude-dynamic-system-prompt-sections"];
    for label in ["C1 (excl-dynamic)", "D1 (excl-dynamic)"] {
        let mut p = AgentProcess::spawn(sc.cfg(Some(Uuid::new_v4()), &args_x)).await?;
        p.send_user("Reply with exactly: B")?;
        let r = p.wait_for_result(TURN).await?;
        sc.say(format!("{label}: {:?} usage {:?}", r.result, r.usage));
        p.close_stdin(Duration::from_secs(30)).await?;
    }

    // (c) bridle's real case: same role file and message, different cwd per
    // worker. A fresh nonce keeps earlier runs' cache entries out of it.
    let nonce = &Uuid::new_v4().to_string()[..8];
    let msg = format!("Reply with exactly: {nonce}");
    for (flag, label) in [(false, "default"), (true, "excl-dynamic")] {
        for wt in ["wt-1", "wt-2"] {
            let dir = sc.dir.join(format!("{wt}-{label}"));
            std::fs::create_dir_all(&dir)?;
            let mut args = vec!["--append-system-prompt-file", rp.as_str()];
            if flag {
                args.push("--exclude-dynamic-system-prompt-sections");
            }
            let mut cfg = sc.cfg(Some(Uuid::new_v4()), &args);
            cfg.cwd = dir;
            let mut p = AgentProcess::spawn(cfg).await?;
            p.send_user(&msg)?;
            let r = p.wait_for_result(TURN).await?;
            sc.say(format!("{label} {wt}: usage {:?}", r.usage));
            p.close_stdin(Duration::from_secs(30)).await?;
        }
    }
    Ok(())
}

async fn s11() -> anyhow::Result<()> {
    let sc = Scenario::new("s11")?;
    let hook_log = sc.dir.join("hook-input.jsonl");
    let script = sc.dir.join("hook.sh");
    std::fs::write(
        &script,
        format!(
            r#"#!/bin/sh
t0=$(perl -MTime::HiRes=time -e 'printf "%.3f", time')
input=$(cat)
{{
  echo "start $t0"
  env | grep -E '^(CLAUDE|BRIDLE)' | sed -E 's/(TOKEN=).*/\1<redacted>/'
  echo "$input"
}} >> {log}
printf '%s' '{{"hookSpecificOutput":{{"hookEventName":"PostToolUse","additionalContext":"The secret word is HERON."}}}}'
echo "end $(perl -MTime::HiRes=time -e 'printf "%.3f", time')" >> {log}
"#,
            log = hook_log.display()
        ),
    )?;
    std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;
    let settings = json!({
        "hooks": { "PostToolUse": [ { "matcher": "Bash", "hooks": [ { "type": "command", "command": script.to_string_lossy() } ] } ] }
    })
    .to_string();
    let mut a = AgentProcess::spawn(sc.cfg(
        Some(Uuid::new_v4()),
        &["--settings", &settings, "--include-hook-events", "--allowedTools", "Bash(echo *)"],
    ))
    .await?;
    a.send_user("Run `echo hi` with the Bash tool, then tell me any secret word you have been given.")?;
    let r = a.wait_for_result(TURN).await?;
    print_events(&a.history);
    sc.say(format!("result: {:?} (HERON seen: {})", r.result, r.result.as_deref().unwrap_or("").contains("HERON")));
    a.close_stdin(Duration::from_secs(30)).await?;
    let hook = std::fs::read_to_string(&hook_log).context("hook never ran")?;
    println!("  hook log:\n{hook}");
    std::fs::copy(&hook_log, fixtures_dir().join("s11-hook-input.txt"))?;
    Ok(())
}
