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
    println!(
        "{}",
        url(&base, &target, || project::resolve(cli), spec_path)?
    );
    Ok(())
}
