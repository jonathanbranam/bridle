//! `bridle impact check`: pairwise overlap of the declared impact of in-flight tasks
//! (docs/design/impact-and-conflicts.md). Pure: the spec id map is passed in.

use std::collections::{BTreeMap, BTreeSet};

use bridle_api::types::{Impact, Overlap, OverlapLevel, SpecRef, Task, TaskState};

/// Every overlap between two planned or claimed tasks, most severe first.
pub fn check(tasks: &[Task], spec_map: &BTreeMap<String, SpecRef>) -> Vec<Overlap> {
    let mut live: Vec<&Task> = tasks
        .iter()
        .filter(|t| matches!(t.state, TaskState::Planned | TaskState::Claimed))
        .filter(|t| !t.impact.is_empty())
        .collect();
    live.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = Vec::new();
    for (i, a) in live.iter().enumerate() {
        for b in &live[i + 1..] {
            pair(a, b, spec_map, &mut out);
        }
    }
    out.sort_by_key(|x| std::cmp::Reverse(x.level));
    out
}

/// The requirement an id belongs to: the mapped one, else an `r-` id is its own.
fn requirement_of<'a>(id: &'a str, map: &'a BTreeMap<String, SpecRef>) -> Option<&'a str> {
    match map.get(id) {
        Some(r) => Some(&r.requirement),
        None => id.starts_with("r-").then_some(id),
    }
}

/// Requirement -> the ids under it that the impact touches (modify/remove/add-under).
fn touched<'a>(
    i: &'a Impact,
    map: &'a BTreeMap<String, SpecRef>,
) -> BTreeMap<&'a str, BTreeSet<&'a str>> {
    let mut m: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for id in i.modify.iter().chain(&i.remove).chain(&i.add_under) {
        if let Some(r) = requirement_of(id, map) {
            m.entry(r).or_default().insert(id);
        }
    }
    m
}

fn pair(a: &Task, b: &Task, map: &BTreeMap<String, SpecRef>, out: &mut Vec<Overlap>) {
    let mut push = |level, kind: &str, key: String| {
        out.push(Overlap {
            level,
            tasks: [a.id.clone(), b.id.clone()],
            kind: kind.into(),
            key,
        });
    };

    // Conflict: one scenario changed or removed by both.
    let changed = |t: &Task| -> BTreeSet<String> {
        t.impact
            .modify
            .iter()
            .chain(&t.impact.remove)
            .filter(|id| id.starts_with("s-"))
            .cloned()
            .collect()
    };
    let (ca, cb) = (changed(a), changed(b));
    let conflicts: BTreeSet<&String> = ca.intersection(&cb).collect();
    for s in &conflicts {
        push(OverlapLevel::Conflict, "scenario", (*s).clone());
    }

    // Warn: same requirement, unless the only thing shared under it is a conflicting
    // scenario (already reported above).
    let (ta, tb) = (touched(&a.impact, map), touched(&b.impact, map));
    for (req, ids_a) in &ta {
        let Some(ids_b) = tb.get(req) else { continue };
        let only_conflicts = ids_a
            .iter()
            .chain(ids_b)
            .all(|id| conflicts.contains(&id.to_string()));
        if !only_conflicts {
            push(OverlapLevel::Warn, "requirement", (*req).into());
        }
    }

    // Info: same capability, different requirements (needs the map).
    let caps = |t: &BTreeMap<&str, BTreeSet<&str>>| -> BTreeMap<String, BTreeSet<String>> {
        let mut m: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for id in t.values().flatten() {
            if let Some(r) = map.get(*id) {
                m.entry(r.capability.clone())
                    .or_default()
                    .insert(r.requirement.clone());
            }
        }
        m
    };
    let (pa, pb) = (caps(&ta), caps(&tb));
    for (cap, reqs_a) in &pa {
        if let Some(reqs_b) = pb.get(cap)
            && reqs_a.is_disjoint(reqs_b)
        {
            push(OverlapLevel::Info, "capability", cap.clone());
        }
    }

    // Warn: overlapping file globs.
    for ga in &a.impact.files {
        for gb in &b.impact.files {
            if globs_overlap(ga, gb) {
                push(OverlapLevel::Warn, "files", format!("{ga} ~ {gb}"));
            }
        }
    }
}

