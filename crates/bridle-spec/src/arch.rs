//! Parser for architecture-tier files (`design/architecture/*.md`,
//! `docs/design/architecture-tier.md`).
//!
//! An element is a level-2 heading with a hand-written id and an optional
//! `invariant` flag; everything up to the next heading is its body:
//!
//! ```markdown
//! ## The engine referees every rule   {#a-12cd invariant}
//! text...
//! **Alternatives rejected:** a paragraph, kept as text.
//! ```
//!
//! Like the spec parser it is strict and reports every problem. `a-` ids are
//! never assigned by bridle, so a missing id is an error, and ids must be
//! unique across all the files given to [`parse_files`].

use std::collections::HashMap;
use std::path::PathBuf;

use crate::Diagnostic;
use crate::parse::{col, is_id, offset_in};

const ALTERNATIVES_LABEL: &str = "**Alternatives rejected:**";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Element {
    pub id: String,
    pub title: String,
    pub invariant: bool,
    /// The body, minus the alternatives paragraph, trimmed.
    pub text: String,
    /// The text of the `**Alternatives rejected:**` paragraph, if any.
    pub alternatives_rejected: Option<String>,
    pub file: String,
    /// 1-based line of the heading.
    pub line: usize,
}

/// Parse one file's text. `file` only labels diagnostics and elements.
pub fn parse_str(file: &str, text: &str) -> Result<Vec<Element>, Vec<Diagnostic>> {
    parse_all(&[(file.to_string(), text.to_string())])
}

/// Read and parse every file, checking ids are unique across all of them.
pub fn parse_files(paths: &[PathBuf]) -> Result<Vec<Element>, Vec<Diagnostic>> {
    let mut inputs = Vec::new();
    let mut diags = Vec::new();
    for p in paths {
        let name = p.display().to_string();
        match std::fs::read_to_string(p) {
            Ok(t) => inputs.push((name, t)),
            Err(e) => diags.push(Diagnostic {
                file: name,
                line: 1,
                column: 1,
                message: format!("expected a readable file, found: {e}"),
            }),
        }
    }
    match parse_all(&inputs) {
        Ok(_) if !diags.is_empty() => Err(diags),
        Ok(els) => Ok(els),
        Err(mut ds) => {
            diags.append(&mut ds);
            Err(diags)
        }
    }
}

fn parse_all(inputs: &[(String, String)]) -> Result<Vec<Element>, Vec<Diagnostic>> {
    let mut elements = Vec::new();
    let mut diags = Vec::new();
    // id -> "file:line" of first use, across every file.
    let mut seen: HashMap<String, String> = HashMap::new();
    for (file, text) in inputs {
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let Some(rest) = lines[i].strip_prefix("## ") else {
                i += 1;
                continue;
            };
            let end = (i + 1..lines.len())
                .find(|&j| lines[j].starts_with('#'))
                .unwrap_or(lines.len());
            let mut diag = |column: usize, message: String| {
                diags.push(Diagnostic {
                    file: file.clone(),
                    line: i + 1,
                    column,
                    message,
                });
            };
            let (title, id, invariant) = heading(lines[i], rest, &mut diag);
            if let Some(id) = &id {
                let here = format!("{file}:{}", i + 1);
                if let Some(first) = seen.get(id) {
                    diag(1, format!("duplicate id {id:?}: first used at {first}"));
                } else {
                    seen.insert(id.clone(), here);
                }
            }
            if let Some(id) = id {
                let (text, alternatives_rejected) = body(&lines[i + 1..end]);
                elements.push(Element {
                    id,
                    title,
                    invariant,
                    text,
                    alternatives_rejected,
                    file: file.clone(),
                    line: i + 1,
                });
            }
            i = end;
        }
    }
    if diags.is_empty() {
        Ok(elements)
    } else {
        Err(diags)
    }
}

