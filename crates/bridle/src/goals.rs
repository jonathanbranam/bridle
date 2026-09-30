//! `bridle goals list` and `bridle goals propose` (docs/design/goals-tier.md).

use anyhow::anyhow;
use bridle_api::{NewTaskRequest, TaskKind};
use bridle_spec::{Goal, parse_goals};

use crate::cli::{Cli, GoalsAction, GoalsArgs, GoalsListArgs, GoalsProposeProposeArgs};
use crate::commands::{client_for, print_task_row, spec_inputs};
use crate::error::CliError;
use crate::render;

#[derive(serde::Serialize)]
struct Row<'a> {
    file: &'a str,
    #[serde(flatten)]
    goal: &'a Goal,
}

#[derive(serde::Serialize)]
struct Diag {
    file: String,
    line: usize,
    column: usize,
    severity: &'static str,
    message: String,
}

#[derive(serde::Serialize)]
struct Report<'a> {
    goals: Vec<Row<'a>>,
    diagnostics: Vec<Diag>,
}

pub async fn run(cli: &Cli, args: &GoalsArgs) -> Result<(), CliError> {
    match &args.action {
        GoalsAction::List(a) => list(cli, a),
        GoalsAction::Propose(a) => propose(cli, a).await,
    }
}

fn list(cli: &Cli, args: &GoalsListArgs) -> Result<(), CliError> {
    let files = spec_inputs(&[], Some(&args.root))?;
    let mut parsed = Vec::new();
    let mut diagnostics = Vec::new();
    for f in &files {
        let name = f.display().to_string();
        let text = std::fs::read_to_string(f).map_err(|e| anyhow!("reading {name}: {e}"))?;
        let g = parse_goals(&name, &text);
        let mut push = |ds: Vec<bridle_spec::Diagnostic>, severity| {
            diagnostics.extend(ds.into_iter().map(|d| Diag {
                file: d.file,
                line: d.line,
                column: d.column,
                severity,
                message: d.message,
            }));
        };
        push(g.errors, "error");
        push(g.warnings, "warning");
        parsed.extend(g.goals.into_iter().map(|g| (name.clone(), g)));
    }
    let keep = |g: &Goal| {
        args.priority
            .as_deref()
            .is_none_or(|p| g.priority.as_str() == p)
            && args
                .stance
                .as_deref()
                .is_none_or(|s| g.stance.as_str() == s)
    };
    let rows: Vec<Row> = parsed
        .iter()
        .filter(|(_, g)| keep(g))
        .map(|(file, goal)| Row { file, goal })
        .collect();
    let errors = diagnostics.iter().filter(|d| d.severity == "error").count();

    if cli.json {
        render::print_json(&Report {
            goals: rows,
            diagnostics,
        })?;
    } else {
        for d in &diagnostics {
            let prefix = if d.severity == "warning" {
                "warning: "
            } else {
                ""
            };
            eprintln!("{}:{}:{}: {prefix}{}", d.file, d.line, d.column, d.message);
        }
        for r in &rows {
            let g = r.goal;
            println!(
                "{}  {:<6}  {:<7}  {:<11}  {}",
                g.id, g.firmness, g.priority, g.stance, g.title
            );
        }
    }
    if errors > 0 {
        return Err(anyhow!("goals list found {errors} error(s)").into());
    }
    Ok(())
}

async fn propose(cli: &Cli, args: &GoalsProposeProposeArgs) -> Result<(), CliError> {
    let files = spec_inputs(&[], Some(&args.goals_root))?;
    let mut parsed = Vec::new();
    for f in &files {
        let name = f.display().to_string();
        let text = std::fs::read_to_string(f).map_err(|e| anyhow!("reading {name}: {e}"))?;
        let g = parse_goals(&name, &text);
        if !g.errors.is_empty() || !g.warnings.is_empty() {
            for d in &g.errors {
                eprintln!("{}:{}:{}: {}", d.file, d.line, d.column, d.message);
            }
            for d in &g.warnings {
                eprintln!("warning: {}:{}:{}: {}", d.file, d.line, d.column, d.message);
            }
        }
        parsed.extend(g.goals.into_iter().map(|g| (name.clone(), g)));
    }

    let _goal = parsed
        .iter()
        .find(|(_, g)| g.id == args.goal_id)
        .ok_or_else(|| anyhow!("goal {} not found", args.goal_id))?;

    let title = format!("Goal {}: change {}", args.goal_id, args.change.join(", "));
    let body = format!(
        "Proposed changes:\n{}\n\nWhy:\n{}",
        args.change.join("\n"),
        args.why
    );

    let client = client_for(cli).await?;
    let req = NewTaskRequest {
        for_human: false,
        priority: None,
        title,
        kind: TaskKind::Question,
        body,
        components: Vec::new(),
        size: None,
    };
    let task = client.new_task(&req).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}
