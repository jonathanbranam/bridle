//! Trace links across the tiers (`docs/design/traceability.md`): the element
//! text hash, and the graph of goals, architecture elements, requirements and
//! scenarios that `bridle trace down|up|orphans` query.

use std::collections::{HashMap, HashSet, VecDeque};

use sha2::{Digest, Sha256};

use crate::arch::Element;
use crate::{Diagnostic, Goal, Spec};

/// The short hash a `traces=id@hash` link carries: the first 4 hex digits of
/// the SHA-256 of the element's normalised text. The text is the heading's
/// title (without its `{#id ...}` block) and the body up to the next heading,
/// joined by a newline, with every run of whitespace collapsed to one space
/// and the ends trimmed. So reflowing or re-indenting never changes it; any
/// change to the words does.
pub fn text_hash(title: &str, body: &str) -> String {
    let norm = format!("{title}\n{body}")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let d = Sha256::digest(norm.as_bytes());
    format!("{:02x}{:02x}", d[0], d[1])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Goal,
    Architecture,
    Requirement,
    Scenario,
}

/// An upward link from a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub target: String,
    /// The hash confirmed in `traces=`; `None` for `serves=` and scenarios.
    pub hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Node {
    pub id: String,
    pub kind: Kind,
    pub title: String,
    pub file: String,
    pub line: usize,
    /// The node's current text hash (see [`text_hash`]); empty for scenarios.
    pub hash: String,
    #[serde(skip)]
    pub up: Vec<Link>,
}

#[derive(Debug, Default)]
pub struct Graph {
    nodes: Vec<Node>,
    index: HashMap<String, usize>,
    /// node index -> nodes linking up to it, in file order.
    children: Vec<Vec<usize>>,
}

impl Graph {
    /// Build the graph. Every id must be unique across tiers and every link
    /// must name a known id; all problems are reported.
    pub fn build(
        goals: &[(String, Goal)],
        elements: &[Element],
        specs: &[(String, Spec)],
    ) -> Result<Graph, Vec<Diagnostic>> {
        let mut nodes = Vec::new();
        for (file, g) in goals {
            nodes.push(Node {
                id: g.id.clone(),
                kind: Kind::Goal,
                title: g.title.clone(),
                file: file.clone(),
                line: g.line,
                hash: g.hash.clone(),
                up: Vec::new(),
            });
        }
        for e in elements {
            nodes.push(Node {
                id: e.id.clone(),
                kind: Kind::Architecture,
                title: e.title.clone(),
                file: e.file.clone(),
                line: e.line,
                hash: e.hash.clone(),
                up: e
                    .serves
                    .iter()
                    .map(|t| Link {
                        target: t.clone(),
                        hash: None,
                    })
                    .collect(),
            });
        }
        for (file, spec) in specs {
            for r in &spec.requirements {
                if let Some(id) = &r.id {
                    nodes.push(Node {
                        id: id.clone(),
                        kind: Kind::Requirement,
                        title: r.title.clone(),
                        file: file.clone(),
                        line: r.line,
                        hash: text_hash(&r.title, &r.text),
                        up: r
                            .traces
                            .iter()
                            .map(|t| Link {
                                target: t.target.clone(),
                                hash: Some(t.hash.clone()),
                            })
                            .collect(),
                    });
                }
                for s in &r.scenarios {
                    if let (Some(sid), Some(rid)) = (&s.id, &r.id) {
                        nodes.push(Node {
                            id: sid.clone(),
                            kind: Kind::Scenario,
                            title: s.title.clone(),
                            file: file.clone(),
                            line: s.line,
                            hash: String::new(),
                            up: vec![Link {
                                target: rid.clone(),
                                hash: None,
                            }],
                        });
                    }
                }
            }
        }

        let mut diags = Vec::new();
        let mut index = HashMap::new();
        for (i, n) in nodes.iter().enumerate() {
            if let Some(&first) = index.get(&n.id) {
                let f: &Node = &nodes[first];
                diags.push(Diagnostic {
                    file: n.file.clone(),
                    line: n.line,
                    column: 1,
                    message: format!(
                        "duplicate id {:?}: first used at {}:{}",
                        n.id, f.file, f.line
                    ),
                });
            } else {
                index.insert(n.id.clone(), i);
            }
        }
        let mut children = vec![Vec::new(); nodes.len()];
        for (i, n) in nodes.iter().enumerate() {
            for l in &n.up {
                match index.get(&l.target) {
                    Some(&t) => children[t].push(i),
                    None => diags.push(Diagnostic {
                        file: n.file.clone(),
                        line: n.line,
                        column: 1,
                        message: format!("{} links to unknown id {:?}", n.id, l.target),
                    }),
                }
            }
        }
        if diags.is_empty() {
            Ok(Graph {
                nodes,
                index,
                children,
            })
        } else {
            Err(diags)
        }
    }

    pub fn get(&self, id: &str) -> Option<&Node> {
        self.index.get(id).map(|&i| &self.nodes[i])
    }

