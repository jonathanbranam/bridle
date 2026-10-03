//! `bridle cost audit`: a static count of what bridle injects into every agent's
//! context with no work done (docs/design/usage-and-budget.md, "Designing for fewer
//! tokens", and "Tracking token use over time").
//!
//! The design names five categories: prime, the rendered system-prompt file, skill
//! descriptions, MCP tool schemas and hook boilerplate. Only one exists in bridle
//! today — the rendered system-prompt file, specifically its role-scoped, cacheable
//! part ([`config::stable_system_prompt`]; see that function's own doc comment for why
//! the agent-specific identity sentence appended after it is excluded). Prime and
//! skills are unbuilt; the MCP server and hooks are deferred nice-to-haves
//! (docs/tickets/open/v1-follow-ups-from-the-build-9c6e.md). [`measure`] folds each
//! in once it exists, rather than reporting a placeholder zero for it now.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{Config, stable_system_prompt};

/// `.bridle/cost-baseline.json`, relative to the repo.
pub const BASELINE_FILENAME: &str = "cost-baseline.json";

/// How much a role's measured cost may grow over its baseline before `--check` fails,
/// as a percentage. A constant, not buried in the comparison logic, so it's easy to
/// find and retune. The design doc doesn't pin down a number; 10% is a starting point
/// that should catch a rules or prompt-file change worth reviewing without flagging
/// ordinary noise (a role's stable prompt only changes when someone edits it, so any
/// change here is deliberate; 10% gives a little room before that shows up as a
/// failure).
pub const GROWTH_THRESHOLD_PERCENT: f64 = 10.0;

/// Bridle has no tokenizer dependency (checked before writing this: nothing in
/// `Cargo.toml` counts tokens), and the design treats this count as "only a relative
/// unit, useful for comparing things with each other" rather than billing — so an
/// approximation is enough. ~4 bytes per token is the rule of thumb Anthropic's own
/// docs give for English text; good enough to catch a role's fixed overhead growing,
/// not exact. Pulling in a real tokenizer crate would fix the exact count but cost a
/// dependency for a number the design explicitly doesn't need to be exact.
pub fn approx_tokens(text: &str) -> usize {
    text.len().div_ceil(4)
}

/// Every configured role's [`stable_system_prompt`] size, in approximate tokens, from
/// a fresh render (not a cached figure), so an edit to a rule or prompt file shows up
/// immediately.
pub fn measure(config: &Config, repo: &Path) -> BTreeMap<String, usize> {
    config
        .roles
        .iter()
        .map(|(name, role)| {
            let prompt = stable_system_prompt(
                name,
                role,
                repo,
                &config.branches,
                &config.commands,
                &config.role_rules_text(repo, name),
            );
            (name.clone(), approx_tokens(&prompt))
        })
        .collect()
}

/// `.bridle/cost-baseline.json`: a record a human or worker commits deliberately when
/// they mean to move the baseline, not output this tool regenerates on every run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Baseline {
    pub roles: BTreeMap<String, usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum BaselineError {
    #[error("reading {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parsing {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl Baseline {
    /// `None` when the file doesn't exist yet: a repo with no committed baseline just
    /// gets an empty one, so every role reports as new rather than failing to load.
    pub fn load(repo: &Path) -> Result<Option<Self>, BaselineError> {
        let path = repo.join(".bridle").join(BASELINE_FILENAME);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let baseline =
                    serde_json::from_str(&text).map_err(|source| BaselineError::Parse {
                        path: path.clone(),
                        source,
                    })?;
                Ok(Some(baseline))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(BaselineError::Read { path, source }),
        }
    }
}

/// One role's measured size next to its baseline, and whether it grew past
/// [`GROWTH_THRESHOLD_PERCENT`].
#[derive(Debug, Clone, Serialize)]
pub struct RoleAudit {
    pub role: String,
    pub current_tokens: usize,
    /// `None` for a role the baseline doesn't mention yet (a new role, or one added
    /// since the baseline was last committed).
    pub baseline_tokens: Option<usize>,
    /// `None` alongside `baseline_tokens: None`.
    pub change_percent: Option<f64>,
    /// Always `false` with no baseline to compare against: nothing to fail yet.
    pub over_threshold: bool,
}

