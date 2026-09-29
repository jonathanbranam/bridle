//! `bridle orchestrator note-session`: the orchestrator launcher's SessionStart hook
//! (docs/design/agent-host/orchestrator-supervision.md, section 1). `/clear` gives the same
//! process a new session id, so the id and transcript path are recorded per session start.

use std::path::Path;

use serde_json::Value;

/// Writes `$BRIDLE_HOME/orchestrator.session` as `<session id> <transcript path>`. Never fails:
/// a hook that errors would show in the session, and a missing note costs only a stale reading.
pub fn note_session(input: &Value) {
    write_session_to(&bridle_api::discovery::bridle_home(), input);
    // Until the daemon reads the context itself, scripts/context-check.sh finds the session
    // through this older file (the launcher no longer writes it).
    if let (Some(id), Some(home)) = (session_id(input), std::env::var_os("HOME")) {
        let _ = std::fs::write(
            Path::new(&home).join(".bridle-orchestrator-session"),
            format!("{id}\n"),
        );
    }
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
