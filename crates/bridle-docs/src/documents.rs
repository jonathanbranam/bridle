//! Reads, searches and writes one document file in a repository, for the document views of the
//! gateway (ticket x8jt, a narrow piece of v8kn) and the daemon (br-5e4k). The whole file goes
//! back on a write and the caller commits nothing: this module does.
//!
//! Safety: the path is repo-relative plain names only and must resolve inside the repo; the
//! file must already exist and be text; a write must carry the hash it read (409 if the file
//! has changed since); and it only commits on the branch checked out in the project's working
//! tree, whichever it is, and never on a detached HEAD. See docs/design/human-web-ui.md section 3.

use std::path::{Component, Path as FsPath, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

/// Bigger than any document worth reviewing in a browser.
const MAX_BYTES: usize = 2 * 1024 * 1024;

/// A document as read: its text, and the hash to send back with an edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Document {
    pub project: String,
    pub path: String,
    pub content: String,
    /// SHA-256 of the content, hex. A write must send it back.
    pub hash: String,
    /// The branch checked out in the project's working tree; the one a write commits to.
    pub branch: String,
}

/// The body of a write: the whole file, and the hash of the version it was edited from.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct DocumentWrite {
    pub content: String,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct DocumentSaved {
    pub project: String,
    pub path: String,
    /// The hash of the content now on disk, for the next write.
    pub hash: String,
    pub branch: String,
    /// The commit made; the current HEAD when the content was unchanged and nothing was committed.
    pub commit: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DocError {
    #[error("no project '{0}' is known on this machine")]
    UnknownProject(String),
    #[error("{0}")]
    NotAvailable(String),
    #[error("bad path: {0}")]
    BadPath(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("not a text file: {0}")]
    NotText(String),
    #[error("the file changed since it was read; reload it and apply the edit again")]
    Stale,
    #[error("the working tree is on a detached HEAD, which the gateway doesn't commit to")]
    DetachedHead,
    #[error("{0}")]
    Internal(String),
}

impl DocError {
    /// The HTTP status each failure maps to, the same on the daemon and the gateway.
    pub fn status(&self) -> u16 {
        match self {
            Self::UnknownProject(_) | Self::NotFound(_) => 404,
            Self::BadPath(_) => 400,
            Self::NotText(_) => 415,
            Self::Stale => 409,
            Self::DetachedHead => 403,
            Self::NotAvailable(_) => 503,
            Self::Internal(_) => 500,
        }
    }
}

/// What `GET .../documents?q=` returns: repo-relative paths, best first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct DocumentMatches {
    pub project: String,
    pub paths: Vec<String>,
}

/// The `?q=` of a document search.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
}

/// `POST .../links/resolve` body: link targets as written in the docs.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct LinkResolveRequest {
    pub targets: Vec<String>,
}

/// One target and the existing document it names, if any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ResolvedLink {
    pub target: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ResolvedLinks {
    pub project: String,
    pub links: Vec<ResolvedLink>,
}

/// A spec id resolves to its file under `design/specs/` plus a `#anchor`, a capability name to
/// the file. A target with a folder is a path under `docs/` (a wiki link may omit `.md`); one without is a
/// ticket stem, looked up in `open/` then `resolved/` since tickets move. Anything that escapes
/// the repo, leaves `docs/` or doesn't exist resolves to none; bare names like README never do.
pub fn resolve_links(repo: &FsPath, targets: &[String]) -> Vec<ResolvedLink> {
    let exists = |rel: &str| rel.starts_with("docs/") && resolve(repo, rel).is_ok();
    let specs = std::cell::OnceCell::new();
    targets
        .iter()
        .map(|t| {
            // A spec id (`r-xxxx`, `s-xxxx`) has the shape of a task id, so it goes first.
            let spec = crate::specs::is_spec_id(t)
                .then(|| crate::specs::path_for(specs.get_or_init(|| crate::specs::load(repo)), t))
                .flatten();
            let path = if spec.is_some() {
                spec
            } else if t.contains('/') {
                [t.clone(), format!("{t}.md")]
                    .into_iter()
                    .find(|c| exists(c))
            } else {
                let stem = t.strip_suffix(".md").unwrap_or(t);
                ["open", "resolved"]
                    .iter()
                    .map(|d| format!("docs/tickets/{d}/{stem}.md"))
                    .find(|c| exists(c))
                    .or_else(|| ticket_by_id(repo, t))
                    .or_else(|| {
                        crate::specs::path_for(specs.get_or_init(|| crate::specs::load(repo)), t)
                    })
            };
            ResolvedLink {
                target: t.clone(),
                path,
            }
        })
        .collect()
}

