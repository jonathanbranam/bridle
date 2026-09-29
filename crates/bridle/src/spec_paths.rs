//! Which spec files a `bridle spec <cmd>` acts on: shared by every `spec`
//! subcommand so the defaults agree (docs/design/specs.md).

use std::path::{Path, PathBuf};

use anyhow::Context;

/// The specs directory used when no paths are given and no `--root` is set.
pub const DEFAULT_ROOT: &str = "design/specs";

/// Expand explicit paths (or `root`, else [`DEFAULT_ROOT`]) into spec files:
/// a directory yields every `*.md` under it, recursively, sorted so output is
/// stable.
pub fn resolve(paths: &[PathBuf], root: Option<&Path>) -> anyhow::Result<Vec<PathBuf>> {
    let default = [root.map_or_else(|| PathBuf::from(DEFAULT_ROOT), Path::to_path_buf)];
    let roots = if paths.is_empty() {
        &default[..]
    } else {
        paths
    };
    let mut out = Vec::new();
    for r in roots {
        collect(r, &mut out)?;
    }
    Ok(out)
}

fn collect(path: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if path.is_dir() {
        let mut entries = std::fs::read_dir(path)
            .with_context(|| format!("reading {}", path.display()))?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("reading {}", path.display()))?;
        entries.sort();
        for e in entries {
            if e.is_dir() || e.extension().is_some_and(|x| x == "md") {
                collect(&e, out)?;
            }
        }
    } else if path.exists() {
        out.push(path.to_path_buf());
    } else {
        anyhow::bail!("no such file or directory: {}", path.display());
    }
    Ok(())
}
