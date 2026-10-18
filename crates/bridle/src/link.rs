//! `bridle link <id|path>`: the bridle UI URL for a ticket, a document or a spec (tickets yfjc,
//! zqnp), so agents talking to the human don't build URLs by hand. The formats are the "Final
//! formats" table of bridle-ui's `docs/design/url-scheme.md`. Prints nothing and exits 0 with
//! no base URL.

use bridle_api::discovery::bridle_home;
use bridle_daemon::config::ui_base_url;
use bridle_daemon::mentions::{encode, target_url};
use bridle_docs::specs;

use crate::cli::{Cli, LinkArgs};
use crate::error::CliError;
use crate::project;

/// The URL for `target` under `base`. A spec ID needs `spec_path` to find the file it lives in.
pub(crate) fn url(
    base: &str,
    target: &str,
    project: impl FnOnce() -> Result<String, CliError>,
    spec_path: impl FnOnce(&str) -> Option<String>,
) -> Result<String, CliError> {
    let project = project()?;
    let p = encode(&project, false);
    if specs::is_spec_id(target) {
        let path = spec_path(target).ok_or_else(|| {
            anyhow::anyhow!("no spec file has the requirement or scenario {target}")
        })?;
        let (file, id) = path.split_once('#').unwrap_or((&path, ""));
        let anchor = if id.is_empty() {
            String::new()
        } else {
            format!("#{id}")
        };
        return Ok(format!(
            "{base}/p/{p}/specs?path={}{anchor}",
            encode(file, true)
        ));
    }
    Ok(target_url(base, &project, target))
}

/// The to-do page URL for a task id: a human's to-do is a task claimed by `human`, and the ticket
/// page does not show it (br-bnfs). The UI scrolls that to-do into view.
pub(crate) fn todo_url(base: &str, project: &str, id: &str) -> String {
    format!(
        "{base}/p/{}/todos/{}",
        encode(project, false),
        encode(id, false)
    )
}

/// Whether the daemon holds `id` as a to-do of the human. Any lookup failure is "no": the link
/// must still print with the daemon down.
async fn is_human_todo(cli: &Cli, id: &str) -> bool {
    let Ok(client) = crate::commands::client_for_read(cli).await else {
        return false;
    };
    client.get_task(id).await.is_ok_and(|t| {
        t.claimed_by.as_deref() == Some("human")
            && t.state == bridle_api::types::TicketState::Claimed
    })
}

pub async fn run(cli: &Cli, args: &LinkArgs) -> Result<(), CliError> {
    let repo = crate::ticket::repo_root().ok();
    let Some(base) = ui_base_url(&bridle_home(), repo.as_deref()).map_err(anyhow::Error::from)?
    else {
        return Ok(());
    };
    let spec_path = |id: &str| {
        let loaded = specs::load(repo.as_deref()?);
        specs::path_for(&loaded, id)
    };
    // The daemon knows only project-qualified row ids, so a bare ticket id gets the prefix
    // (br-wayb); the UI also expands bare ids in old links.
    let mut target = args.id.clone();
    if crate::ticket::is_bare_id(&target)
        && let Some(repo) = &repo
        && let Ok(prefix) = crate::ticket::own_prefix(cli, repo).await
    {
        target = format!("{prefix}-{target}");
    }
    let link = url(&base, &target, || project::resolve(cli), spec_path)?;
    let is_ticket =
        !target.contains('/') && !target.ends_with(".md") && !specs::is_spec_id(&target);
    if is_ticket && is_human_todo(cli, &target).await {
        println!("{}", todo_url(&base, &project::resolve(cli)?, &target));
    } else {
        println!("{link}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_todo_links_to_the_todo_page() {
        assert_eq!(
            todo_url("http://ui", "bridle", "br-q7ua"),
            "http://ui/p/bridle/todos/br-q7ua"
        );
    }
}
