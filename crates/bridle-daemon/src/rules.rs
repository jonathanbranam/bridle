//! Layer resolution for bridle's workflow rules (docs/design/workflow-layers.md,
//! "Rules have ids, and overrides are explicit"). Resolves L1 base, L2 packs and L3
//! project rule layers by id, later layers winning by default. L0 core has no
//! file-backed layer yet — it's "built into the binary" per the design's layer
//! table, and nothing in the binary defines rules that way today — so it isn't
//! given a directory here; wiring it in is a follow-up once L0 has real content.
//! L4 component/path-scoped rules are out of scope (blocked on spike vxp6).
//!
//! Each layer is a directory of `<id>.md` files (`Layer::dir` already points at
//! the `rules/` directory itself, e.g. `<workflow>/base/rules` for L1,
//! `<workflow>/packs/<name>/rules` for L2, `<repo>/.bridle/rules` for L3 —
//! see [`Layer`], [`discover_layers`]). A rule file's frontmatter carries its id, and for anything
//! after the first layer that defines that id, an explicit `override` kind
//! (`replace` | `append` | `disable`) — silent redefinition is an error, per the
//! "overrides are explicit" section. `disable` requires a `reason`. A rule marked
//! `locked: true` can never be redefined by a later layer, whatever kind of
//! override it attempts.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LayerKind {
    Base,
    Pack,
    Project,
}

impl fmt::Display for LayerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LayerKind::Base => "base",
            LayerKind::Pack => "pack",
            LayerKind::Project => "project",
        };
        f.write_str(s)
    }
}

/// One directory of `<id>.md` rule files. Layers are resolved in the order
/// given to [`resolve`]; within a `Pack` layer, `name` distinguishes it from
/// other packs (e.g. `"typescript"`, `"web-ui"`).
#[derive(Debug, Clone)]
pub struct Layer {
    pub kind: LayerKind,
    pub name: String,
    pub dir: PathBuf,
}

/// Builds the layer list for a repo from `workflow`/`packs` in
/// `<repo>/.bridle/config.toml` (`config::Config`): L1 base and L2 packs
/// from the workflow checkout, if one is configured, then L3 project from
/// `<repo>/.bridle/rules`. With no `workflow` set, resolution only sees the
/// project layer. `workflow_root`, if relative, is resolved against `repo`
/// — bridle's own workflow checkout is in-repo (`workflow = "workflow"`,
/// decision r2uq), but the field also accepts a path to a sibling checkout
/// or, per docs/design/workflow-layers.md, a git url (not resolved here;
/// a git-url `workflow` currently yields an empty base layer, since nothing
/// clones it yet — a documented follow-up, not this task's scope). Layout
/// inside the workflow root: `base/rules/<id>.md` for L1,
/// `packs/<name>/rules/<id>.md` for L2.
pub fn discover_layers(repo: &Path, workflow_root: Option<&Path>, packs: &[String]) -> Vec<Layer> {
    let mut layers = Vec::new();
    if let Some(root) = workflow_root {
        let root = if root.is_absolute() {
            root.to_path_buf()
        } else {
            repo.join(root)
        };
        layers.push(Layer {
            kind: LayerKind::Base,
            name: "base".to_string(),
            dir: root.join("base").join("rules"),
        });
        for pack in packs {
            layers.push(Layer {
                kind: LayerKind::Pack,
                name: pack.clone(),
                dir: root.join("packs").join(pack).join("rules"),
            });
        }
    }
    layers.push(Layer {
        kind: LayerKind::Project,
        name: "project".to_string(),
        dir: repo.join(".bridle").join("rules"),
    });
    layers
}

/// Loads every layer's rule files and resolves them, in one step, for CLI
/// commands that just want the end result.
pub fn load_and_resolve(layers: &[Layer]) -> Result<Resolution, LoadAndResolveError> {
    let mut loaded = Vec::with_capacity(layers.len());
    for layer in layers {
        let rules = load_layer_rules(&layer.dir)?;
        loaded.push(LoadedLayer {
            layer: layer.into(),
            rules,
        });
    }
    Ok(resolve(&loaded)?)
}

