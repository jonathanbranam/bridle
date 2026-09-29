//! `bridle spec coverage`: list executable scenarios with no bound test.
//! See docs/design/specs-to-tests.md.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bridle_spec::Verification;
use serde::Serialize;

use crate::cli::{Cli, SpecCoverageArgs};
use crate::error::CliError;
use crate::render;

#[derive(Serialize)]
struct CoverageReport {
    executable: usize,
    bound: usize,
    unbound: usize,
    scenarios: Vec<UnboundScenario>,
}

#[derive(Serialize, Clone)]
struct UnboundScenario {
    file: String,
    line: usize,
    id: String,
    title: String,
}

pub fn run(cli: &Cli, args: &SpecCoverageArgs) -> Result<(), CliError> {
    let specs_root = args.root.as_deref().unwrap_or(Path::new("design/specs"));
    let specs_root = if specs_root.is_absolute() {
        specs_root.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| CliError::from(anyhow::Error::new(e)))?
            .join(specs_root)
    };

    let test_dirs = if args.tests.is_empty() {
        let mut defaults = Vec::new();
        if Path::new("tests").is_dir() {
            defaults.push(PathBuf::from("tests"));
        }
        if Path::new("test").is_dir() {
            defaults.push(PathBuf::from("test"));
        }
        defaults
    } else {
        args.tests
            .iter()
            .map(|p| {
                if p.is_absolute() {
                    p.clone()
                } else {
                    std::env::current_dir()
                        .ok()
                        .map(|cwd| cwd.join(p))
                        .unwrap_or_else(|| p.clone())
                }
            })
            .collect()
    };

    // Collect all executable scenarios
    let scenarios = collect_scenarios(&specs_root)?;

    // Find which ones are bound in tests
    let mut bound = HashSet::new();
    for test_dir in &test_dirs {
        find_bound_ids(test_dir, &mut bound)?;
    }

    // Report coverage
    let executable = scenarios.iter().filter(|s| s.id.is_some()).count();
    let mut unbound = Vec::new();

    for scenario in scenarios {
        if let Some(ref id) = scenario.id {
            if !bound.contains(id) {
                unbound.push(UnboundScenario {
                    file: scenario.file,
                    line: scenario.line,
                    id: id.clone(),
                    title: scenario.title,
                });
            }
        } else {
            unbound.push(UnboundScenario {
                file: scenario.file,
                line: scenario.line,
                id: "no id, run bridle spec id".to_string(),
                title: scenario.title,
            });
        }
    }

    unbound.sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.line.cmp(&b.line)));

    let bound_count = unbound
        .iter()
        .filter(|u| u.id != "no id, run bridle spec id")
        .count();
    let unbound_count = unbound.len();

    let report = CoverageReport {
        executable,
        bound: executable - bound_count,
        unbound: unbound_count,
        scenarios: unbound.clone(),
    };

    if cli.json {
        render::print_json(&report)?;
    } else {
        println!("executable: {}", report.executable);
        println!("bound: {}", report.bound);
        println!("unbound: {}", report.unbound);
        for scenario in &report.scenarios {
            println!(
                "{}:{}: {} {}",
                scenario.file, scenario.line, scenario.id, scenario.title
            );
        }
    }

    if args.require_all && !report.scenarios.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

struct Scenario {
    file: String,
    line: usize,
    id: Option<String>,
    title: String,
}

fn collect_scenarios(root: &Path) -> anyhow::Result<Vec<Scenario>> {
    let mut scenarios = Vec::new();
    collect_scenarios_recursive(root, &mut scenarios)?;
    Ok(scenarios)
}

fn collect_scenarios_recursive(path: &Path, scenarios: &mut Vec<Scenario>) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            match bridle_spec::parse_file(path) {
                Ok(spec) => {
                    for req in spec.requirements {
                        for scenario in req.scenarios {
                            if scenario.verification == Verification::Executable {
                                scenarios.push(Scenario {
                                    file: path.display().to_string(),
                                    line: scenario.line,
                                    id: scenario.id,
                                    title: scenario.title,
                                });
                            }
                        }
                    }
                }
                Err(_) => {
                    // Skip files with parse errors
                }
            }
        }
        return Ok(());
    }

    if path.is_dir() {
        if let Some(name) = path.file_name()
            && let Some(s) = name.to_str()
            && (s == "node_modules" || s == "target" || s == ".git" || s.starts_with('.'))
        {
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            collect_scenarios_recursive(&path, scenarios)?;
        }
    }

    Ok(())
}

fn find_bound_ids(path: &Path, bound: &mut HashSet<String>) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_file() {
        if is_text_file(path) {
            let content = std::fs::read_to_string(path)?;
            find_ids_in_text(&content, bound);
        }
        return Ok(());
    }

    if path.is_dir() {
        if let Some(name) = path.file_name()
            && let Some(s) = name.to_str()
            && (s == "node_modules" || s == "target" || s == ".git" || s.starts_with('.'))
        {
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            find_bound_ids(&path, bound)?;
        }
    }

    Ok(())
}

fn is_text_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Common text file extensions
    matches!(
        ext,
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "md"
            | "txt"
            | "json"
            | "toml"
            | "yaml"
            | "yml"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "go"
            | "rb"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "swift"
            | "kt"
            | "scala"
            | "clj"
            | "ex"
            | "exs"
            | "erl"
            | "hrl"
            | "php"
            | "pl"
            | "pm"
            | "t"
            | "gherkin"
            | "feature"
            | "html"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "xml"
            | "svg"
            | "gradle"
            | "maven"
            | "sql"
    )
}

fn find_ids_in_text(text: &str, bound: &mut HashSet<String>) {
    for line in text.lines() {
        // Look for 's-' followed by hex characters
        if let Some(pos) = line.find("s-") {
            let rest = &line[pos + 2..];
            let id_part: String = rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
            if !id_part.is_empty() {
                bound.insert(format!("s-{}", id_part));
            }
        }
    }
}
