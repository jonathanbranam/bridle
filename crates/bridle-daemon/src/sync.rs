//! `bridle sync`: renders resolved workflow layers into what the Claude Code
//! harness reads (docs/design/workflow-layers.md, "Rendering into what the
//! agent harness reads"). Local and static, like `rules explain`/`diff` —
//! just the layer directories [`crate::rules::discover_layers`] finds, no
//! daemon round trip.
//!
//! - `CLAUDE.md` gets a small managed block, replaced idempotently between
//!   markers; everything else in the file is untouched.
//! - `.claude/skills/bridle-<name>/` is rendered from each layer's
//!   `skills/<name>/` directory, later layers overlaying earlier ones file
//!   by file, except `SKILL.md`, which is *appended* rather than replaced —
//!   the "project addenda" workflow-layers.md describes.
//! - `.claude/agents/<role>.md` is rendered from each layer's
//!   `agents/<role>.md`, later layers replacing earlier ones wholesale.
//! - `.claude/settings.json`'s `hooks` object gets each layer's
//!   `hooks/<event>.json` (a JSON array of Claude Code hook-config entries
//!   for that event), merged in without disturbing entries bridle didn't
//!   add. A gitignored sidecar, `.claude/.bridle-sync-hooks.json`, records
//!   exactly which entries bridle wrote last time, so a later sync (or a
//!   change to a layer's hooks) removes only those, never a hand-written
//!   entry for the same event.
//!
//! Skips L4 component/path-scoped rule rendering: workflow-layers.md flags
//! the Claude Code mechanism for that as unverified (see
//! docs/questions/open/ — filed as a follow-up, not guessed at here).
//!
//! `hooks/<event>.json` and the "later layer replaces wholesale" convention
//! for both it and `agents/<role>.md` are this module's own convention, not
//! yet written down anywhere else; nothing in `workflow/` uses either yet.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value};

use crate::config::{BranchesConfig, CommandsConfig};
use crate::rules::Layer;

pub const CLAUDE_MD_START: &str = "<!-- bridle:managed:start -->";
pub const CLAUDE_MD_END: &str = "<!-- bridle:managed:end -->";

const HOOKS_STATE_FILE: &str = ".bridle-sync-hooks.json";

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("resolving workflow rules: {0}")]
    Rules(#[from] crate::rules::LoadAndResolveError),
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{path}: writing JSON: {source}")]
    Serialize {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{path}: expected a JSON object")]
    NotAnObject { path: PathBuf },
    #[error("{path}: expected {event:?} to be a JSON array")]
    HookNotArray { path: PathBuf, event: String },
}

/// What one `sync` run did, for `--json` and the human summary.
#[derive(Debug, Serialize)]
pub struct SyncReport {
    pub claude_md_changed: bool,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub hook_events: Vec<String>,
}

/// Runs every render step against `repo`. Validates that the layers'
/// rules resolve cleanly first ([`crate::rules::load_and_resolve`]) so a
/// broken override kind or locked-rule conflict fails loudly before
/// anything on disk is touched, even though rule *content* itself isn't
/// rendered anywhere by sync.
pub fn sync(
    repo: &Path,
    layers: &[Layer],
    commands: &CommandsConfig,
    branches: &BranchesConfig,
) -> Result<SyncReport, SyncError> {
    crate::rules::load_and_resolve(layers)?;

    let claude_md_changed = sync_claude_md(repo)?;

    let skills = discover_skills(layers)?;
    write_skills(repo, &skills, commands, branches)?;

    let agents = discover_agents(layers)?;
    write_agents(repo, &agents)?;

    let hooks = discover_hooks(layers)?;
    sync_hooks(repo, &hooks)?;

    Ok(SyncReport {
        claude_md_changed,
        skills: skills.into_keys().collect(),
        agents: agents.into_keys().collect(),
        hook_events: hooks.into_keys().collect(),
    })
}

fn managed_block() -> String {
    format!(
        "{CLAUDE_MD_START}\n\
This project's workflow rules, current task and role priming are rendered by\n\
bridle, not written here. Read the rule files (markdown, one per rule id)\n\
in `.bridle/rules/` and in the workflow checkout's `base/rules/` (`workflow`\n\
in `.bridle/config.toml`) at the start of a session — don't rely on this\n\
file for rule content. The orchestrator also runs `bridle prime orchestrator`.\n\
{CLAUDE_MD_END}\n"
    )
}

/// Inserts or replaces the managed block in `existing`, leaving everything
/// else byte-for-byte. Appends the block (after a blank line) if the
/// markers aren't present yet.
pub fn render_claude_md(existing: &str) -> String {
    let block = managed_block();
    if let (Some(start), Some(end_marker)) =
        (existing.find(CLAUDE_MD_START), existing.find(CLAUDE_MD_END))
        && start < end_marker
    {
        let mut end = end_marker + CLAUDE_MD_END.len();
        if existing[end..].starts_with('\n') {
            end += 1;
        }
        let mut out = String::with_capacity(existing.len());
        out.push_str(&existing[..start]);
        out.push_str(&block);
        out.push_str(&existing[end..]);
        return out;
    }

    let mut out = existing.to_string();
    if !out.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(&block);
    out
}

fn sync_claude_md(repo: &Path) -> Result<bool, SyncError> {
    let path = repo.join("CLAUDE.md");
    let existing = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(source) => return Err(SyncError::Io { path, source }),
    };
    let rendered = render_claude_md(&existing);
    let changed = rendered != existing;
    if changed {
        fs::write(&path, &rendered).map_err(|source| SyncError::Io {
            path: path.clone(),
            source,
        })?;
    }
    Ok(changed)
}

