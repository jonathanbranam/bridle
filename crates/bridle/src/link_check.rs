//! `bridle hook link-check`: the Stop hook that reminds an interactive session, once, to link the
//! task and ticket IDs in its reply (ticket 9dcz, rule `link-ids-for-the-human`). Pure scanning
//! lives here; the command in `commands/hook.rs` supplies the reply, the prefix and the links.
//! It never blocks twice and never fails: the caller allows on any error.

use std::collections::BTreeSet;
use std::path::Path;

use crate::ticket::is_bare_id;
use bridle_api::types::is_reserved_id;

/// Blank out fenced and inline code so IDs inside it are skipped.
fn without_code(text: &str) -> Vec<char> {
    let mut out = String::new();
    let mut fenced = false;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            out.push('\n');
        } else if fenced {
            out.push('\n');
        } else {
            let mut inline = false;
            for c in line.chars() {
                if c == '`' {
                    inline = !inline;
                    out.push(' ');
                } else if inline && c != '\n' {
                    out.push(' ');
                } else {
                    out.push(c);
                }
            }
        }
    }
    out.chars().collect()
}

/// Char ranges inside link syntax: `[[wiki]]`, `[text](url)` and bare `http(s)://` URLs.
fn link_spans(chars: &[char]) -> Vec<(usize, usize)> {
    let find = |from: usize, pat: &str| -> Option<usize> {
        let tail: String = chars[from..].iter().collect();
        tail.find(pat).map(|b| from + tail[..b].chars().count())
    };
    let mut spans = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let rest_starts = |p: &str| chars[i..].iter().take(p.len()).collect::<String>() == p;
        if rest_starts("[[")
            && let Some(end) = find(i + 2, "]]")
        {
            spans.push((i, end + 2));
            i = end + 2;
        } else if chars[i] == '['
            && let Some(close) = find(i + 1, "](")
            && !chars[i + 1..close].contains(&'\n')
            && let Some(end) = find(close + 2, ")")
        {
            spans.push((i, end + 1));
            i = end + 1;
        } else if rest_starts("http://") || rest_starts("https://") {
            let end = chars[i..]
                .iter()
                .position(|c| c.is_whitespace())
                .map_or(chars.len(), |p| i + p);
            spans.push((i, end));
            i = end;
        } else {
            i += 1;
        }
    }
    spans
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || "-_/.:".contains(c)
}

/// The words of `text` outside code, with whether each sits in link syntax.
fn words(text: &str) -> Vec<(String, bool)> {
    let chars = without_code(text);
    let spans = link_spans(&chars);
    let in_link = |i: usize| spans.iter().any(|&(a, b)| i >= a && i < b);
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !is_word_char(chars[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && is_word_char(chars[i]) {
            i += 1;
        }
        let word: String = chars[start..i].iter().collect();
        let word = word.trim_end_matches(['.', ':']).to_string();
        out.push((word, in_link(start)));
    }
    out
}

/// IDs in `text` that are not part of a link, as written (`br-9dcz` or `9dcz`), once each. A task
/// ID of any of the `prefixes` always counts; a bare 4-character ID only when `exists` says a
/// ticket has it. An ID linked anywhere in the reply counts as linked everywhere.
pub fn unlinked_ids(text: &str, prefixes: &[&str], exists: impl Fn(&str) -> bool) -> Vec<String> {
    let tail = |w: &str| -> Option<String> {
        for prefix in prefixes {
            if let Some(rest) = w.strip_prefix(prefix).and_then(|r| r.strip_prefix('-')) {
                return (is_bare_id(rest) && !is_reserved_id(rest)).then(|| rest.to_string());
            }
        }
        (is_bare_id(w) && !is_reserved_id(w) && exists(w)).then(|| w.to_string())
    };
    let mut linked = BTreeSet::new();
    let mut found: Vec<(String, String)> = Vec::new();
    for (word, in_link) in words(text) {
        let Some(id) = tail(&word) else { continue };
        if in_link {
            linked.insert(id);
        } else if !found.iter().any(|(w, _)| *w == word) {
            found.push((word, id));
        }
    }
    found
        .into_iter()
        .filter(|(_, id)| !linked.contains(id))
        .map(|(w, _)| w)
        .collect()
}

/// A project whose tickets an ID may belong to: its name, id prefix and the ticket IDs it has.
pub struct Owner {
    pub project: String,
    pub prefix: String,
    pub ids: BTreeSet<String>,
}

/// The project that owns `id` (as written) and the qualified id to link. A qualified id names its
/// owner by prefix; a bare one belongs to the first project that has it, the session's `own`
/// first, and falls back to `own`. `None` for a prefix no project has.
pub fn owning<'a>(id: &str, own: &'a Owner, others: &'a [Owner]) -> Option<(&'a str, String)> {
    let all = || std::iter::once(own).chain(others);
    if let Some((prefix, bare)) = id.split_once('-') {
        let o = all().find(|o| o.prefix == prefix)?;
        return Some((&o.project, format!("{prefix}-{bare}")));
    }
    let o = all().find(|o| o.ids.contains(id)).unwrap_or(own);
    Some((&o.project, format!("{}-{id}", o.prefix)))
}