#[derive(Debug, thiserror::Error)]
pub enum LoadAndResolveError {
    #[error(transparent)]
    Load(#[from] RuleLoadError),
    #[error(transparent)]
    Resolve(#[from] ResolveError),
}

/// Identifies a layer in resolution output, without the filesystem path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerRef {
    pub kind: LayerKind,
    pub name: String,
}

impl fmt::Display for LayerRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.kind)
    }
}

impl From<&Layer> for LayerRef {
    fn from(l: &Layer) -> Self {
        LayerRef {
            kind: l.kind,
            name: l.name.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Must,
    Should,
    May,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Must => "must",
            Severity::Should => "should",
            Severity::May => "may",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OverrideKind {
    Replace,
    Append,
    Disable,
}

impl fmt::Display for OverrideKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OverrideKind::Replace => "replace",
            OverrideKind::Append => "append",
            OverrideKind::Disable => "disable",
        };
        f.write_str(s)
    }
}

/// One `<id>.md` file, parsed but not yet resolved against any other layer.
#[derive(Debug, Clone)]
pub struct RuleFile {
    pub id: String,
    pub severity: Option<Severity>,
    pub roles: Vec<String>,
    /// `None` means "not stated in this file", distinct from an explicit
    /// `locked: false` — needed so a `replace` that omits `locked` inherits
    /// the flag from what it's replacing rather than silently clearing it.
    pub locked: Option<bool>,
    pub override_kind: Option<OverrideKind>,
    pub reason: Option<String>,
    pub body: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RuleParseError {
    #[error("{path}: file doesn't start with a `---` frontmatter block")]
    NoFrontmatter { path: PathBuf },
    #[error("{path}: frontmatter block has no closing `---`")]
    UnterminatedFrontmatter { path: PathBuf },
    #[error("{path}: frontmatter has no `id`")]
    MissingId { path: PathBuf },
    #[error("{path}: unknown severity {value:?} (expected must, should or may)")]
    UnknownSeverity { path: PathBuf, value: String },
    #[error("{path}: unknown override {value:?} (expected replace, append or disable)")]
    UnknownOverride { path: PathBuf, value: String },
    #[error("{path}: unknown frontmatter key {key:?}")]
    UnknownKey { path: PathBuf, key: String },
    #[error("{path}: `override: disable` requires a `reason`")]
    DisableRequiresReason { path: PathBuf },
}

/// Parses one rule file's text (frontmatter + body). `path` is only used to
/// label errors.
pub fn parse_rule_file(path: &Path, text: &str) -> Result<RuleFile, RuleParseError> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let rest = text
        .strip_prefix("---\n")
        .ok_or_else(|| RuleParseError::NoFrontmatter {
            path: path.to_path_buf(),
        })?;
    let end = rest
        .find("\n---")
        .ok_or_else(|| RuleParseError::UnterminatedFrontmatter {
            path: path.to_path_buf(),
        })?;
    let frontmatter = &rest[..end];
    // Skip the closing `---` line itself, and the newline right after it if present.
    let after_marker = &rest[end + "\n---".len()..];
    let body = after_marker
        .strip_prefix('\n')
        .unwrap_or(after_marker)
        .trim_start_matches('\n')
        .to_string();

