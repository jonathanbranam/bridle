+++
id = "br-p4f2"
title = "bridle migrate 0006 refuses every project that works on main (protected-branch check ignores [branches] integration), even with nothing to change"
kind = "bug"
state = "open"
created_at = "2026-10-16T01:52:45.667Z"
updated_at = "2026-10-17T02:07:16.901872Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "p4f2"
+++

docs/tickets/open/bridle-migrate-0006-refuses-every-project-that-works-on-main-p4f2.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T02:06:31.627Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:16.901Z
Brief (pm-1): the ask is the brief. Model: Sonnet. File: crates/bridle/src/migrate.rs (refuse_on_protected_branch ~734; find every caller, "check the other migrations that call the same guard"). Do: (1) the guard allows the project's own integration branch ([branches] integration) and still refuses main/dev/release when that is not the project's integration branch; (2) a migration with nothing to change does not refuse at all (check before the guard). Tests: a project with integration = main and tickets to backfill proceeds; a project with nothing to change on main does not refuse; bridle's own rules unchanged; every other migration using the guard covered. Acceptance: just check. Rule existing-projects still holds: the guard stays for any branch that is not the project's declared integration branch. Out of scope: running migrations on the human's projects.
