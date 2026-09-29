//! Assigning stable ids to headings that lack one (`bridle spec id`).
//!
//! Edits are made line by line on the original text so everything except the
//! touched heading lines stays byte-for-byte identical.

use std::collections::BTreeSet;

use crate::{Diagnostic, parse_str};

/// One id written into a heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assigned {
    /// 1-based line of the heading.
    pub line: usize,
    pub id: String,
    pub title: String,
}

/// The longest id (hex digits) before giving up on finding a free one.
const MAX_HEX: usize = 16;

/// Give every requirement and scenario in `text` without an id a fresh one and
/// return the edited text with what was assigned. `taken` holds every id in
/// use or ever used (other files, the ledger); the new ids are added to it.
/// `rng` supplies the randomness, so tests can seed it. A candidate is 4 hex
/// digits, and only a collision makes the next attempt one digit longer.
pub fn assign_ids(
    file: &str,
    text: &str,
    taken: &mut BTreeSet<String>,
    rng: &mut impl FnMut() -> u64,
) -> Result<(String, Vec<Assigned>), Vec<Diagnostic>> {
    let spec = parse_str(file, text)?;
    // (0-based line, prefix, title) for every heading lacking an id.
    let mut todo: Vec<(usize, char, &str)> = Vec::new();
    for r in &spec.requirements {
        if r.id.is_none() {
            todo.push((r.line - 1, 'r', &r.title));
        }
        for s in r.scenarios.iter().filter(|s| s.id.is_none()) {
            todo.push((s.line - 1, 's', &s.title));
        }
    }
    if todo.is_empty() {
        return Ok((text.to_string(), Vec::new()));
    }

    let mut assigned = Vec::new();
    let mut out = String::with_capacity(text.len() + todo.len() * 14);
    let mut todo = todo.into_iter().peekable();
    for (n, raw) in text.split_inclusive('\n').enumerate() {
        let Some(&(_, prefix, title)) = todo.next_if(|t| t.0 == n).as_ref() else {
            out.push_str(raw);
            continue;
        };
        let id = fresh_id(prefix, taken, rng).map_err(|message| {
            vec![Diagnostic {
                file: file.to_string(),
                line: n + 1,
                column: 1,
                message,
            }]
        })?;
        let body = raw.trim_end_matches(['\n', '\r']);
        out.push_str(&with_id(body, &id));
        out.push_str(&raw[body.len()..]);
        assigned.push(Assigned {
            line: n + 1,
            id,
            title: title.to_string(),
        });
    }
    Ok((out, assigned))
}

fn fresh_id(
    prefix: char,
    taken: &mut BTreeSet<String>,
    rng: &mut impl FnMut() -> u64,
) -> Result<String, String> {
    for len in 4..=MAX_HEX {
        let hex = format!("{:016x}", rng());
        let id = format!("{prefix}-{}", &hex[..len]);
        if taken.insert(id.clone()) {
            return Ok(id);
        }
    }
    Err("expected a free id, but every candidate was taken".into())
}

/// The heading with `#id` added: first in an existing `{...}` block, or in a
/// new one at the end.
fn with_id(heading: &str, id: &str) -> String {
    let h = heading.trim_end();
    if h.ends_with('}')
        && let Some(open) = h.rfind('{')
    {
        let inner = h[open + 1..h.len() - 1].trim();
        return if inner.is_empty() {
            format!("{}{{#{id}}}", &h[..open])
        } else {
            format!("{}{{#{id} {}", &h[..open], &h[open + 1..])
        };
    }
    format!("{h}  {{#{id}}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "# Cap\n\nintro\n\n### Requirement: One   {protected}\nThe system SHALL.\n\n#### Scenario: A\n*Verification*: **non-executable**\nprose\n\n### Requirement: Two {#r-abcd}\nThe system SHALL.\n\n#### Scenario: B  {#s-1234}\n*Verification*: **non-executable**\n";

    fn counter() -> impl FnMut() -> u64 {
        let mut n = 0u64;
        move || {
            n += 1;
            n << 48
        }
    }

    #[test]
    fn assigns_only_where_missing_and_keeps_the_rest() {
        let mut taken = BTreeSet::from(["r-abcd".to_string(), "s-1234".to_string()]);
        let (out, got) = assign_ids("t.md", SPEC, &mut taken, &mut counter()).expect("ok");
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].id, "r-0001");
        assert_eq!(got[1].id, "s-0002");
        let want = SPEC
            .replace("{protected}", "{#r-0001 protected}")
            .replace("Scenario: A\n", "Scenario: A  {#s-0002}\n");
        assert_eq!(out, want);
        assert!(taken.contains("r-0001") && taken.contains("s-0002"));
    }

    #[test]
    fn idempotent() {
        let mut taken = BTreeSet::new();
        let (once, _) = assign_ids("t.md", SPEC, &mut taken, &mut counter()).expect("ok");
        let (twice, got) = assign_ids("t.md", &once, &mut taken, &mut counter()).expect("ok");
        assert_eq!(once, twice);
        assert!(got.is_empty());
    }

    #[test]
    fn collision_lengthens_the_id() {
        let mut taken = BTreeSet::from(["r-abcd".to_string()]);
        let mut rng = || 0xabcd_ef01_2345_6789u64;
        let (out, got) = assign_ids(
            "t.md",
            "### Requirement: X\nSHALL.\n#### Scenario: Y\n*Verification*: **non-executable**\n",
            &mut taken,
            &mut rng,
        )
        .expect("ok");
        assert_eq!(got[0].id, "r-abcde");
        assert_eq!(got[1].id, "s-abcd");
        assert!(out.contains("{#r-abcde}"));
    }

    #[test]
    fn preserves_crlf_and_missing_final_newline() {
        let text = "### Requirement: X\r\nSHALL.\r\n#### Scenario: Y\r\n*Verification*: **non-executable**";
        let (out, _) = assign_ids("t.md", text, &mut BTreeSet::new(), &mut counter()).expect("ok");
        assert_eq!(
            out,
            "### Requirement: X  {#r-0001}\r\nSHALL.\r\n#### Scenario: Y  {#s-0002}\r\n*Verification*: **non-executable**"
        );
    }

    #[test]
    fn parse_errors_are_returned_untouched() {
        assert!(
            assign_ids(
                "t.md",
                "### Requirement: X\n",
                &mut BTreeSet::new(),
                &mut counter()
            )
            .is_err()
        );
    }
}
