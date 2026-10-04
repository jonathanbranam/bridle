+++
id = "br-m7mp"
title = "Interactive roles (aide, advisor, orchestrator) get their resolved rules, project rules included, at startup"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T13:34:26.873Z"
updated_at = "2026-10-04T17:19:43.416381Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/prime-rules"
commit = "5b6f1e5bbbd2f2a92f723a4e7782e86bb1ce57a9"
summary = "bridle prime orchestrator|advisor|aide|prototyper|document-reviewer now ends with a '## Rules' section from Config::role_rules_text (the daemon's spawn resolver), so project .bridle/rules/ and each rule's roles: are honored; bridle session's opening prompt gets it via prime. Before: prime printed role file + project addendum only; there is no 'prime manager' (manager is daemon-spawned and already had rules in its system prompt). Code: commands/orchestrator.rs; test: tests/prime_rules_test.rs; docs: workflow-layers.md (how a project adds a rule to roles), roles-and-config.md, CHANGELOG."
+++

original id: m7mp
Feature. Ticket: docs/tickets/open/interactive-roles-aide-advisor-orchestrator-get-their-resolv-m7mp.md (read it, and docs/tickets/open/role-and-rule-files-misstate-how-rules-reach-agents-orchestr-98xt.md section on 'The orchestrator and advisor get none'). The human, 2026-10-04: 'approve m7mp and kae5 afterwards' (after br-fc9a). Daemon-spawned agents get their resolved rules (project .bridle/rules/ included) in the system prompt since br-2242; the interactive roles (orchestrator, advisor, aide) start from 'bridle prime <role>' / 'bridle session', which print no rules. Goal: those roles get their resolved rules, project rules included, at startup. First verify what prime and session inject today (crates/bridle/src/prime.rs, session.rs, the rules resolver the daemon uses for spawn); then reuse the same resolver so 'bridle prime <role>' (and so the session's opening prompt) includes the role's resolved rules for the current project, honoring each rule's roles: frontmatter. Say in the docs what a project does to add a rule to specific roles (the NUC's notes project example: .bridle/rules/worked-on-log.md with roles: [advisor, orchestrator, manager, worker]). Also check bridle prime manager. Files: prime.rs, session.rs (br-fc9a also edits session.rs: land after it), docs (roles-and-config.md, workflow-layers.md), CHANGELOG. Tests: prime advisor includes a project rule listing advisor in roles and omits one that doesn't. Acceptance: just check passes. Model: Sonnet. Migration: none (projects' rules unchanged; they start being delivered). Out of scope: a man page; kae5.

## Thread

### note · external:orchestrator · 2026-10-04T13:39:53.771Z
The human, 2026-10-04: 'approve m7mp and kae5 afterwards' (after br-fc9a).

### note · agent:prime-rules · 2026-10-04T17:16:23.791Z
done: prime orchestrator|advisor|aide|prototyper|document-reviewer now print the role's resolved rules (project rules, roles: honored) via the daemon's role_rules_text; there is no 'prime manager' (daemon-spawned, already had them); docs+CHANGELOG+test added; just check green (1156 tests; one earlier run had 2 load-timeouts in unrelated tests, passed on rerun); b3d6882f9a742930a53b03a0da5908bbe8d7206c

### note · agent:manager-2 · 2026-10-04T17:16:30.116Z
integrated: 5b6f1e5bbbd2f2a92f723a4e7782e86bb1ce57a9 (branch bridle/prime-rules)

### note · agent:manager-2 · 2026-10-04T17:19:43.416Z
cleanup: removed agent prime-rules, branch bridle/prime-rules
