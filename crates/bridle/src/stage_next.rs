//! `bridle ticket next <id>` (br-quvh, epic br-tj7q): the next unblocked stage of a version 2
//! ticket with everything its writer needs at the moment it writes: the stage's rule (layered
//! like any rule), its template, the output path and the files it depends on. Read-only.
//! Stage rules are tagged `roles: [stage-writer]`, a role no agent has, so they reach a writer
//! only through this command. Design: `docs/design/stages.md`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use bridle_daemon::config::Config;
use bridle_daemon::rules::{self, Layer, RuleState};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::stages::{Stage, State};

/// The human's comments since the stage was written, for a redo.
#[derive(Debug, Clone, Serialize)]
pub struct Comment {
    pub at: DateTime<Utc>,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct Next {
    pub ticket: String,
    /// `None` when every stage is written (and signed where reviewed): "prepared".
    pub stage: Option<String>,
    pub instructions: String,
    pub template: String,
    pub output: Option<PathBuf>,
    pub depends_on: Vec<PathBuf>,
    pub comments: Vec<Comment>,
}

/// The stage file a writer produces; the proposal is a section of the ticket file.
fn output_of(stage: &str, ticket: &Path) -> PathBuf {
    match stage {
        "proposal" => ticket.to_path_buf(),
        _ => crate::stages::folder_of(ticket).join(format!("{stage}.md")),
    }
}

/// The first unblocked stage, `Ok(None)` when none is left, an error naming the gate when the
/// only stages left wait on a signature.
pub fn pick(stages: &[Stage], states: &[(String, State)], id: &str) -> Result<Option<String>> {
    if let Some((n, _)) = states.iter().find(|(_, s)| *s == State::Unblocked) {
        return Ok(Some(n.clone()));
    }
    if let Some((n, _)) = states
        .iter()
        .find(|(_, s)| *s == State::WaitingForSignature)
    {
        bail!("{id}: the next stage waits on the human's signature of {n}");
    }
    if states.iter().any(|(_, s)| *s == State::Waiting) {
        // Waiting with nothing unsigned means a required stage is itself waiting; the graph
        // is acyclic (validated), so this is unreachable short of a malformed ticket.
        let n = &stages[0].name;
        bail!("{id}: no stage is unblocked (check the stages of {n})");
    }
    Ok(None)
}

/// The template for `stage`: the last layer that has `<stage-rule-id>.template` beside its rules.
fn template_of(layers: &[Layer], stage: &str) -> Result<String> {
    let file = format!("stage-{stage}.template");
    for l in layers.iter().rev() {
        let p = l.dir.join(&file);
        match std::fs::read_to_string(&p) {
            Ok(t) => return Ok(t),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).with_context(|| format!("reading {}", p.display())),
        }
    }
    Err(anyhow!("no template {file} in any workflow layer"))
}

/// The resolved body of rule `stage-<stage>`.
fn instructions_of(repo: &Path, config: &Config, stage: &str) -> Result<(String, Vec<Layer>)> {
    let root = config.workflow_root(repo)?;
    let layers = rules::discover_layers(repo, root.as_deref(), &config.packs);
    let res = rules::load_and_resolve(&layers).context("resolving workflow rules")?;
    let id = format!("stage-{stage}");
    let rule = rules::explain(&res, &id).ok_or_else(|| anyhow!("no rule {id} in any layer"))?;
    match rule.state() {
        RuleState::Active { body, .. } => Ok((body.trim().to_string(), layers)),
        RuleState::Disabled { reason } => bail!("rule {id} is disabled: {reason}"),
    }
}

