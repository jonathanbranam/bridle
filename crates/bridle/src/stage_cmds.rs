//! `ticket stage-done | approve | changes | restage` (br-ussh, epic br-tj7q): marking the stages
//! of a version 2 ticket written, signed or sent back. The marks live in the ticket's
//! frontmatter (`written`, `signed`), who did what on its thread (which tells the watchers), and
//! `prepared: true` tells the daemon to open the ticket. Design: `docs/design/stages.md`.

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow, bail};

use crate::cli::{Cli, TicketChangesArgs, TicketStageArgs};
use crate::commands::client_for;
use crate::error::CliError;
use crate::stages::{self, Marks, Stage, State};
use crate::ticket::{drop_line, find_by_id, projects, set_line, thread_row_id, tickets_root};

/// `ticket ready` for a version 2 ticket: refused until the ticket is prepared.
pub fn check_ready_allowed(text: &str) -> anyhow::Result<()> {
    let front: Vec<&str> = text.lines().skip(1).take_while(|l| *l != "---").collect();
    if front.contains(&"version: 2") && !front.contains(&"prepared: true") {
        bail!(
            "version 2 ticket: it becomes ready by itself once every stage is written and every gate signed (see `bridle ticket show`)"
        );
    }
    Ok(())
}

/// A ticket file loaded for a stage command.
struct Loaded {
    path: PathBuf,
    text: String,
    stages: Vec<Stage>,
    id: String,
}

fn load(repo: &Path, id: &str, local: &str, stage: &str) -> anyhow::Result<Loaded> {
    let path = find_by_id(&tickets_root(repo).join("open"), local)
        .map_err(|_| anyhow!("no open ticket {id}"))?;
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let front: Vec<&str> = text.lines().skip(1).take_while(|l| *l != "---").collect();
    let stages = stages::parse(&front)
        .map_err(|e| anyhow!("{e}"))?
        .ok_or_else(|| anyhow!("{id} is not a version 2 ticket (no stages)"))?;
    if !stages.iter().any(|s| s.name == stage) {
        bail!("{id} does not list the stage {stage}");
    }
    Ok(Loaded {
        path,
        text,
        stages,
        id: local.to_string(),
    })
}

fn state_of(l: &Loaded, stage: &str) -> State {
    let m = stages::marks(&l.text);
    stages::states(&l.stages, &m)
        .into_iter()
        .find(|(n, _)| n == stage)
        .map(|(_, s)| s)
        .expect("load checked the stage is listed")
}

/// Rewrites the marks of `text`: the two lists, `prepared`, and `claimed_by` (`Some(None)` drops
/// it, `Some(Some(who))` sets it, `None` leaves it).
fn write_marks(
    text: &str,
    stages: &[Stage],
    m: &Marks,
    claim: Option<Option<&str>>,
) -> anyhow::Result<String> {
    let mut out = text.to_string();
    for (key, items) in [("written", &m.written), ("signed", &m.signed)] {
        out = if items.is_empty() {
            drop_line(&out, key)
        } else {
            set_line(&out, key, &stages::render_list(items))
                .ok_or_else(|| anyhow!("no frontmatter"))?
        };
    }
    out = if stages::prepared(stages, m) {
        set_line(&out, "prepared", "true").ok_or_else(|| anyhow!("no frontmatter"))?
    } else {
        drop_line(&out, "prepared")
    };
    match claim {
        Some(None) => out = drop_line(&out, "claimed_by"),
        Some(Some(who)) => {
            out = set_line(&out, "claimed_by", who).ok_or_else(|| anyhow!("no frontmatter"))?
        }
        None => {}
    }
    Ok(out)
}

fn add(list: &mut Vec<String>, stage: &str) {
    if !list.iter().any(|s| s == stage) {
        list.push(stage.to_string());
    }
}

/// The ticket is claimed by the human while any stage waits for their signature.
fn claim_after(l: &Loaded, m: &Marks) -> Option<&'static str> {
    stages::states(&l.stages, m)
        .iter()
        .any(|(_, s)| *s == State::WaitingForSignature)
        .then_some("human")
}

/// Writes the new marks, then tells the thread (and so the watchers).
async fn finish(
    cli: &Cli,
    repo: &Path,
    l: &Loaded,
    m: Marks,
    claim: Option<Option<&str>>,
    note: &str,
) -> Result<(), CliError> {
    let out = write_marks(&l.text, &l.stages, &m, claim)?;
    std::fs::write(&l.path, out).with_context(|| format!("writing {}", l.path.display()))?;
    let row = thread_row_id(cli, repo, &l.id).await?;
    client_for(cli).await?.note_task(&row, note).await?;
    if stages::prepared(&l.stages, &m) {
        println!("{}: prepared; the daemon opens it", l.id);
    }
    Ok(())
}

async fn local(cli: &Cli, repo: &Path, id: &str) -> Result<String, CliError> {
    Ok(projects(cli, repo).await?.local(id)?.to_string())
}