/// A layer's root directory — the one containing `rules/`, `skills/`,
/// `agents/`, `hooks/` — derived from [`Layer::dir`], which already points
/// at `<root>/rules`.
fn layer_root(layer: &Layer) -> &Path {
    layer.dir.parent().unwrap_or(&layer.dir)
}

/// One rendered skill's files, keyed by path relative to the skill
/// directory.
type SkillFiles = BTreeMap<PathBuf, Vec<u8>>;

/// Collects every layer's `skills/<name>/` directory into one file map per
/// skill name, later layers overlaying earlier ones file-by-file, except
/// `SKILL.md`: a later layer's `SKILL.md` is appended to the earlier one's
/// rather than replacing it, so a project's `skills/<name>/SKILL.md` reads
/// as an addendum, not a full rewrite.
pub fn discover_skills(layers: &[Layer]) -> Result<BTreeMap<String, SkillFiles>, SyncError> {
    let mut skills: BTreeMap<String, SkillFiles> = BTreeMap::new();
    for layer in layers {
        let skills_dir = layer_root(layer).join("skills");
        let Ok(entries) = fs::read_dir(&skills_dir) else {
            continue;
        };
        let mut dirs: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        for skill_dir in dirs {
            let name = skill_dir
                .file_name()
                .expect("read_dir entries have a file name")
                .to_string_lossy()
                .to_string();
            let files = skills.entry(name).or_default();
            overlay_dir(&skill_dir, &skill_dir, files)?;
        }
    }
    Ok(skills)
}

fn overlay_dir(root: &Path, dir: &Path, files: &mut SkillFiles) -> Result<(), SyncError> {
    let entries = fs::read_dir(dir).map_err(|source| SyncError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            overlay_dir(root, &path, files)?;
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .expect("walked from root, so always under it")
            .to_path_buf();
        let content = fs::read(&path).map_err(|source| SyncError::Io {
            path: path.clone(),
            source,
        })?;
        if rel == Path::new("SKILL.md") {
            files
                .entry(rel)
                .and_modify(|existing| {
                    existing.extend_from_slice(b"\n\n");
                    existing.extend_from_slice(&content);
                })
                .or_insert(content);
        } else {
            files.insert(rel, content);
        }
    }
    Ok(())
}

