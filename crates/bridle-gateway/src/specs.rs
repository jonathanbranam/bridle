//! A project's specs as bridle organises them (`design/specs/**/*.md`, parsed by `bridle-spec`),
//! for the UI's read-only spec view, and spec ids for link resolution. Read-only: nothing here
//! writes. See docs/design/human-web-ui.md.

use std::path::Path as FsPath;

use axum::Json;
use axum::extract::Path;
use bridle_spec::{Spec, Verification};
use serde::Serialize;
use ts_rs::TS;

use crate::documents::{DocError, blocking, collect_markdown, read_text, repo_of, resolve};

const SPECS_DIR: &str = "design/specs";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SpecScenario {
    pub id: Option<String>,
    pub heading: String,
    pub executable: bool,
    /// 1-based line of the heading.
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SpecRequirement {
    pub id: Option<String>,
    pub heading: String,
    pub protected: bool,
    pub line: usize,
    pub scenarios: Vec<SpecScenario>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SpecDiagnostic {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

/// One spec file. `capability` is the file's name without `.md`, under `design/specs/`. A file
/// that doesn't parse has `diagnostics` and no requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SpecFile {
    pub path: String,
    pub capability: String,
    pub title: Option<String>,
    pub requirements: Vec<SpecRequirement>,
    pub diagnostics: Vec<SpecDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ProjectSpecs {
    pub project: String,
    pub specs: Vec<SpecFile>,
}

/// `GET /api/v1/projects/{project}/specs`.
pub async fn specs_route(Path(project): Path<String>) -> Result<Json<ProjectSpecs>, DocError> {
    let repo = repo_of(&project).await?;
    let specs = blocking(move || Ok(load(&repo))).await?;
    Ok(Json(ProjectSpecs { project, specs }))
}

/// Every spec file under `design/specs/`, by path. No such folder is an empty list.
pub fn load(repo: &FsPath) -> Vec<SpecFile> {
    let mut paths = Vec::new();
    collect_markdown(repo, &repo.join(SPECS_DIR), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .filter_map(|rel| {
            // The same checks as a document read: no symlink out of the repo, text, bounded size.
            let text = read_text(&resolve(repo, &rel).ok()?, &rel).ok()?;
            let capability = rel
                .rsplit('/')
                .next()
                .unwrap_or(&rel)
                .trim_end_matches(".md")
                .to_string();
            let (title, requirements, diagnostics) = match bridle_spec::parse_str(&rel, &text) {
                Ok(s) => shape(s),
                Err(ds) => (
                    None,
                    Vec::new(),
                    ds.into_iter()
                        .map(|d| SpecDiagnostic {
                            line: d.line,
                            column: d.column,
                            message: d.message,
                        })
                        .collect(),
                ),
            };
            Some(SpecFile {
                path: rel,
                capability,
                title,
                requirements,
                diagnostics,
            })
        })
        .collect()
}

fn shape(s: Spec) -> (Option<String>, Vec<SpecRequirement>, Vec<SpecDiagnostic>) {
    let reqs = s
        .requirements
        .into_iter()
        .map(|r| SpecRequirement {
            id: r.id,
            heading: r.title,
            protected: r.protected,
            line: r.line,
            scenarios: r
                .scenarios
                .into_iter()
                .map(|sc| SpecScenario {
                    id: sc.id,
                    heading: sc.title,
                    executable: sc.verification == Verification::Executable,
                    line: sc.line,
                })
                .collect(),
        })
        .collect();
    (s.title, reqs, Vec::new())
}

/// `r-xxxx` or `s-xxxx`, the shape of a requirement or scenario id.
pub fn is_spec_id(t: &str) -> bool {
    t.len() == 6 && (t.starts_with("r-") || t.starts_with("s-"))
}

/// The file a target names: a capability name gives the path; a requirement or scenario id the
/// path plus `#<id>`, the explicit anchor on its heading (`{#r-xxxx}`). None when unknown.
pub fn path_for(specs: &[SpecFile], target: &str) -> Option<String> {
    if is_spec_id(target) {
        return specs
            .iter()
            .find(|f| {
                f.requirements.iter().any(|r| {
                    r.id.as_deref() == Some(target)
                        || r.scenarios.iter().any(|s| s.id.as_deref() == Some(target))
                })
            })
            .map(|f| format!("{}#{target}", f.path));
    }
    specs
        .iter()
        .find(|f| f.capability == target)
        .map(|f| f.path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::resolve_links;

    const SPEC: &str = "# Thing\n\n## Requirements\n\n### Requirement: It works  {#r-ab23}\n\nIt SHALL work.\n\n#### Scenario: Runs  {#s-cd34}\n\n*Verification*: **executable**\n\n- **GIVEN** a\n- **WHEN** b\n- **THEN** c\n\n#### Scenario: Reads well  {#s-ef45}\n\n*Verification*: **non-executable**\n\nLooks fine.\n";

    fn repo_with(files: &[(&str, &str)]) -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tempdir");
        for (p, c) in files {
            let full = d.path().join(p);
            std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
            std::fs::write(full, c).expect("write");
        }
        d
    }

    #[test]
    fn lists_a_spec_with_ids_and_executability() {
        let d = repo_with(&[("design/specs/thing.md", SPEC)]);
        let got = load(d.path());
        assert_eq!(got.len(), 1);
        let f = &got[0];
        assert_eq!(f.capability, "thing");
        assert!(f.diagnostics.is_empty());
        let r = &f.requirements[0];
        assert_eq!(r.id.as_deref(), Some("r-ab23"));
        assert_eq!(r.heading, "It works");
        assert_eq!(r.scenarios.len(), 2);
        assert!(r.scenarios[0].executable);
        assert!(!r.scenarios[1].executable);
        assert_eq!(r.scenarios[1].id.as_deref(), Some("s-ef45"));
    }

    #[test]
    fn project_without_specs_is_empty() {
        let d = repo_with(&[("docs/a.md", "x")]);
        assert!(load(d.path()).is_empty());
    }

    #[test]
    fn a_parse_error_is_a_diagnostic_on_that_file() {
        let d = repo_with(&[
            ("design/specs/good.md", SPEC),
            (
                "design/specs/bad.md",
                "### Requirement: No scenarios\n\nText.\n",
            ),
        ]);
        let got = load(d.path());
        assert_eq!(got.len(), 2);
        let bad = got.iter().find(|f| f.capability == "bad").expect("bad");
        assert!(!bad.diagnostics.is_empty());
        assert!(bad.requirements.is_empty());
        let good = got.iter().find(|f| f.capability == "good").expect("good");
        assert!(good.diagnostics.is_empty());
    }

    #[test]
    fn resolves_spec_ids_and_capability_names() {
        let d = repo_with(&[("design/specs/thing.md", SPEC)]);
        let got = |t: &str| resolve_links(d.path(), &[t.to_string()]).remove(0).path;
        assert_eq!(got("r-ab23"), Some("design/specs/thing.md#r-ab23".into()));
        assert_eq!(got("s-cd34"), Some("design/specs/thing.md#s-cd34".into()));
        assert_eq!(got("thing"), Some("design/specs/thing.md".into()));
        assert_eq!(got("r-zz99"), None);
        assert_eq!(got("nothing"), None);
    }

    #[test]
    fn spec_files_are_readable_and_traversal_is_refused() {
        let d = repo_with(&[("design/specs/thing.md", SPEC), ("secret.md", "x")]);
        assert!(resolve(d.path(), "design/specs/thing.md").is_ok());
        assert!(resolve(d.path(), "design/specs/../../etc/passwd").is_err());
        let got = resolve_links(d.path(), &["design/specs/../../secret".to_string()]);
        assert_eq!(got[0].path, None);
    }
}