pub async fn stage_done(cli: &Cli, repo: &Path, a: &TicketStageArgs) -> Result<(), CliError> {
    let tail = local(cli, repo, &a.id).await?;
    let l = load(repo, &a.id, &tail, &a.stage)?;
    let state = state_of(&l, &a.stage);
    if state == State::Waiting {
        return Err(anyhow!("{} requires a stage that is not done yet", a.stage).into());
    }
    stages::check_written(&l.text, &stages::folder_of(&l.path), &a.stage)
        .map_err(|e| anyhow!("{}: {e}", a.stage))?;
    let mut m = stages::marks(&l.text);
    add(&mut m.written, &a.stage);
    let review = l.stages.iter().any(|s| s.name == a.stage && s.human_review);
    // The writer's claim ends; the human's begins while a signature is waited for.
    let claim = Some(claim_after(&l, &m));
    let note = if review {
        format!(
            "stage {} written, waiting for the human's signature",
            a.stage
        )
    } else {
        format!("stage {} written", a.stage)
    };
    finish(cli, repo, &l, m, claim, &note).await
}

pub async fn approve(cli: &Cli, repo: &Path, a: &TicketStageArgs) -> Result<(), CliError> {
    let tail = local(cli, repo, &a.id).await?;
    let l = load(repo, &a.id, &tail, &a.stage)?;
    if state_of(&l, &a.stage) != State::WaitingForSignature {
        return Err(anyhow!(
            "{} is {}; only a written stage that needs the human's signature can be approved",
            a.stage,
            state_of(&l, &a.stage)
        )
        .into());
    }
    let caller = client_for(cli).await?.status().await?.principal;
    let who = if caller == "human" {
        "human".to_string()
    } else {
        format!("human via {caller}")
    };
    let mut m = stages::marks(&l.text);
    add(&mut m.signed, &a.stage);
    let claim = Some(claim_after(&l, &m));
    let note = format!(
        "stage {} signed by {who} at {}",
        a.stage,
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    );
    finish(cli, repo, &l, m, claim, &note).await
}

pub async fn changes(cli: &Cli, repo: &Path, a: &TicketChangesArgs) -> Result<(), CliError> {
    let tail = local(cli, repo, &a.id).await?;
    let l = load(repo, &a.id, &tail, &a.stage)?;
    let mut m = stages::marks(&l.text);
    if !m.written.contains(&a.stage) {
        return Err(anyhow!("{} is not written; there is nothing to send back", a.stage).into());
    }
    m.written.retain(|s| s != &a.stage);
    m.signed.retain(|s| s != &a.stage);
    let claim = Some(claim_after(&l, &m));
    let note = match a.text.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(t) => format!("stage {} sent back for changes: {t}", a.stage),
        None => format!("stage {} sent back for changes", a.stage),
    };
    finish(cli, repo, &l, m, claim, &note).await
}

pub async fn restage(cli: &Cli, repo: &Path, a: &TicketStageArgs) -> Result<(), CliError> {
    let tail = local(cli, repo, &a.id).await?;
    let l = load(repo, &a.id, &tail, &a.stage)?;
    let mut m = stages::marks(&l.text);
    if m.signed.contains(&a.stage) {
        return Err(anyhow!("{} is signed; send it back with `changes`", a.stage).into());
    }
    m.written.retain(|s| s != &a.stage);
    let claim = Some(claim_after(&l, &m));
    let note = format!(
        "stage {} put back for bridle (unwritten, unclaimed)",
        a.stage
    );
    finish(cli, repo, &l, m, claim, &note).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_2_cannot_be_readied_by_hand() {
        assert!(check_ready_allowed("---\nid: x\n---\n").is_ok());
        assert!(check_ready_allowed("---\nversion: 1\n---\n").is_ok());
        assert!(check_ready_allowed("---\nversion: 2\n---\n").is_err());
        assert!(check_ready_allowed("---\nversion: 2\nprepared: true\n---\n").is_ok());
    }

    #[test]
    fn marks_round_trip_and_prepared_follows() {
        let st = stages::build(&[], &["proposal".into()]).unwrap();
        let text = "---\nid: x\nversion: 2\nstages:\n  proposal: {review: human}\n---\nbody\n";
        let mut m = Marks::default();
        add(&mut m.written, "proposal");
        let out = write_marks(text, &st, &m, Some(Some("human"))).unwrap();
        assert!(out.contains("written: [proposal]") && out.contains("claimed_by: human"));
        assert!(!out.contains("prepared"));
        add(&mut m.signed, "proposal");
        let out = write_marks(&out, &st, &m, Some(None)).unwrap();
        assert!(out.contains("signed: [proposal]") && out.contains("prepared: true"));
        assert!(!out.contains("claimed_by"));
        assert_eq!(stages::marks(&out), m);
        let back = write_marks(&out, &st, &Marks::default(), None).unwrap();
        assert_eq!(back, text);
    }
}
