//! `bridle ticket new` / `resolve` (docs/design/cli.md, ticket 7gk7): mint a ticket file under
//! `docs/tickets/open/` and move it to `resolved/`. The file logic (id mint, frontmatter, find by
//! id) takes a tickets root and does no I/O beyond it, so `check`/`set`/`link` can reuse it.

use std::collections::HashSet;
use std::hash::{BuildHasher, RandomState};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, anyhow, bail};
use bridle_api::{NewTaskRequest, TaskKind};

use crate::cli::{Cli, TicketAction, TicketArgs, TicketNewArgs, TicketResolveArgs};
use crate::commands::client_for;
use crate::error::CliError;
use crate::launchd::project_name;

const ID_ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
const ID_LEN: usize = 4;
const MAX_SLUG: usize = 60;

pub async fn run(cli: &Cli, args: &TicketArgs) -> Result<(), CliError> {
    let repo = repo_root()?;
    match &args.action {
        TicketAction::New(a) => new(cli, &repo, a).await,
        TicketAction::Resolve(a) => resolve_cmd(&repo, a),
    }
}

fn repo_root() -> anyhow::Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!("not in a git repository");
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

pub fn tickets_root(repo: &Path) -> PathBuf {
    repo.join("docs/tickets")
}

async fn new(cli: &Cli, repo: &Path, args: &TicketNewArgs) -> Result<(), CliError> {
    let repos = if args.repos.is_empty() {
        vec![project_name(cli, repo)]
    } else {
        args.repos.clone()
    };
    let fields = Fields {
        title: &args.title,
        repos: &repos,
        needs: &args.needs,
        see: &args.see,
    };
    let today = chrono::Utc::now().date_naive().to_string();
    let path = create(&tickets_root(repo), &fields, &today)?;
    let rel = path
        .strip_prefix(repo)
        .unwrap_or(&path)
        .display()
        .to_string();
    println!("{rel}");

    if !args.no_task {
        let id = id_of(&path).expect("create names the file <slug>-<id>.md");
        if let Err(e) = make_task(cli, &args.title, &id, &rel).await {
            eprintln!("warning: ticket written, but no bridle task was created: {e}");
        }
    }
    Ok(())
}

async fn make_task(cli: &Cli, title: &str, id: &str, rel: &str) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    client
        .new_task(&NewTaskRequest {
            for_human: false,
            priority: None,
            title: title.to_string(),
            kind: TaskKind::Question,
            body: format!("original id: {id}\n{rel}"),
            components: Vec::new(),
            size: None,
        })
        .await?;
    Ok(())
}

fn resolve_cmd(repo: &Path, args: &TicketResolveArgs) -> Result<(), CliError> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let dest = resolve(&tickets_root(repo), &args.id, &now)?;
    println!("{}", dest.strip_prefix(repo).unwrap_or(&dest).display());
    Ok(())
}

pub struct Fields<'a> {
    pub title: &'a str,
    pub repos: &'a [String],
    pub needs: &'a [String],
    pub see: &'a [String],
}

