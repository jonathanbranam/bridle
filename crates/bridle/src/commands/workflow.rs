//! Workflow commands: rules, sync, spec, arch, explore, pane.

use super::*;

/// `bridle rules explain`/`diff`: local and static, like `cost audit` — no
/// daemon call, just `.bridle/config.toml` and the layer directories it
/// points at, read from the current directory (docs/design/workflow-layers.md,
/// `bridle_daemon::rules`).
pub(super) async fn rules(cli: &Cli, args: &RulesArgs) -> Result<(), CliError> {
    match &args.action {
        RulesAction::Explain(e) => rules_explain(cli, e).await,
        RulesAction::Diff(d) => rules_diff(cli, d).await,
    }
}

pub(super) fn resolve_workflow_rules(
    repo: &Path,
    component: Option<&str>,
) -> Result<bridle_daemon::rules::Resolution, CliError> {
    use bridle_daemon::config::Config;
    use bridle_daemon::rules;

    let config = Config::load(repo).context("loading .bridle/config.toml")?;
    let workflow_root = config.workflow_root(repo).map_err(anyhow::Error::new)?;
    let mut layers = rules::discover_layers(repo, workflow_root.as_deref(), &config.packs);
    if let Some(id) = component {
        let chain = rules::discover_component_layers(repo, &config, id)
            .ok_or_else(|| anyhow::anyhow!("no component {id:?} in .bridle/config.toml"))?;
        layers.extend(chain);
    }
    rules::load_and_resolve(&layers)
        .map_err(|e| CliError::from(anyhow::Error::new(e).context("resolving workflow rules")))
}

pub(super) async fn rules_explain(cli: &Cli, args: &RulesExplainArgs) -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let resolution = resolve_workflow_rules(&repo, args.component.as_deref())?;
    let Some(rule) = bridle_daemon::rules::explain(&resolution, &args.id) else {
        return Err(anyhow::anyhow!("no rule with id {:?} in any layer", args.id).into());
    };

    if cli.json {
        render::print_json(rule)?;
        return Ok(());
    }

    println!("{}: won by {}", rule.id, rule.winning_layer());
    for entry in &rule.history {
        let action = match entry.override_kind {
            None => "defines".to_string(),
            Some(kind) => format!("{kind}s it"),
        };
        print!("  {} {action}", entry.layer);
        if let Some(reason) = &entry.reason {
            print!(" ({reason})");
        }
        println!();
    }
    match rule.state() {
        bridle_daemon::rules::RuleState::Active { severity, body, .. } => {
            if let Some(s) = severity {
                println!("severity: {s}");
            }
            println!("{body}");
        }
        bridle_daemon::rules::RuleState::Disabled { reason } => {
            println!("disabled: {reason}");
        }
    }
    Ok(())
}

pub(super) async fn rules_diff(cli: &Cli, args: &RulesDiffArgs) -> Result<(), CliError> {
    if !args.project_layer && args.component.is_none() {
        return Err(anyhow::anyhow!(
            "rules diff needs a mode: --project-layer or --component <id>"
        )
        .into());
    }
    let repo = std::env::current_dir().context("current directory")?;
    let resolution = resolve_workflow_rules(&repo, args.component.as_deref())?;
    if args.component.is_some() {
        let diffs = bridle_daemon::rules::diff_components(&resolution);
        if cli.json {
            render::print_json(&diffs)?;
            return Ok(());
        }
        if diffs.is_empty() {
            println!("the component chain changes nothing");
        }
        for d in &diffs {
            let action = d
                .diff
                .override_kind
                .map(|k| format!("{k}s"))
                .unwrap_or_else(|| "defines".to_string());
            print!("{}: {} {action}", d.component, d.diff.id);
            if let Some(reason) = &d.diff.reason {
                print!(" ({reason})");
            }
            println!();
        }
        return Ok(());
    }
    let diffs = bridle_daemon::rules::diff_project(&resolution);

    if cli.json {
        render::print_json(&diffs)?;
        return Ok(());
    }

    if diffs.is_empty() {
        println!("project layer changes nothing");
        return Ok(());
    }
    for d in &diffs {
        let action = d
            .override_kind
            .map(|k| format!("{k}s"))
            .unwrap_or_else(|| "defines".to_string());
        print!("{} {action}", d.id);
        if let Some(reason) = &d.reason {
            print!(" ({reason})");
        }
        println!();
    }
    Ok(())
}

