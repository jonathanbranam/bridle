//! `bridle case-guard`: Claude Code's PreToolUse hook that refuses a Write/Edit/NotebookEdit
//! creating a path that collides, case-insensitively, with an existing sibling (rule
//! `no-case-colliding-paths`, ticket 4fpk). Linux CI keeps `TimeLog.tsx` and `timelog.ts` apart;
//! a macOS or Windows checkout does not, and a bundler then binds an import to the wrong file.
//! Two names collide when they differ but are equal lowercased, or when their stems differ
//! only by case (`TimeLog.tsx` vs `timelog.ts`). Same stem and same case with another extension
//! (`Cargo.toml`, `Cargo.lock`) is fine. Rewriting the exact existing file is allowed. Siblings
//! are read from disk, which covers tracked files in a checkout.

use std::path::{Path, PathBuf};

fn stem(name: &str) -> &str {
    Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name)
}

fn collides(a: &str, b: &str) -> bool {
    a != b
        && (a.to_lowercase() == b.to_lowercase()
            || (stem(a) != stem(b) && stem(a).to_lowercase() == stem(b).to_lowercase()))
}

/// Pairs of tracked paths (files and their directories) that collide on a case-insensitive disk.
/// Backstop for files written other ways than the Write tool (`bridle case-check`).
pub fn tracked_collisions(paths: &[String]) -> Vec<(String, String)> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut entries: BTreeSet<String> = BTreeSet::new();
    for p in paths {
        let mut cur = Path::new(p);
        loop {
            entries.insert(cur.to_string_lossy().into_owned());
            match cur.parent() {
                Some(par) if !par.as_os_str().is_empty() => cur = par,
                _ => break,
            }
        }
    }
    // Group by (parent, lowercased stem): only same-directory names can collide.
    let mut groups: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for e in &entries {
        let path = Path::new(e);
        let parent = path.parent().map(|p| p.to_string_lossy().to_lowercase());
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or(e);
        groups
            .entry((parent.unwrap_or_default(), stem(name).to_lowercase()))
            .or_default()
            .push(e.clone());
    }
    let mut out = Vec::new();
    for g in groups.values() {
        for (i, a) in g.iter().enumerate() {
            for b in &g[i + 1..] {
                let name = |x: &str| {
                    Path::new(x)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(x)
                        .to_string()
                };
                if a.to_lowercase().rsplit_once('/').map(|x| x.0)
                    == b.to_lowercase().rsplit_once('/').map(|x| x.0)
                    && collides(&name(a), &name(b))
                {
                    out.push((a.clone(), b.clone()));
                }
            }
        }
    }
    out
}

/// The refusal reason when `path` (resolved against `cwd` when relative) would collide.
pub fn collision(path: &str, cwd: Option<&str>) -> Option<String> {
    let path = Path::new(path);
    let full: PathBuf = match cwd {
        Some(cwd) if path.is_relative() => Path::new(cwd).join(path),
        _ => path.to_path_buf(),
    };
    let name = full.file_name()?.to_str()?;
    let dir = full.parent()?;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let Some(other) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if collides(name, &other) {
            return Some(format!(
                "{} collides with the existing {} on a case-insensitive disk (macOS, Windows; \
                 rule no-case-colliding-paths). Edit the existing file, or pick a name that \
                 differs by more than case and extension.",
                full.display(),
                dir.join(&other).display()
            ));
        }
    }
    None
}

/// The refusal reason when the hook input is a file write that collides.
pub fn refusal(input: &serde_json::Value) -> Option<String> {
    let path = crate::arch_guard::edited_path(input)?;
    collision(&path, input.get("cwd").and_then(|c| c.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, name: &str) -> Option<String> {
        refusal(&serde_json::json!({
            "tool_name": "Write",
            "cwd": dir.to_str().unwrap(),
            "tool_input": {"file_path": name},
        }))
    }

    fn dir_with(files: &[&str]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for f in files {
            std::fs::write(d.path().join(f), "").unwrap();
        }
        d
    }

    #[test]
    fn refuses_same_name_differing_by_case() {
        let d = dir_with(&["foo.md"]);
        let why = write(d.path(), "Foo.md").expect("refused");
        assert!(why.contains("Foo.md") && why.contains("foo.md"), "{why}");
    }

    #[test]
    fn refuses_same_stem_differing_by_case_and_extension() {
        let d = dir_with(&["timelog.ts"]);
        assert!(write(d.path(), "TimeLog.tsx").is_some());
    }

    #[test]
    fn allows_exact_rewrite_and_unrelated_names() {
        let d = dir_with(&["foo.md", "Cargo.toml", "Cargo.lock"]);
        assert!(write(d.path(), "foo.md").is_none());
        assert!(write(d.path(), "bar.md").is_none());
        assert!(write(d.path(), "Cargo.lock").is_none());
        assert!(write(d.path(), "foo.test.md").is_none());
        assert!(write(d.path(), "new/dir/Foo.md").is_none());
    }

    #[test]
    fn tracked_collisions_finds_case_pairs_only() {
        let v = |xs: &[&str]| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let hit = tracked_collisions(&v(&["src/timelog.ts", "src/TimeLog.tsx", "a.md"]));
        assert_eq!(
            hit,
            vec![("src/TimeLog.tsx".into(), "src/timelog.ts".into())]
        );
        assert_eq!(tracked_collisions(&v(&["Foo/a.ts", "foo/b.ts"])).len(), 1);
        assert!(
            tracked_collisions(&v(&["Cargo.toml", "Cargo.lock", "src/a.ts", "lib/A.ts"]))
                .is_empty()
        );
    }

    #[test]
    fn ignores_other_tools_and_bad_input() {
        let d = dir_with(&["foo.md"]);
        let bash = serde_json::json!({"tool_name": "Bash", "tool_input": {"file_path": "Foo.md"}});
        assert!(refusal(&bash).is_none());
        assert!(refusal(&serde_json::Value::Null).is_none());
        let _ = d;
    }
}
