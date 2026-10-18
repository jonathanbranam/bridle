//! Stages of a version 2 ticket (br-9mqh, epic br-tj7q): the `stages:` frontmatter block, the
//! proposal section of the body, the stage folder, and each stage's state. Design:
//! `docs/design/stages.md`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Every stage name, in OpenSpec's order.
pub const NAMES: [&str; 4] = ["proposal", "design", "specs", "tasks"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub name: String,
    pub requires: Vec<String>,
    pub human_review: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Waiting,
    Unblocked,
    Written,
    WaitingForSignature,
    Signed,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            State::Waiting => "waiting",
            State::Unblocked => "unblocked",
            State::Written => "written",
            State::WaitingForSignature => "waiting for signature",
            State::Signed => "signed",
        })
    }
}

/// What a new version 2 ticket's body ends with: the proposal, between two rules.
pub const PROPOSAL_TEMPLATE: &str = "\n---\n\n## Proposal\n\n### Why\n\n### What changes\n\n### What does not change\n\n### Risk and impact\n\n### Out of scope\n\n### Open questions\n\n---\n";

/// OpenSpec's graph: what `stage` requires, restricted to the listed stages.
fn default_requires(stage: &str, listed: &[String]) -> Vec<String> {
    let want: &[&str] = match stage {
        "design" | "specs" => &["proposal"],
        "tasks" => &["design", "specs"],
        _ => &[],
    };
    want.iter()
        .filter(|w| listed.iter().any(|l| l == *w))
        .map(|w| w.to_string())
        .collect()
}

/// The stages `ticket new --stages ... --review ...` asks for, `proposal` always included.
pub fn build(stages: &[String], review: &[String]) -> anyhow::Result<Vec<Stage>> {
    for n in stages.iter().chain(review) {
        if !NAMES.contains(&n.as_str()) {
            anyhow::bail!("unknown stage {n} (known: {})", NAMES.join(", "));
        }
    }
    let listed: Vec<String> = NAMES
        .iter()
        .filter(|n| **n == "proposal" || stages.iter().any(|s| s == *n))
        .map(|n| n.to_string())
        .collect();
    if let Some(r) = review.iter().find(|r| !listed.contains(r)) {
        anyhow::bail!("--review names {r}, which is not a listed stage");
    }
    Ok(listed
        .iter()
        .map(|n| Stage {
            name: n.clone(),
            requires: default_requires(n, &listed),
            human_review: review.contains(n),
        })
        .collect())
}

/// The frontmatter block for `stages`, ending in a newline.
pub fn render(stages: &[Stage]) -> String {
    if stages.is_empty() {
        return String::new();
    }
    let mut out = String::from("stages:\n");
    for s in stages {
        let mut parts = Vec::new();
        if !s.requires.is_empty() {
            parts.push(format!("requires: [{}]", s.requires.join(", ")));
        }
        if s.human_review {
            parts.push("review: human".to_string());
        }
        out.push_str(&format!("  {}: {{{}}}\n", s.name, parts.join(", ")));
    }
    out
}