/// Regenerates `.claude/skills/bridle-*/` from scratch: removes every
/// existing `bridle-*` skill directory, then writes the current set. Never
/// commited (docs/design/workflow-layers.md), so a full wipe-and-rewrite is
/// always safe.
fn write_skills(
    repo: &Path,
    skills: &BTreeMap<String, SkillFiles>,
    commands: &CommandsConfig,
    branches: &BranchesConfig,
) -> Result<(), SyncError> {
    let skills_root = repo.join(".claude").join("skills");
    if let Ok(entries) = fs::read_dir(&skills_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let is_bridle = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("bridle-"));
            if path.is_dir() && is_bridle {
                fs::remove_dir_all(&path).map_err(|source| SyncError::Io {
                    path: path.clone(),
                    source,
                })?;
            }
        }
    }
    for (name, files) in skills {
        let dir = skills_root.join(format!("bridle-{name}"));
        for (rel, content) in files {
            let out_path = dir.join(rel);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|source| SyncError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            let rendered = substitute_placeholders(content, commands, branches);
            fs::write(&out_path, &rendered).map_err(|source| SyncError::Io {
                path: out_path.clone(),
                source,
            })?;
        }
    }
    Ok(())
}

/// Substitutes `{{commands.check}}` for the project's bound check command
/// (default `just check`), and `{{branches.integration}}`/`{{branches.release}}`
/// for the project's branch pattern (`[branches]`, config.rs; default
/// integration `main`, no release branch), in skill source text, so e.g.
/// `workflow/base/skills/worker/SKILL.md` can reference the project's
/// definition-of-done command and integration branch instead of hardcoding
/// either. Applied to every skill file; binary content (not valid UTF-8)
/// passes through unchanged since there's nothing to substitute in it.
/// `{{branches.release}}` is left as-is when no release branch is
/// configured -- there's nothing sensible to substitute for the trunk
/// pattern, and no shipped skill text references it unconditionally.
fn substitute_placeholders(
    content: &[u8],
    commands: &CommandsConfig,
    branches: &BranchesConfig,
) -> Vec<u8> {
    match std::str::from_utf8(content) {
        Ok(text) => {
            let text = text.replace("{{commands.check}}", &commands.check);
            let text = text.replace("{{commands.check_worker}}", commands.worker_check());
            let text = text.replace("{{branches.integration}}", &branches.integration);
            let text = match &branches.release {
                Some(release) => text.replace("{{branches.release}}", release),
                None => text,
            };
            text.into_bytes()
        }
        Err(_) => content.to_vec(),
    }
}

/// Collects every layer's `agents/<role>.md`, later layers replacing
/// earlier ones wholesale (unlike skills, there's no addendum convention
/// for role definitions).
pub fn discover_agents(layers: &[Layer]) -> Result<BTreeMap<String, Vec<u8>>, SyncError> {
    let mut agents = BTreeMap::new();
    for layer in layers {
        let agents_dir = layer_root(layer).join("agents");
        let Ok(entries) = fs::read_dir(&agents_dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
            .collect();
        paths.sort();
        for path in paths {
            let role = path
                .file_stem()
                .expect("filtered to .md files")
                .to_string_lossy()
                .to_string();
            let content = fs::read(&path).map_err(|source| SyncError::Io {
                path: path.clone(),
                source,
            })?;
            agents.insert(role, content);
        }
    }
    Ok(agents)
}

/// Regenerates `.claude/agents/*.md` from scratch: never committed, so a
/// full wipe-and-rewrite is always safe.
fn write_agents(repo: &Path, agents: &BTreeMap<String, Vec<u8>>) -> Result<(), SyncError> {
    let dir = repo.join(".claude").join("agents");
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|source| SyncError::Io {
            path: dir.clone(),
            source,
        })?;
    }
    if agents.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(&dir).map_err(|source| SyncError::Io {
        path: dir.clone(),
        source,
    })?;
    for (role, content) in agents {
        let path = dir.join(format!("{role}.md"));
        fs::write(&path, content).map_err(|source| SyncError::Io {
            path: path.clone(),
            source,
        })?;
    }
    Ok(())
}

