//! `bridle session orchestrator|advisor`: the command lines, env, tags and files the launch
//! scripts produced. `claude` and `tmux` are stubs on PATH; no real claude, no real orchestrator.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Run {
    out: Output,
    home: tempfile::TempDir,
    /// One argument per line, then a line of the env the stub saw.
    claude: String,
    tmux: String,
    /// BRIDLE_HOME's files as claude saw them, while it ran.
    files_during: String,
}

fn stub(dir: &Path, name: &str, body: &str) {
    let p = dir.join(name);
    fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

fn session(args: &[&str], envs: &[(&str, &str)], claude_exit: i32) -> Run {
    let bin = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let rec = bin.path().join("claude.rec");
    let tmux_rec = bin.path().join("tmux.rec");
    stub(
        bin.path(),
        "claude",
        &format!(
            "for a in \"$@\"; do printf '%s\\n' \"$a\"; done > {rec}\n\
             echo \"AS=$BRIDLE_AS PROJECT=$BRIDLE_PROJECT ADVISOR=$BRIDLE_ADVISOR_NAME\" >> {rec}\n\
             ls {home} > {rec}.files\nexit {claude_exit}",
            rec = rec.display(),
            home = home.path().display()
        ),
    );
    stub(
        bin.path(),
        "tmux",
        &format!("echo \"$@\" >> {}", tmux_rec.display()),
    );
    let cwd: PathBuf = tempfile::tempdir().unwrap().keep();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bridle"));
    cmd.args(["session"])
        .args(args)
        .current_dir(cwd)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.path().display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("BRIDLE_HOME", home.path())
        .env("BRIDLE_AGENT_ID", "a-test")
        .env("BRIDLE_LAUNCHER_TEST", "1")
        .env("TMUX_PANE", "%9")
        .env_remove("BRIDLE_PROJECT")
        .env_remove("BRIDLE_URL")
        .env_remove("BRIDLE_TOKEN")
        .env_remove("BRIDLE_SESSION_SUFFIX");
    for (k, v) in envs {
        // "CWD" is not an env var: it moves the run into that folder.
        if *k == "CWD" {
            cmd.current_dir(v);
        } else {
            cmd.env(k, v);
        }
    }
    let out = cmd.output().unwrap();
    Run {
        out,
        home,
        claude: fs::read_to_string(&rec).unwrap_or_default(),
        tmux: fs::read_to_string(&tmux_rec).unwrap_or_default(),
        files_during: fs::read_to_string(bin.path().join("claude.rec.files")).unwrap_or_default(),
    }
}

fn lines(r: &Run) -> Vec<&str> {
    r.claude.lines().collect()
}

#[test]
fn orchestrator_command_line_matches_the_script() {
    let r = session(&["--project", "bridle", "orchestrator"], &[], 0);
    let l = lines(&r);
    assert_eq!(l[0], "--settings");
    assert!(l[1].contains("bridle orchestrator note-session") && l[1].contains("disableWorkflows"));
    assert_eq!(
        &l[2..7],
        [
            "--strict-mcp-config",
            "--name",
            "orch-bridle",
            "--remote-control",
            "orch-bridle"
        ]
    );
    assert_eq!(
        l[7],
        "Run `bridle prime orchestrator` and follow what it prints."
    );
    assert_eq!(l[8], "AS=orchestrator PROJECT=bridle ADVISOR=");
    assert!(
        r.tmux.contains("set-option -p -t %9 @bridle orchestrator"),
        "{}",
        r.tmux
    );
    let pid = fs::read_to_string(r.home.path().join("orchestrator.pid")).unwrap();
    assert!(pid.split_whitespace().count() >= 3, "{pid}");
    let exits = fs::read_to_string(r.home.path().join("orchestrator.exits")).unwrap();
    assert!(
        exits.contains("orchestrator claude ended: exit 0"),
        "{exits}"
    );
}

#[test]
fn orchestrator_uses_project_suffix_and_passes_extra_args() {
    let r = session(
        &["--project", "meta-notes", "orchestrator", "--resume", "x"],
        &[("BRIDLE_SESSION_SUFFIX", "nuc")],
        0,
    );
    let l = lines(&r);
    assert_eq!(l[4], "orch-meta-notes-nuc");
    assert_eq!(
        &l[6..10],
        [
            "orch-meta-notes-nuc",
            "--resume",
            "x",
            "Run `bridle prime orchestrator` and follow what it prints."
        ]
    );
    assert!(l[10].contains("PROJECT=meta-notes"));
}

#[test]
fn orchestrator_records_a_nonzero_exit() {
    let r = session(&["--project", "p", "orchestrator"], &[], 3);
    assert_eq!(r.out.status.code(), Some(3));
    let exits = fs::read_to_string(r.home.path().join("orchestrator.exits")).unwrap();
    assert!(exits.contains("exit 3"), "{exits}");
}

#[test]
fn advisor_unnamed_has_a_pid_file_only_while_running() {
    let r = session(&["--project", "p", "advisor"], &[], 0);
    let l = lines(&r);
    assert_eq!(
        &l[3..7],
        ["--name", "advisor-p", "--remote-control", "advisor-p"]
    );
    assert!(l[7].starts_with("Run `bridle prime advisor`"));
    assert_eq!(l[8], "AS=advisor PROJECT=p ADVISOR=");
    assert!(r.tmux.contains("@bridle advisor\n"), "{}", r.tmux);
    assert!(
        r.files_during.contains("advisor-p.pid"),
        "{}",
        r.files_during
    );
    assert!(!r.home.path().join("advisor-p.pid").exists());
}

#[test]
fn aide_signs_as_aide_and_tags_its_pane() {
    let r = session(&["--project", "p", "aide"], &[], 0);
    let l = lines(&r);
    assert_eq!(&l[3..7], ["--name", "aide-p", "--remote-control", "aide-p"]);
    assert_eq!(l[7], "Run `bridle prime aide` and follow what it prints.");
    assert_eq!(l[8], "AS=aide PROJECT=p ADVISOR=");
    assert!(r.tmux.contains("@bridle aide\n"), "{}", r.tmux);
}

#[test]
fn named_advisor_signs_tags_and_skips_the_pid_file() {
    let r = session(
        &["--project", "p", "advisor", "alice", "--model", "m"],
        &[("BRIDLE_SESSION_SUFFIX", "nuc")],
        0,
    );
    let l = lines(&r);
    assert_eq!(l[4], "advisor-alice-p-nuc");
    assert_eq!(l[7], "--model");
    assert_eq!(l[8], "m");
    assert_eq!(l[10], "AS=advisor PROJECT=p ADVISOR=alice");
    assert!(r.tmux.contains("@bridle advisor-alice"), "{}", r.tmux);
    assert!(!r.home.path().join("advisor-p.pid").exists());
}

/// `bridle advisor start`, with a stub tmux that records its arguments.
fn advisor_start(tmux_env: bool, panes: &str, config: &str, agent: bool) -> (Output, String) {
    let bin = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    fs::write(home.path().join("config.toml"), config).unwrap();
    let rec = bin.path().join("tmux.rec");
    stub(
        bin.path(),
        "tmux",
        &format!(
            "echo \"$@\" >> {rec}\n[ \"$1\" = list-panes ] && printf '%b' '{panes}'\nexit 0",
            rec = rec.display()
        ),
    );
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bridle"));
    cmd.args(["advisor", "start", "fred"])
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.path().display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("BRIDLE_HOME", home.path())
        .env("BRIDLE_PROJECT", "bridle")
        .env_remove("BRIDLE_AGENT_ID")
        .env_remove("TMUX");
    if agent {
        cmd.env("BRIDLE_AGENT_ID", "a-test");
    }
    if tmux_env {
        cmd.env("TMUX", "/tmp/tmux-stub,1,0");
    }
    let out = cmd.output().unwrap();
    (out, fs::read_to_string(&rec).unwrap_or_default())
}

#[test]
fn advisor_start_splits_the_orchestrator_window() {
    let (out, tmux) = advisor_start(true, "%1 \\n%4 orchestrator\\n", "", false);
    assert!(out.status.success(), "{out:?}");
    assert!(
        tmux.contains("split-window -d -t %4 bridle session advisor --project bridle fred"),
        "{tmux}"
    );
}

#[test]
fn advisor_start_window_mode_opens_a_window() {
    let (out, tmux) = advisor_start(
        true,
        "%4 orchestrator\\n",
        "[tmux]\nadvisor_pane = \"window\"\n",
        false,
    );
    assert!(out.status.success(), "{out:?}");
    assert!(
        tmux.contains("new-window -d bridle session advisor --project bridle fred"),
        "{tmux}"
    );
}

#[test]
fn advisor_start_outside_tmux_prints_the_command() {
    let (out, tmux) = advisor_start(false, "", "", false);
    assert!(out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .contains("bridle session advisor --project bridle fred")
    );
    assert_eq!(tmux, "");
}

#[test]
fn advisor_start_refuses_a_worker() {
    let (out, tmux) = advisor_start(true, "", "", true);
    assert!(!out.status.success());
    assert_eq!(tmux, "");
}

#[test]
fn advisor_session_tags_its_pane() {
    let r = session(&["--project", "p", "advisor", "test"], &[], 0);
    // Verify tmux was called with the tag
    assert!(
        r.tmux.contains("set-option -p -t %9 @bridle advisor-test"),
        "{}",
        r.tmux
    );
}

#[test]
fn advisor_session_without_tmux_succeeds() {
    let bin = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let rec = bin.path().join("claude.rec");
    stub(
        bin.path(),
        "claude",
        &format!(
            "for a in \"$@\"; do printf '%s\\n' \"$a\"; done > {rec}\n\
             echo \"AS=$BRIDLE_AS PROJECT=$BRIDLE_PROJECT ADVISOR=$BRIDLE_ADVISOR_NAME\" >> {rec}\n\
             exit 0",
            rec = rec.display()
        ),
    );
    // NO tmux stub - session should succeed anyway since tagging is best-effort
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bridle"));
    cmd.args(["session", "--project", "p", "advisor", "test"])
        .current_dir(tempfile::tempdir().unwrap().keep())
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.path().display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("BRIDLE_HOME", home.path())
        .env("BRIDLE_AGENT_ID", "a-test")
        .env("BRIDLE_LAUNCHER_TEST", "1")
        // Explicitly remove TMUX_PANE to simulate being outside tmux
        .env_remove("TMUX_PANE")
        .env_remove("BRIDLE_PROJECT")
        .env_remove("BRIDLE_URL")
        .env_remove("BRIDLE_TOKEN")
        .env_remove("BRIDLE_SESSION_SUFFIX");
    let out = cmd.output().unwrap();
    // Session should succeed even without tmux
    assert!(
        out.status.success(),
        "advisor session failed outside tmux: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn session_targets_the_project_of_the_folder_it_runs_in() {
    let ws = tempfile::tempdir().unwrap();
    let clone = ws.path().join("clone");
    fs::create_dir_all(ws.path().join(".bridle")).unwrap();
    fs::create_dir_all(&clone).unwrap();
    fs::write(
        ws.path().join(".bridle/daemon.json"),
        format!(
            r#"{{"project":"other","workspace":"{}","repo":"","url":"http://127.0.0.1:1","pid":1,"started_at":"2026-01-01T00:00:00Z","version":""}}"#,
            ws.path().display()
        ),
    )
    .unwrap();
    let r = session(&["aide"], &[("CWD", clone.to_str().unwrap())], 0);
    assert!(
        r.out.status.success(),
        "{}",
        String::from_utf8_lossy(&r.out.stderr)
    );
    assert!(r.claude.contains("AS=aide PROJECT=other"), "{}", r.claude);
}

#[test]
fn session_outside_any_workspace_refuses_instead_of_defaulting_to_bridle() {
    let r = session(&["aide"], &[], 0);
    assert!(!r.out.status.success());
    assert!(r.claude.is_empty(), "claude must not start: {}", r.claude);
    assert!(String::from_utf8_lossy(&r.out.stderr).contains("--project"));
}

/// A one-shot-per-connection fake daemon that answers every request with `body`.
fn fake_daemon(body: String) -> String {
    use std::io::{Read, Write};
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    std::thread::spawn(move || {
        for mut c in l.incoming().flatten() {
            let mut buf = [0u8; 4096];
            let _ = c.read(&mut buf);
            let _ = write!(
                c,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    url
}

fn registered(identity: &str, pid: u32) -> String {
    format!(
        r#"[{{"identity":"{identity}","pid":{pid},"pane":"%3","started_at":"2026-01-01T00:00:00Z","project":"p"}}]"#
    )
}

#[test]
fn a_second_live_session_of_an_identity_is_refused() {
    let url = fake_daemon(registered("aide", std::process::id()));
    let r = session(
        &["--project", "p", "--url", &url, "--token", "t", "aide"],
        &[],
        0,
    );
    assert!(!r.out.status.success());
    assert!(r.claude.is_empty(), "claude must not start: {}", r.claude);
    let err = String::from_utf8_lossy(&r.out.stderr);
    assert!(
        err.contains(&format!("pid {}", std::process::id())),
        "{err}"
    );
    assert!(err.contains("%3"), "{err}");
    assert!(err.contains("bridle session restart aide"), "{err}");
}

#[test]
fn a_registered_session_whose_process_is_gone_does_not_block() {
    // Above any pid the OS hands out, so nothing can be running as it.
    let url = fake_daemon(registered("advisor/alice", 2_000_000_000));
    let r = session(
        &[
            "--project",
            "p",
            "--url",
            &url,
            "--token",
            "t",
            "advisor",
            "alice",
        ],
        &[],
        0,
    );
    assert!(
        r.out.status.success(),
        "{}",
        String::from_utf8_lossy(&r.out.stderr)
    );
    assert!(!r.claude.is_empty());
}

/// A fake daemon with one handover note, for `role=aide` only: any other role has none.
fn handover_daemon() -> String {
    use std::io::{Read, Write};
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    std::thread::spawn(move || {
        for mut c in l.incoming().flatten() {
            let mut buf = [0u8; 4096];
            let n = c.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).into_owned();
            let line = req.lines().next().unwrap_or("").to_string();
            let body = if line.contains("/v1/handovers/latest") && line.contains("role=aide ") {
                r#"{"id":"h-0042","role":"aide","project":"p","body":"b","created_at":"2026-01-01T00:00:00Z","created_by":"external:aide"}"#
            } else if line.contains("/v1/handovers/latest") {
                "null"
            } else {
                "[]"
            };
            let _ = write!(
                c,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    url
}

#[test]
fn a_session_prompt_names_the_newest_note_of_its_own_identity() {
    let url = handover_daemon();
    let args = |who: &[&str]| {
        let mut a = vec!["--project", "p", "--url", &url, "--token", "t"];
        a.extend_from_slice(who);
        session(&a, &[], 0)
    };
    let aide = args(&["aide"]);
    assert!(
        aide.claude.contains("bridle handover show h-0042"),
        "{}",
        aide.claude
    );
    let advisor = args(&["advisor", "alice"]);
    assert!(!advisor.claude.is_empty());
    assert!(!advisor.claude.contains("h-0042"), "{}", advisor.claude);
}