/// Split a `## Title {#a-12cd invariant}` heading; reports problems through
/// `diag(column, message)`. The id is `None` if absent or malformed.
fn heading(
    line: &str,
    rest: &str,
    diag: &mut impl FnMut(usize, String),
) -> (String, Option<String>, bool) {
    let rest = rest.trim();
    let Some(open) = rest.find("{#") else {
        diag(
            col(line, line.len()),
            format!("expected '{{#a-xxxx}}' after the title, found {rest:?} with no id"),
        );
        return (rest.to_string(), None, false);
    };
    let block = &rest[open..];
    let title = rest[..open].trim_end().to_string();
    if !block.ends_with('}') {
        diag(
            col(line, offset_in(line, block)),
            format!(
                "expected the '{{#id ...}}' block to end the heading with '}}', found {block:?}"
            ),
        );
        return (title, None, false);
    }
    if title.is_empty() {
        diag(
            1,
            "expected a title before the '{#id}' block, found none".into(),
        );
    }
    let mut id = None;
    let mut invariant = false;
    for (n, tok) in block[1..block.len() - 1].split_whitespace().enumerate() {
        let c = col(line, offset_in(line, tok));
        if let Some(t) = tok.strip_prefix('#') {
            if n != 0 {
                diag(
                    c,
                    format!(
                        "expected the '#id' first in the braces, found {tok:?} after other tokens"
                    ),
                );
            } else if !is_id(t, 'a') {
                diag(
                    c,
                    format!(
                        "expected an id like '#a-12cd' (lowercase hex, 4+ digits), found {tok:?}"
                    ),
                );
            } else {
                id = Some(t.to_string());
            }
        } else if tok == "invariant" {
            invariant = true;
        } else {
            diag(
                c,
                format!("expected one of '#id', 'invariant', found {tok:?}"),
            );
        }
    }
    (title, id, invariant)
}

/// The body text, with the `**Alternatives rejected:**` paragraph (up to the
/// next blank line) split out.
fn body(lines: &[&str]) -> (String, Option<String>) {
    let Some(start) = lines
        .iter()
        .position(|l| l.trim_start().starts_with(ALTERNATIVES_LABEL))
    else {
        return (lines.join("\n").trim().to_string(), None);
    };
    let len = lines[start..]
        .iter()
        .position(|l| l.trim().is_empty())
        .unwrap_or(lines.len() - start);
    let first = lines[start].trim_start()[ALTERNATIVES_LABEL.len()..].trim();
    let mut alt = vec![first];
    alt.extend(&lines[start + 1..start + len]);
    let alt = alt.join("\n").trim().to_string();
    let mut rest = lines[..start].to_vec();
    rest.extend(&lines[start + len..]);
    (rest.join("\n").trim().to_string(), Some(alt))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# Arch\n\nintro\n\n## Engine referees {#a-12cd invariant}\nThe engine decides.\n\n## Use SQLite {#a-00ff}\nWhy.\n\n**Alternatives rejected:** Postgres,\nbecause ops.\n\nAfter.\n";

    #[test]
    fn parses_ids_flags_and_alternatives() {
        let els = parse_str("a.md", DOC).expect("parses");
        assert_eq!(els.len(), 2);
        assert_eq!(els[0].id, "a-12cd");
        assert!(els[0].invariant);
        assert_eq!(els[0].title, "Engine referees");
        assert_eq!(els[0].text, "The engine decides.");
        assert_eq!(els[0].line, 5);
        assert!(!els[1].invariant);
        assert_eq!(
            els[1].alternatives_rejected.as_deref(),
            Some("Postgres,\nbecause ops.")
        );
        assert_eq!(els[1].text, "Why.\n\nAfter.");
    }

    #[test]
    fn missing_id_is_an_error() {
        let ds = parse_str("a.md", "## No id here\nbody\n").expect_err("fails");
        assert_eq!(ds.len(), 1);
        assert_eq!((ds[0].line, ds[0].file.as_str()), (1, "a.md"));
        assert!(ds[0].message.contains("no id"));
    }

    #[test]
    fn bad_id_and_unknown_flag_are_errors() {
        let ds = parse_str("a.md", "## X {#r-12cd}\n## Y {#a-12cd bogus}\n").expect_err("fails");
        assert_eq!(ds.len(), 2);
    }

    #[test]
    fn duplicate_ids_within_and_across_files() {
        let ds = parse_str("a.md", "## X {#a-12cd}\n## Y {#a-12cd}\n").expect_err("fails");
        assert!(ds[0].message.contains("duplicate"));
        let ds = parse_all(&[
            ("a.md".into(), "## X {#a-12cd}\n".into()),
            ("b.md".into(), "## Y {#a-12cd}\n".into()),
        ])
        .expect_err("fails");
        assert_eq!(ds[0].file, "b.md");
        assert!(ds[0].message.contains("a.md:1"));
    }
}
