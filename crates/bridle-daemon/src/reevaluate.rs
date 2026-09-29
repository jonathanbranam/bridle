//! Re-evaluate tasks for suspect requirements (docs/design/traceability.md): when an
//! `arch-revision` lands, each capability with a requirement whose upstream text changed
//! gets one open `re-evaluate` task listing those requirements.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bridle_spec::trace::Graph;
use bridle_spec::{Diagnostic, Goal, Spec, parse_goals};

/// Suspect requirement ids per capability (spec file stem), computed over the
/// `design/{goals,architecture,specs}` dirs under `repo`. A missing dir counts as empty.
pub fn suspects_by_capability(repo: &Path) -> anyhow::Result<BTreeMap<String, Vec<String>>> {
    let design = repo.join("design");
    let mut diags: Vec<Diagnostic> = Vec::new();

    let mut goals: Vec<(String, Goal)> = Vec::new();
    for f in md_files(&design.join("goals")) {
        let name = f.display().to_string();
        let g = parse_goals(&name, &std::fs::read_to_string(&f)?);
        diags.extend(g.errors);
        goals.extend(g.goals.into_iter().map(|g| (name.clone(), g)));
    }
    let elements = match bridle_spec::arch::parse_files(&md_files(&design.join("architecture"))) {
        Ok(e) => e,
        Err(ds) => {
            diags.extend(ds);
            Vec::new()
        }
    };
    let mut specs: Vec<(String, Spec)> = Vec::new();
    for f in md_files(&design.join("specs")) {
        let name = f.display().to_string();
        match bridle_spec::parse_str(&name, &std::fs::read_to_string(&f)?) {
            Ok(s) => specs.push((name, s)),
            Err(ds) => diags.extend(ds),
        }
    }
    if let Some(d) = diags.first() {
        anyhow::bail!("trace inputs don't parse: {d}");
    }
    let graph = Graph::build(&goals, &elements, &specs).map_err(|ds| {
        anyhow::anyhow!(
            "trace inputs don't link: {}",
            ds.first().map_or_else(String::new, |d| d.to_string())
        )
    })?;

    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for s in graph.suspects() {
        let cap = Path::new(&s.file)
            .file_stem()
            .map_or_else(|| s.file.clone(), |x| x.to_string_lossy().into_owned());
        let ids = out.entry(cap).or_default();
        if !ids.contains(&s.id) {
            ids.push(s.id);
        }
    }
    Ok(out)
}

fn md_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    let mut out = Vec::new();
    for e in entries {
        if e.is_dir() {
            out.extend(md_files(&e));
        } else if e.extension().is_some_and(|x| x == "md") {
            out.push(e);
        }
    }
    out
}

pub fn title(capability: &str, arch_task: &str) -> String {
    format!("re-evaluate {capability} after {arch_task}")
}

pub fn body(ids: &[String]) -> String {
    format!(
        "These requirements trace to architecture text that changed: {}.\n\nFor each, read the \
         upstream change, then either `bridle trace confirm <id>` (still valid) or edit the \
         requirement (which may make its scenarios suspect).",
        ids.join(", ")
    )
}