/// `bridle sync`: local and static, like `rules explain`/`diff` — renders
/// the resolved workflow layers into CLAUDE.md, .claude/skills,
/// .claude/agents and .claude/settings.json's hooks
/// (docs/design/workflow-layers.md, `bridle_daemon::sync`).
pub(super) async fn sync(cli: &Cli) -> Result<(), CliError> {
    use bridle_daemon::config::Config;
    use bridle_daemon::rules;

    let repo = std::env::current_dir().context("current directory")?;
    let config = Config::load(&repo).context("loading .bridle/config.toml")?;
    let workflow_root = config.workflow_root(&repo).map_err(anyhow::Error::new)?;
    let layers = rules::discover_layers(&repo, workflow_root.as_deref(), &config.packs);
    let report = bridle_daemon::sync::sync(&repo, &layers, &config.commands, &config.branches)
        .map_err(|e| CliError::from(anyhow::Error::new(e).context("syncing workflow layers")))?;

    if cli.json {
        render::print_json(&report)?;
        return Ok(());
    }

    println!(
        "CLAUDE.md: {}",
        if report.claude_md_changed {
            "updated"
        } else {
            "unchanged"
        }
    );
    let list = |items: &[String]| {
        if items.is_empty() {
            "none".to_string()
        } else {
            items.join(", ")
        }
    };
    println!("skills: {}", list(&report.skills));
    println!("agents: {}", list(&report.agents));
    println!("hooks: {}", list(&report.hook_events));
    Ok(())
}

#[derive(serde::Serialize)]
pub(super) struct SpecDiagnostic {
    file: String,
    line: usize,
    column: usize,
    severity: &'static str,
    message: String,
}

#[derive(serde::Serialize)]
pub(super) struct SpecCheckReport {
    files: usize,
    errors: usize,
    warnings: usize,
    diagnostics: Vec<SpecDiagnostic>,
}

/// Every `*.md` under `path` (or `path` itself if it's a file), sorted so
/// output is stable.
pub(super) fn spec_files(path: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if path.is_dir() {
        let mut entries = std::fs::read_dir(path)
            .with_context(|| format!("reading {}", path.display()))?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("reading {}", path.display()))?;
        entries.sort();
        for e in entries {
            if e.is_dir() || e.extension().is_some_and(|x| x == "md") {
                spec_files(&e, out)?;
            }
        }
    } else if path.exists() {
        out.push(path.to_path_buf());
    } else {
        anyhow::bail!("no such file or directory: {}", path.display());
    }
    Ok(())
}

/// The spec files named by `paths`, else those under `root` (default
/// `design/specs`): the defaults every `bridle spec` subcommand shares.
pub(crate) fn spec_inputs(paths: &[PathBuf], root: Option<&Path>) -> anyhow::Result<Vec<PathBuf>> {
    let default = [root.map_or_else(|| PathBuf::from("design/specs"), Path::to_path_buf)];
    let roots = if paths.is_empty() {
        &default[..]
    } else {
        paths
    };
    let mut files = Vec::new();
    for r in roots {
        spec_files(r, &mut files)?;
    }
    Ok(files)
}

/// `bridle spec check`: local, no daemon call (docs/design/specs.md).
pub(super) fn spec(cli: &Cli, args: &SpecArgs) -> Result<(), CliError> {
    let args = match &args.action {
        SpecAction::Check(args) => args,
        SpecAction::Export(_) => unreachable!("dispatched to spec_export"),
        SpecAction::Id(args) => return crate::specid::run(cli, args),
        SpecAction::Import(args) => return crate::spec_import::run(cli, args),
        SpecAction::Coverage(args) => return crate::spec_coverage::run(cli, args),
    };
    let files = spec_inputs(&args.paths, args.root.as_deref())?;

    let mut diagnostics = Vec::new();
    for file in &files {
        let name = file.display().to_string();
        match bridle_spec::parse_file(file) {
            Ok(spec) => {
                for r in spec.requirements.iter().filter(|r| r.id.is_none()) {
                    diagnostics.push(SpecDiagnostic {
                        file: name.clone(),
                        line: r.line,
                        column: 1,
                        severity: if args.require_ids { "error" } else { "warning" },
                        message: format!(
                            "requirement {:?} has no id (expected '{{#r-xxxx}}' after the title)",
                            r.title
                        ),
                    });
                }
            }
            Err(bridle_spec::ParseFileError::Diagnostics(ds)) => {
                diagnostics.extend(ds.into_iter().map(|d| SpecDiagnostic {
                    file: d.file,
                    line: d.line,
                    column: d.column,
                    severity: "error",
                    message: d.message,
                }));
            }
            Err(e) => return Err(anyhow::Error::new(e).into()),
        }
    }
    let count = |sev| diagnostics.iter().filter(|d| d.severity == sev).count();
    let report = SpecCheckReport {
        files: files.len(),
        errors: count("error"),
        warnings: count("warning"),
        diagnostics,
    };

    if cli.json {
        render::print_json(&report)?;
    } else {
        for d in &report.diagnostics {
            let prefix = if d.severity == "warning" {
                "warning: "
            } else {
                ""
            };
            println!("{}:{}:{}: {prefix}{}", d.file, d.line, d.column, d.message);
        }
        println!(
            "{} file(s) checked: {} error(s), {} warning(s)",
            report.files, report.errors, report.warnings
        );
    }
    if report.errors > 0 {
        return Err(anyhow::anyhow!("spec check found {} error(s)", report.errors).into());
    }
    Ok(())
}

