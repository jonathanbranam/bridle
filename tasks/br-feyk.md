+++
id = "br-feyk"
title = "Rename the triage role to aide (r8kv)"
kind = "chore"
state = "planned"
created_at = "2026-10-03T22:52:27.602Z"
updated_at = "2026-10-03T22:52:59.979914Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

original id: r8kv
Ticket: docs/tickets/open/seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv.md, section "Renamed: triage is now aide". The human, 2026-10-03 (via the advisor): "let's rename triage to aid. I think that's a better name. A-I-D-E. Sounds good." Approved; do it now, before anyone mints a triage token (none exists yet).
Goal: rename the ROLE introduced by br-r8kv (ec0847e) from `triage` to `aide` everywhere: workflow/base/roles/triage.md -> aide.md; principal `external:triage` -> `external:aide`; `bridle session triage` -> `bridle session aide`; `bridle prime triage` -> `bridle prime aide`; credentials section [triage] -> [aide]; session identifiers, wake targets, the [sessions] config and warnings routed "to triage" (br-jttf, 9f67724) -> aide; tests; docs (cli.md, roles-and-config.md, orchestrator-supervision.md, principals.md, orchestrator/advisor/product-manager role text, .bridle/roles/orchestrator.md, .bridle/config.toml), CHANGELOG.
CAREFUL: "triage" as a verb or for the product manager's triage of open tasks, ticket submissions "for triage", and the planned bridle-triage skill (skills.md) are NOT the role: leave those. Rename only the role and its identifiers.
No compatibility alias needed (the role landed today and nobody has used it). Acceptance: just check passes; `grep -rn "external:triage\|session triage\|roles/triage"` finds nothing. Model: Sonnet (Haiku is fine if mechanical). Migration: none (no project has a triage token or config yet).

## Thread

### note · agent:pm-1 · 2026-10-03T22:52:59.979Z
Planned and queued as tier 1 (human-approved). May touch files br-jttf's worker (session-context) also touches (warnings routed to the role): watch for merge conflicts, and rebase on whichever lands first.
