//! `bridle prime worker|planner`: the role's rules, facts and guide pointers, plus the
//! scope of any named components (docs/design/components.md, "What an agent is handed").
//! Everything is read from disk and printed; nothing is rendered to files.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bridle_api::TaskKind;
use bridle_daemon::config::Config;
use bridle_daemon::rules::{self, Layer, LayerKind, Resolution, RuleState};

/// README.md is printed in full only up to this many lines; longer, it's a pointer.
const README_MAX_LINES: usize = 40;

/// What an agent on an `explore` task is told first (docs/design/explorations.md).
const EXPLORE_PARAGRAPH: &str = "You are exploring an alternative. Contradicting the \
architecture, goals and specs is intended: don't bring the code back into line and \
don't flag the divergence. Exploration code stays on `explore/<id>`; only the findings \
doc merges, into `design/explore/<id>/`.";

/// `role` is the rule-tag name (`worker`, `product-manager`), `title` what the heading says.
/// `kind` is the kind of the task being worked, when known.
pub fn render(
    repo: &Path,
    config: &Config,
    role: &str,
    title: &str,
    components: &[String],
    kind: Option<TaskKind>,
) -> Result<String> {
    let workflow_root = config.workflow.as_deref().map(Path::new);
    let base_layers = rules::discover_layers(repo, workflow_root, &config.packs);
    let base = rules::load_and_resolve(&base_layers).context("resolving workflow rules")?;

    let mut out = format!("# Role: {title}\n\n");
    if role == "worker" && kind == Some(TaskKind::Explore) {
        let _ = write!(out, "{EXPLORE_PARAGRAPH}\n\n");
    }
    out.push_str("## Rules\n\n");
    out.push_str(&rules_section(&base, role, None));
    out.push_str(&facts_and_guides(&base_layers));

    for id in components {
        let mut layers = base_layers.clone();
        let chain = rules::discover_component_layers(repo, config, id)
            .with_context(|| format!("no component {id:?} in .bridle/config.toml"))?;
        let chain_names: Vec<String> = chain.iter().map(|l| l.name.clone()).collect();
        layers.extend(chain.iter().cloned());
        let res = rules::load_and_resolve(&layers)
            .with_context(|| format!("resolving rules for component {id}"))?;
        let _ = write!(
            out,
            "\n# Component: {id} ({})\n\n## Rules\n\n",
            chain_names.join(" -> ")
        );
        out.push_str(&rules_section(&res, role, Some(LayerKind::Component)));
        out.push_str(&facts_and_guides(&chain));
        out.push_str(&docs_section(repo, config, id));
    }

    let others: Vec<String> = config
        .components
        .iter()
        .filter(|(id, _)| !components.contains(id))
        .map(|(id, c)| match &c.docs {
            Some(d) => format!("{id} ({d})"),
            None => id.clone(),
        })
        .collect();
    if !others.is_empty() {
        let _ = write!(out, "\nOther components: {}\n", others.join(", "));
    }
    Ok(out)
}

/// Active rules tagged for `role` (or untagged). With `only_kind`, just those a layer of
/// that kind wins, plus disables it made: the delta on top of what's already printed.
fn rules_section(res: &Resolution, role: &str, only_kind: Option<LayerKind>) -> String {
    let mut out = String::new();
    for rule in res.rules.values() {
        let winner = rule.winning_layer();
        if only_kind.is_some_and(|k| winner.kind != k) {
            continue;
        }
        match rule.state() {
            RuleState::Active {
                severity,
                roles,
                body,
                ..
            } => {
                if !roles.is_empty() && !roles.iter().any(|r| r == role) {
                    continue;
                }
                let sev = severity.map(|s| format!("{s}, ")).unwrap_or_default();
                let _ = writeln!(out, "- {} [{sev}{winner}]: {}", rule.id, body.trim());
            }
            RuleState::Disabled { reason } => {
                let _ = writeln!(out, "- {} disabled by {winner}: {reason}", rule.id);
            }
        }
    }
    if out.is_empty() {
        out.push_str("(none)\n");
    }
    out
}

/// Facts (`facts.md`) and guide paths (`guides/*.md`) beside each layer's `rules/` dir.
fn facts_and_guides(layers: &[Layer]) -> String {
    let mut facts = String::new();
    let mut guides: Vec<PathBuf> = Vec::new();
    for layer in layers {
        let Some(root) = layer.dir.parent() else {
            continue;
        };
        if let Ok(text) = std::fs::read_to_string(root.join("facts.md")) {
            let _ = writeln!(facts, "{} ({}):\n{}", layer.name, layer.kind, text.trim());
        }
        if let Ok(rd) = std::fs::read_dir(root.join("guides")) {
            let mut found: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "md"))
                .collect();
            found.sort();
            guides.extend(found);
        }
    }
    let mut out = String::new();
    if !facts.is_empty() {
        let _ = write!(out, "\n## Facts\n\n{facts}");
    }
    if !guides.is_empty() {
        out.push_str("\n## Guides\n\n");
        for g in guides {
            let _ = writeln!(out, "- {}", g.display());
        }
    }
    out
}