    let mut id = None;
    let mut severity = None;
    let mut roles = Vec::new();
    let mut locked = None;
    let mut override_kind = None;
    let mut reason = None;

    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "id" => id = Some(unquote(value).to_string()),
            "severity" => {
                severity = Some(match unquote(value) {
                    "must" => Severity::Must,
                    "should" => Severity::Should,
                    "may" => Severity::May,
                    other => {
                        return Err(RuleParseError::UnknownSeverity {
                            path: path.to_path_buf(),
                            value: other.to_string(),
                        });
                    }
                });
            }
            "roles" => roles = parse_list(value),
            "locked" => locked = Some(unquote(value) == "true"),
            "override" => {
                override_kind = Some(match unquote(value) {
                    "replace" => OverrideKind::Replace,
                    "append" => OverrideKind::Append,
                    "disable" => OverrideKind::Disable,
                    other => {
                        return Err(RuleParseError::UnknownOverride {
                            path: path.to_path_buf(),
                            value: other.to_string(),
                        });
                    }
                });
            }
            "reason" => reason = Some(unquote(value).to_string()),
            other => {
                return Err(RuleParseError::UnknownKey {
                    path: path.to_path_buf(),
                    key: other.to_string(),
                });
            }
        }
    }

    let id = id.ok_or_else(|| RuleParseError::MissingId {
        path: path.to_path_buf(),
    })?;

    if override_kind == Some(OverrideKind::Disable) && reason.is_none() {
        return Err(RuleParseError::DisableRequiresReason {
            path: path.to_path_buf(),
        });
    }

    Ok(RuleFile {
        id,
        severity,
        roles,
        locked,
        override_kind,
        reason,
        body,
    })
}

fn unquote(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2
        && let (Some(first), Some(last)) = (s.chars().next(), s.chars().last())
        && ((first == '"' && last == '"') || (first == '\'' && last == '\''))
    {
        return &s[1..s.len() - 1];
    }
    s
}

fn parse_list(value: &str) -> Vec<String> {
    let inner = value.trim().trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| unquote(s).to_string())
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum RuleLoadError {
    #[error("reading {dir}: {source}")]
    Read {
        dir: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Parse(#[from] RuleParseError),
    #[error("{dir}: duplicate rule id {id:?} in {first} and {second}")]
    DuplicateId {
        dir: PathBuf,
        id: String,
        first: PathBuf,
        second: PathBuf,
    },
}

/// Loads every `*.md` file directly under `dir` as a rule file, sorted by
/// filename for a deterministic load order. `dir` not existing loads as
/// empty, not an error — a project or pack with no rules yet is normal.
pub fn load_layer_rules(dir: &Path) -> Result<Vec<RuleFile>, RuleLoadError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(RuleLoadError::Read {
                dir: dir.to_path_buf(),
                source,
            });
        }
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();

    let mut rules = Vec::new();
    let mut seen: BTreeMap<String, PathBuf> = BTreeMap::new();
    for path in paths {
        let text = std::fs::read_to_string(&path).map_err(|source| RuleLoadError::Read {
            dir: dir.to_path_buf(),
            source,
        })?;
        let rule = parse_rule_file(&path, &text)?;
        if let Some(first) = seen.get(&rule.id) {
            return Err(RuleLoadError::DuplicateId {
                dir: dir.to_path_buf(),
                id: rule.id,
                first: first.clone(),
                second: path,
            });
        }
        seen.insert(rule.id.clone(), path);
        rules.push(rule);
    }
    Ok(rules)
}

/// A layer's rule files, already loaded.
#[derive(Debug, Clone)]
pub struct LoadedLayer {
    pub layer: LayerRef,
    pub rules: Vec<RuleFile>,
}

/// A rule id's resolved content, or that it's disabled.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum RuleState {
    Active {
        severity: Option<Severity>,
        roles: Vec<String>,
        locked: bool,
        body: String,
    },
    Disabled {
        reason: String,
    },
}

/// One layer's contribution to a rule id's history: what it did (define, or
/// override with a kind) and the state that resulted.
#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
    pub layer: LayerRef,
    /// `None` for the layer that first defines this id.
    pub override_kind: Option<OverrideKind>,
    pub reason: Option<String>,
    pub state: RuleState,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedRule {
    pub id: String,
    /// In layer order, earliest first; the last entry is the winner.
    pub history: Vec<HistoryEntry>,
}

impl ResolvedRule {
    pub fn state(&self) -> &RuleState {
        &self
            .history
            .last()
            .expect("a resolved rule always has at least one history entry")
            .state
    }

