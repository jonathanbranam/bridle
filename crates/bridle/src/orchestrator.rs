//! `bridle orchestrator note-session`: the orchestrator launcher's SessionStart hook
//! (docs/design/agent-host/orchestrator-supervision.md, section 1). `/clear` gives the same
//! process a new session id, so the id and transcript path are recorded per session start.

use std::path::Path;

use serde_json::Value;

/// Writes `$BRIDLE_HOME/orchestrator.session` as `<session id> <transcript path>`. Never fails:
/// a hook that errors would show in the session, and a missing note costs only a stale reading.
pub fn note_session(input: &Value) {
    write_session_to(&bridle_api::discovery::bridle_home(), input);
}

/// `bridle orchestrator clear-recovery`: removes the recovery flag the daemon's supervisor
/// set. A file under `$BRIDLE_HOME`, like the pid file, so no daemon call.
pub fn clear_recovery() -> Result<(), crate::error::CliError> {
    // The name the daemon's supervisor checks (bridle-daemon orchestrator.rs, RECOVERY_FILE).
    let path = bridle_api::discovery::bridle_home().join("orchestrator.recovery");
    match std::fs::remove_file(&path) {
        Ok(()) => println!("recovery mode cleared; the orchestrator supervisor resumes"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("not in recovery mode")
        }
        Err(e) => return Err(anyhow::Error::from(e).into()),
    }
    Ok(())
}

/// `bridle orchestrator hold`.
pub async fn hold(
    cli: &crate::cli::Cli,
    reason: Option<String>,
) -> Result<(), crate::error::CliError> {
    let client = crate::commands::client_for(cli).await?;
    let hold = client.orchestrator_hold(reason).await?;
    println!(
        "orchestrator relaunch held since {}",
        hold.since.to_rfc3339()
    );
    Ok(())
}

/// `bridle orchestrator release`.
pub async fn release(cli: &crate::cli::Cli) -> Result<(), crate::error::CliError> {
    let client = crate::commands::client_for(cli).await?;
    let (released, config_held) = client.orchestrator_release().await?;
    if released {
        println!("hold released; the orchestrator supervisor resumes");
    } else {
        println!("not held");
    }
    if config_held {
        println!(
            "autostart = false in ~/.bridle/config.toml still keeps the orchestrator down; set it true (no restart needed)"
        );
    }
    Ok(())
}

fn session_id(input: &Value) -> Option<&str> {
    input
        .get("session_id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && !s.contains(char::is_whitespace))
}

fn write_session_to(home: &Path, input: &Value) {
    let Some(id) = session_id(input) else {
        return;
    };
    let transcript = input
        .get("transcript_path")
        .and_then(Value::as_str)
        .unwrap_or("");
    let _ = std::fs::create_dir_all(home);
    let _ = std::fs::write(
        home.join("orchestrator.session"),
        format!("{id} {transcript}\n"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn writes_id_and_transcript() {
        let home = tempfile::tempdir().expect("tempdir");
        write_session_to(
            home.path(),
            &json!({"session_id": "abc", "transcript_path": "/t/abc.jsonl", "source": "clear"}),
        );
        let got = std::fs::read_to_string(home.path().join("orchestrator.session")).expect("file");
        assert_eq!(got, "abc /t/abc.jsonl\n");
    }

    #[test]
    fn junk_input_writes_nothing() {
        let home = tempfile::tempdir().expect("tempdir");
        write_session_to(home.path(), &Value::Null);
        write_session_to(home.path(), &json!({"session_id": "a b"}));
        assert!(!home.path().join("orchestrator.session").exists());
    }
}