/// `bridle arch list`: local, no daemon call (docs/design/architecture-tier.md).
pub(super) async fn arch(cli: &Cli, args: &crate::cli::ArchArgs) -> Result<(), CliError> {
    match &args.action {
        crate::cli::ArchAction::List(args) => arch_list(cli, args),
        crate::cli::ArchAction::Propose(args) => arch_propose(cli, args).await,
    }
}

pub(super) fn arch_list(cli: &Cli, args: &crate::cli::ArchListArgs) -> Result<(), CliError> {
    let files = spec_inputs(std::slice::from_ref(&args.root), None)?;
    let mut elements = match bridle_spec::arch::parse_files(&files) {
        Ok(els) => els,
        Err(ds) => {
            for d in &ds {
                eprintln!("{d}");
            }
            return Err(anyhow::anyhow!("arch list found {} error(s)", ds.len()).into());
        }
    };
    if args.invariants {
        elements.retain(|e| e.invariant);
    }
    if cli.json {
        render::print_json(&elements)?;
    } else {
        for e in &elements {
            let flag = if e.invariant { " invariant" } else { "" };
            println!("{}{flag}  {}  ({}:{})", e.id, e.title, e.file, e.line);
        }
    }
    Ok(())
}

pub(super) async fn arch_propose(cli: &Cli, args: &ArchProposeArgs) -> Result<(), CliError> {
    let files = spec_inputs(std::slice::from_ref(&args.arch_root), None)?;
    if let Err(ds) = bridle_spec::arch::parse_files(&files) {
        for d in &ds {
            eprintln!("{d}");
        }
        return Err(anyhow::anyhow!("arch parse found {} error(s)", ds.len()).into());
    }

    let argument = if args.argument.is_some() || args.argument_file.is_some() {
        read_text(&args.argument, &args.argument_file, "argument")?
    } else {
        String::new()
    };

    let client = client_for(cli).await?;
    let req = NewTaskRequest {
        ticket: None,
        parent: None,
        for_human: false,
        priority: None,
        title: args.title.clone(),
        kind: TaskKind::ArchRevision,
        body: argument,
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

/// `bridle explore`: local, no daemon call (docs/design/explorations.md).
pub(super) fn explore(action: &crate::cli::ExploreAction) -> Result<(), CliError> {
    use crate::cli::ExploreAction as A;
    let (id, status) = match action {
        A::Check(args) => {
            let files = spec_inputs(&args.paths, Some(Path::new("design/explore")))?;
            let mut errors = 0;
            for f in &files {
                let name = f.display().to_string();
                let text = std::fs::read_to_string(f).with_context(|| format!("reading {name}"))?;
                if let Err(ds) = bridle_spec::explore::parse_str(&name, &text) {
                    errors += ds.len();
                    ds.iter().for_each(|d| println!("{d}"));
                }
            }
            println!("{} file(s) checked: {errors} error(s)", files.len());
            if errors > 0 {
                return Err(anyhow::anyhow!("explore check found {errors} error(s)").into());
            }
            return Ok(());
        }
        A::New(a) => (&a.id, None),
        A::Conclude(a) => (&a.id, Some("concluded")),
        A::Abandon(a) => (&a.id, Some("abandoned")),
    };
    if id.is_empty() || id.contains(['/', '\\']) || id.starts_with('.') {
        return Err(anyhow::anyhow!("invalid exploration id '{id}'").into());
    }
    let path = Path::new("design/explore").join(id).join("findings.md");
    let Some(status) = status else {
        if path.exists() {
            return Err(anyhow::anyhow!("{} already exists", path.display()).into());
        }
        std::fs::create_dir_all(path.parent().expect("has a parent"))
            .with_context(|| format!("creating {}", path.display()))?;
        std::fs::write(&path, bridle_spec::explore::scaffold(id))
            .with_context(|| format!("writing {}", path.display()))?;
        println!("{}", path.display());
        return Ok(());
    };
    let name = path.display().to_string();
    let text = std::fs::read_to_string(&path).with_context(|| format!("reading {name}"))?;
    match bridle_spec::explore::set_status(&name, &text, status) {
        Ok(out) => std::fs::write(&path, out).with_context(|| format!("writing {name}"))?,
        Err(ds) => {
            ds.iter().for_each(|d| eprintln!("{d}"));
            return Err(anyhow::anyhow!("{name} does not check; not changing its status").into());
        }
    }
    println!("{name}: status {status}");
    Ok(())
}

pub(super) fn pane(action: &PaneAction) -> Result<(), CliError> {
    match action {
        PaneAction::Tag { name } => {
            crate::pane::tag_pane_with_error(name)?;
            println!("pane tagged: @bridle={name}");
        }
        PaneAction::Untag => {
            crate::pane::untag_pane_with_error()?;
            println!("pane untagged");
        }
    }
    Ok(())
}

/// `bridle spec export`: local, except `--task`, which asks the daemon for the task's
/// declared impact (docs/design/specs-to-tests.md).
pub(super) async fn spec_export(cli: &Cli, args: &SpecExportArgs) -> Result<(), CliError> {
    let files = spec_inputs(&args.paths, args.root.as_deref())?;

    let mut selection: Option<HashSet<String>> = None;
    if !args.scenario.is_empty() {
        selection = Some(args.scenario.iter().cloned().collect());
    }
    if let Some(task) = &args.task {
        let impact = client_for_read(cli).await?.get_task(task).await?.impact;
        if impact.is_empty() {
            return Err(anyhow::anyhow!("{task} declares no impact; nothing to select").into());
        }
        selection.get_or_insert_with(HashSet::new).extend(
            impact
                .modify
                .into_iter()
                .chain(impact.add_under)
                .chain(impact.remove),
        );
    }

    let mut specs = Vec::new();
    let mut diagnostics = Vec::new();
    for file in &files {
        match bridle_spec::parse_file(file) {
            Ok(mut spec) => {
                if let Some(ids) = &selection {
                    crate::spec_export::select(&mut spec, ids);
                    if spec.requirements.is_empty() {
                        continue;
                    }
                }
                specs.push((file, spec));
            }
            Err(bridle_spec::ParseFileError::Diagnostics(ds)) => diagnostics.extend(ds),
            Err(e) => return Err(anyhow::Error::new(e).into()),
        }
    }
    if !diagnostics.is_empty() {
        for d in &diagnostics {
            eprintln!("{d}");
        }
        return Err(
            anyhow::anyhow!("not exporting: {} spec diagnostic(s)", diagnostics.len()).into(),
        );
    }

    let write = |dir: &Path, name: &str, text: &str| -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        let path = dir.join(name);
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    };
    match args.format {
        SpecFormatArg::Gherkin => {
            let dir = args
                .out
                .clone()
                .unwrap_or_else(|| PathBuf::from(".bridle/cache/features"));
            let mut written = Vec::new();
            for (file, spec) in &specs {
                let cap = crate::spec_export::capability(file);
                let text = crate::spec_export::gherkin(&cap, spec);
                written.push(
                    write(&dir, &format!("{cap}.feature"), &text)?
                        .display()
                        .to_string(),
                );
            }
            if cli.json {
                render::print_json(&written)?;
            } else {
                for p in &written {
                    println!("{p}");
                }
            }
        }
        SpecFormatArg::Json => {
            let doc = crate::spec_export::JsonExport {
                version: crate::spec_export::JSON_VERSION,
                specs: specs
                    .iter()
                    .map(|(f, s)| crate::spec_export::json_spec(f, s))
                    .collect(),
            };
            match &args.out {
                Some(dir) => {
                    let text = serde_json::to_string_pretty(&doc).context("encoding json")?;
                    let path = write(dir, "specs.json", &format!("{text}\n"))?;
                    println!("{}", path.display());
                }
                None => render::print_json(&doc)?,
            }
        }
    }
    Ok(())
}