/// Collects every layer's `hooks/<event>.json` — a JSON array of Claude
/// Code hook-config entries for that Claude Code event name — later layers
/// replacing earlier ones wholesale, same as agents.
pub fn discover_hooks(layers: &[Layer]) -> Result<BTreeMap<String, Value>, SyncError> {
    let mut hooks = BTreeMap::new();
    for layer in layers {
        let hooks_dir = layer_root(layer).join("hooks");
        let Ok(entries) = fs::read_dir(&hooks_dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let event = path
                .file_stem()
                .expect("filtered to .json files")
                .to_string_lossy()
                .to_string();
            let text = fs::read_to_string(&path).map_err(|source| SyncError::Io {
                path: path.clone(),
                source,
            })?;
            let value: Value = serde_json::from_str(&text).map_err(|source| SyncError::Json {
                path: path.clone(),
                source,
            })?;
            if !value.is_array() {
                return Err(SyncError::HookNotArray {
                    path: path.clone(),
                    event,
                });
            }
            hooks.insert(event, value);
        }
    }
    Ok(hooks)
}

fn read_json_object(path: &Path) -> Result<Map<String, Value>, SyncError> {
    match fs::read_to_string(path) {
        Ok(text) => {
            let value: Value = serde_json::from_str(&text).map_err(|source| SyncError::Json {
                path: path.to_path_buf(),
                source,
            })?;
            match value {
                Value::Object(map) => Ok(map),
                _ => Err(SyncError::NotAnObject {
                    path: path.to_path_buf(),
                }),
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Map::new()),
        Err(source) => Err(SyncError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn write_json_pretty(path: &Path, value: &Value) -> Result<(), SyncError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SyncError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut text = serde_json::to_string_pretty(value).map_err(|source| SyncError::Serialize {
        path: path.to_path_buf(),
        source,
    })?;
    text.push('\n');
    fs::write(path, text).map_err(|source| SyncError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Merges `hooks` (this sync's discovered `hooks/<event>.json` entries)
/// into `.claude/settings.json`'s `hooks` object, without disturbing
/// anything bridle didn't add. A gitignored sidecar next to settings.json
/// records exactly which entries the previous sync wrote per event, so
/// they — and only they — are removed before the current entries go in;
/// a hand-written hook for the same event, added outside sync, is never
/// touched because it was never recorded there.
fn sync_hooks(repo: &Path, hooks: &BTreeMap<String, Value>) -> Result<(), SyncError> {
    let claude_dir = repo.join(".claude");
    let settings_path = claude_dir.join("settings.json");
    let state_path = claude_dir.join(HOOKS_STATE_FILE);

    let mut settings = read_json_object(&settings_path)?;
    let previous = read_json_object(&state_path)?;

    {
        let hooks_entry = settings
            .entry("hooks".to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let Value::Object(hooks_obj) = hooks_entry else {
            return Err(SyncError::NotAnObject {
                path: settings_path.clone(),
            });
        };

        for (event, prev_value) in &previous {
            let Value::Array(prev_entries) = prev_value else {
                continue;
            };
            if let Some(Value::Array(current)) = hooks_obj.get_mut(event) {
                current.retain(|entry| !prev_entries.contains(entry));
            }
        }
        hooks_obj.retain(|_, v| !matches!(v, Value::Array(a) if a.is_empty()));

        for (event, new_value) in hooks {
            let Value::Array(new_entries) = new_value else {
                return Err(SyncError::HookNotArray {
                    path: state_path.clone(),
                    event: event.clone(),
                });
            };
            match hooks_obj.get_mut(event) {
                Some(Value::Array(existing)) => existing.extend(new_entries.iter().cloned()),
                _ => {
                    hooks_obj.insert(event.clone(), Value::Array(new_entries.clone()));
                }
            }
        }
        if hooks_obj.is_empty() {
            settings.remove("hooks");
        }
    }

    // No hooks and no settings file: leave the project without a .claude/settings.json.
    if !hooks.is_empty() || settings_path.exists() {
        write_json_pretty(&settings_path, &Value::Object(settings))?;
    }

    if hooks.is_empty() {
        if state_path.exists() {
            fs::remove_file(&state_path).map_err(|source| SyncError::Io {
                path: state_path.clone(),
                source,
            })?;
        }
    } else {
        let state_value =
            Value::Object(hooks.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        write_json_pretty(&state_path, &state_value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::LayerKind;

    fn layer(kind: LayerKind, name: &str, root: &Path) -> Layer {
        Layer {
            kind,
            name: name.to_string(),
            dir: root.join("rules"),
        }
    }

    // -- CLAUDE.md managed block -----------------------------------------

    #[test]
    fn inserts_the_block_into_an_empty_file() {
        let rendered = render_claude_md("");
        assert!(rendered.contains(CLAUDE_MD_START));
        assert!(rendered.contains(CLAUDE_MD_END));
        assert!(rendered.contains("bridle prime"));
    }

    #[test]
    fn appends_the_block_after_human_content_when_absent() {
        let human = "# My project\n\nSome human-written notes.\n";
        let rendered = render_claude_md(human);
        assert!(rendered.starts_with(human));
        assert!(rendered.contains(CLAUDE_MD_START));
    }

    #[test]
    fn replaces_only_the_block_leaving_human_content_untouched() {
        let human_before = "# My project\n\nHuman notes above.\n\n";
        let old_block = format!("{CLAUDE_MD_START}\nold content\n{CLAUDE_MD_END}\n");
        let human_after = "\nMore human notes below.\n";
        let existing = format!("{human_before}{old_block}{human_after}");

        let rendered = render_claude_md(&existing);
        assert!(rendered.contains(human_before));
        assert!(rendered.contains(human_after));
        assert!(!rendered.contains("old content"));
        assert!(rendered.contains(CLAUDE_MD_START));
    }

    #[test]
    fn resyncing_twice_is_idempotent() {
        let human = "# My project\n\nNotes.\n";
        let once = render_claude_md(human);
        let twice = render_claude_md(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn sync_writes_claude_md_once_then_leaves_it_alone_on_resync() {
        let repo = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            repo.path().join("CLAUDE.md"),
            "# Human title\n\nHuman body.\n",
        )
        .expect("write");

        let report = sync(
            repo.path(),
            &[],
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert!(report.claude_md_changed);
        let after_first = std::fs::read_to_string(repo.path().join("CLAUDE.md")).expect("read");
        assert!(after_first.contains("# Human title"));
        assert!(after_first.contains(CLAUDE_MD_START));

        let report2 = sync(
            repo.path(),
            &[],
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert!(!report2.claude_md_changed);
        let after_second = std::fs::read_to_string(repo.path().join("CLAUDE.md")).expect("read");
        assert_eq!(after_first, after_second);
    }

    // -- skills -----------------------------------------------------------

    #[test]
    fn renders_skills_from_base_with_project_addendum_appended() {
        let repo = tempfile::tempdir().expect("tempdir");
        let base_skill = repo.path().join("workflow/base/skills/verify");
        std::fs::create_dir_all(&base_skill).expect("mkdir");
        std::fs::write(base_skill.join("SKILL.md"), "base instructions\n").expect("write");
        std::fs::write(base_skill.join("script.sh"), "#!/bin/sh\necho base\n").expect("write");

        let project_skill = repo.path().join(".bridle/skills/verify");
        std::fs::create_dir_all(&project_skill).expect("mkdir");
        std::fs::write(project_skill.join("SKILL.md"), "project addendum\n").expect("write");

        let layers = vec![
            layer(LayerKind::Base, "base", &repo.path().join("workflow/base")),
            layer(LayerKind::Project, "project", &repo.path().join(".bridle")),
        ];

        let report = sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert_eq!(report.skills, vec!["verify".to_string()]);

        let out_dir = repo.path().join(".claude/skills/bridle-verify");
        let skill_md = std::fs::read_to_string(out_dir.join("SKILL.md")).expect("read");
        assert!(skill_md.contains("base instructions"));
        assert!(skill_md.contains("project addendum"));
        let script = std::fs::read_to_string(out_dir.join("script.sh")).expect("read");
        assert_eq!(script, "#!/bin/sh\necho base\n");
    }

    #[test]
    fn skill_check_command_placeholder_is_substituted_per_project() {
        let repo = tempfile::tempdir().expect("tempdir");
        let base_skill = repo.path().join("workflow/base/skills/worker");
        std::fs::create_dir_all(&base_skill).expect("mkdir");
        std::fs::write(
            base_skill.join("SKILL.md"),
            "run `{{commands.check}}` before handoff\n",
        )
        .expect("write");

        let layers = vec![layer(
            LayerKind::Base,
            "base",
            &repo.path().join("workflow/base"),
        )];

        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        let default_rendered =
            std::fs::read_to_string(repo.path().join(".claude/skills/bridle-worker/SKILL.md"))
                .expect("read");
        assert!(default_rendered.contains("run `just check` before handoff"));

        let custom = CommandsConfig {
            check: "make check".to_string(),
            check_worker: None,
        };
        sync(repo.path(), &layers, &custom, &BranchesConfig::default()).expect("sync");
        let custom_rendered =
            std::fs::read_to_string(repo.path().join(".claude/skills/bridle-worker/SKILL.md"))
                .expect("read");
        assert!(custom_rendered.contains("run `make check` before handoff"));
    }

    #[test]
    fn manager_skill_renders_project_neutral() {
        let src = include_bytes!("../../../workflow/base/skills/manager/SKILL.md");
        let branches = BranchesConfig {
            integration: "trunk-x".to_string(),
            ..BranchesConfig::default()
        };
        let commands = CommandsConfig {
            check: "make ci".to_string(),
            check_worker: None,
        };
        let out =
            String::from_utf8(substitute_placeholders(src, &commands, &branches)).expect("utf8");
        assert!(out.contains("make ci") && out.contains("trunk-x"));
        assert!(!out.contains("just check") && !out.contains("{{"));
        for word in out.split(|c: char| !(c.is_alphanumeric() || c == '/' || c == '-')) {
            assert!(word != "main", "stray `main`");
        }
    }

    #[test]
    fn base_arch_guard_hook_is_rendered_into_settings() {
        let repo = tempfile::tempdir().expect("tempdir");
        let base_root = repo.path().join("workflow/base");
        std::fs::create_dir_all(base_root.join("hooks")).expect("mkdir");
        std::fs::write(
            base_root.join("hooks/PreToolUse.json"),
            include_bytes!("../../../workflow/base/hooks/PreToolUse.json"),
        )
        .expect("write");
        let layers = vec![layer(LayerKind::Base, "base", &base_root)];
        let report = sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert_eq!(report.hook_events, vec!["PreToolUse".to_string()]);
        let settings: Value = serde_json::from_str(
            &std::fs::read_to_string(repo.path().join(".claude/settings.json")).expect("read"),
        )
        .expect("json");
        let entry = &settings["hooks"]["PreToolUse"][0];
        assert_eq!(entry["matcher"], "Edit|Write|MultiEdit");
        assert_eq!(entry["hooks"][0]["command"], "bridle arch-guard");
    }

    #[test]
    fn skill_branches_placeholders_are_substituted_per_project() {
        let repo = tempfile::tempdir().expect("tempdir");
        let base_skill = repo.path().join("workflow/base/skills/worker");
        std::fs::create_dir_all(&base_skill).expect("mkdir");
        std::fs::write(
            base_skill.join("SKILL.md"),
            "merge `{{branches.integration}}`; never touch `{{branches.release}}`\n",
        )
        .expect("write");

        let layers = vec![layer(
            LayerKind::Base,
            "base",
            &repo.path().join("workflow/base"),
        )];

        // Trunk pattern (default): no release branch, so its placeholder is
        // left untouched.
        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        let rendered =
            std::fs::read_to_string(repo.path().join(".claude/skills/bridle-worker/SKILL.md"))
                .expect("read");
        assert!(rendered.contains("merge `main`"));
        assert!(rendered.contains("{{branches.release}}"));

        // Dev + release pattern: both are substituted.
        let dev_release = BranchesConfig {
            integration: "dev".to_string(),
            integration_set: true,
            release: Some("main".to_string()),
        };
        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &dev_release,
        )
        .expect("sync");
        let rendered =
            std::fs::read_to_string(repo.path().join(".claude/skills/bridle-worker/SKILL.md"))
                .expect("read");
        assert!(rendered.contains("merge `dev`"));
        assert!(rendered.contains("never touch `main`"));
    }

    #[test]
    fn resyncing_skills_removes_a_skill_dropped_from_the_layers() {
        let repo = tempfile::tempdir().expect("tempdir");
        let base_root = repo.path().join("workflow/base");
        let skill_a = base_root.join("skills/a");
        std::fs::create_dir_all(&skill_a).expect("mkdir");
        std::fs::write(skill_a.join("SKILL.md"), "a\n").expect("write");

        let layers = vec![layer(LayerKind::Base, "base", &base_root)];
        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert!(
            repo.path()
                .join(".claude/skills/bridle-a")
                .join("SKILL.md")
                .exists()
        );

        std::fs::remove_dir_all(&skill_a).expect("remove");
        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert!(!repo.path().join(".claude/skills/bridle-a").exists());
    }

    // -- agents -------------------------------------------------------------

    #[test]
    fn renders_agents_with_later_layers_replacing_earlier_ones() {
        let repo = tempfile::tempdir().expect("tempdir");
        let base_root = repo.path().join("workflow/base");
        std::fs::create_dir_all(base_root.join("agents")).expect("mkdir");
        std::fs::write(base_root.join("agents/worker.md"), "base worker\n").expect("write");

        let project_root = repo.path().join(".bridle");
        std::fs::create_dir_all(project_root.join("agents")).expect("mkdir");
        std::fs::write(project_root.join("agents/worker.md"), "project worker\n").expect("write");

        let layers = vec![
            layer(LayerKind::Base, "base", &base_root),
            layer(LayerKind::Project, "project", &project_root),
        ];

        let report = sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert_eq!(report.agents, vec!["worker".to_string()]);
        let rendered =
            std::fs::read_to_string(repo.path().join(".claude/agents/worker.md")).expect("read");
        assert_eq!(rendered, "project worker\n");
    }

    #[test]
    fn resyncing_agents_removes_a_role_dropped_from_the_layers() {
        let repo = tempfile::tempdir().expect("tempdir");
        let base_root = repo.path().join("workflow/base");
        std::fs::create_dir_all(base_root.join("agents")).expect("mkdir");
        std::fs::write(base_root.join("agents/worker.md"), "worker\n").expect("write");

        let layers = vec![layer(LayerKind::Base, "base", &base_root)];
        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert!(repo.path().join(".claude/agents/worker.md").exists());

        std::fs::remove_file(base_root.join("agents/worker.md")).expect("remove");
        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert!(!repo.path().join(".claude/agents/worker.md").exists());
    }

    // -- settings.json hooks ------------------------------------------------

    #[test]
    fn merges_a_hook_without_touching_a_preexisting_unrelated_entry() {
        let repo = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(repo.path().join(".claude")).expect("mkdir");
        std::fs::write(
            repo.path().join(".claude/settings.json"),
            serde_json::json!({
                "hooks": {
                    "PreToolUse": [
                        { "matcher": "Bash", "hooks": [{ "type": "command", "command": "human-added-hook" }] }
                    ]
                }
            })
            .to_string(),
        )
        .expect("write");

        let base_root = repo.path().join("workflow/base");
        std::fs::create_dir_all(base_root.join("hooks")).expect("mkdir");
        std::fs::write(
            base_root.join("hooks/SessionStart.json"),
            serde_json::json!([
                { "matcher": "", "hooks": [{ "type": "command", "command": "bridle sync" }] }
            ])
            .to_string(),
        )
        .expect("write");

        let layers = vec![layer(LayerKind::Base, "base", &base_root)];
        let report = sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");
        assert_eq!(report.hook_events, vec!["SessionStart".to_string()]);

        let settings: Value = serde_json::from_str(
            &std::fs::read_to_string(repo.path().join(".claude/settings.json")).expect("read"),
        )
        .expect("json");
        // the human's own hook is untouched
        assert_eq!(
            settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "human-added-hook"
        );
        // bridle's hook is present
        assert_eq!(
            settings["hooks"]["SessionStart"][0]["hooks"][0]["command"],
            "bridle sync"
        );
    }

    #[test]
    fn resyncing_updates_only_bridles_own_hook_entries() {
        let repo = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(repo.path().join(".claude")).expect("mkdir");
        std::fs::write(
            repo.path().join(".claude/settings.json"),
            serde_json::json!({
                "hooks": {
                    "PreToolUse": [
                        { "matcher": "Bash", "hooks": [{ "type": "command", "command": "human-added-hook" }] }
                    ]
                }
            })
            .to_string(),
        )
        .expect("write");

        let base_root = repo.path().join("workflow/base");
        std::fs::create_dir_all(base_root.join("hooks")).expect("mkdir");
        let hook_path = base_root.join("hooks/SessionStart.json");
        std::fs::write(
            &hook_path,
            serde_json::json!([
                { "matcher": "", "hooks": [{ "type": "command", "command": "bridle sync v1" }] }
            ])
            .to_string(),
        )
        .expect("write");

        let layers = vec![layer(LayerKind::Base, "base", &base_root)];
        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");

        std::fs::write(
            &hook_path,
            serde_json::json!([
                { "matcher": "", "hooks": [{ "type": "command", "command": "bridle sync v2" }] }
            ])
            .to_string(),
        )
        .expect("write");
        sync(
            repo.path(),
            &layers,
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");

        let settings: Value = serde_json::from_str(
            &std::fs::read_to_string(repo.path().join(".claude/settings.json")).expect("read"),
        )
        .expect("json");
        let session_start = settings["hooks"]["SessionStart"].as_array().expect("array");
        assert_eq!(session_start.len(), 1);
        assert_eq!(session_start[0]["hooks"][0]["command"], "bridle sync v2");
        // the human's own hook still untouched
        assert_eq!(
            settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "human-added-hook"
        );
    }

    #[test]
    fn no_hooks_in_any_layer_leaves_settings_json_untouched() {
        let repo = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(repo.path().join(".claude")).expect("mkdir");
        let original = serde_json::json!({ "someOtherKey": true }).to_string();
        std::fs::write(repo.path().join(".claude/settings.json"), &original).expect("write");

        sync(
            repo.path(),
            &[],
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");

        let settings: Value = serde_json::from_str(
            &std::fs::read_to_string(repo.path().join(".claude/settings.json")).expect("read"),
        )
        .expect("json");
        assert_eq!(settings["someOtherKey"], true);
        assert!(settings.get("hooks").is_none());
    }

    #[test]
    fn no_hooks_and_no_settings_json_creates_nothing() {
        let repo = tempfile::tempdir().expect("tempdir");

        sync(
            repo.path(),
            &[],
            &CommandsConfig::default(),
            &BranchesConfig::default(),
        )
        .expect("sync");

        assert!(!repo.path().join(".claude/settings.json").exists());
    }
}
