//! `bridle review add|remove|list`: edits the project's list of documents under review
//! (`.bridle/review-documents.txt`); the daemon reads it each tick.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, bail};
use bridle_api::types::ReviewNowRequest;
use bridle_daemon::doc_watch::{read_registry, set_registered};

use crate::cli::{Cli, ReviewAction, ReviewArgs};
use crate::commands::client_for;
use crate::error::CliError;
use crate::render;

pub async fn run(cli: &Cli, args: &ReviewArgs) -> Result<(), CliError> {
    if let ReviewAction::Now { path, resend } = &args.action {
        let client = client_for(cli).await?;
        let r = client
            .review_now(&ReviewNowRequest {
                path: path.trim_start_matches("./").to_string(),
                resend: *resend,
            })
            .await?;
        if cli.json {
            render::print_json(&r)?;
        } else if r.threads == 0 {
            println!("nothing to send: no unsent comment threads in {}", r.path);
        } else {
            println!("sent {} thread(s) in {} to {}", r.threads, r.path, r.agent);
        }
        return Ok(());
    }
    Ok(edit(args)?)
}

fn edit(args: &ReviewArgs) -> anyhow::Result<()> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!("not in a git repository");
    }
    let repo = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    match &args.action {
        ReviewAction::Add { path } => {
            if !repo.join(path).is_file() {
                bail!("{path} is not a file under {}", repo.display());
            }
            set_registered(&repo, path, true).context("writing the review list")?;
        }
        ReviewAction::Remove { path } => {
            set_registered(&repo, path, false).context("writing the review list")?;
        }
        ReviewAction::Now { .. } => unreachable!("handled in run"),
        ReviewAction::List => {
            for p in read_registry(&repo) {
                println!("{p}");
            }
        }
    }
    Ok(())
}