/// The ticket IDs on disk: the end of each file name in `docs/tickets/{open,resolved}`.
pub fn ticket_ids(repo: &Path) -> BTreeSet<String> {
    ["open", "resolved"]
        .iter()
        .filter_map(|d| std::fs::read_dir(repo.join("docs/tickets").join(d)).ok())
        .flatten()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter_map(|n| {
            let stem = n.strip_suffix(".md")?;
            let id = stem.rsplit('-').next()?;
            is_bare_id(id).then(|| id.to_string())
        })
        .collect()
}

/// The one reminder. `links` pairs each ID (as written) with its URL.
pub fn reason(links: &[(String, String)]) -> String {
    let ids: Vec<&str> = links.iter().map(|(i, _)| i.as_str()).collect();
    let urls: Vec<String> = links.iter().map(|(i, u)| format!("{i} {u}")).collect();
    format!(
        "Your reply names these without a link: {}. Add each as a Markdown link, e.g. [{}]({}). \
         Links: {}. Ignore any that are not tickets.",
        ids.join(", "),
        links[0].0,
        links[0].1,
        urls.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn real(id: &str) -> bool {
        id == "9dcz" || id == "rest"
    }

    #[test]
    fn the_reserved_placeholder_is_never_flagged() {
        assert!(unlinked_ids("br-xxxx and xxxx", &["br"], |_| true).is_empty());
    }

    #[test]
    fn task_ids_always_count() {
        assert_eq!(
            unlinked_ids("see br-ab3k now", &["br"], |_| false),
            ["br-ab3k"]
        );
    }

    #[test]
    fn a_word_that_is_not_a_ticket_is_not_flagged() {
        assert!(unlinked_ids("then we wait, easy", &["br"], real).is_empty());
    }

    #[test]
    fn an_existing_ticket_is_flagged() {
        assert_eq!(unlinked_ids("Filed (9dcz).", &["br"], real), ["9dcz"]);
        assert_eq!(unlinked_ids("the rest.", &["br"], real), ["rest"]);
    }

    #[test]
    fn linked_code_path_and_longer_words_are_skipped() {
        let t = "[9dcz](http://x/9dcz) [[tickets/foo-9dcz|br-ab3k]] `9dcz`\n```\n9dcz br-ab3k\n```\n\
                 docs/tickets/open/foo-9dcz.md http://h/p/tickets/br-ab3k 9dczx";
        assert!(unlinked_ids(t, &["br"], real).is_empty());
    }

    #[test]
    fn linked_anywhere_counts_as_linked() {
        assert!(unlinked_ids("9dcz is [9dcz](http://x/9dcz)", &["br"], real).is_empty());
    }

    #[test]
    fn other_prefixes_and_capitals_are_ignored() {
        assert!(unlinked_ids("pre-test BR-ab3k", &["br"], |_| false).is_empty());
    }

    #[test]
    fn ids_come_from_ticket_file_names() {
        let d = tempfile::tempdir().expect("tmp");
        std::fs::create_dir_all(d.path().join("docs/tickets/open")).expect("mk");
        std::fs::write(d.path().join("docs/tickets/open/a-thing-9dcz.md"), "").expect("w");
        let ids = ticket_ids(d.path());
        assert!(ids.contains("9dcz") && ids.len() == 1);
    }

    fn owner(project: &str, prefix: &str, ids: &[&str]) -> Owner {
        Owner {
            project: project.into(),
            prefix: prefix.into(),
            ids: ids.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn a_qualified_id_of_another_project_links_there() {
        let own = owner("dalek-home", "da", &[]);
        let others = [owner("bridle", "br", &["gb97"])];
        assert_eq!(
            owning("br-gb97", &own, &others),
            Some(("bridle", "br-gb97".to_string()))
        );
        assert_eq!(owning("zz-gb97", &own, &others), None);
    }

    #[test]
    fn a_bare_id_links_to_the_project_that_has_it() {
        let own = owner("dalek-home", "da", &["aaaa"]);
        let others = [owner("bridle", "br", &["gb97"])];
        assert_eq!(
            owning("gb97", &own, &others),
            Some(("bridle", "br-gb97".to_string()))
        );
        assert_eq!(
            owning("aaaa", &own, &others),
            Some(("dalek-home", "da-aaaa".to_string()))
        );
        // Nowhere: the session's project.
        assert_eq!(
            owning("zzzz", &own, &others),
            Some(("dalek-home", "da-zzzz".to_string()))
        );
    }

    #[test]
    fn other_projects_prefixes_count_as_ids() {
        assert_eq!(
            unlinked_ids("see br-ab3k and da-cd4e", &["da", "br"], |_| false),
            ["br-ab3k", "da-cd4e"]
        );
    }

    #[test]
    fn reason_lists_ids_and_links() {
        let r = reason(&[("9dcz".into(), "http://u/9dcz".into())]);
        assert!(r.contains("9dcz") && r.contains("http://u/9dcz"));
    }
}
