+++
id = "br-1ce4"
title = "Explore kind: locked base rule and prime paragraph"
kind = "feature"
state = "planned"
created_at = "2026-09-29T04:01:16.352Z"
updated_at = "2026-09-29T04:01:18.507871Z"
+++

Goal: deliver the explore task kind's rules (docs/design/explorations.md). Kind explore exists in bridle-api types; nothing tells agents how to treat it yet.

Do:
1. Add a locked base rule workflow/base/rules/explorations.md (follow the frontmatter of the other rules there and how 'locked' is expressed; see docs/design/workflow-layers.md and crates/bridle-daemon/src/rules.rs) with the text from explorations.md: material under design/explore/, or marked exploratory: true, deliberately contradicts the current design; it is not a defect, not stale, not to be reconciled or reported. As a base rule `bridle prime` already prints it to every role; verify with a test. If sync rendering already supports path-scoped rules, also scope a copy to design/explore/**; if not, skip and say so in the task thread.
2. `bridle prime worker` (crates/bridle/src/prime.rs): when the task is kind explore, prepend the exploring agent's paragraph: you are exploring an alternative; contradicting architecture, goals and specs is intended; don't bring the code back into line and don't flag the divergence; exploration code stays on explore/<id>, only the findings doc merges, into design/explore/<id>/. Find how prime learns the task (a --task ID flag or the caller's claim; pick the least invasive).
3. Add a Built section to docs/design/explorations.md.

Acceptance: `just check` passes; prime tests with an explore task and a feature task (paragraph absent). Model: Sonnet.

Out of scope: frontmatter checking, `bridle explore` commands, exemptions from gates (not built yet). Independent of the parser tasks; runs in parallel with them.
