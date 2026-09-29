//! Frontmatter of an exploration's findings doc (`design/explore/<id>/*.md`,
//! `docs/design/explorations.md`):
//!
//! ```yaml
//! ---
//! exploratory: true
//! task: tw-e41a
//! diverges-from: [a-12cd, r-7fa2]
//! status: open            # open | concluded | adopted | abandoned
//! ---
//! ```
//!
//! Only this flat shape is understood, so it is read by hand rather than with
//! a YAML parser. Like the other parsers it reports every problem.

use crate::Diagnostic;
use crate::parse::is_id;

pub const STATUSES: [&str; 4] = ["open", "concluded", "adopted", "abandoned"];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Frontmatter {
    pub task: String,
    pub diverges_from: Vec<String>,
    pub status: String,
}

/// The findings doc scaffolded by `bridle explore new`.
pub fn scaffold(task: &str) -> String {
    format!(
        "---\nexploratory: true\ntask: {task}\ndiverges-from: []\nstatus: open\n---\n\n\
         # Findings\n\nWhat was tried, what happened, and the recommendation.\n"
    )
}

fn diag(file: &str, line: usize, message: String) -> Diagnostic {
    Diagnostic {
        file: file.to_string(),
        line,
        column: 1,
        message,
    }
}

/// `key: value  # comment` -> (key, value), for a frontmatter line.
fn split_kv(line: &str) -> Option<(&str, &str)> {
    let (k, v) = line.split_once(':')?;
    let v = v.split_once(" #").map_or(v, |(v, _)| v);
    Some((k.trim(), v.trim()))
}

/// Lines of the frontmatter block, as (1-based line, text), or `None` when the
/// text doesn't open with `---` and close it.
fn block(text: &str) -> Option<Vec<(usize, &str)>> {
    let mut lines = text.lines().enumerate();
    if lines.next()?.1.trim_end() != "---" {
        return None;
    }
    let mut out = Vec::new();
    for (i, l) in lines {
        if l.trim_end() == "---" {
            return Some(out);
        }
        out.push((i + 1, l));
    }
    None
}

/// Parse and validate one findings doc. `file` only labels diagnostics.
pub fn parse_str(file: &str, text: &str) -> Result<Frontmatter, Vec<Diagnostic>> {
    let Some(lines) = block(text) else {
        return Err(vec![diag(
            file,
            1,
            "expected frontmatter (a '---' block opening the file), found none".into(),
        )]);
    };
    let mut diags = Vec::new();
    let (mut exploratory, mut task, mut diverges, mut status) = (None, None, None, None);
    for (n, l) in lines {
        let Some((k, v)) = split_kv(l) else { continue };
        match k {
            "exploratory" => exploratory = Some((n, v)),
            "task" => task = Some((n, v)),
            "diverges-from" => diverges = Some((n, v)),
            "status" => status = Some((n, v)),
            _ => {}
        }
    }
    match exploratory {
        Some((_, "true")) => {}
        Some((n, v)) => diags.push(diag(
            file,
            n,
            format!("expected 'exploratory: true', found 'exploratory: {v}'"),
        )),
        None => diags.push(diag(file, 1, "missing required key 'exploratory'".into())),
    }
    let task = match task {
        Some((_, v)) if !v.is_empty() => v.to_string(),
        Some((n, _)) => {
            diags.push(diag(file, n, "expected a task id after 'task:'".into()));
            String::new()
        }
        None => {
            diags.push(diag(file, 1, "missing required key 'task'".into()));
            String::new()
        }
    };
    let mut ids = Vec::new();
    match diverges {
        Some((n, v)) => match v.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
            Some(inner) => {
                for id in inner.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    if ['g', 'a', 'r', 's'].iter().any(|p| is_id(id, *p)) {
                        ids.push(id.to_string());
                    } else {
                        diags.push(diag(
                            file,
                            n,
                            format!("expected a g-/a-/r-/s- id in 'diverges-from', found '{id}'"),
                        ));
                    }
                }
            }
            None => diags.push(diag(
                file,
                n,
                format!("expected 'diverges-from: [id, ...]', found '{v}'"),
            )),
        },
        None => diags.push(diag(file, 1, "missing required key 'diverges-from'".into())),
    }
    let status = match status {
        Some((_, v)) if STATUSES.contains(&v) => v.to_string(),
        Some((n, v)) => {
            diags.push(diag(
                file,
                n,
                format!(
                    "expected status one of {}, found '{v}'",
                    STATUSES.join(" | ")
                ),
            ));
            String::new()
        }
        None => {
            diags.push(diag(file, 1, "missing required key 'status'".into()));
            String::new()
        }
    };
    if diags.is_empty() {
        Ok(Frontmatter {
            task,
            diverges_from: ids,
            status,
        })
    } else {
        Err(diags)
    }
}