    pub fn winning_layer(&self) -> &LayerRef {
        &self
            .history
            .last()
            .expect("a resolved rule always has at least one history entry")
            .layer
    }
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub rules: BTreeMap<String, ResolvedRule>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error(
        "{id}: {layer} redefines a rule already defined by an earlier layer without an `override` kind"
    )]
    MissingOverride { id: String, layer: LayerRef },
    #[error("{id}: {layer} gives an `override` kind but no earlier layer defines this rule")]
    OverrideWithoutPriorDefinition { id: String, layer: LayerRef },
    #[error(
        "{id}: {layer} attempts to override a rule locked by an earlier layer; change it there instead"
    )]
    LockedRuleOverridden { id: String, layer: LayerRef },
    #[error("{id}: {layer} tries to `append` to a rule disabled by an earlier layer")]
    AppendToDisabled { id: String, layer: LayerRef },
}

/// Resolves a rule id across layers, later layers winning by default, per
/// docs/design/workflow-layers.md. `layers` must already be in resolution
/// order (base, then each pack, then project).
pub fn resolve(layers: &[LoadedLayer]) -> Result<Resolution, ResolveError> {
    let mut rules: BTreeMap<String, ResolvedRule> = BTreeMap::new();

    for loaded in layers {
        for rule in &loaded.rules {
            match rules.get_mut(&rule.id) {
                None => {
                    if rule.override_kind.is_some() {
                        return Err(ResolveError::OverrideWithoutPriorDefinition {
                            id: rule.id.clone(),
                            layer: loaded.layer.clone(),
                        });
                    }
                    let state = RuleState::Active {
                        severity: rule.severity,
                        roles: rule.roles.clone(),
                        locked: rule.locked.unwrap_or(false),
                        body: rule.body.clone(),
                    };
                    rules.insert(
                        rule.id.clone(),
                        ResolvedRule {
                            id: rule.id.clone(),
                            history: vec![HistoryEntry {
                                layer: loaded.layer.clone(),
                                override_kind: None,
                                reason: rule.reason.clone(),
                                state,
                            }],
                        },
                    );
                }
                Some(existing) => {
                    let was_locked =
                        matches!(existing.state(), RuleState::Active { locked: true, .. });
                    if was_locked {
                        return Err(ResolveError::LockedRuleOverridden {
                            id: rule.id.clone(),
                            layer: loaded.layer.clone(),
                        });
                    }
                    let Some(kind) = rule.override_kind else {
                        return Err(ResolveError::MissingOverride {
                            id: rule.id.clone(),
                            layer: loaded.layer.clone(),
                        });
                    };
                    let new_state = match kind {
                        OverrideKind::Replace => RuleState::Active {
                            severity: rule.severity.or_else(|| severity_of(existing.state())),
                            roles: if rule.roles.is_empty() {
                                roles_of(existing.state()).to_vec()
                            } else {
                                rule.roles.clone()
                            },
                            locked: rule.locked.unwrap_or_else(|| locked_of(existing.state())),
                            body: rule.body.clone(),
                        },
                        OverrideKind::Append => match existing.state() {
                            RuleState::Active {
                                severity,
                                roles,
                                locked,
                                body,
                            } => RuleState::Active {
                                severity: *severity,
                                roles: roles.clone(),
                                locked: *locked,
                                body: format!("{body}\n\n{}", rule.body),
                            },
                            RuleState::Disabled { .. } => {
                                return Err(ResolveError::AppendToDisabled {
                                    id: rule.id.clone(),
                                    layer: loaded.layer.clone(),
                                });
                            }
                        },
                        OverrideKind::Disable => RuleState::Disabled {
                            reason: rule
                                .reason
                                .clone()
                                .expect("parse_rule_file enforces a reason on disable"),
                        },
                    };
                    existing.history.push(HistoryEntry {
                        layer: loaded.layer.clone(),
                        override_kind: Some(kind),
                        reason: rule.reason.clone(),
                        state: new_state,
                    });
                }
            }
        }
    }

    Ok(Resolution { rules })
}

fn severity_of(state: &RuleState) -> Option<Severity> {
    match state {
        RuleState::Active { severity, .. } => *severity,
        RuleState::Disabled { .. } => None,
    }
}

fn roles_of(state: &RuleState) -> &[String] {
    match state {
        RuleState::Active { roles, .. } => roles,
        RuleState::Disabled { .. } => &[],
    }
}