/// Builds the answer. `comments` are the human's, already filtered to those since the stage was
/// last written; `None` when `--comments` was not asked.
pub fn build(
    repo: &Path,
    ticket: &Path,
    id: &str,
    stages: &[Stage],
    states: &[(String, State)],
    comments: impl FnOnce(&Path) -> Result<Vec<Comment>>,
) -> Result<Next> {
    let Some(stage) = pick(stages, states, id)? else {
        return Ok(Next {
            ticket: id.to_string(),
            stage: None,
            instructions: String::new(),
            template: String::new(),
            output: None,
            depends_on: Vec::new(),
            comments: Vec::new(),
        });
    };
    let config = Config::load(repo).context("loading .bridle/config.toml")?;
    let (instructions, layers) = instructions_of(repo, &config, &stage)?;
    let template = template_of(&layers, &stage)?;
    let output = output_of(&stage, ticket);
    let requires = stages
        .iter()
        .find(|s| s.name == stage)
        .map(|s| s.requires.clone())
        .unwrap_or_default();
    let depends_on = requires.iter().map(|r| output_of(r, ticket)).collect();
    Ok(Next {
        ticket: id.to_string(),
        stage: Some(stage),
        instructions,
        template,
        comments: comments(&output)?,
        output: Some(output),
        depends_on,
    })
}

/// Human-readable form, in the order a writer wants it.
pub fn render(n: &Next) -> String {
    use std::fmt::Write as _;
    let Some(stage) = &n.stage else {
        return format!("{}: prepared (every stage is written)\n", n.ticket);
    };
    let mut out = format!("Stage: {stage} of {}\n", n.ticket);
    if let Some(o) = &n.output {
        let _ = writeln!(out, "Output: {}", o.display());
    }
    if n.depends_on.is_empty() {
        out.push_str("Depends on: (nothing)\n");
    } else {
        out.push_str("Depends on:\n");
        for d in &n.depends_on {
            let _ = writeln!(out, "  {}", d.display());
        }
    }
    let _ = write!(out, "\n## Instructions\n\n{}\n", n.instructions);
    let _ = write!(out, "\n## Template\n\n{}\n", n.template.trim_end());
    if !n.comments.is_empty() {
        out.push_str("\n## Comments since this stage was last written\n\n");
        for c in &n.comments {
            let _ = writeln!(out, "- {}: {}", c.at.to_rfc3339(), c.body);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stages() -> Vec<Stage> {
        crate::stages::build(
            &["design".into(), "specs".into(), "tasks".into()],
            &["proposal".into(), "specs".into()],
        )
        .unwrap()
    }

    fn st(v: &[State]) -> Vec<(String, State)> {
        crate::stages::NAMES
            .iter()
            .map(|n| n.to_string())
            .zip(v.iter().copied())
            .collect()
    }

    #[test]
    fn pick_takes_the_first_unblocked_stage() {
        use State::*;
        let s = stages();
        let p = |v: &[State]| pick(&s, &st(v), "br-x");
        assert_eq!(
            p(&[Unblocked, Waiting, Waiting, Waiting]).unwrap(),
            Some("proposal".into())
        );
        assert_eq!(
            p(&[Signed, Written, Unblocked, Waiting]).unwrap(),
            Some("specs".into())
        );
        assert_eq!(p(&[Signed, Written, Signed, Written]).unwrap(), None);
    }

    #[test]
    fn pick_refuses_on_an_unsigned_gate() {
        use State::*;
        let e = pick(
            &stages(),
            &st(&[WaitingForSignature, Waiting, Waiting, Waiting]),
            "br-x",
        )
        .unwrap_err();
        assert!(e.to_string().contains("signature of proposal"));
    }

    #[test]
    fn outputs_and_templates_follow_the_layout() {
        let t = Path::new("docs/tickets/open/a-b1c2.md");
        assert_eq!(output_of("proposal", t), t);
        assert_eq!(
            output_of("design", t),
            Path::new("docs/tickets/open/a-b1c2/design.md")
        );
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("base");
        let proj = dir.path().join("proj");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(base.join("stage-design.template"), "base").unwrap();
        let layer = |k, n: &str, d: &Path| Layer {
            kind: k,
            name: n.into(),
            dir: d.to_path_buf(),
        };
        let layers = [
            layer(rules::LayerKind::Base, "base", &base),
            layer(rules::LayerKind::Project, "project", &proj),
        ];
        assert_eq!(template_of(&layers, "design").unwrap(), "base");
        std::fs::write(proj.join("stage-design.template"), "mine").unwrap();
        assert_eq!(template_of(&layers, "design").unwrap(), "mine");
        assert!(template_of(&layers, "tasks").is_err());
    }
}
