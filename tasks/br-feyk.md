+++
id = "br-feyk"
title = "Rename the triage role to aide (r8kv)"
kind = "chore"
state = "integrated"
created_at = "2026-10-03T22:52:27.602Z"
updated_at = "2026-10-03T23:21:10.973255Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/aide-rename"
commit = "6164a550e992fd8b1c8c082c6402364634ebfcbc"
summary = "Renamed the triage role to aide: workflow/base/roles/triage.md -> aide.md, principal external:aide, bridle session aide / prime aide, [aide] credentials, [sessions.aide], session identity and wake targets, tests, design docs, orchestrator/advisor role text, CHANGELOG. 'triage' as a verb, PM triage and the planned bridle-triage skill left alone. No alias."
+++

original id: r8kv
Ticket: docs/tickets/open/seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv.md, section "Renamed: triage is now aide". The human, 2026-10-03 (via the advisor): "let's rename triage to aid. I think that's a better name. A-I-D-E. Sounds good." Approved; do it now, before anyone mints a triage token (none exists yet).
Goal: rename the ROLE introduced by br-r8kv (ec0847e) from `triage` to `aide` everywhere: workflow/base/roles/triage.md -> aide.md; principal `external:triage` -> `external:aide`; `bridle session triage` -> `bridle session aide`; `bridle prime triage` -> `bridle prime aide`; credentials section [triage] -> [aide]; session identifiers, wake targets, the [sessions] config and warnings routed "to triage" (br-jttf, 9f67724) -> aide; tests; docs (cli.md, roles-and-config.md, orchestrator-supervision.md, principals.md, orchestrator/advisor/product-manager role text, .bridle/roles/orchestrator.md, .bridle/config.toml), CHANGELOG.
CAREFUL: "triage" as a verb or for the product manager's triage of open tasks, ticket submissions "for triage", and the planned bridle-triage skill (skills.md) are NOT the role: leave those. Rename only the role and its identifiers.
No compatibility alias needed (the role landed today and nobody has used it). Acceptance: just check passes; `grep -rn "external:triage\|session triage\|roles/triage"` finds nothing. Model: Sonnet (Haiku is fine if mechanical). Migration: none (no project has a triage token or config yet).

## Thread

### note · agent:pm-1 · 2026-10-03T22:52:59.979Z
Planned and queued as tier 1 (human-approved). May touch files br-jttf's worker (session-context) also touches (warnings routed to the role): watch for merge conflicts, and rebase on whichever lands first.

### note · agent:aide-rename · 2026-10-03T23:19:50.572Z
done: triage role renamed to aide, just check green (1112 tests); b9e6588

### note · agent:manager-2 · 2026-10-03T23:19:56.616Z
integrated: 6164a550e992fd8b1c8c082c6402364634ebfcbc (branch bridle/aide-rename)

### note · agent:manager-2 · 2026-10-03T23:21:10.973Z
cleanup: removed agent aide-rename, branch bridle/aide-rename