fn locked_of(state: &RuleState) -> bool {
    match state {
        RuleState::Active { locked, .. } => *locked,
        RuleState::Disabled { .. } => false,
    }
}

/// `bridle rules explain <id>`: which layer won, and what it shadowed.
/// `None` if the id isn't defined in any layer.
pub fn explain<'a>(resolution: &'a Resolution, id: &str) -> Option<&'a ResolvedRule> {
    resolution.rules.get(id)
}

/// One rule id's before/after across the project layer, for `bridle rules
/// diff --project`.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectDiff {
    pub id: String,
    /// The state below the project layer, or `None` if the project layer is
    /// what defines this id in the first place.
    pub before: Option<RuleState>,
    pub after: RuleState,
    pub override_kind: Option<OverrideKind>,
    pub reason: Option<String>,
}

/// `bridle rules diff --project`: everything the project layer does
/// differently from the layers below it (base + packs), by walking each
/// rule's history for the entry the project layer itself contributed.
pub fn diff_project(resolution: &Resolution) -> Vec<ProjectDiff> {
    let mut diffs = Vec::new();
    for rule in resolution.rules.values() {
        let Some(idx) = rule
            .history
            .iter()
            .position(|h| h.layer.kind == LayerKind::Project)
        else {
            continue;
        };
        let entry = &rule.history[idx];
        let before = if idx == 0 {
            None
        } else {
            Some(rule.history[idx - 1].state.clone())
        };
        diffs.push(ProjectDiff {
            id: rule.id.clone(),
            before,
            after: entry.state.clone(),
            override_kind: entry.override_kind,
            reason: entry.reason.clone(),
        });
    }
    diffs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(kind: LayerKind, name: &str, rules: Vec<RuleFile>) -> LoadedLayer {
        LoadedLayer {
            layer: LayerRef {
                kind,
                name: name.to_string(),
            },
            rules,
        }
    }

    fn rule(id: &str) -> RuleFile {
        RuleFile {
            id: id.to_string(),
            severity: Some(Severity::Must),
            roles: vec![],
            locked: None,
            override_kind: None,
            reason: None,
            body: format!("body of {id}"),
        }
    }

    fn overriding(id: &str, kind: OverrideKind, reason: Option<&str>) -> RuleFile {
        RuleFile {
            override_kind: Some(kind),
            reason: reason.map(str::to_string),
            body: format!("override body of {id}"),
            ..rule(id)
        }
    }

    // -- parsing --------------------------------------------------------

    #[test]
    fn parses_a_plain_rule() {
        let text = "---\nid: servers.never-restart\nseverity: must\nroles: [manager, worker]\n---\nNever kill or restart a dev server.\n";
        let f = parse_rule_file(Path::new("x.md"), text).expect("parse");
        assert_eq!(f.id, "servers.never-restart");
        assert_eq!(f.severity, Some(Severity::Must));
        assert_eq!(f.roles, vec!["manager", "worker"]);
        assert_eq!(f.locked, None);
        assert_eq!(f.body, "Never kill or restart a dev server.\n");
    }

    #[test]
    fn parses_an_override() {
        let text = "---\nid: verify.browser\noverride: replace\nreason: no browser path here\n---\nVerify with the CLI harness.\n";
        let f = parse_rule_file(Path::new("x.md"), text).expect("parse");
        assert_eq!(f.override_kind, Some(OverrideKind::Replace));
        assert_eq!(f.reason.as_deref(), Some("no browser path here"));
    }

    #[test]
    fn parses_locked() {
        let text = "---\nid: a\nlocked: true\n---\nbody\n";
        let f = parse_rule_file(Path::new("x.md"), text).expect("parse");
        assert_eq!(f.locked, Some(true));
    }

    #[test]
    fn disable_without_reason_is_an_error() {
        let text = "---\nid: a\noverride: disable\n---\nbody\n";
        let err = parse_rule_file(Path::new("x.md"), text).unwrap_err();
        assert!(matches!(err, RuleParseError::DisableRequiresReason { .. }));
    }

    #[test]
    fn missing_id_is_an_error() {
        let text = "---\nseverity: must\n---\nbody\n";
        let err = parse_rule_file(Path::new("x.md"), text).unwrap_err();
        assert!(matches!(err, RuleParseError::MissingId { .. }));
    }

    #[test]
    fn no_frontmatter_is_an_error() {
        let err = parse_rule_file(Path::new("x.md"), "just body, no frontmatter\n").unwrap_err();
        assert!(matches!(err, RuleParseError::NoFrontmatter { .. }));
    }

    // -- resolution -------------------------------------------------------

    #[test]
    fn a_single_layer_defines_the_rule() {
        let layers = vec![layer(LayerKind::Base, "base", vec![rule("a")])];
        let res = resolve(&layers).expect("resolve");
        let a = &res.rules["a"];
        assert_eq!(a.winning_layer().name, "base");
        assert_eq!(a.history.len(), 1);
    }

    #[test]
    fn later_layer_wins_with_replace() {
        let layers = vec![
            layer(LayerKind::Base, "base", vec![rule("a")]),
            layer(
                LayerKind::Project,
                "project",
                vec![overriding("a", OverrideKind::Replace, None)],
            ),
        ];
        let res = resolve(&layers).expect("resolve");
        let a = &res.rules["a"];
        assert_eq!(a.winning_layer().name, "project");
        match a.state() {
            RuleState::Active { body, .. } => assert_eq!(body, "override body of a"),
            other => panic!("expected active, got {other:?}"),
        }
        assert_eq!(a.history.len(), 2);
    }

    #[test]
    fn append_concatenates_onto_the_earlier_body() {
        let layers = vec![
            layer(LayerKind::Base, "base", vec![rule("a")]),
            layer(
                LayerKind::Project,
                "project",
                vec![overriding("a", OverrideKind::Append, None)],
            ),
        ];
        let res = resolve(&layers).expect("resolve");
        match res.rules["a"].state() {
            RuleState::Active { body, .. } => {
                assert!(body.contains("body of a"));
                assert!(body.contains("override body of a"));
            }
            other => panic!("expected active, got {other:?}"),
        }
    }

    #[test]
    fn disable_removes_the_rule_and_requires_a_reason() {
        let layers = vec![
            layer(LayerKind::Base, "base", vec![rule("a")]),
            layer(
                LayerKind::Project,
                "project",
                vec![overriding(
                    "a",
                    OverrideKind::Disable,
                    Some("no longer needed"),
                )],
            ),
        ];
        let res = resolve(&layers).expect("resolve");
        match res.rules["a"].state() {
            RuleState::Disabled { reason } => assert_eq!(reason, "no longer needed"),
            other => panic!("expected disabled, got {other:?}"),
        }
    }

    #[test]
    fn redefining_an_id_without_an_override_kind_is_an_error() {
        let layers = vec![
            layer(LayerKind::Base, "base", vec![rule("a")]),
            layer(LayerKind::Project, "project", vec![rule("a")]),
        ];
        let err = resolve(&layers).unwrap_err();
        assert!(matches!(err, ResolveError::MissingOverride { id, .. } if id == "a"));
    }

    #[test]
    fn locked_rule_cannot_be_overridden() {
        let mut locked_rule = rule("a");
        locked_rule.locked = Some(true);
        let layers = vec![
            layer(LayerKind::Base, "base", vec![locked_rule]),
            layer(
                LayerKind::Project,
                "project",
                vec![overriding("a", OverrideKind::Replace, None)],
            ),
        ];
        let err = resolve(&layers).unwrap_err();
        assert!(matches!(err, ResolveError::LockedRuleOverridden { id, .. } if id == "a"));
    }

    #[test]
    fn locked_rule_rejects_even_a_silent_redefinition() {
        let mut locked_rule = rule("a");
        locked_rule.locked = Some(true);
        let layers = vec![
            layer(LayerKind::Base, "base", vec![locked_rule]),
            layer(LayerKind::Project, "project", vec![rule("a")]),
        ];
        let err = resolve(&layers).unwrap_err();
        assert!(matches!(err, ResolveError::LockedRuleOverridden { id, .. } if id == "a"));
    }

    #[test]
    fn replace_inherits_severity_and_roles_when_omitted() {
        let mut base = rule("a");
        base.severity = Some(Severity::Must);
        base.roles = vec!["worker".to_string()];
        let mut replace = overriding("a", OverrideKind::Replace, None);
        replace.severity = None;
        replace.roles = vec![];
        let layers = vec![
            layer(LayerKind::Base, "base", vec![base]),
            layer(LayerKind::Project, "project", vec![replace]),
        ];
        let res = resolve(&layers).expect("resolve");
        match res.rules["a"].state() {
            RuleState::Active {
                severity, roles, ..
            } => {
                assert_eq!(*severity, Some(Severity::Must));
                assert_eq!(roles, &["worker".to_string()]);
            }
            other => panic!("expected active, got {other:?}"),
        }
    }

    #[test]
    fn append_to_a_disabled_rule_is_an_error() {
        let layers = vec![
            layer(LayerKind::Base, "base", vec![rule("a")]),
            layer(
                LayerKind::Pack,
                "typescript",
                vec![overriding("a", OverrideKind::Disable, Some("dead in ts"))],
            ),
            layer(
                LayerKind::Project,
                "project",
                vec![overriding("a", OverrideKind::Append, None)],
            ),
        ];
        let err = resolve(&layers).unwrap_err();
        assert!(matches!(err, ResolveError::AppendToDisabled { id, .. } if id == "a"));
    }

    #[test]
    fn multiple_pack_layers_apply_in_order() {
        let layers = vec![
            layer(LayerKind::Base, "base", vec![rule("a")]),
            layer(
                LayerKind::Pack,
                "typescript",
                vec![overriding("a", OverrideKind::Append, None)],
            ),
            layer(
                LayerKind::Pack,
                "web-ui",
                vec![overriding("a", OverrideKind::Replace, None)],
            ),
        ];
        let res = resolve(&layers).expect("resolve");
        assert_eq!(res.rules["a"].winning_layer().name, "web-ui");
        assert_eq!(res.rules["a"].history.len(), 3);
    }

    // -- explain / diff -----------------------------------------------------

    #[test]
    fn explain_shows_the_winner_and_what_it_shadowed() {
        let layers = vec![
            layer(LayerKind::Base, "base", vec![rule("a")]),
            layer(
                LayerKind::Project,
                "project",
                vec![overriding(
                    "a",
                    OverrideKind::Replace,
                    Some("project-specific"),
                )],
            ),
        ];
        let res = resolve(&layers).expect("resolve");
        let explained = explain(&res, "a").expect("rule exists");
        assert_eq!(explained.winning_layer().name, "project");
        assert_eq!(explained.history[0].layer.name, "base");
        assert_eq!(explained.history[0].override_kind, None);
    }

    #[test]
    fn explain_returns_none_for_an_unknown_id() {
        let layers = vec![layer(LayerKind::Base, "base", vec![rule("a")])];
        let res = resolve(&layers).expect("resolve");
        assert!(explain(&res, "nope").is_none());
    }

    #[test]
    fn diff_project_reports_new_changed_and_disabled() {
        let layers = vec![
            layer(
                LayerKind::Base,
                "base",
                vec![rule("kept-same"), rule("overridden"), rule("disabled")],
            ),
            layer(
                LayerKind::Project,
                "project",
                vec![
                    overriding("overridden", OverrideKind::Replace, None),
                    overriding("disabled", OverrideKind::Disable, Some("not needed here")),
                    rule("brand-new"),
                ],
            ),
        ];
        let res = resolve(&layers).expect("resolve");
        let diffs = diff_project(&res);
        let by_id: BTreeMap<&str, &ProjectDiff> =
            diffs.iter().map(|d| (d.id.as_str(), d)).collect();

        assert!(!by_id.contains_key("kept-same"));

        let overridden = by_id["overridden"];
        assert!(overridden.before.is_some());
        assert_eq!(overridden.override_kind, Some(OverrideKind::Replace));

        let disabled = by_id["disabled"];
        assert!(matches!(disabled.after, RuleState::Disabled { .. }));
        assert_eq!(disabled.reason.as_deref(), Some("not needed here"));

        let new = by_id["brand-new"];
        assert!(new.before.is_none());
    }

    // -- loading a layer from disk -------------------------------------

    #[test]
    fn load_layer_rules_reads_every_md_file_sorted() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("b.md"), "---\nid: b\n---\nbody b\n").expect("write");
        std::fs::write(dir.path().join("a.md"), "---\nid: a\n---\nbody a\n").expect("write");
        std::fs::write(dir.path().join("not-a-rule.txt"), "ignored").expect("write");

        let rules = load_layer_rules(dir.path()).expect("load");
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "a");
        assert_eq!(rules[1].id, "b");
    }

    #[test]
    fn load_layer_rules_missing_dir_is_empty() {
        let rules = load_layer_rules(Path::new("/nonexistent/rules/dir")).expect("load");
        assert!(rules.is_empty());
    }

    #[test]
    fn load_layer_rules_rejects_duplicate_ids() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("a.md"), "---\nid: a\n---\nbody\n").expect("write");
        std::fs::write(dir.path().join("a2.md"), "---\nid: a\n---\nother body\n").expect("write");

        let err = load_layer_rules(dir.path()).unwrap_err();
        assert!(matches!(err, RuleLoadError::DuplicateId { id, .. } if id == "a"));
    }

    // -- discovery and end-to-end ---------------------------------------

    #[test]
    fn discover_layers_with_no_workflow_is_project_only() {
        let layers = discover_layers(Path::new("/repo"), None, &[]);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].kind, LayerKind::Project);
        assert_eq!(layers[0].dir, Path::new("/repo/.bridle/rules"));
    }

    #[test]
    fn discover_layers_with_workflow_and_packs() {
        let layers = discover_layers(
            Path::new("/repo"),
            Some(Path::new("/wf")),
            &["typescript".to_string(), "web-ui".to_string()],
        );
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[0].kind, LayerKind::Base);
        assert_eq!(layers[0].dir, Path::new("/wf/base/rules"));
        assert_eq!(layers[1].kind, LayerKind::Pack);
        assert_eq!(layers[1].name, "typescript");
        assert_eq!(layers[1].dir, Path::new("/wf/packs/typescript/rules"));
        assert_eq!(layers[2].name, "web-ui");
        assert_eq!(layers[3].kind, LayerKind::Project);
    }

    #[test]
    fn discover_layers_resolves_a_relative_workflow_path_against_repo() {
        let layers = discover_layers(Path::new("/repo"), Some(Path::new("../wf")), &[]);
        assert_eq!(layers[0].dir, Path::new("/repo/../wf/base/rules"));
    }

    #[test]
    fn load_and_resolve_runs_end_to_end_on_real_directories() {
        let repo = tempfile::tempdir().expect("tempdir");
        let base_rules = repo.path().join("workflow").join("base").join("rules");
        std::fs::create_dir_all(&base_rules).expect("mkdir");
        std::fs::write(
            base_rules.join("verify.browser.md"),
            "---\nid: verify.browser\nseverity: must\n---\nUse playwright.\n",
        )
        .expect("write");

        let project_rules = repo.path().join(".bridle").join("rules");
        std::fs::create_dir_all(&project_rules).expect("mkdir");
        std::fs::write(
            project_rules.join("verify.browser.md"),
            "---\nid: verify.browser\noverride: replace\nreason: no browser path here\n---\nUse the CLI harness.\n",
        )
        .expect("write");

        let layers = discover_layers(repo.path(), Some(&repo.path().join("workflow")), &[]);
        let resolution = load_and_resolve(&layers).expect("resolve");
        let rule = &resolution.rules["verify.browser"];
        assert_eq!(rule.winning_layer().kind, LayerKind::Project);
        match rule.state() {
            RuleState::Active { body, .. } => assert!(body.contains("CLI harness")),
            other => panic!("expected active, got {other:?}"),
        }
    }
}
