//! Filesystem layout for a bridle workspace. See docs/agent-host.md §3.1.

use std::io;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// Resolves every path the daemon reads or writes, from the repo clone and
/// an optional workspace override. Nothing here touches the filesystem
/// except [`Workspace::ensure_dirs`] and [`write_secret_file`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub workspace: PathBuf,
    pub repo: PathBuf,
}

impl Workspace {
    /// `workspace` defaults to the repo's parent directory (docs/agent-host.md §3.1).
    pub fn new(repo: impl Into<PathBuf>, workspace: Option<PathBuf>) -> Self {
        let repo = repo.into();
        let workspace = workspace.unwrap_or_else(|| {
            repo.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| repo.clone())
        });
        Self { workspace, repo }
    }

    pub fn state_dir(&self) -> PathBuf {
        self.workspace.join(".bridle")
    }

    pub fn db(&self) -> PathBuf {
        self.state_dir().join("bridle.db")
    }

    pub fn daemon_json(&self) -> PathBuf {
        self.state_dir().join("daemon.json")
    }

    pub fn daemon_log(&self) -> PathBuf {
        self.state_dir().join("daemon.log")
    }

    pub fn tokens_dir(&self) -> PathBuf {
        self.state_dir().join("tokens")
    }

    pub fn human_token(&self) -> PathBuf {
        self.tokens_dir().join("human")
    }

    pub fn agents_dir(&self) -> PathBuf {
        self.state_dir().join("agents")
    }

    pub fn agent_dir(&self, id: &str) -> PathBuf {
        self.agents_dir().join(id)
    }

    pub fn transcript(&self, id: &str) -> PathBuf {
        self.agent_dir(id).join("transcript.jsonl")
    }

    pub fn system_prompt(&self, id: &str) -> PathBuf {
        self.agent_dir(id).join("system-prompt.md")
    }

    pub fn wt_dir(&self) -> PathBuf {
        self.workspace.join("wt")
    }

    pub fn worktree(&self, name: &str) -> PathBuf {
        self.wt_dir().join(name)
    }

    /// Creates the directories the daemon needs before it can write
    /// anything. The tokens directory is tightened to 0700 (§5.3: the human
    /// token must not be group/world readable).
    pub fn ensure_dirs(&self) -> io::Result<()> {
        std::fs::create_dir_all(self.state_dir())?;
        std::fs::create_dir_all(self.agents_dir())?;
        std::fs::create_dir_all(self.wt_dir())?;
        let tokens_dir = self.tokens_dir();
        std::fs::create_dir_all(&tokens_dir)?;
        std::fs::set_permissions(&tokens_dir, std::fs::Permissions::from_mode(0o700))?;
        Ok(())
    }
}

/// Writes `contents` to `path` with 0600 permissions from the start (no
/// window where the file is world-readable). Used for token files.
pub fn write_secret_file(path: &Path, contents: &str) -> io::Result<()> {
    use std::io::Write;

    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true).mode(0o600);
    let mut f = opts.open(path)?;
    f.write_all(contents.as_bytes())?;
    // Belt and suspenders: if the file already existed with looser
    // permissions, `mode()` on create doesn't retroactively fix them.
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_workspace_is_repo_parent() {
        let ws = Workspace::new("/home/jo/proj/repo", None);
        assert_eq!(ws.workspace, PathBuf::from("/home/jo/proj"));
    }

    #[test]
    fn explicit_workspace_override() {
        let ws = Workspace::new("/home/jo/proj/repo", Some(PathBuf::from("/srv/ws")));
        assert_eq!(ws.workspace, PathBuf::from("/srv/ws"));
        assert_eq!(ws.state_dir(), PathBuf::from("/srv/ws/.bridle"));
        assert_eq!(ws.db(), PathBuf::from("/srv/ws/.bridle/bridle.db"));
        assert_eq!(
            ws.human_token(),
            PathBuf::from("/srv/ws/.bridle/tokens/human")
        );
        assert_eq!(
            ws.agent_dir("a-abcde"),
            PathBuf::from("/srv/ws/.bridle/agents/a-abcde")
        );
        assert_eq!(
            ws.transcript("a-abcde"),
            PathBuf::from("/srv/ws/.bridle/agents/a-abcde/transcript.jsonl")
        );
        assert_eq!(ws.worktree("w1"), PathBuf::from("/srv/ws/wt/w1"));
    }

    #[test]
    fn ensure_dirs_creates_layout_with_tight_perms() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).expect("mkdir repo");
        let ws = Workspace::new(&repo, Some(tmp.path().to_path_buf()));
        ws.ensure_dirs().expect("ensure_dirs");

        assert!(ws.state_dir().is_dir());
        assert!(ws.agents_dir().is_dir());
        assert!(ws.wt_dir().is_dir());
        assert!(ws.tokens_dir().is_dir());

        let mode = std::fs::metadata(ws.tokens_dir())
            .expect("stat tokens dir")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o700);
    }

    #[test]
    fn write_secret_file_is_0600() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("human");
        write_secret_file(&path, "deadbeef").expect("write token");
        let mode = std::fs::metadata(&path)
            .expect("stat token")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        assert_eq!(
            std::fs::read_to_string(&path).expect("read token"),
            "deadbeef"
        );
    }
}
