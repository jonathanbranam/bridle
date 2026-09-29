//! Parser for goal files (`design/goals/*.md`, `docs/design/goals-tier.md`).
//!
//! ```markdown
//! ## Offline-first clients                         {#g-03}
//! firmness: firm · priority: later · stance: unaddressed
//!
//! Body prose.
//!
//! **Why unaddressed:** it needs a sync engine we don't have.
//! ```
//!
//! Like the spec parser it is strict and reports every problem in the file.
//! Goal ids are written by hand, never assigned.

use std::collections::BTreeSet;
use std::fmt;

use crate::Diagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Firmness {
    Fixed,
    Firm,
    Soft,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Now,
    Next,
    Later,
    Someday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stance {
    Build,
    KeepOpen,
    Unaddressed,
}

impl Firmness {
    pub fn as_str(self) -> &'static str {
        match self {
            Firmness::Fixed => "fixed",
            Firmness::Firm => "firm",
            Firmness::Soft => "soft",
            Firmness::Open => "open",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "fixed" => Firmness::Fixed,
            "firm" => Firmness::Firm,
            "soft" => Firmness::Soft,
            "open" => Firmness::Open,
            _ => return None,
        })
    }
}

impl Priority {
    pub fn as_str(self) -> &'static str {
        match self {
            Priority::Now => "now",
            Priority::Next => "next",
            Priority::Later => "later",
            Priority::Someday => "someday",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "now" => Priority::Now,
            "next" => Priority::Next,
            "later" => Priority::Later,
            "someday" => Priority::Someday,
            _ => return None,
        })
    }
    fn default_stance(self) -> Stance {
        match self {
            Priority::Now => Stance::Build,
            Priority::Next => Stance::KeepOpen,
            Priority::Later | Priority::Someday => Stance::Unaddressed,
        }
    }
}

impl Stance {
    pub fn as_str(self) -> &'static str {
        match self {
            Stance::Build => "build",
            Stance::KeepOpen => "keep-open",
            Stance::Unaddressed => "unaddressed",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "build" => Stance::Build,
            "keep-open" => Stance::KeepOpen,
            "unaddressed" => Stance::Unaddressed,
            _ => return None,
        })
    }
}

impl fmt::Display for Firmness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl fmt::Display for Stance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Goal {
    /// `g-NN`, as written in the heading.
    pub id: String,
    pub title: String,
    pub firmness: Firmness,
    pub priority: Priority,
    /// Explicit, or defaulted from the priority.
    pub stance: Stance,
    /// The prose after the attribute line, trimmed.
    pub body: String,
    /// 1-based line of the heading.
    pub line: usize,
}

/// What [`parse_goals`] found. Goals with errors are left out; a warning
/// leaves its goal in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Goals {
    pub goals: Vec<Goal>,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

fn col(line: &str, byte: usize) -> usize {
    line[..byte].chars().count() + 1
}