/// Rewrite the `status:` value in the frontmatter, changing nothing else (a
/// trailing `# comment` on the line is kept). The doc must already check clean.
pub fn set_status(file: &str, text: &str, new: &str) -> Result<String, Vec<Diagnostic>> {
    parse_str(file, text)?;
    let n = block(text)
        .and_then(|b| {
            b.into_iter()
                .find(|(_, l)| split_kv(l).is_some_and(|(k, _)| k == "status"))
        })
        .map(|(n, _)| n)
        .expect("a checked doc has a status line");
    let mut out = String::with_capacity(text.len());
    for (i, l) in text.split_inclusive('\n').enumerate() {
        if i + 1 != n {
            out.push_str(l);
            continue;
        }
        let body = l.trim_end_matches(['\n', '\r']);
        let eol = &l[body.len()..];
        let comment = body.find(" #").map_or("", |p| &body[p..]);
        out.push_str(&format!("status: {new}{comment}{eol}"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "---\r\nexploratory: true\r\ntask: tw-e41a\r\ndiverges-from: [a-12cd, r-7fa2]\r\nstatus: open   # open | concluded\r\n---\r\n\r\nbody  \r\nno newline";

    #[test]
    fn valid() {
        let f = parse_str("f.md", DOC).unwrap();
        assert_eq!(f.task, "tw-e41a");
        assert_eq!(f.diverges_from, ["a-12cd", "r-7fa2"]);
        assert_eq!(f.status, "open");
        assert!(parse_str("f.md", &scaffold("br-1")).is_ok());
    }

    #[test]
    fn missing_key_and_no_frontmatter() {
        let ds = parse_str("f.md", "---\nexploratory: true\nstatus: open\n---\n").unwrap_err();
        let msgs: Vec<_> = ds.iter().map(|d| d.message.as_str()).collect();
        assert!(msgs.contains(&"missing required key 'task'"));
        assert!(msgs.contains(&"missing required key 'diverges-from'"));
        let ds = parse_str("f.md", "# just a doc\n").unwrap_err();
        assert!(ds[0].message.contains("expected frontmatter"));
    }

    #[test]
    fn bad_status_and_bad_id() {
        let text = DOC
            .replace("status: open", "status: done")
            .replace("a-12cd", "x-12cd");
        let ds = parse_str("f.md", &text).unwrap_err();
        assert_eq!(ds.len(), 2);
        assert!(
            ds.iter()
                .any(|d| d.message.contains("found 'done'") && d.line == 5)
        );
        assert!(
            ds.iter()
                .any(|d| d.message.contains("found 'x-12cd'") && d.line == 4)
        );
    }

    #[test]
    fn set_status_preserves_the_rest() {
        let out = set_status("f.md", DOC, "concluded").unwrap();
        assert_eq!(out, DOC.replace("status: open  ", "status: concluded"));
        assert!(out.contains("status: concluded # open | concluded\r\n"));
        assert!(out.ends_with("body  \r\nno newline"));
        assert_eq!(parse_str("f.md", &out).unwrap().status, "concluded");
    }
}
