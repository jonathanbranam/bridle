//! Shared pane tagging for interactive sessions and `bridle pane tag` command.

use anyhow::Context;
use std::process::Command;

/// Tags the current tmux pane with the given identifier. Best effort: if outside tmux
/// or tmux fails, succeeds silently. Used by sessions.
pub fn tag_pane(tag: &str) {
    let Ok(pane) = std::env::var("TMUX_PANE") else {
        return;
    };
    let _ = Command::new("tmux")
        .args(["set-option", "-p", "-t", &pane, "@bridle", tag])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Tags the current tmux pane with an error if not in tmux. Used by `bridle pane tag` command.
pub fn tag_pane_with_error(tag: &str) -> anyhow::Result<()> {
    let pane = std::env::var("TMUX_PANE")
        .map_err(|_| anyhow::anyhow!("not in a tmux pane (TMUX_PANE not set)"))?;
    let status = Command::new("tmux")
        .args(["set-option", "-p", "-t", &pane, "@bridle", tag])
        .status()
        .context("running tmux set-option")?;
    if !status.success() {
        return Err(anyhow::anyhow!("tmux set-option failed"));
    }
    Ok(())
}

/// Clears the @bridle tag from the current tmux pane, with error if not in tmux.
pub fn untag_pane_with_error() -> anyhow::Result<()> {
    let pane = std::env::var("TMUX_PANE")
        .map_err(|_| anyhow::anyhow!("not in a tmux pane (TMUX_PANE not set)"))?;
    let status = Command::new("tmux")
        .args(["set-option", "-p", "-t", &pane, "-u", "@bridle"])
        .status()
        .context("running tmux set-option")?;
    if !status.success() {
        return Err(anyhow::anyhow!("tmux set-option failed"));
    }
    Ok(())
}