/// Ids whose text differs between two versions of one spec file (`None` = the file
/// is absent on that side, or doesn't parse). Line numbers are ignored so an insertion
/// doesn't flag everything below it. Prose of a non-executable scenario isn't parsed, so
/// editing only that isn't seen. A changed scenario also flags its requirement.
pub fn changed_spec_ids(old: Option<&str>, new: Option<&str>) -> BTreeSet<String> {
    fn index(text: Option<&str>) -> BTreeMap<String, (String, Option<String>)> {
        let mut m = BTreeMap::new();
        let Some(spec) = text.and_then(|t| bridle_spec::parse_str("spec", t).ok()) else {
            return m;
        };
        for r in spec.requirements {
            let Some(rid) = r.id.clone() else { continue };
            for mut s in r.scenarios.iter().cloned() {
                s.line = 0;
                if let Some(sid) = s.id.take() {
                    m.insert(sid, (format!("{s:?}"), Some(rid.clone())));
                }
            }
            let mut head = r.clone();
            head.line = 0;
            head.scenarios.clear();
            m.insert(rid, (format!("{head:?}"), None));
        }
        m
    }
    let (old, new) = (index(old), index(new));
    let mut out = BTreeSet::new();
    for id in old.keys().chain(new.keys()) {
        if old.get(id).map(|x| &x.0) != new.get(id).map(|x| &x.0) {
            out.insert(id.clone());
            if let Some(Some(r)) = old.get(id).or(new.get(id)).map(|x| &x.1) {
                out.insert(r.clone());
            }
        }
    }
    out
}

/// What a landing touched that `impact` declares: the spec ids and file paths, as a
/// short phrase, or `None` when there is no overlap (or no declared impact).
pub fn landing_overlap(
    impact: &Impact,
    ids: &BTreeSet<String>,
    files: &[String],
) -> Option<String> {
    let hit_ids: BTreeSet<&str> = impact
        .modify
        .iter()
        .chain(&impact.remove)
        .chain(&impact.add_under)
        .filter(|i| ids.contains(*i))
        .map(String::as_str)
        .collect();
    let hit_files: BTreeSet<&str> = files
        .iter()
        .filter(|f| impact.files.iter().any(|g| globs_overlap(g, f)))
        .map(String::as_str)
        .collect();
    if hit_ids.is_empty() && hit_files.is_empty() {
        return None;
    }
    let all: Vec<&str> = hit_ids.into_iter().chain(hit_files).collect();
    Some(all.join(", "))
}

/// Two globs overlap when the literal text before the first wildcard of one is a
/// prefix of the other's. Deliberately coarse (`src/a*.rs` vs `src/ab/**` overlaps;
/// `src/a.rs` vs `src/a.rs.bak` too): an early warning, not a proof.
fn globs_overlap(a: &str, b: &str) -> bool {
    let (pa, pb) = (literal_prefix(a), literal_prefix(b));
    pa.starts_with(pb) || pb.starts_with(pa)
}

