//! What the bridge asks of its own machine: does this daemon own the project (hw6c's owner
//! record), and is the main advisor running (its launcher's pid file).

use std::path::PathBuf;

use async_trait::async_trait;

#[async_trait]
pub trait Local: Send + Sync {
    /// Mail for the project is taken only while this is true.
    async fn owns_project(&self) -> bool;
    /// The unnamed advisor is up, so mail goes to it rather than the orchestrator.
    async fn advisor_running(&self) -> bool;
}

/// Fixed answers, for a bridge with no state to read and for tests.
pub struct FixedLocal {
    pub owns: bool,
    pub advisor: bool,
}

#[async_trait]
impl Local for FixedLocal {
    async fn owns_project(&self) -> bool {
        self.owns
    }

    async fn advisor_running(&self) -> bool {
        self.advisor
    }
}

/// The real thing, from files.
pub struct FileLocal {
    /// `<workspace>/.bridle/state/owner.toml`: the state branch's worktree.
    pub owner_file: PathBuf,
    /// `<bridle home>/advisor-<project>.pid`, written by the advisor launcher.
    pub advisor_pid_file: PathBuf,
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

    async fn advisor_running(&self) -> bool {
        let Some((pid, start)) = std::fs::read_to_string(&self.advisor_pid_file)
            .ok()
            .and_then(|t| parse_pid_file(&t))
        else {
            return false;
        };
        // `ps` prints the start time the launcher recorded, so a reused pid doesn't match.
        let out = tokio::process::Command::new("ps")
            .args(["-o", "lstart=", "-p", &pid.to_string()])
            .output()
            .await;
        out.ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .is_some_and(|s| squeeze(&s) == start)
    }
}

fn owner_host(text: &str) -> Option<String> {
    toml::from_str::<toml::Table>(text)
        .ok()?
        .get("host")?
        .as_str()
        .map(str::to_string)
}

/// `<pid> <ps lstart> <launch epoch>`, as the orchestrator launcher writes; returns the pid
/// and the start time with whitespace squeezed.
fn parse_pid_file(text: &str) -> Option<(i32, String)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let (pid, rest) = words.split_first()?;
    let start = rest.get(..rest.len().checked_sub(1)?)?.join(" ");
    (!start.is_empty()).then_some((pid.parse().ok()?, start))
}

fn squeeze(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_file_parses() {
        assert_eq!(
            parse_pid_file("123 Tue Sep 30 10:00:00 2026 1790000000\n"),
            Some((123, "Tue Sep 30 10:00:00 2026".to_string()))
        );
        assert_eq!(parse_pid_file(""), None);
        assert_eq!(parse_pid_file("123"), None);
    }

    #[tokio::test]
    async fn owner_record_decides() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("owner.toml");
        let local = |host: &str| FileLocal {
            owner_file: file.clone(),
            advisor_pid_file: dir.path().join("advisor.pid"),
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

    #[tokio::test]
    async fn advisor_runs_only_while_its_pid_is_that_process() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("advisor.pid");
        let local = FileLocal {
            owner_file: dir.path().join("owner.toml"),
            advisor_pid_file: pid_file.clone(),
            host: "a".to_string(),
        };
        assert!(!local.advisor_running().await, "no pid file");

        let me = std::process::id();
        let lstart = std::process::Command::new("ps")
            .args(["-o", "lstart=", "-p", &me.to_string()])
            .output()
            .unwrap();
        let lstart = squeeze(&String::from_utf8(lstart.stdout).unwrap());
        std::fs::write(&pid_file, format!("{me} {lstart} 1790000000\n")).unwrap();
        assert!(local.advisor_running().await);

        std::fs::write(
            &pid_file,
            format!("{me} Mon Jan 1 00:00:00 2001 1790000000\n"),
        )
        .unwrap();
        assert!(
            !local.advisor_running().await,
            "a reused pid is not the advisor"
        );
        std::fs::write(&pid_file, "999999 Mon Jan 1 00:00:00 2001 1\n").unwrap();
        assert!(!local.advisor_running().await, "a dead pid");
    }
}
