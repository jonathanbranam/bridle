+++
id = "br-jstr"
title = "Recipient hint: suggest external:advisor/<name> for a bare advisor/<name>"
kind = "bug"
state = "integrated"
created_at = "2026-10-11T03:48:54.303Z"
updated_at = "2026-10-11T11:58:52.178265Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wjstr",
]
priority = "low"
priority_at = "2026-10-11T03:48:55.855541Z"
branch = "bridle/wjstr"
commit = "c222400f11b55d63874db9517e21fd2eb1f814a7"
summary = "Moot after br-3k7d: canonical_target now resolves advisor, advisor/<name>, aide, orchestrator (with optional @machine) to external:<name>, so the 'did you mean external:...' hint in resolve_targets could never fire. Removed is_known_external_principal, the hint and its test (server.rs); CHANGELOG line added. Unknown names after advisor/ resolve as external, so nothing is left to hint."
ticket = "jstr"
+++

Ticket: docs/tickets/open/recipient-hint-suggest-external-advisor-name-for-a-bare-advi-jstr.md (read "Fix"; it is the spec). Low priority, small.

Goal: `bridle send --to advisor/product-manager` (and `schedule add --to`) fails with not_found and no hint, while `--to aide` hints "did you mean external:aide?". Fix is_known_external_principal in crates/bridle-daemon/src/server.rs (~line 1496) to compare only the role part: strip a /<name> suffix and an @<machine> suffix before matching, and keep the full typed name in the hint (external:advisor/product-manager).

Order: runs after br-3k7d (same resolver area, server.rs). Start only once br-3k7d is integrated. If 3k7d already resolves some names outright, keep the hint for what it does not resolve (an unknown name after advisor/), or drop it if nothing is left; say which on the task thread.

Files: crates/bridle-daemon/src/server.rs, CHANGELOG.
Acceptance: just check passes; extend is_known_external_principal_recognizes_valid_names with advisor/product-manager and aide@nuc.
Migration: none. Model: Haiku. Out of scope: resolving unknown names, any other recipient rules.

## Thread

### note · external:advisor/product-manager · 2026-10-11T03:48:55.855Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-11T11:05:11.673Z
advisor (product-manager): check first whether this is already moot: br-3k7d (integrated 2026-10-11) made a bare known external name (advisor, advisor/<name>, aide, orchestrator) resolve as external:<name> in send and schedule add. If advisor/<name> now resolves, the hint is only needed where resolution still fails; say so and keep the change minimal (or report it done-as-moot).

### note · agent:wjstr · 2026-10-11T11:28:20.428Z
done: hint removed as moot after br-3k7d (no code left to fix); check exit 0, 1513 tests; 21656197

### note · agent:wjstr · 2026-10-11T11:28:23.246Z
Dropped the hint rather than fixing it: br-3k7d resolves these names outright, nothing left to hint. just check exit 0, 1513 tests passed, sha 21656197.

### note · agent:manager-2 · 2026-10-11T11:28:32.279Z
main moved. Merge main into your branch, re-run just check, and message me the new sha with exit status and test count.

### note · agent:wjstr · 2026-10-11T11:46:26.653Z
merged main; just check exit 0, 1513 tests passed (6 skipped); sha a07ec80e

### note · agent:manager-2 · 2026-10-11T11:52:31.037Z
main moved again (gd43 landed). Merge main, re-run just check, and message me the sha.

### note · agent:wjstr · 2026-10-11T11:58:31.975Z
merged main; just check exit 0, 1516 tests passed (6 skipped); sha ffdb2fdf

### note · agent:manager-2 · 2026-10-11T11:58:35.983Z
integrated: c222400f11b55d63874db9517e21fd2eb1f814a7 (branch bridle/wjstr)

### note · agent:manager-2 · 2026-10-11T11:58:52.178Z
cleanup: removed agent wjstr, branch bridle/wjstr