fn literal_prefix(g: &str) -> &str {
    let end = g.find(['*', '?', '[', '{']).unwrap_or(g.len());
    &g[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridle_api::types::{Impact, TaskKind};

    fn task(id: &str, state: TaskState, impact: Impact) -> Task {
        let mut t: Task = serde_json::from_value(serde_json::json!({
            "id": id, "title": "t", "kind": serde_json::to_value(TaskKind::Feature).unwrap(),
            "state": serde_json::to_value(state).unwrap(), "body": "", "thread": [],
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z",
        }))
        .expect("task");
        t.impact = impact;
        t
    }

    fn imp(modify: &[&str], add_under: &[&str], remove: &[&str], files: &[&str]) -> Impact {
        let v = |x: &[&str]| x.iter().map(|s| s.to_string()).collect();
        Impact {
            modify: v(modify),
            add_under: v(add_under),
            remove: v(remove),
            files: v(files),
        }
    }

    fn map() -> BTreeMap<String, SpecRef> {
        let r = |req: &str, cap: &str| SpecRef {
            requirement: req.into(),
            capability: cap.into(),
        };
        BTreeMap::from([
            ("s-1".into(), r("r-1", "c1")),
            ("s-2".into(), r("r-1", "c1")),
            ("r-1".into(), r("r-1", "c1")),
            ("s-3".into(), r("r-2", "c1")),
            ("r-2".into(), r("r-2", "c1")),
            ("s-4".into(), r("r-3", "c2")),
        ])
    }

    fn run(a: Impact, b: Impact, m: &BTreeMap<String, SpecRef>) -> Vec<(OverlapLevel, String)> {
        let ts = [
            task("a", TaskState::Planned, a),
            task("b", TaskState::Claimed, b),
        ];
        check(&ts, m)
            .into_iter()
            .map(|o| (o.level, o.kind))
            .collect()
    }

    #[test]
    fn table() {
        use OverlapLevel::*;
        let e = |l, k: &str| (l, k.to_string());
        let cases = [
            // modify vs remove of the same scenario
            (
                imp(&["s-1"], &[], &[], &[]),
                imp(&[], &[], &["s-1"], &[]),
                vec![e(Conflict, "scenario")],
            ),
            // same requirement, different scenarios
            (
                imp(&["s-1"], &[], &[], &[]),
                imp(&["s-2"], &[], &[], &[]),
                vec![e(Warn, "requirement")],
            ),
            // same capability, different requirements
            (
                imp(&["s-1"], &[], &[], &[]),
                imp(&["s-3"], &[], &[], &[]),
                vec![e(Info, "capability")],
            ),
            // different capabilities
            (
                imp(&["s-1"], &[], &[], &[]),
                imp(&["s-4"], &[], &[], &[]),
                vec![],
            ),
            // add-under a requirement another task modifies a scenario of
            (
                imp(&["s-1"], &[], &[], &[]),
                imp(&[], &["r-1"], &[], &[]),
                vec![e(Warn, "requirement")],
            ),
            // overlapping globs
            (
                imp(&[], &[], &[], &["a/**"]),
                imp(&[], &[], &[], &["a/b/*.rs"]),
                vec![e(Warn, "files")],
            ),
            // disjoint globs
            (
                imp(&[], &[], &[], &["a/**"]),
                imp(&[], &[], &[], &["b/**"]),
                vec![],
            ),
        ];
        for (a, b, want) in cases {
            assert_eq!(run(a.clone(), b.clone(), &map()), want, "{a:?} vs {b:?}");
        }
    }

    #[test]
    fn without_a_map_r_ids_still_match_and_info_is_skipped() {
        let none = BTreeMap::new();
        let got = run(
            imp(&["r-1"], &[], &[], &[]),
            imp(&[], &["r-1"], &[], &[]),
            &none,
        );
        assert_eq!(got, vec![(OverlapLevel::Warn, "requirement".to_string())]);
        // Unmapped scenario ids still conflict.
        let got = run(
            imp(&["s-9"], &[], &[], &[]),
            imp(&["s-9"], &[], &[], &[]),
            &none,
        );
        assert_eq!(got, vec![(OverlapLevel::Conflict, "scenario".to_string())]);
    }

    #[test]
    fn only_in_flight_tasks_count() {
        let i = imp(&["s-1"], &[], &[], &[]);
        let ts = [
            task("a", TaskState::Planned, i.clone()),
            task("b", TaskState::Integrated, i.clone()),
            task("c", TaskState::Open, i),
        ];
        assert!(check(&ts, &map()).is_empty());
    }

    const SPEC: &str = "### Requirement: A   {#r-aaaa}\n\nSHALL A.\n\n#### Scenario: S   {#s-bbbb}\n\n*Verification*: **executable**\n\n- **GIVEN** one\n- **WHEN** it runs\n- **THEN** ok\n";

    #[test]
    fn changed_ids_ignore_untouched_and_flag_edits() {
        assert!(changed_spec_ids(Some(SPEC), Some(SPEC)).is_empty());
        let edited = SPEC.replace("GIVEN** one", "GIVEN** two");
        let ids = changed_spec_ids(Some(SPEC), Some(&edited));
        assert_eq!(ids.into_iter().collect::<Vec<_>>(), ["r-aaaa", "s-bbbb"]);
        // A new file: everything in it is new.
        assert!(changed_spec_ids(None, Some(SPEC)).contains("s-bbbb"));
    }

    #[test]
    fn landing_overlap_names_ids_and_files() {
        let ids: BTreeSet<String> = ["s-bbbb".to_string()].into();
        let files = vec!["src/a/x.rs".to_string(), "README.md".to_string()];
        let i = imp(&["s-bbbb", "s-zzzz"], &[], &[], &["src/a/**"]);
        assert_eq!(
            landing_overlap(&i, &ids, &files).as_deref(),
            Some("s-bbbb, src/a/x.rs")
        );
        assert_eq!(
            landing_overlap(&imp(&[], &[], &[], &["docs/**"]), &ids, &files),
            None
        );
        assert_eq!(landing_overlap(&Impact::default(), &ids, &files), None);
    }
}