/// Parse goal text. `file` only labels diagnostics.
pub fn parse_goals(file: &str, text: &str) -> Goals {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Goals::default();
    let mut seen = BTreeSet::new();
    let diag = |line: usize, column: usize, message: String| Diagnostic {
        file: file.to_string(),
        line,
        column,
        message,
    };

    // Heading line indexes, skipping fenced code (goals-tier.md shows one).
    let mut fenced = false;
    let mut heads = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if l.trim_start().starts_with("```") {
            fenced = !fenced;
        } else if !fenced && l.starts_with("## ") {
            heads.push(i);
        }
    }

    for (n, &h) in heads.iter().enumerate() {
        let end = heads.get(n + 1).copied().unwrap_or(lines.len());
        let heading = lines[h];
        let errs_before = out.errors.len();

        let rest = heading[3..].trim_end();
        let (title, id) = match rest.rfind("{#") {
            Some(open) if rest.ends_with('}') => {
                let id = &rest[open + 2..rest.len() - 1];
                let c = col(heading, 3 + open);
                if id.len() > 2
                    && id.starts_with("g-")
                    && id[2..]
                        .chars()
                        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
                {
                    if !seen.insert(id.to_string()) {
                        out.errors
                            .push(diag(h + 1, c, format!("duplicate goal id {id:?}")));
                    }
                    (rest[..open].trim_end(), id)
                } else {
                    out.errors.push(diag(
                        h + 1,
                        c,
                        format!(
                            "expected a goal id like '{{#g-03}}', found {:?}",
                            &rest[open..]
                        ),
                    ));
                    (rest[..open].trim_end(), "")
                }
            }
            _ => {
                out.errors.push(diag(
                    h + 1,
                    col(heading, heading.len()),
                    format!("expected '{{#g-NN}}' to end the goal heading, found {rest:?}"),
                ));
                (rest, "")
            }
        };

        let attr_i = (h + 1..end).find(|&i| !lines[i].trim().is_empty());
        let mut firmness = None;
        let mut priority = None;
        let mut stance = None;
        match attr_i {
            Some(i) if lines[i].trim_start().starts_with("firmness:") => {
                let line = lines[i];
                for part in line.split('·') {
                    let part_c = col(line, part.as_ptr() as usize - line.as_ptr() as usize);
                    let Some((key, value)) = part.split_once(':') else {
                        out.errors.push(diag(
                            i + 1,
                            part_c,
                            format!("expected 'key: value', found {:?}", part.trim()),
                        ));
                        continue;
                    };
                    let (key, value) = (key.trim(), value.trim());
                    let vc = col(line, value.as_ptr() as usize - line.as_ptr() as usize);
                    let bad = |out: &mut Goals, set: &str| {
                        out.errors.push(diag(
                            i + 1,
                            vc,
                            format!("unknown {key} {value:?}, expected one of {set}"),
                        ));
                    };
                    match key {
                        "firmness" => match Firmness::parse(value) {
                            Some(v) => firmness = Some(v),
                            None => bad(&mut out, "fixed, firm, soft, open"),
                        },
                        "priority" => match Priority::parse(value) {
                            Some(v) => priority = Some(v),
                            None => bad(&mut out, "now, next, later, someday"),
                        },
                        "stance" => match Stance::parse(value) {
                            Some(v) => stance = Some(v),
                            None => bad(&mut out, "build, keep-open, unaddressed"),
                        },
                        _ => out.errors.push(diag(
                            i + 1,
                            col(line, key.as_ptr() as usize - line.as_ptr() as usize),
                            format!("unknown attribute {key:?}, expected firmness, priority or stance"),
                        )),
                    }
                }
                // A bad value was already reported; only report absence.
                if out.errors.len() == errs_before {
                    for (name, present) in [("firmness", firmness.is_some()), ("priority", priority.is_some())] {
                        if !present {
                            out.errors.push(diag(i + 1, 1, format!("missing '{name}:' in the attribute line")));
                        }
                    }
                }
            }
            Some(i) => out.errors.push(diag(
                i + 1,
                1,
                format!(
                    "expected 'firmness: X · priority: Y' under the heading, found {:?}",
                    lines[i].trim()
                ),
            )),
            None => out.errors.push(diag(
                h + 1,
                col(heading, heading.len()),
                "expected an attribute line 'firmness: X · priority: Y' under the heading, found none".into(),
            )),
        }

        if out.errors.len() > errs_before {
            continue;
        }
        let (Some(firmness), Some(priority), Some(i)) = (firmness, priority, attr_i) else {
            continue;
        };
        let stance = stance.unwrap_or_else(|| priority.default_stance());
        let body = lines[i + 1..end].join("\n");
        if stance == Stance::Unaddressed
            && !body
                .lines()
                .any(|l| l.trim_start().starts_with("**Why unaddressed:**"))
        {
            out.warnings.push(diag(
                h + 1,
                1,
                format!("goal {id} is unaddressed but has no '**Why unaddressed:**' line"),
            ));
        }
        out.goals.push(Goal {
            id: id.to_string(),
            title: title.to_string(),
            firmness,
            priority,
            stance,
            body: body.trim().to_string(),
            line: h + 1,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "# Goals\n\n## Offline {#g-03}\nfirmness: firm · priority: later · stance: unaddressed\n\nBody.\n\n**Why unaddressed:** no sync.\n\n## Fast {#g-04}\n\nfirmness: fixed · priority: now\n\nGo fast.\n";

    #[test]
    fn parses_and_defaults_stance() {
        let g = parse_goals("f.md", GOOD);
        assert!(g.errors.is_empty(), "{:?}", g.errors);
        assert!(g.warnings.is_empty(), "{:?}", g.warnings);
        assert_eq!(g.goals.len(), 2);
        assert_eq!(g.goals[0].id, "g-03");
        assert_eq!(g.goals[0].title, "Offline");
        assert_eq!(g.goals[0].stance, Stance::Unaddressed);
        assert_eq!(g.goals[1].stance, Stance::Build);
        assert_eq!(g.goals[1].body, "Go fast.");
        for (p, s) in [
            ("next", Stance::KeepOpen),
            ("later", Stance::Unaddressed),
            ("someday", Stance::Unaddressed),
        ] {
            let t =
                format!("## X {{#g-01}}\nfirmness: soft · priority: {p}\n**Why unaddressed:** y\n");
            assert_eq!(parse_goals("f", &t).goals[0].stance, s);
        }
    }

    #[test]
    fn bad_values_are_errors_with_positions() {
        let t = "## X {#g-01}\nfirmness: hard · priority: soon · stance: maybe\n";
        let g = parse_goals("f.md", t);
        assert!(g.goals.is_empty());
        let msgs: Vec<String> = g.errors.iter().map(ToString::to_string).collect();
        assert_eq!(msgs.len(), 3, "{msgs:?}");
        assert!(
            msgs[0].starts_with("f.md:2:11: unknown firmness \"hard\""),
            "{msgs:?}"
        );
        assert!(msgs[1].contains("unknown priority \"soon\""), "{msgs:?}");
        assert!(msgs[2].contains("unknown stance \"maybe\""), "{msgs:?}");
    }

    #[test]
    fn structural_errors() {
        let g = parse_goals("f", "## No id\nfirmness: firm · priority: now\n");
        assert!(g.errors[0].message.contains("{#g-NN}"), "{:?}", g.errors);
        let g = parse_goals("f", "## X {#g-01}\n\nno attrs\n");
        assert!(
            g.errors[0].message.contains("under the heading"),
            "{:?}",
            g.errors
        );
        let g = parse_goals("f", "## X {#g-01}\nfirmness: firm\n");
        assert!(g.errors[0].message.contains("priority"), "{:?}", g.errors);
        let two = "## A {#g-01}\nfirmness: firm · priority: now\n## B {#g-01}\nfirmness: firm · priority: now\n";
        assert!(
            parse_goals("f", two).errors[0]
                .message
                .contains("duplicate")
        );
    }

    #[test]
    fn unaddressed_without_why_warns() {
        let g = parse_goals(
            "f",
            "## X {#g-01}\nfirmness: open · priority: someday\n\nProse.\n",
        );
        assert!(g.errors.is_empty());
        assert_eq!(g.goals.len(), 1);
        assert_eq!(g.warnings.len(), 1);
        assert!(g.warnings[0].message.contains("Why unaddressed"));
    }

    #[test]
    fn headings_in_code_fences_are_ignored() {
        let t = "```markdown\n## Example {#g-99}\n```\n\n## Real {#g-01}\nfirmness: firm · priority: now\n";
        let g = parse_goals("f", t);
        assert_eq!(g.goals.len(), 1);
        assert!(g.errors.is_empty());
    }
}