/// Write a new ticket into `<root>/open/`, creating the folders; returns its path.
pub fn create(root: &Path, f: &Fields, today: &str) -> anyhow::Result<PathBuf> {
    let (open, resolved) = (root.join("open"), root.join("resolved"));
    for d in [&open, &resolved] {
        std::fs::create_dir_all(d).with_context(|| format!("creating {}", d.display()))?;
    }
    let id = mint_id(&existing_ids(root));
    let slug = slugify(f.title);
    let name = if slug.is_empty() {
        format!("ticket-{id}.md")
    } else {
        format!("{slug}-{id}.md")
    };
    let text = format!(
        "---\nid: {id}\ntitle: {}\nopened: {today}\nrepos: {}\nchanges: []\nspecs: []\nneeds: {}\nsee: {}\n---\n\n## The ask\n\n",
        yaml_scalar(f.title),
        list(f.repos),
        list(f.needs),
        list(f.see),
    );
    let path = open.join(name);
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Move the open ticket `id` to `resolved/`, stamping `closed:` first. Returns the new path.
pub fn resolve(root: &Path, id: &str, closed: &str) -> anyhow::Result<PathBuf> {
    let found = find_by_id(&root.join("open"), id)?;
    let dest = root
        .join("resolved")
        .join(found.file_name().expect("a file"));
    if find_by_id(&root.join("resolved"), id).is_ok() {
        bail!("ticket {id} is already in resolved/");
    }
    let text =
        std::fs::read_to_string(&found).with_context(|| format!("reading {}", found.display()))?;
    let stamped = set_closed(&text, closed)
        .ok_or_else(|| anyhow!("{} has no frontmatter", found.display()))?;
    std::fs::create_dir_all(root.join("resolved")).context("creating resolved/")?;
    std::fs::write(&dest, stamped).with_context(|| format!("writing {}", dest.display()))?;
    std::fs::remove_file(&found).with_context(|| format!("removing {}", found.display()))?;
    Ok(dest)
}

/// The one `*-<id>.md` file in `dir`; errors on none or several.
pub fn find_by_id(dir: &Path, id: &str) -> anyhow::Result<PathBuf> {
    let suffix = format!("-{id}.md");
    let mut hits: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with(&suffix))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    hits.sort();
    match hits.len() {
        0 => bail!("no ticket with id {id} in {}", dir.display()),
        1 => Ok(hits.remove(0)),
        _ => bail!(
            "ticket id {id} is ambiguous in {}: {}",
            dir.display(),
            hits.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// The id a ticket file name ends with (`<slug>-<id>.md`).
pub fn id_of(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy();
    let id = stem.rsplit('-').next()?;
    (id.len() == ID_LEN && id.bytes().all(|b| ID_ALPHABET.contains(&b))).then(|| id.to_string())
}

fn existing_ids(root: &Path) -> HashSet<String> {
    ["open", "resolved"]
        .iter()
        .filter_map(|d| std::fs::read_dir(root.join(d)).ok())
        .flat_map(|rd| rd.filter_map(Result::ok))
        .filter_map(|e| id_of(&e.path()))
        .collect()
}

fn mint_id(taken: &HashSet<String>) -> String {
    let state = RandomState::new();
    for n in 0u64.. {
        let mut h = state.hash_one(n);
        let id: String = (0..ID_LEN)
            .map(|_| {
                let c = ID_ALPHABET[(h % ID_ALPHABET.len() as u64) as usize] as char;
                h /= ID_ALPHABET.len() as u64;
                c
            })
            .collect();
        if !taken.contains(&id) {
            return id;
        }
    }
    unreachable!("the id space is finite but far larger than any repo's tickets")
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    if slug.len() > MAX_SLUG {
        slug.truncate(MAX_SLUG);
    }
    slug.trim_end_matches('-').to_string()
}

fn list(items: &[String]) -> String {
    format!("[{}]", items.join(", "))
}

/// A frontmatter scalar, double-quoted only when plain YAML would misread it.
fn yaml_scalar(s: &str) -> String {
    let risky = s.contains(": ")
        || s.contains(" #")
        || s.ends_with(':')
        || s.starts_with(|c: char| "-?:,[]{}#&*!|>'\"%@`".contains(c) || c.is_whitespace());
    if risky {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// `text` with `closed: <ts>` as the last frontmatter line (replacing any existing one);
/// `None` when there is no frontmatter block.
fn set_closed(text: &str, closed: &str) -> Option<String> {
    let mut lines: Vec<&str> = text.lines().collect();
    if lines.first() != Some(&"---") {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    lines.retain({
        let mut i = 0;
        move |l| {
            i += 1;
            !(i > 1 && i <= end && l.starts_with("closed:"))
        }
    });
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    let stamp = format!("closed: {closed}");
    lines.insert(end, &stamp);
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields<'a>(title: &'a str, repos: &'a [String]) -> Fields<'a> {
        Fields {
            title,
            repos,
            needs: &[],
            see: &[],
        }
    }

    #[test]
    fn new_writes_named_file_with_frontmatter_and_makes_folders() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["proj".to_string()];
        let p = create(
            dir.path(),
            &fields("Fix the: thing (now)", &repos),
            "2026-09-30",
        )
        .unwrap();
        assert!(dir.path().join("resolved").is_dir());
        let id = id_of(&p).unwrap();
        assert_eq!(p.parent().unwrap(), dir.path().join("open"));
        assert_eq!(
            p.file_name().unwrap().to_string_lossy(),
            format!("fix-the-thing-now-{id}.md")
        );
        let text = std::fs::read_to_string(&p).unwrap();
        assert_eq!(
            text,
            format!(
                "---\nid: {id}\ntitle: \"Fix the: thing (now)\"\nopened: 2026-09-30\nrepos: [proj]\nchanges: []\nspecs: []\nneeds: []\nsee: []\n---\n\n## The ask\n\n"
            )
        );
    }

    #[test]
    fn ids_are_unique_across_open_and_resolved() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let mut seen = HashSet::new();
        for i in 0..200 {
            let p = create(dir.path(), &fields(&format!("t{i}"), &repos), "2026-09-30").unwrap();
            let id = id_of(&p).unwrap();
            assert!(seen.insert(id.clone()));
            if i % 2 == 0 {
                resolve(dir.path(), &id, "2026-09-30T00:00:00Z").unwrap();
            }
        }
        // A taken id is never minted again.
        let taken: HashSet<String> = ID_ALPHABET
            .iter()
            .map(|c| format!("{0}{0}{0}{0}", *c as char))
            .collect();
        let id = mint_id(&taken);
        assert!(!taken.contains(&id));
    }

    #[test]
    fn resolve_moves_and_stamps_closed() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let p = create(dir.path(), &fields("Thing", &repos), "2026-09-30").unwrap();
        let id = id_of(&p).unwrap();
        let dest = resolve(dir.path(), &id, "2026-10-01T12:00:00Z").unwrap();
        assert!(!p.exists());
        assert_eq!(dest.parent().unwrap(), dir.path().join("resolved"));
        let text = std::fs::read_to_string(&dest).unwrap();
        assert!(
            text.contains("see: []\nclosed: 2026-10-01T12:00:00Z\n---\n"),
            "{text}"
        );
        assert!(resolve(dir.path(), &id, "x").is_err());
    }

    #[test]
    fn resolve_refuses_unknown_and_ambiguous_ids() {
        let dir = tempfile::tempdir().unwrap();
        let open = dir.path().join("open");
        std::fs::create_dir_all(&open).unwrap();
        let e = resolve(dir.path(), "zzzz", "x").unwrap_err().to_string();
        assert!(e.contains("no ticket"), "{e}");
        for n in ["a-abcd.md", "b-abcd.md"] {
            std::fs::write(open.join(n), "---\nid: abcd\n---\n").unwrap();
        }
        let e = resolve(dir.path(), "abcd", "x").unwrap_err().to_string();
        assert!(e.contains("ambiguous"), "{e}");
    }

    #[test]
    fn set_closed_replaces_an_existing_stamp() {
        let t = set_closed("---\nid: a\nclosed: old\n---\nbody\n", "new").unwrap();
        assert_eq!(t, "---\nid: a\nclosed: new\n---\nbody\n");
        assert!(set_closed("no frontmatter", "x").is_none());
    }
}