/// Compares a fresh [`measure`] against the committed [`Baseline`]. A role missing
/// from the baseline is reported, not failed — that's a role added since the baseline
/// was last committed, not growth.
pub fn compare(current: &BTreeMap<String, usize>, baseline: &Baseline) -> Vec<RoleAudit> {
    current
        .iter()
        .map(|(role, &current_tokens)| {
            let baseline_tokens = baseline.roles.get(role).copied();
            let change_percent = baseline_tokens.map(|base| {
                if base == 0 {
                    0.0
                } else {
                    (current_tokens as f64 - base as f64) / base as f64 * 100.0
                }
            });
            let over_threshold = change_percent.is_some_and(|c| c > GROWTH_THRESHOLD_PERCENT);
            RoleAudit {
                role: role.clone(),
                current_tokens,
                baseline_tokens,
                change_percent,
                over_threshold,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Role;

    #[test]
    fn approx_tokens_rounds_up() {
        assert_eq!(approx_tokens(""), 0);
        assert_eq!(approx_tokens("abcd"), 1);
        assert_eq!(approx_tokens("abcde"), 2);
    }

    #[test]
    fn measure_covers_every_role_and_matches_stable_system_prompt() {
        let config = Config::default();
        let repo = Path::new("/nonexistent");
        let sizes = measure(&config, repo);
        assert_eq!(sizes.len(), config.roles.len());
        let worker = &config.roles["worker"];
        let expected = approx_tokens(&stable_system_prompt(
            "worker",
            worker,
            repo,
            &config.branches,
            &config.commands,
            "",
        ));
        assert_eq!(sizes["worker"], expected);
    }

    #[test]
    fn measure_reflects_a_role_prompt_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bridle_dir = dir.path().join(".bridle");
        std::fs::create_dir_all(&bridle_dir).expect("mkdir");
        std::fs::write(bridle_dir.join("worker.md"), "x".repeat(400)).expect("write");

        let mut config = Config::default();
        let mut role = config.roles["worker"].clone();
        role.system_prompt = Some(PathBuf::from(".bridle/worker.md"));
        config.roles.insert("worker".to_string(), role);

        let without_file = approx_tokens(&stable_system_prompt(
            "worker",
            &Role {
                system_prompt: None,
                ..config.roles["worker"].clone()
            },
            dir.path(),
            &config.branches,
            &config.commands,
            "",
        ));
        let sizes = measure(&config, dir.path());
        assert!(sizes["worker"] > without_file);
    }

    #[test]
    fn baseline_load_missing_file_is_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(Baseline::load(dir.path()).expect("load").is_none());
    }

    #[test]
    fn baseline_load_reads_committed_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bridle_dir = dir.path().join(".bridle");
        std::fs::create_dir_all(&bridle_dir).expect("mkdir");
        std::fs::write(
            bridle_dir.join(BASELINE_FILENAME),
            r#"{"roles": {"worker": 100}}"#,
        )
        .expect("write");

        let baseline = Baseline::load(dir.path())
            .expect("load")
            .expect("some baseline");
        assert_eq!(baseline.roles["worker"], 100);
    }

    #[test]
    fn compare_flags_growth_past_threshold() {
        let mut current = BTreeMap::new();
        current.insert("worker".to_string(), 115);
        current.insert("manager".to_string(), 100);
        current.insert("newrole".to_string(), 42);
        let mut baseline = Baseline::default();
        baseline.roles.insert("worker".to_string(), 100);
        baseline.roles.insert("manager".to_string(), 100);

        let rows = compare(&current, &baseline);

        let worker = rows.iter().find(|r| r.role == "worker").expect("worker");
        assert_eq!(worker.change_percent, Some(15.0));
        assert!(worker.over_threshold);

        let manager = rows.iter().find(|r| r.role == "manager").expect("manager");
        assert_eq!(manager.change_percent, Some(0.0));
        assert!(!manager.over_threshold);

        let newrole = rows.iter().find(|r| r.role == "newrole").expect("newrole");
        assert_eq!(newrole.baseline_tokens, None);
        assert_eq!(newrole.change_percent, None);
        assert!(!newrole.over_threshold);
    }

    #[test]
    fn compare_stays_under_threshold_at_exactly_the_limit() {
        let mut current = BTreeMap::new();
        current.insert("worker".to_string(), 110);
        let mut baseline = Baseline::default();
        baseline.roles.insert("worker".to_string(), 100);

        let rows = compare(&current, &baseline);
        assert!(!rows[0].over_threshold, "exactly at threshold should pass");
    }
}
