//! `bridle project take-over` (53gy A): claims a project for this machine and exits. It commits
//! `owner.toml` on `bridle/state`, confirms origin has it, then moves the project's `[projects]`
//! entry to this machine on every machine through `bridle projects sync`'s code. It never serves
//! and needs no daemon; starting one is a separate step.

use std::io::Write;
use std::path::Path;

use anyhow::Context;
use bridle_api::discovery::bridle_home;
use bridle_api::machines::{MachineMap, owner_names};
use bridle_daemon::config::Config;
use bridle_daemon::paths::Workspace;
use bridle_daemon::state_branch::{MainPush, TakeOverOutcome, take_over_project};

use crate::cli::{Cli, TakeOverArgs};
use crate::error::CliError;
use crate::project_sync::{Diff, Entry, Files, Real, apply_text, sync};
use crate::token_pair::{Fleet, Where};

pub async fn run(cli: &Cli, args: &TakeOverArgs) -> Result<(), CliError> {
    let repo = match &args.repo {
        Some(r) => r.clone(),
        None => std::env::current_dir().context("current directory")?,
    }
    .canonicalize()
    .context("repo path")?;
    let project = cli.project.clone().unwrap_or_else(|| {
        repo.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    let home = bridle_home();
    let fleet = Fleet::from_config(&MachineMap::load(&home).map_err(|e| anyhow::anyhow!(e))?);
    // Before claiming anything: a listed project needs a machine name to be moved to.
    if fleet.placed.contains_key(&project) && fleet.local == "local" {
        return Err(anyhow::anyhow!(
            "{project} is listed in [projects], but this machine has no [machine] name in {}: \
             set it, then run `bridle project take-over` again",
            home.join("config.toml").display()
        )
        .into());
    }
    let integration = Config::load(&repo)
        .context("loading .bridle/config.toml")?
        .branches
        .integration;
    let ws = Workspace::new(repo.clone(), args.workspace.clone());
    let names = owner_names(&home);
    let outcome = take_over_project(
        &repo,
        &ws.state_branch_dir(),
        &project,
        &integration,
        &names,
        args.force,
    )
    .await
    .map_err(|e| CliError::Other(e.context(format!("take-over of {project} refused"))))?;
    let mut out = std::io::stdout();
    match &outcome {
        TakeOverOutcome::AlreadyOwner => {
            let _ = writeln!(
                out,
                "{project}: already owned by this machine ({})",
                names[0]
            );
        }
        TakeOverOutcome::ReleasedByThisMachine { released } => {
            let _ = writeln!(
                out,
                "{project}: released by this machine ({}) at {released}; it is not owned now. \
                 To take it back, run `bridle project start` here; to move it, run take-over \
                 on the new machine.",
                names[0]
            );
        }
        TakeOverOutcome::Claimed { from, main_push } => {
            let _ = writeln!(
                out,
                "{project}: now owned by {} (was {}); origin has it",
                names[0],
                from.as_deref().unwrap_or("no one")
            );
            match main_push {
                MainPush::NotNeeded => {}
                MainPush::Pushed => {
                    let _ = writeln!(
                        out,
                        "{project}: local {integration} had commits origin lacked; pushed them"
                    );
                }
                MainPush::Failed(e) => {
                    let _ = writeln!(
                        out,
                        "{project}: local {integration} has commits origin lacks and the push \
                         failed ({e}); run `git push origin {integration}` here"
                    );
                }
            }
            // Tokens and scheduled messages are made by the new daemon, so `project start`
            // finishes the move once it is up.
            crate::project_start::note_moved_in(&ws, from.as_deref()).map_err(|e| {
                CliError::Other(e.context("recording what `project start` carries"))
            })?;
            if let Some(dir) = crate::project_start::fresh_database(&ws)
                .await
                .map_err(|e| CliError::Other(e.context("setting the database aside")))?
            {
                let _ = writeln!(
                    out,
                    "{project}: the old database is kept in {}; the daemon builds a fresh one",
                    dir.display()
                );
            }
            let _ = writeln!(
                out,
                "next: `bridle project start` here re-creates the tokens and carries the scheduled messages"
            );
        }
    }
    // A machine that gave the project up (`project stop`, kx5u K5) listed its clone as tools-only;
    // owning it again means serving it again.
    if crate::tools_only::remove_entry(&repo, &home).map_err(CliError::Other)? {
        let _ = writeln!(
            out,
            "{project}: {} removed from [machine] tools_only; `bridle serve` works here again",
            repo.display()
        );
    }
    // Also on a repeat: it is how an interrupted run finishes.
    update_projects(&home, &project, &mut out).map_err(|e| {
        CliError::Other(anyhow::anyhow!(
            "{project} is owned by this machine, but [projects] was not updated everywhere: {e}; \
             fix it, then run `bridle projects sync --apply`"
        ))
    })
}

/// This machine's port for `project` once it lives here: its listed port, moved up past any
/// other project already placed on this machine with the same one.
fn port_here(fleet: &Fleet, project: &str) -> Option<u16> {
    let mut port = fleet.placed.get(project)?.1;
    while fleet
        .placed
        .iter()
        .any(|(p, (m, q))| p != project && *m == fleet.local && *q == port)
    {
        port += 1;
    }
    Some(port)
}

/// Points `project`'s `[projects]` entry at this machine in the local file, then has
/// `projects sync` bring every `[machines]` entry to match (this machine wins). No second writer
/// of the list. An unlisted project has no entry to move and is left unlisted.
pub fn update_projects(home: &Path, project: &str, out: &mut dyn Write) -> Result<(), String> {
    let files = Real {
        home: home.to_path_buf(),
    };
    let map = MachineMap::load(home).map_err(|e| e.to_string())?;
    let fleet = Fleet::from_config(&map);
    if let Some(port) = port_here(&fleet, project) {
        let here = Entry::Place(fleet.local.clone(), port);
        let current = fleet
            .placed
            .get(project)
            .map(|(m, p)| Entry::Place(m.clone(), *p));
        if current.as_ref() != Some(&here) {
            let key = ("projects", project.to_string());
            let diff = Diff::Differs(
                format!("[projects] {project}"),
                here,
                current.unwrap_or(Entry::Host(String::new())),
            );
            let text = files.read(&Where::Local)?;
            files.write(&Where::Local, &apply_text(&text, &[(key, diff)])?)?;
        }
    } else {
        let _ = writeln!(out, "{project} is not in [projects]; left unlisted");
    }
    let map = MachineMap::load(home).map_err(|e| e.to_string())?;
    if sync(&Fleet::from_config(&map), &files, true, out) {
        Ok(())
    } else {
        Err("some machines could not be read or written".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_project_entry_moves_here_and_keeps_its_port() {
        let home = tempfile::tempdir().unwrap();
        std::fs::write(
            home.path().join("config.toml"),
            "[machine]\nname = \"new-box\"\n[projects]\np = { machine = \"old-box\", port = 7500 }\n",
        )
        .unwrap();
        let mut out = Vec::new();
        update_projects(home.path(), "p", &mut out).unwrap();
        let text = std::fs::read_to_string(home.path().join("config.toml")).unwrap();
        assert!(
            text.contains("machine = \"new-box\""),
            "{text}\n{}",
            String::from_utf8_lossy(&out)
        );
        assert!(text.contains("port = 7500"), "{text}");
    }
}