fn hash_of(content: &str) -> String {
    hex::encode(Sha256::digest(content.as_bytes()))
}

/// The file's absolute path, after checking `rel` is plain names that resolve inside `repo`.
pub fn resolve(repo: &FsPath, rel: &str) -> Result<PathBuf, DocError> {
    let mut clean = PathBuf::new();
    for c in FsPath::new(rel).components() {
        match c {
            Component::Normal(n) if n != ".git" => clean.push(n),
            _ => return Err(DocError::BadPath(rel.to_string())),
        }
    }
    if clean.as_os_str().is_empty() {
        return Err(DocError::BadPath(rel.to_string()));
    }
    let root = repo
        .canonicalize()
        .map_err(|e| DocError::Internal(format!("repo: {e}")))?;
    // Canonicalising also follows a symlink out of the repo, which the prefix check then catches.
    let full = root
        .join(&clean)
        .canonicalize()
        .map_err(|_| DocError::NotFound(rel.to_string()))?;
    if !full.starts_with(&root) || !full.is_file() {
        return Err(DocError::BadPath(rel.to_string()));
    }
    Ok(full)
}

pub fn read_text(full: &FsPath, rel: &str) -> Result<String, DocError> {
    let bytes = std::fs::read(full).map_err(|e| DocError::Internal(e.to_string()))?;
    if bytes.len() > MAX_BYTES || bytes.contains(&0) {
        return Err(DocError::NotText(rel.to_string()));
    }
    String::from_utf8(bytes).map_err(|_| DocError::NotText(rel.to_string()))
}

