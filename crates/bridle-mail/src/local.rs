//! What the bridge asks of its own machine: does this daemon own the project (hw6c's owner
//! record), and is the project's aide running (the daemon's session list).

use std::path::PathBuf;

use async_trait::async_trait;

#[async_trait]
pub trait Local: Send + Sync {
    /// Mail for the project is taken only while this is true.
    async fn owns_project(&self) -> bool;
    /// The project's aide is up, so mail goes to it rather than the orchestrator.
    async fn aide_running(&self) -> bool;
}

/// Fixed answers, for a bridge with no state to read and for tests.
pub struct FixedLocal {
    pub owns: bool,
    pub aide: bool,
}

#[async_trait]
impl Local for FixedLocal {
    async fn owns_project(&self) -> bool {
        self.owns
    }

    async fn aide_running(&self) -> bool {
        self.aide
    }
}

/// The real thing: the owner file, and the daemon for liveness.
pub struct FileLocal {
    /// `<workspace>/.bridle/state/owner.toml`: the state branch's worktree.
    pub owner_file: PathBuf,
    /// The project's daemon, which lists the live sessions.
    pub client: bridle_api::Client,
    /// This machine's name, as `bridle serve` writes it to `owner.toml`.
    pub host: String,
}

#[async_trait]
impl Local for FileLocal {
    async fn owns_project(&self) -> bool {
        // No record (a single machine with no origin, or a first serve) means nobody else does.
        match std::fs::read_to_string(&self.owner_file) {
            Err(_) => true,
            Ok(text) => owner_host(&text).is_none_or(|h| h == self.host),
        }
    }

    async fn aide_running(&self) -> bool {
        // The daemon drops a session whose pid is gone, so a listed aide is a live one. A daemon
        // that can't answer means no aide: the orchestrator is the safe fallback.
        self.client.sessions().await.is_ok_and(|s| has_aide(&s))
    }
}

/// The daemon is per project, so any `aide` session in its list is this project's.
fn has_aide(sessions: &[bridle_api::types::SessionInfo]) -> bool {
    sessions.iter().any(|s| s.identity == "aide" && s.pid > 0)
}

fn owner_host(text: &str) -> Option<String> {
    toml::from_str::<toml::Table>(text)
        .ok()?
        .get("host")?
        .as_str()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(identity: &str, pid: i32) -> bridle_api::types::SessionInfo {
        serde_json::from_value(serde_json::json!({
            "identity": identity, "pid": pid, "started_at": "2026-10-07T00:00:00Z",
        }))
        .unwrap()
    }

    #[test]
    fn only_an_aide_session_counts() {
        assert!(!has_aide(&[]));
        assert!(!has_aide(&[
            session("advisor", 5),
            session("advisor/alice", 6)
        ]));
        assert!(has_aide(&[session("advisor", 5), session("aide", 7)]));
    }

    #[tokio::test]
    async fn owner_record_decides() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("owner.toml");
        let local = |host: &str| FileLocal {
            owner_file: file.clone(),
            client: bridle_api::Client::new("http://127.0.0.1:1", None),
            host: host.to_string(),
        };
        assert!(
            local("a").owns_project().await,
            "no record: nobody else owns it"
        );
        std::fs::write(&file, "host = \"a\"\nsince = \"2026-09-30T00:00:00Z\"\n").unwrap();
        assert!(local("a").owns_project().await);
        assert!(!local("b").owns_project().await);
    }
}