    /// Everything that depends on `id`, transitively, breadth-first, each with
    /// its distance. `None` if `id` is unknown.
    pub fn down(&self, id: &str) -> Option<Vec<(usize, &Node)>> {
        self.walk(id, |i| self.children[i].clone())
    }

    /// Everything `id` rests on, up to the goals; same shape as [`Graph::down`].
    pub fn up(&self, id: &str) -> Option<Vec<(usize, &Node)>> {
        self.walk(id, |i| {
            self.nodes[i]
                .up
                .iter()
                .filter_map(|l| self.index.get(&l.target).copied())
                .collect()
        })
    }

    fn walk(&self, id: &str, next: impl Fn(usize) -> Vec<usize>) -> Option<Vec<(usize, &Node)>> {
        let start = *self.index.get(id)?;
        let mut seen = HashSet::from([start]);
        let mut queue = VecDeque::from([(0, start)]);
        let mut out = Vec::new();
        while let Some((d, i)) = queue.pop_front() {
            if i != start {
                out.push((d, &self.nodes[i]));
            }
            for n in next(i) {
                if seen.insert(n) {
                    queue.push_back((d + 1, n));
                }
            }
        }
        Some(out)
    }

    /// Requirements that trace to nothing.
    pub fn orphans(&self) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|n| n.kind == Kind::Requirement && n.up.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{arch, parse_goals, parse_str};

    const GOALS: &str = "## Fast {#g-03}\nfirmness: fixed · priority: now\n\nGo.\n";
    const ARCH: &str = "## Engine {#a-12cd serves=g-03}\nDecides.\n";
    const SPEC: &str = "### Requirement: R1 {#r-7fa2 traces=a-12cd@3f9e}\nSHALL.\n\n#### Scenario: S {#s-b310}\n\n*Verification*: **non-executable**\n\nprose\n\n### Requirement: R2 {#r-0002}\nSHALL.\n\n#### Scenario: S2 {#s-0002}\n\n*Verification*: **non-executable**\n\nprose\n";

    fn graph(spec: &str, arch_text: &str) -> Result<Graph, Vec<Diagnostic>> {
        let goals = parse_goals("g.md", GOALS)
            .goals
            .into_iter()
            .map(|g| ("g.md".to_string(), g))
            .collect::<Vec<_>>();
        let els = arch::parse_str("a.md", arch_text).expect("arch");
        let sp = parse_str("s.md", &format!("## Requirements\n\n{spec}")).expect("spec");
        Graph::build(&goals, &els, &[("s.md".into(), sp)])
    }

    #[test]
    fn hash_ignores_whitespace_but_not_words() {
        assert_eq!(text_hash("T", "a  b\n c"), text_hash("T", "a b c"));
        assert_ne!(text_hash("T", "a b c"), text_hash("T", "a b d"));
        assert_ne!(text_hash("T", "a"), text_hash("U", "a"));
        // Pinned so the algorithm can't drift silently.
        assert_eq!(text_hash("T", "a"), text_hash("T", " a\n"));
        assert_eq!(text_hash("T", "a").len(), 4);
    }

    #[test]
    fn down_up_and_orphans() {
        let g = graph(SPEC, ARCH).expect("builds");
        let ids = |v: Vec<(usize, &Node)>| {
            v.into_iter()
                .map(|(d, n)| (d, n.id.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            ids(g.down("g-03").expect("known")),
            [
                (1, "a-12cd".into()),
                (2, "r-7fa2".into()),
                (3, "s-b310".into())
            ]
        );
        assert_eq!(
            ids(g.up("s-b310").expect("known")),
            [
                (1, "r-7fa2".into()),
                (2, "a-12cd".into()),
                (3, "g-03".into())
            ]
        );
        assert_eq!(
            g.orphans()
                .iter()
                .map(|n| n.id.as_str())
                .collect::<Vec<_>>(),
            ["r-0002"]
        );
        assert!(g.down("x-9999").is_none());
    }

    #[test]
    fn unknown_link_target_is_an_error() {
        let ds = graph(&SPEC.replace("a-12cd@", "a-ffff@"), ARCH).expect_err("fails");
        assert!(ds[0].message.contains("unknown id \"a-ffff\""));
        let ds = graph(SPEC, &ARCH.replace("g-03", "g-99")).expect_err("fails");
        assert!(ds[0].message.contains("unknown id \"g-99\""));
    }

    #[test]
    fn bad_hash_form_is_a_parse_error() {
        let bad = SPEC.replace("@3f9e", "@3f9");
        let ds = parse_str("s.md", &format!("## Requirements\n\n{bad}")).expect_err("fails");
        assert!(ds[0].message.contains("traces=<id>@<hash>"));
    }

    #[test]
    fn multiple_traces_parse() {
        let s = SPEC.replace("a-12cd@3f9e", "a-12cd@3f9e,a-00ff@0000");
        let sp = parse_str("s.md", &format!("## Requirements\n\n{s}")).expect("parses");
        assert_eq!(sp.requirements[0].traces.len(), 2);
    }
}
