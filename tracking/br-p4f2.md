+++
id = "br-p4f2"
title = "bridle migrate 0006 refuses every project that works on main (protected-branch check ignores [branches] integration), even with nothing to change"
kind = "bug"
state = "integrated"
created_at = "2026-10-16T01:52:45.667Z"
updated_at = "2026-10-17T07:02:35.323145Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:p4f2",
]
branch = "bridle/p4f2"
commit = "c9dfa15ac2d8e789ac29fbf0949625ac387cbbc6"
summary = "migrate.rs: refuse_on_protected_branch now allows a branch the project declares as [branches] integration (integration_set, and not also the release branch); main/dev/release otherwise still refuse. The guard moved from the top of 0003/0004/0005/0006/0007 and the links migration to just before each writes, so a migration with nothing to change never refuses. The rename migration had no guard and is unchanged. Test added; CHANGELOG line. No design docs mention the guard."
ticket = "p4f2"
+++

docs/tickets/open/bridle-migrate-0006-refuses-every-project-that-works-on-main-p4f2.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T02:06:31.627Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:16.901Z
Brief (pm-1): the ask is the brief. Model: Sonnet. File: crates/bridle/src/migrate.rs (refuse_on_protected_branch ~734; find every caller, "check the other migrations that call the same guard"). Do: (1) the guard allows the project's own integration branch ([branches] integration) and still refuses main/dev/release when that is not the project's integration branch; (2) a migration with nothing to change does not refuse at all (check before the guard). Tests: a project with integration = main and tickets to backfill proceeds; a project with nothing to change on main does not refuse; bridle's own rules unchanged; every other migration using the guard covered. Acceptance: just check. Rule existing-projects still holds: the guard stays for any branch that is not the project's declared integration branch. Out of scope: running migrations on the human's projects.

### note · agent:p4f2 · 2026-10-17T06:44:21.318Z
built: 629c71c4 (branch bridle/p4f2)

### note · agent:p4f2 · 2026-10-17T06:44:21.359Z
done: migrate guard honours [branches] integration and is skipped when nothing to change; just check exit 0, 1793 tests passed, sha 629c71c4

### note · agent:manager-2 · 2026-10-17T07:02:19.229Z
integrated: c9dfa15ac2d8e789ac29fbf0949625ac387cbbc6 (branch bridle/p4f2)

### note · agent:manager-2 · 2026-10-17T07:02:35.323Z
cleanup: removed agent p4f2, branch bridle/p4f2