fn git(repo: &FsPath, args: &[&str]) -> Result<String, DocError> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| DocError::Internal(format!("git: {e}")))?;
    if !out.status.success() {
        return Err(DocError::Internal(format!(
            "git {}: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn branch_of(repo: &FsPath) -> Result<String, DocError> {
    git(repo, &["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Most paths a search returns.
const MAX_MATCHES: usize = 30;

/// Markdown files under `docs/` whose path contains `q` (case-insensitive), best first: a ticket
/// whose ID is exactly `q`, then open tickets, then open spikes, then the rest, each alphabetical.
/// An empty `q` lists the open tickets.
pub fn search_documents(repo: &FsPath, q: &str) -> Vec<String> {
    let q = q.trim().to_lowercase();
    let mut found = Vec::new();
    collect_markdown(repo, &repo.join("docs"), &mut found);
    let mut ranked: Vec<(u8, String)> = found
        .into_iter()
        .filter(|p| {
            if q.is_empty() {
                p.starts_with("docs/tickets/open/")
            } else {
                p.to_lowercase().contains(&q)
            }
        })
        .map(|p| {
            let stem = p.rsplit('/').next().unwrap_or("").trim_end_matches(".md");
            let rank = if !q.is_empty() && stem.to_lowercase().ends_with(&format!("-{q}")) {
                0
            } else if p.starts_with("docs/tickets/open/") {
                1
            } else if p.starts_with("docs/spikes/open/") {
                2
            } else {
                3
            };
            (rank, p)
        })
        .collect();
    ranked.sort();
    ranked
        .into_iter()
        .take(MAX_MATCHES)
        .map(|(_, p)| p)
        .collect()
}

/// Repo-relative paths of the `.md` files under `dir`, not following symlinks.
pub fn collect_markdown(repo: &FsPath, dir: &FsPath, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        match e.file_type() {
            Ok(t) if t.is_dir() => collect_markdown(repo, &path, out),
            Ok(t) if t.is_file() && path.extension().is_some_and(|x| x == "md") => {
                if let Ok(rel) = path.strip_prefix(repo) {
                    out.push(rel.to_string_lossy().into_owned());
                }
            }
            _ => {}
        }
    }
}

pub fn read_document(repo: &FsPath, project: &str, rel: &str) -> Result<Document, DocError> {
    let full = resolve(repo, rel)?;
    let content = read_text(&full, rel)?;
    Ok(Document {
        project: project.to_string(),
        path: rel.to_string(),
        hash: hash_of(&content),
        content,
        branch: branch_of(repo)?,
    })
}

pub fn write_document(
    repo: &FsPath,
    project: &str,
    rel: &str,
    req: &DocumentWrite,
) -> Result<DocumentSaved, DocError> {
    let full = resolve(repo, rel)?;
    let branch = branch_of(repo)?;
    // A detached HEAD reads as "HEAD": no branch to commit to.
    if branch == "HEAD" {
        return Err(DocError::DetachedHead);
    }
    if req.content.len() > MAX_BYTES || req.content.contains('\0') {
        return Err(DocError::NotText(rel.to_string()));
    }
    let current = read_text(&full, rel)?;
    if hash_of(&current) != req.hash {
        return Err(DocError::Stale);
    }
    let saved = |commit: String| DocumentSaved {
        project: project.to_string(),
        path: rel.to_string(),
        hash: hash_of(&req.content),
        branch: branch.clone(),
        commit,
    };
    if current == req.content {
        return Ok(saved(git(repo, &["rev-parse", "HEAD"])?));
    }
    std::fs::write(&full, &req.content).map_err(|e| DocError::Internal(e.to_string()))?;
    let msg = format!("review: human comments on {rel}");
    // `--only` commits just this file, whatever else is staged in the human's tree.
    let committed = git(repo, &["add", "--", rel])
        .and_then(|_| git(repo, &["commit", "-q", "-m", &msg, "--only", "--", rel]));
    if let Err(e) = committed {
        // Leave the file as it was read, not half-saved.
        std::fs::write(&full, &current).ok();
        return Err(e);
    }
    Ok(saved(git(repo, &["rev-parse", "HEAD"])?))
}

/// A bare ticket ID, or a task ID `<prefix>-<id>` whose `<id>` is a ticket ID (a ticket's first
/// task takes its id), is the ticket file ending `-<id>.md`, in `open/` then `resolved/`. Other
/// task IDs resolve to none: the UI has no task view.
fn ticket_by_id(repo: &FsPath, target: &str) -> Option<String> {
    const ALPHABET: &str = "abcdefghjkmnpqrstuvwxyz23456789";
    let id = match target.split_once('-') {
        None => target,
        Some((prefix, id))
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_lowercase()) =>
        {
            id
        }
        Some(_) => return None,
    };
    if id.len() != 4 || !id.chars().all(|c| ALPHABET.contains(c)) {
        return None;
    }
    let suffix = format!("-{id}.md");
    ["open", "resolved"].iter().find_map(|d| {
        let dir = format!("docs/tickets/{d}");
        let mut names: Vec<String> = std::fs::read_dir(repo.join(&dir))
            .ok()?
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter(|n| n.ends_with(&suffix))
            .collect();
        names.sort();
        names
            .into_iter()
            .map(|n| format!("{dir}/{n}"))
            .find(|rel| resolve(repo, rel).is_ok())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_ticket_and_task_ids() {
        let d = repo("main");
        let root = d.path();
        for dir in ["docs/tickets/open", "docs/tickets/resolved"] {
            std::fs::create_dir_all(root.join(dir)).expect("mkdir");
        }
        std::fs::write(root.join("docs/tickets/open/a-thing-ab23.md"), "x").expect("write");
        std::fs::write(root.join("docs/tickets/resolved/old-one-cd34.md"), "x").expect("write");
        let got = |t: &str| resolve_links(root, &[t.to_string()]).remove(0).path;
        assert_eq!(
            got("ab23"),
            Some("docs/tickets/open/a-thing-ab23.md".into())
        );
        assert_eq!(
            got("cd34"),
            Some("docs/tickets/resolved/old-one-cd34.md".into())
        );
        assert_eq!(
            got("br-ab23"),
            Some("docs/tickets/open/a-thing-ab23.md".into())
        );
        assert_eq!(
            got("br-cd34"),
            Some("docs/tickets/resolved/old-one-cd34.md".into())
        );
        assert_eq!(got("zz99"), None);
        assert_eq!(got("br-zz99"), None);
        assert_eq!(got("a-b-ab23"), None);
    }

    #[test]
    fn resolves_paths_and_ticket_stems_only_inside_docs() {
        let d = repo("main");
        let root = d.path();
        for dir in ["docs/tickets/open", "docs/tickets/resolved"] {
            std::fs::create_dir_all(root.join(dir)).expect("mkdir");
        }
        std::fs::write(root.join("docs/tickets/open/a-thing-ab12.md"), "x").expect("write");
        std::fs::write(root.join("docs/tickets/resolved/old-one-cd34.md"), "x").expect("write");
        std::fs::write(root.join("README.md"), "x").expect("write");
        let got = |t: &str| resolve_links(root, &[t.to_string()]).remove(0).path;
        assert_eq!(got("docs/doc"), Some("docs/doc.md".into()));
        assert_eq!(got("docs/doc.md"), Some("docs/doc.md".into()));
        assert_eq!(
            got("a-thing-ab12"),
            Some("docs/tickets/open/a-thing-ab12.md".into())
        );
        assert_eq!(
            got("old-one-cd34"),
            Some("docs/tickets/resolved/old-one-cd34.md".into())
        );
        assert_eq!(got("docs/missing"), None);
        assert_eq!(got("nope-ef56"), None);
        assert_eq!(got("README"), None);
        assert_eq!(got("README.md"), None);
        assert_eq!(got("docs/../README.md"), None);
        assert_eq!(got("/etc/passwd"), None);
        assert_eq!(got("../x/docs/doc.md"), None);
        assert_eq!(got("blob.bin"), None);
    }

    fn run(dir: &FsPath, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "{args:?}: {out:?}");
    }

    /// A repo on `branch` with `docs/doc.md` and a binary file committed.
    fn repo(branch: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        run(dir.path(), &["init", "-q", "-b", branch]);
        run(dir.path(), &["config", "user.name", "t"]);
        run(dir.path(), &["config", "user.email", "t@t"]);
        run(dir.path(), &["config", "commit.gpgsign", "false"]);
        std::fs::create_dir(dir.path().join("docs")).expect("mkdir");
        std::fs::write(dir.path().join("docs/doc.md"), "# Title\n").expect("write");
        std::fs::write(dir.path().join("blob.bin"), [0u8, 159, 146]).expect("write");
        run(dir.path(), &["add", "."]);
        run(dir.path(), &["commit", "-q", "-m", "init"]);
        dir
    }

    fn write(content: &str, hash: &str) -> DocumentWrite {
        DocumentWrite {
            content: content.into(),
            hash: hash.into(),
        }
    }

    #[test]
    fn reads_text_with_hash_and_branch() {
        let d = repo("review");
        let doc = read_document(d.path(), "p", "docs/doc.md").expect("read");
        assert_eq!(doc.content, "# Title\n");
        assert_eq!(doc.branch, "review");
        assert_eq!(doc.hash, hash_of("# Title\n"));
    }

    #[test]
    fn write_commits_the_file_and_the_new_hash_works_again() {
        let d = repo("review");
        let doc = read_document(d.path(), "p", "docs/doc.md").expect("read");
        let saved = write_document(
            d.path(),
            "p",
            "docs/doc.md",
            &write("# Title\nnote\n", &doc.hash),
        )
        .expect("write");
        assert_eq!(saved.hash, hash_of("# Title\nnote\n"));
        assert_eq!(
            std::fs::read_to_string(d.path().join("docs/doc.md")).expect("read"),
            "# Title\nnote\n"
        );
        let log = git(d.path(), &["log", "-1", "--format=%H %s"]).expect("log");
        assert_eq!(
            log,
            format!("{} review: human comments on docs/doc.md", saved.commit)
        );
        assert_eq!(
            git(d.path(), &["status", "--porcelain"]).expect("status"),
            ""
        );
        write_document(
            d.path(),
            "p",
            "docs/doc.md",
            &write("# Title\nagain\n", &saved.hash),
        )
        .expect("second write");
    }

    #[test]
    fn stale_write_is_refused_and_changes_nothing() {
        let d = repo("review");
        let doc = read_document(d.path(), "p", "docs/doc.md").expect("read");
        std::fs::write(d.path().join("docs/doc.md"), "# Edited elsewhere\n").expect("write");
        let err = write_document(d.path(), "p", "docs/doc.md", &write("mine", &doc.hash))
            .expect_err("stale");
        assert!(matches!(err, DocError::Stale));
        assert_eq!(
            std::fs::read_to_string(d.path().join("docs/doc.md")).expect("read"),
            "# Edited elsewhere\n"
        );
    }

    #[test]
    fn paths_outside_the_repo_are_refused() {
        let outer = tempfile::tempdir().expect("tempdir");
        std::fs::write(outer.path().join("secret.txt"), "secret").expect("write");
        let d = repo("review");
        std::os::unix::fs::symlink(outer.path().join("secret.txt"), d.path().join("link.txt"))
            .expect("symlink");
        for p in [
            "../secret.txt",
            "docs/../../secret.txt",
            "/etc/passwd",
            "",
            ".git/config",
            "link.txt",
        ] {
            let err = read_document(d.path(), "p", p).expect_err(p);
            assert!(matches!(err, DocError::BadPath(_)), "{p}: {err}");
            let err = write_document(d.path(), "p", p, &write("x", "h")).expect_err(p);
            assert!(matches!(err, DocError::BadPath(_)), "{p}: {err}");
        }
        assert!(matches!(
            read_document(d.path(), "p", "nope.md"),
            Err(DocError::NotFound(_))
        ));
    }

    #[test]
    fn search_ranks_the_id_then_open_tickets_then_the_rest() {
        let d = repo("review");
        for f in [
            "docs/tickets/open/alpha-x8jt.md",
            "docs/tickets/open/x8jt-guide-zzzz.md",
            "docs/tickets/resolved/old-x8jt.md",
            "docs/spikes/open/spike-x8jt-notes.md",
            "docs/design/x8jt.md",
            "docs/tickets/open/beta-b2b2.md",
            "docs/notes.txt",
        ] {
            std::fs::create_dir_all(d.path().join(f).parent().expect("parent")).expect("mkdir");
            std::fs::write(d.path().join(f), "x").expect("write");
        }
        assert_eq!(
            search_documents(d.path(), "X8JT"),
            [
                "docs/tickets/open/alpha-x8jt.md",
                "docs/tickets/resolved/old-x8jt.md",
                "docs/tickets/open/x8jt-guide-zzzz.md",
                "docs/spikes/open/spike-x8jt-notes.md",
                "docs/design/x8jt.md",
            ]
        );
        assert_eq!(
            search_documents(d.path(), ""),
            [
                "docs/tickets/open/alpha-x8jt.md",
                "docs/tickets/open/beta-b2b2.md",
                "docs/tickets/open/x8jt-guide-zzzz.md",
            ]
        );
        assert!(search_documents(d.path(), "nothing").is_empty());
    }

    #[test]
    fn non_text_is_refused() {
        let d = repo("review");
        assert!(matches!(
            read_document(d.path(), "p", "blob.bin"),
            Err(DocError::NotText(_))
        ));
    }

    #[test]
    fn writes_on_any_branch_but_not_a_detached_head() {
        for branch in ["main", "dev", "review"] {
            let d = repo(branch);
            let doc = read_document(d.path(), "p", "docs/doc.md").expect("read");
            let saved =
                write_document(d.path(), "p", "docs/doc.md", &write("x", &doc.hash)).expect(branch);
            assert_eq!(saved.branch, branch);
        }
        let d = repo("review");
        run(d.path(), &["checkout", "-q", "--detach"]);
        let doc = read_document(d.path(), "p", "docs/doc.md").expect("read");
        assert!(matches!(
            write_document(d.path(), "p", "docs/doc.md", &write("x", &doc.hash)),
            Err(DocError::DetachedHead)
        ));
    }
}