/// The stages of the frontmatter `lines` (between the `---` fences): the indented lines under
/// `stages:`. `Ok(None)` when there is no `stages:` key.
pub fn parse(front: &[&str]) -> Result<Option<Vec<Stage>>, String> {
    let Some(at) = front.iter().position(|l| l.trim_end() == "stages:") else {
        return Ok(None);
    };
    let mut out: Vec<Stage> = Vec::new();
    for line in front[at + 1..]
        .iter()
        .take_while(|l| l.starts_with(' ') || l.starts_with('\t'))
    {
        let (name, rest) = line
            .trim()
            .split_once(':')
            .ok_or_else(|| format!("stages: line `{}` is not `name: {{...}}`", line.trim()))?;
        let name = name.trim();
        if !NAMES.contains(&name) {
            return Err(format!("unknown stage {name}"));
        }
        if out.iter().any(|s| s.name == name) {
            return Err(format!("stage {name} is listed twice"));
        }
        let inner = rest
            .trim()
            .strip_prefix('{')
            .and_then(|r| r.strip_suffix('}'))
            .ok_or_else(|| format!("stage {name}: expected {{...}}"))?;
        let mut stage = Stage {
            name: name.to_string(),
            requires: Vec::new(),
            human_review: false,
        };
        // Split on commas outside the `[...]` list.
        let (mut depth, mut start, mut parts) = (0, 0, Vec::new());
        for (i, c) in inner.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => depth -= 1,
                ',' if depth == 0 => {
                    parts.push(&inner[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        parts.push(&inner[start..]);
        for p in parts.into_iter().map(str::trim).filter(|p| !p.is_empty()) {
            match p.split_once(':').map(|(k, v)| (k.trim(), v.trim())) {
                Some(("requires", v)) => {
                    let list = v
                        .strip_prefix('[')
                        .and_then(|v| v.strip_suffix(']'))
                        .ok_or_else(|| format!("stage {name}: requires is not a [a, b] list"))?;
                    stage.requires = list
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                Some(("review", "human")) => stage.human_review = true,
                Some(("review", v)) => {
                    return Err(format!("stage {name}: review {v} is not `human`"));
                }
                _ => return Err(format!("stage {name}: unknown setting `{p}`")),
            }
        }
        out.push(stage);
    }
    if out.is_empty() {
        return Err("stages: lists no stage".into());
    }
    Ok(Some(out))
}

/// What is wrong with the graph of `stages`: a missing proposal, a `requires` on an unlisted
/// stage, or a cycle.
pub fn validate(stages: &[Stage]) -> Result<(), String> {
    if !stages.iter().any(|s| s.name == "proposal") {
        return Err("stages must include proposal".into());
    }
    for s in stages {
        for r in &s.requires {
            if !stages.iter().any(|o| &o.name == r) {
                return Err(format!(
                    "stage {} requires {r}, which is not listed",
                    s.name
                ));
            }
        }
    }
    // Peel off stages whose requirements are all peeled; what is left is a cycle.
    let mut done: HashSet<&str> = HashSet::new();
    while done.len() < stages.len() {
        let ready: Vec<&str> = stages
            .iter()
            .filter(|s| !done.contains(s.name.as_str()))
            .filter(|s| s.requires.iter().all(|r| done.contains(r.as_str())))
            .map(|s| s.name.as_str())
            .collect();
        if ready.is_empty() {
            return Err("stages require each other in a cycle".into());
        }
        done.extend(ready);
    }
    Ok(())
}

/// The proposal: line indexes (into `body.lines()`) of its opening and closing `---` lines, with
/// `## Proposal` the first non-blank line after the opening one.
pub fn find_proposal(body: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = body.lines().collect();
    let mut in_code = false;
    let mut start = None;
    for (i, l) in lines.iter().enumerate() {
        if l.trim_start().starts_with("```") {
            in_code = !in_code;
        }
        if in_code || l.trim_end() != "---" {
            continue;
        }
        match start {
            None => {
                let next = lines[i + 1..].iter().find(|l| !l.trim().is_empty())?;
                if next.trim_end() == "## Proposal" {
                    start = Some(i);
                }
            }
            Some(s) => return Some((s, i)),
        }
    }
    None
}

/// The folder beside a ticket file: `<slug>-<id>.md` -> `<slug>-<id>/`.
pub fn folder_of(ticket: &Path) -> PathBuf {
    ticket.with_extension("")
}

/// A stage's file in the ticket's folder; the proposal has none (it is in the body).
pub fn file_of(folder: &Path, stage: &str) -> Option<PathBuf> {
    (stage != "proposal").then(|| folder.join(format!("{stage}.md")))
}

/// What a ticket's frontmatter records about its stages: `written: [..]` (set by `stage-done`,
/// cleared by `changes` and `restage`) and `signed: [..]` (set by `approve`). A signature is the
/// human's: who ran it and when are on the thread (br-ussh).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Marks {
    pub written: Vec<String>,
    pub signed: Vec<String>,
}

/// The `written` and `signed` lists of a ticket file's text.
pub fn marks(text: &str) -> Marks {
    let mut m = Marks::default();
    for line in text.lines().skip(1).take_while(|l| *l != "---") {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let items = || -> Vec<String> {
            value
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        match key {
            "written" => m.written = items(),
            "signed" => m.signed = items(),
            _ => {}
        }
    }
    m
}

/// A frontmatter flow list for `items`, as `set_line` writes it.
pub fn render_list(items: &[String]) -> String {
    format!("[{}]", items.join(", "))
}

/// The state of every stage, from the ticket's marks. A stage is unblocked once everything it
/// requires is done: written with no review, or signed.
pub fn states(stages: &[Stage], marks: &Marks) -> Vec<(String, State)> {
    let mut known: Vec<(String, State)> = Vec::new();
    for s in stages {
        let written = marks.written.contains(&s.name);
        let signed = marks.signed.contains(&s.name);
        let done = |r: &String| {
            known
                .iter()
                .any(|(n, st)| n == r && matches!(st, State::Signed | State::Written))
        };
        let state = if !written {
            if s.requires.iter().all(done) {
                State::Unblocked
            } else {
                State::Waiting
            }
        } else if signed {
            State::Signed
        } else if s.human_review {
            State::WaitingForSignature
        } else {
            State::Written
        };
        known.push((s.name.clone(), state));
    }
    known
}

/// Every stage is written and every gate signed: the ticket is prepared (it becomes `open` by
/// itself).
pub fn prepared(stages: &[Stage], marks: &Marks) -> bool {
    states(stages, marks)
        .iter()
        .all(|(_, s)| matches!(s, State::Written | State::Signed))
}

/// Why `stage` cannot be marked written yet, or `Ok`: the proposal's section (or the stage's
/// file) must be there and say something.
pub fn check_written(text: &str, folder: &Path, stage: &str) -> Result<(), String> {
    match file_of(folder, stage) {
        Some(f) => match std::fs::read_to_string(&f) {
            Ok(t) if !t.trim().is_empty() => Ok(()),
            Ok(_) => Err(format!("{} is empty", f.display())),
            Err(_) => Err(format!("{} does not exist", f.display())),
        },
        None => {
            let body: String = text
                .lines()
                .skip(1)
                .skip_while(|l| *l != "---")
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n");
            let (s, e) = find_proposal(&body).ok_or("the body has no ## Proposal section")?;
            let said = body
                .lines()
                .skip(s + 1)
                .take(e - s - 1)
                .any(|l| !l.trim().is_empty() && !l.starts_with('#'));
            said.then_some(())
                .ok_or_else(|| "the proposal section is empty".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(s: &[Stage]) -> Vec<&str> {
        s.iter().map(|s| s.name.as_str()).collect()
    }

    #[test]
    fn build_fills_requires_from_the_openspec_graph() {
        let st = build(
            &["design".into(), "specs".into(), "tasks".into()],
            &["proposal".into(), "specs".into()],
        )
        .unwrap();
        assert_eq!(names(&st), ["proposal", "design", "specs", "tasks"]);
        assert_eq!(st[3].requires, ["design", "specs"]);
        assert!(st[0].human_review && st[2].human_review && !st[1].human_review);
        let only = build(&["tasks".into()], &[]).unwrap();
        assert!(only[1].requires.is_empty());
        assert!(build(&["nope".into()], &[]).is_err());
        assert!(build(&[], &["design".into()]).is_err());
    }

    #[test]
    fn render_then_parse_round_trips() {
        let st = build(
            &["design".into(), "specs".into(), "tasks".into()],
            &["proposal".into(), "specs".into()],
        )
        .unwrap();
        let text = render(&st);
        assert!(text.contains("  proposal: {review: human}\n"));
        assert!(text.contains("  design: {requires: [proposal]}\n"));
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(parse(&lines).unwrap().unwrap(), st);
    }

    #[test]
    fn parse_and_validate_reject_bad_graphs() {
        let p = |t: &str| parse(&t.lines().collect::<Vec<_>>());
        assert_eq!(p("title: x"), Ok(None));
        assert!(p("stages:\n  bogus: {}").is_err());
        assert!(p("stages:\n  design: {review: bot}").is_err());
        let cyc = p("stages:\n  proposal: {requires: [design]}\n  design: {requires: [proposal]}")
            .unwrap()
            .unwrap();
        assert!(validate(&cyc).unwrap_err().contains("cycle"));
        let dangling = p("stages:\n  proposal: {}\n  tasks: {requires: [specs]}")
            .unwrap()
            .unwrap();
        assert!(validate(&dangling).unwrap_err().contains("not listed"));
        let none = p("stages:\n  design: {}").unwrap().unwrap();
        assert!(validate(&none).is_err());
    }

    #[test]
    fn proposal_is_found_between_rules() {
        let body = "## The ask\n\nx\n\n---\n\n## Proposal\n\n### Why\n\n---\n\nafter\n";
        assert_eq!(find_proposal(body), Some((4, 10)));
        assert_eq!(find_proposal("---\n\n## Other\n---\n"), None);
    }

    #[test]
    fn states_follow_the_marks() {
        let st = build(
            &["design".into(), "specs".into()],
            &["proposal".into(), "specs".into()],
        )
        .unwrap();
        let get = |w: &[&str], sg: &[&str]| -> Vec<State> {
            let m = Marks {
                written: w.iter().map(|s| s.to_string()).collect(),
                signed: sg.iter().map(|s| s.to_string()).collect(),
            };
            states(&st, &m).into_iter().map(|(_, s)| s).collect()
        };
        use State::*;
        assert_eq!(get(&[], &[]), [Unblocked, Waiting, Waiting]);
        assert_eq!(
            get(&["proposal"], &[]),
            [WaitingForSignature, Waiting, Waiting]
        );
        assert_eq!(
            get(&["proposal"], &["proposal"]),
            [Signed, Unblocked, Unblocked]
        );
        let all = get(&["proposal", "design", "specs"], &["proposal"]);
        assert_eq!(all, [Signed, Written, WaitingForSignature]);
        let m = |w: &[&str], sg: &[&str]| Marks {
            written: w.iter().map(|s| s.to_string()).collect(),
            signed: sg.iter().map(|s| s.to_string()).collect(),
        };
        assert!(!prepared(
            &st,
            &m(&["proposal", "design", "specs"], &["proposal"])
        ));
        assert!(prepared(
            &st,
            &m(&["proposal", "design", "specs"], &["proposal", "specs"])
        ));
    }

    #[test]
    fn marks_are_read_from_the_frontmatter() {
        let m = marks(
            "---\nid: x\nwritten: [proposal, design]\nsigned: [proposal]\n---\nsigned: [x]\n",
        );
        assert_eq!(m.written, ["proposal", "design"]);
        assert_eq!(m.signed, ["proposal"]);
        assert_eq!(marks("---\nid: x\n---\n"), Marks::default());
    }

    #[test]
    fn check_written_wants_content() {
        let dir = tempfile::tempdir().unwrap();
        let empty = "---\nid: x\n---\n\n---\n\n## Proposal\n\n### Why\n\n---\n";
        assert!(check_written(empty, dir.path(), "proposal").is_err());
        let said = "---\nid: x\n---\n\n---\n\n## Proposal\n\n### Why\n\nbecause\n\n---\n";
        assert!(check_written(said, dir.path(), "proposal").is_ok());
        assert!(check_written(said, dir.path(), "design").is_err());
        std::fs::write(dir.path().join("design.md"), "x").unwrap();
        assert!(check_written(said, dir.path(), "design").is_ok());
    }
}