fn docs_section(repo: &Path, config: &Config, id: &str) -> String {
    let Some(docs) = config.components.get(id).and_then(|c| c.docs.as_deref()) else {
        return String::new();
    };
    let dir = repo.join(docs);
    let mut out = format!("\n## Docs\n\nfolder: {docs}\n");
    for name in ["README.md", "roadmap.md"] {
        if dir.join(name).is_file() {
            let _ = writeln!(out, "- {docs}/{name}");
        }
    }
    if let Ok(text) = std::fs::read_to_string(dir.join("README.md"))
        && text.lines().count() <= README_MAX_LINES
    {
        let _ = write!(out, "\nREADME.md:\n{}\n", text.trim());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, text: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(p, text).expect("write");
    }

    fn fixture() -> (tempfile::TempDir, Config) {
        let dir = tempfile::tempdir().expect("tempdir");
        let r = dir.path();
        write(
            r,
            ".bridle/config.toml",
            "[components.client-games]\ndocs = \"docs/client\"\n\
             [components.dungeon]\nparent = \"client-games\"\ndocs = \"docs/dungeon\"\n\
             [components.client-play]\ndocs = \"docs/play\"\n",
        );
        write(
            r,
            ".bridle/rules/proj.md",
            "---\nid: proj\nroles: [worker]\n---\nproject rule\n",
        );
        write(
            r,
            ".bridle/components/client-games/rules/cg.md",
            "---\nid: cg\n---\nclient rule\n",
        );
        write(
            r,
            ".bridle/components/dungeon/rules/dg.md",
            "---\nid: dg\n---\ndungeon rule\n",
        );
        write(r, ".bridle/components/dungeon/facts.md", "dungeon fact\n");
        write(
            r,
            ".bridle/components/client-play/rules/cp.md",
            "---\nid: cp\n---\nplay rule\n",
        );
        write(r, "docs/dungeon/README.md", "who this is for\n");
        write(r, "docs/dungeon/roadmap.md", "later\n");
        write(r, "docs/client/README.md", &"line\n".repeat(41));
        let config = Config::load(r).expect("config");
        (dir, config)
    }

    #[test]
    fn named_chain_shows_sibling_only_listed() {
        let (dir, config) = fixture();
        let out = render(
            dir.path(),
            &config,
            "worker",
            "worker",
            &["dungeon".into()],
            None,
        )
        .expect("render");
        assert!(out.contains("project rule"), "{out}");
        assert!(
            out.contains("client rule") && out.contains("dungeon rule"),
            "{out}"
        );
        assert!(out.contains("dungeon fact"), "{out}");
        assert!(!out.contains("play rule"), "{out}");
        assert!(
            out.contains("Other components: client-games (docs/client), client-play (docs/play)"),
            "{out}"
        );
        assert!(
            out.contains("- docs/dungeon/README.md") && out.contains("- docs/dungeon/roadmap.md")
        );
        assert!(out.contains("who this is for"));
        // Ancestors get no docs pointers of their own.
        assert!(!out.contains("folder: docs/client"), "{out}");
    }

    #[test]
    fn long_readme_is_a_pointer_only() {
        let (dir, config) = fixture();
        let out = render(
            dir.path(),
            &config,
            "worker",
            "worker",
            &["client-games".into()],
            None,
        )
        .expect("render");
        assert!(out.contains("- docs/client/README.md"));
        assert!(!out.contains("README.md:\n"), "{out}");
    }

    #[test]
    fn rules_for_other_roles_are_left_out_and_no_components_prints_no_component_sections() {
        let (dir, config) = fixture();
        let out =
            render(dir.path(), &config, "product-manager", "planner", &[], None).expect("render");
        assert!(!out.contains("project rule"), "{out}");
        assert!(!out.contains("# Component:"), "{out}");
        assert!(out.contains("Other components: client-games (docs/client), client-play (docs/play), dungeon (docs/dungeon)"), "{out}");
    }

    #[test]
    fn unknown_component_is_an_error() {
        let (dir, config) = fixture();
        assert!(
            render(
                dir.path(),
                &config,
                "worker",
                "worker",
                &["nope".into()],
                None
            )
            .is_err()
        );
    }

    #[test]
    fn explore_task_gets_the_paragraph_and_a_feature_task_does_not() {
        let (dir, config) = fixture();
        let go = |kind| render(dir.path(), &config, "worker", "worker", &[], kind).expect("render");
        let explore = go(Some(TaskKind::Explore));
        assert!(
            explore.contains("You are exploring an alternative"),
            "{explore}"
        );
        assert!(!go(Some(TaskKind::Feature)).contains("exploring an alternative"));
        assert!(!go(None).contains("exploring an alternative"));
    }

    #[test]
    fn repo_base_explorations_rule_reaches_every_role() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = Config::load(&repo).expect("config");
        for role in ["worker", "product-manager"] {
            let out = render(&repo, &config, role, role, &[], None).expect("render");
            assert!(out.contains("- explorations [must, base"), "{role}: {out}");
        }
    }
}
