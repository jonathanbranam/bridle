//! Shared pane tagging for interactive sessions and `bridle pane tag` command.

use anyhow::Context;
use std::process::Command;

/// Clear the given tag from all other panes. Best effort: logs warnings on failure.
fn clear_tag_from_others(tag: &str, current_pane: &str) {
    let output = match Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{pane_id}:#{pane_option_@bridle}",
        ])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("warning: failed to list panes: {e}");
            return;
        }
    };

    if !output.status.success() {
        eprintln!("warning: tmux list-panes failed");
        return;
    }

    let pane_list = String::from_utf8_lossy(&output.stdout);
    for line in pane_list.lines() {
        let Some((pane_id, pane_tag)) = line.split_once(':') else {
            continue;
        };
        if pane_id != current_pane && pane_tag == tag {
            if let Err(e) = Command::new("tmux")
                .args(["set-option", "-p", "-t", pane_id, "-u", "@bridle"])
                .status()
                .context("running tmux set-option to clear tag")
            {
                eprintln!("warning: failed to clear tag from {pane_id}: {e}");
            } else {
                println!("moved {tag} from {pane_id}");
            }
        }
    }
}

/// Tags the current tmux pane with the given identifier. Best effort: if outside tmux
/// or tmux fails, succeeds silently. Used by sessions.
pub fn tag_pane(tag: &str) {
    let Ok(pane) = std::env::var("TMUX_PANE") else {
        return;
    };
    clear_tag_from_others(tag, &pane);
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
    clear_tag_from_others(tag, &pane);
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
