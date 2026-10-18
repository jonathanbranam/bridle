//! Ticket links from the frontmatter (br-3v75, 22ab section 7). `blocked_by`, `parent` and
//! `related` in a ticket file are the one source of truth; the daemon indexes them into the
//! `edges` table so readiness keeps working. Only a ticket's own edges are touched: its
//! `blocks` edges into it, the `parent` edge into it from its parent, and its `related` edges
//! out of it. Other edge kinds (`duplicates`, `supersedes`, `discovered-from`) and edges on
//! rows with no ticket file are left alone.

use std::collections::HashSet;
use std::path::Path;

use bridle_api::types::EdgeKind;

use crate::tasks::TicketManager;

/// One ticket's links, as written in its frontmatter.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TicketLinks {
    pub id: String,
    pub blocked_by: Vec<String>,
    pub parent: Option<String>,
    pub related: Vec<String>,
    /// The ticket version (1 when absent); 2 tickets have stages (br-9mqh).
    pub version: u32,
    /// Every stage written and every gate signed (`prepared: true`, br-ussh): the ticket opens.
    pub prepared: bool,
}

/// The links in a ticket's text; `None` without a frontmatter block or an `id`. The old field
/// names (`needs`, `see`) are read until the migration has run everywhere.
pub fn parse_links(text: &str) -> Option<TicketLinks> {
    let mut lines = text.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut links = TicketLinks {
        version: 1,
        ..Default::default()
    };
    let (mut old_blocked, mut old_related) = (Vec::new(), Vec::new());
    let (mut has_blocked, mut has_related) = (false, false);
    for line in lines {
        if line == "---" {
            break;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key {
            "id" => links.id = unquote(value),
            "blocked_by" => {
                links.blocked_by = list(value);
                has_blocked = true;
            }
            "needs" => old_blocked = list(value),
            "related" => {
                links.related = list(value);
                has_related = true;
            }
            "see" => old_related = list(value),
            "version" => links.version = value.parse().unwrap_or(1),
            "prepared" => links.prepared = value == "true",
            "parent" => links.parent = Some(unquote(value)).filter(|p| !p.is_empty()),
            _ => {}
        }
    }
    if !has_blocked {
        links.blocked_by = old_blocked;
    }
    if !has_related {
        links.related = old_related;
    }
    (!links.id.is_empty()).then_some(links)
}

fn unquote(v: &str) -> String {
    v.trim().trim_matches(|c| c == '"' || c == '\'').to_string()
}

/// A flow list, `[a, b]`.
fn list(v: &str) -> Vec<String> {
    v.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(unquote)
        .filter(|s| !s.is_empty())
        .collect()
}

/// Every ticket in `docs/tickets/open` and `resolved` under `root` (a `docs/tickets` dir).
pub fn read_dir_links(tickets: &Path) -> Vec<TicketLinks> {
    let mut out = Vec::new();
    for sub in ["open", "resolved"] {
        let Ok(rd) = std::fs::read_dir(tickets.join(sub)) else {
            continue;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|e| e == "md")
                && let Ok(text) = std::fs::read_to_string(&p)
                && let Some(l) = parse_links(&text)
            {
                out.push(l);
            }
        }
    }
    out
}

impl TicketManager {
    /// Makes the edges of every ticket that has a row match its frontmatter. A link to an id
    /// with no row here (another project's ticket, a typo) makes no edge. Returns how many
    /// edges were added and removed.
    pub async fn sync_ticket_links(&self, tickets: &Path) -> (usize, usize) {
        let dir = tickets.to_path_buf();
        let links = tokio::task::spawn_blocking(move || read_dir_links(&dir))
            .await
            .unwrap_or_default();
        let (mut added, mut removed) = (0, 0);
        for t in links {
            let Some(row) = self.get_task(&t.id) else {
                continue;
            };
            // Ready means prepared: a version 2 ticket opens itself, as by bridle.
            if t.version >= 2 && t.prepared && row.state == bridle_api::TicketState::Pending {
                let bridle = "bridle".to_string();
                if self.ready_task(&t.id, &bridle).await.is_ok() {
                    let _ = self
                        .note_task(
                            &t.id,
                            &bridle,
                            "prepared: every stage is written and every gate signed; ready",
                        )
                        .await;
                }
            }
            // (from, to, kind) the file asks for.
            let mut want: HashSet<(String, String, &'static str)> = HashSet::new();
            for b in &t.blocked_by {
                want.insert((b.clone(), t.id.clone(), EdgeKind::Blocks.as_str()));
            }
            if let Some(p) = &t.parent {
                want.insert((p.clone(), t.id.clone(), EdgeKind::Parent.as_str()));
            }
            for r in &t.related {
                want.insert((t.id.clone(), r.clone(), EdgeKind::Related.as_str()));
            }
            let have: Vec<_> = self
                .list_edges()
                .into_iter()
                .filter(|e| match e.kind {
                    EdgeKind::Blocks | EdgeKind::Parent => e.to == t.id,
                    EdgeKind::Related => e.from == t.id,
                    _ => false,
                })
                .collect();
            for e in &have {
                if !want.contains(&(e.from.clone(), e.to.clone(), e.kind.as_str()))
                    && self.remove_edge(&e.from, &e.to, e.kind).await.is_ok()
                {
                    removed += 1;
                }
            }
            for (from, to, kind) in want {
                if have
                    .iter()
                    .any(|e| e.from == from && e.to == to && e.kind.as_str() == kind)
                {
                    continue;
                }
                let kind = match kind {
                    "blocks" => EdgeKind::Blocks,
                    "parent" => EdgeKind::Parent,
                    _ => EdgeKind::Related,
                };
                // No row for the other end, a self-link: not an edge.
                if self.add_edge(&from, &to, kind).await.is_ok() {
                    added += 1;
                }
            }
        }
        (added, removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_new_and_old_field_names() {
        let l = parse_links(
            "---\nid: br-aaaa\nblocked_by: [br-bbbb, \"br-cccc\"]\nparent: br-dddd\nrelated: []\n---\n\nbody\nblocked_by: [x]\n",
        )
        .expect("links");
        assert_eq!(l.id, "br-aaaa");
        assert_eq!(l.blocked_by, ["br-bbbb", "br-cccc"]);
        assert_eq!(l.parent.as_deref(), Some("br-dddd"));
        assert!(l.related.is_empty());
        let old = parse_links("---\nid: aaaa\nneeds: [bbbb]\nsee: [cccc]\n---\n").expect("old");
        assert_eq!(old.blocked_by, ["bbbb"]);
        assert_eq!(old.related, ["cccc"]);
        assert!(parse_links("no frontmatter").is_none());
        let v2 = parse_links("---\nid: br-aaaa\nversion: 2\nprepared: true\n---\n").unwrap();
        assert!(v2.prepared && v2.version == 2);
        assert!(!old.prepared);
    }
}
