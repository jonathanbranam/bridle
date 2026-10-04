+++
id = "br-m7mp"
title = "Interactive roles (aide, advisor, orchestrator) get their resolved rules, project rules included, at startup"
kind = "feature"
state = "planned"
created_at = "2026-10-04T13:34:26.873Z"
updated_at = "2026-10-04T13:40:39.123586Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: m7mp
Feature. Ticket: docs/tickets/open/interactive-roles-aide-advisor-orchestrator-get-their-resolv-m7mp.md (read it, and docs/tickets/open/role-and-rule-files-misstate-how-rules-reach-agents-orchestr-98xt.md section on 'The orchestrator and advisor get none'). The human, 2026-10-04: 'approve m7mp and kae5 afterwards' (after br-fc9a). Daemon-spawned agents get their resolved rules (project .bridle/rules/ included) in the system prompt since br-2242; the interactive roles (orchestrator, advisor, aide) start from 'bridle prime <role>' / 'bridle session', which print no rules. Goal: those roles get their resolved rules, project rules included, at startup. First verify what prime and session inject today (crates/bridle/src/prime.rs, session.rs, the rules resolver the daemon uses for spawn); then reuse the same resolver so 'bridle prime <role>' (and so the session's opening prompt) includes the role's resolved rules for the current project, honoring each rule's roles: frontmatter. Say in the docs what a project does to add a rule to specific roles (the NUC's notes project example: .bridle/rules/worked-on-log.md with roles: [advisor, orchestrator, manager, worker]). Also check bridle prime manager. Files: prime.rs, session.rs (br-fc9a also edits session.rs: land after it), docs (roles-and-config.md, workflow-layers.md), CHANGELOG. Tests: prime advisor includes a project rule listing advisor in roles and omits one that doesn't. Acceptance: just check passes. Model: Sonnet. Migration: none (projects' rules unchanged; they start being delivered). Out of scope: a man page; kae5.

## Thread

### note · external:orchestrator · 2026-10-04T13:39:53.771Z
The human, 2026-10-04: 'approve m7mp and kae5 afterwards' (after br-fc9a).
