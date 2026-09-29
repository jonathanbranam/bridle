//! `bridle trace down|up|orphans` (docs/design/traceability.md). Local, no daemon call.

use anyhow::anyhow;
use bridle_spec::trace::{Graph, Node};
use bridle_spec::{Diagnostic, Goal, Spec, parse_goals};

use crate::cli::{Cli, TraceAction, TraceArgs};
use crate::commands::spec_inputs;
use crate::error::CliError;
use crate::render;

#[derive(serde::Serialize)]
struct Row<'a> {
    depth: usize,
    #[serde(flatten)]
    node: &'a Node,
}

pub fn run(cli: &Cli, args: &TraceArgs) -> Result<(), CliError> {
    let graph = load(args)?;
    match &args.action {
        TraceAction::Down { id } => walk(cli, &graph, id, graph.down(id)),
        TraceAction::Up { id } => walk(cli, &graph, id, graph.up(id)),
        TraceAction::Orphans => {
            let orphans = graph.orphans();
            if cli.json {
                render::print_json(&orphans)?;
            } else {
                for n in &orphans {
                    eprintln!("warning: {}:{}: {} traces to nothing", n.file, n.line, n.id);
                    println!("{}  {}", n.id, n.title);
                }
            }
            Ok(())
        }
    }
}

fn walk(
    cli: &Cli,
    graph: &Graph,
    id: &str,
    found: Option<Vec<(usize, &Node)>>,
) -> Result<(), CliError> {
    let found = found.ok_or_else(|| anyhow!("unknown id {id:?}"))?;
    if cli.json {
        let rows: Vec<Row> = found
            .iter()
            .map(|&(depth, node)| Row { depth, node })
            .collect();
        render::print_json(&rows)?;
    } else {
        if let Some(n) = graph.get(id) {
            println!("{}  {}", n.id, n.title);
        }
        for (depth, n) in found {
            println!(
                "{}{}  {}  ({}:{})",
                "  ".repeat(depth),
                n.id,
                n.title,
                n.file,
                n.line
            );
        }
    }
    Ok(())
}

/// Parse the three tiers and link them; any parse or link error is printed
/// (file:line:col: message) and fails the command.
fn load(args: &TraceArgs) -> Result<Graph, CliError> {
    let mut diags: Vec<Diagnostic> = Vec::new();

    let mut goals: Vec<(String, Goal)> = Vec::new();
    for f in spec_inputs(&[], Some(&args.goals))? {
        let name = f.display().to_string();
        let text = std::fs::read_to_string(&f).map_err(|e| anyhow!("reading {name}: {e}"))?;
        let g = parse_goals(&name, &text);
        diags.extend(g.errors);
        goals.extend(g.goals.into_iter().map(|g| (name.clone(), g)));
    }

    let elements =
        match bridle_spec::arch::parse_files(&spec_inputs(std::slice::from_ref(&args.arch), None)?)
        {
            Ok(e) => e,
            Err(ds) => {
                diags.extend(ds);
                Vec::new()
            }
        };

    let mut specs: Vec<(String, Spec)> = Vec::new();
    for f in spec_inputs(&[], Some(&args.specs))? {
        let name = f.display().to_string();
        let text = std::fs::read_to_string(&f).map_err(|e| anyhow!("reading {name}: {e}"))?;
        match bridle_spec::parse_str(&name, &text) {
            Ok(s) => specs.push((name, s)),
            Err(ds) => diags.extend(ds),
        }
    }

    if diags.is_empty() {
        match Graph::build(&goals, &elements, &specs) {
            Ok(g) => return Ok(g),
            Err(ds) => diags = ds,
        }
    }
    for d in &diags {
        eprintln!("{d}");
    }
    Err(anyhow!("trace found {} error(s)", diags.len()).into())
}
