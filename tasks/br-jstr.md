+++
id = "br-jstr"
title = "Recipient hint: suggest external:advisor/<name> for a bare advisor/<name>"
kind = "bug"
state = "planned"
created_at = "2026-10-11T03:48:54.303Z"
updated_at = "2026-10-11T03:49:12.403995Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "low"
priority_at = "2026-10-11T03:48:55.855541Z"
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
