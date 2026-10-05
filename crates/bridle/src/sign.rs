//! `bridle sign setup|binary`: stable local code signing on macOS (ticket p88z). The logic is
//! in `bridle_daemon::signing`; this reads the keychain password and prints.

use std::io::{BufRead, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, anyhow};
use bridle_daemon::signing;

use crate::cli::{SignAction, SignArgs};
use crate::error::CliError;

pub fn run(args: &SignArgs) -> Result<(), CliError> {
    match &args.action {
        SignAction::Setup => {
            let password = match std::env::var("BRIDLE_KEYCHAIN_PASSWORD") {
                Ok(p) if !p.is_empty() => p,
                _ => prompt_password()?,
            };
            for line in signing::setup(&password)? {
                println!("{line}");
            }
            println!("next: `just install` (or any self-upgrade) now signs with it");
        }
        SignAction::Binary { path } => {
            let path = match path {
                Some(p) => p.clone(),
                None => std::env::current_exe().context("finding this binary")?,
            };
            if signing::sign(&path)? {
                println!(
                    "signed {} with \"{}\"",
                    path.display(),
                    signing::identity_name()
                );
            } else {
                println!(
                    "no \"{}\" identity here; keeping the ad-hoc signature",
                    signing::identity_name()
                );
            }
        }
    }
    Ok(())
}

/// Asks for the login keychain's password on the terminal, echo off (works over SSH).
fn prompt_password() -> Result<String, CliError> {
    eprint!("Login keychain password (used once, to let codesign use the key unattended): ");
    std::io::stderr().flush().ok();
    let stty = |arg: &str| {
        Command::new("stty")
            .arg(arg)
            .stdin(Stdio::inherit())
            .status()
    };
    let hidden = stty("-echo").map(|s| s.success()).unwrap_or(false);
    let mut line = String::new();
    let read = std::io::stdin().lock().read_line(&mut line);
    if hidden {
        let _ = stty("echo");
        eprintln!();
    }
    read.map_err(|e| anyhow!("reading the password: {e}"))?;
    let p = line.trim_end_matches(['\n', '\r']).to_string();
    if p.is_empty() {
        return Err(CliError::Other(anyhow!("no password given")));
    }
    Ok(p)
}
