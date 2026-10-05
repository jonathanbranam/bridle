//! `bridle link <id>`: the bridle UI URL for a ticket or task ID (ticket yfjc), so agents
//! talking to the human don't build URLs by hand. Task IDs carry the `br-` prefix style
//! (`<prefix>-<tail>`); ticket IDs are bare. Prints nothing and exits 0 with no base URL.

use bridle_api::discovery::bridle_home;
use bridle_daemon::config::ui_base_url;

use crate::cli::{Cli, LinkArgs};
use crate::error::CliError;
use crate::project;

/// The URL for `id` under `base`; a ticket link needs the project.
fn url(
    base: &str,
    id: &str,
    project: impl FnOnce() -> Result<String, CliError>,
) -> Result<String, CliError> {
    if id.contains('-') {
        Ok(format!("{base}/task?id={id}"))
    } else {
        Ok(format!("{base}/ticket?project={}&id={id}", project()?))
    }
}

pub fn run(cli: &Cli, args: &LinkArgs) -> Result<(), CliError> {
    let repo = crate::ticket::repo_root().ok();
    let Some(base) = ui_base_url(&bridle_home(), repo.as_deref()).map_err(anyhow::Error::from)?
    else {
        return Ok(());
    };
    println!("{}", url(&base, &args.id, || project::resolve(cli))?);
    Ok(())
}
