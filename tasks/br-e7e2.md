+++
id = "br-e7e2"
title = "Migration: backfill ticket kind and two-way task links in every bridle project (v3dk slice B)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T19:46:02.983Z"
updated_at = "2026-10-11T13:31:31.825188Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
size = "S"
+++

Ticket: docs/tickets/open/tickets-get-a-kind-kinds-are-editable-tickets-and-tasks-link-v3dk.md, section 'Migration'. REFRESHED 2026-10-11 against the everything-is-a-ticket epic (22ab), per the PdM thread: the human approved this task and the old parked terms are void. Starts only after br-res3 is integrated (both rewrite ticket files; avoid a merge conflict like 3v75's).

What 22ab already did, so DROP it: field renames (migration 0002-standard-ticket-fields), bug tasks that are fixes (0003), links in frontmatter and the tasks:/ticket pair going away (0004, step 3), a ticket for every active task (0005, step 4/bpku). Two-way task links are gone; do not rebuild them. Code: crates/bridle/src/migrate.rs (the MIGRATIONS list; ids append-only; follow the latest migration as the model), crates/bridle/src/ticket.rs, docs/design/migrations.md.

What is left: 'bridle ticket check' still warns 'missing kind' on a few hundred tickets (docs/tickets/open and resolved, and docs/spikes). Goal: one migration (next id after 0005, e.g. 0006-backfill-ticket-kind) that adds 'kind:' to every ticket or spike with a bridle frontmatter that lacks it: from its row or task if one has a kind (ask the daemon if up, else read the state branch; no daemon must not fail it), else a plain recorded default ('chore'; the human edits with 'ticket set'). Idempotent; honours --dry-run (report what it would change); refuses, as the others do, on uncommitted changes in the files; never touches main or dev of an existing project that is not on a trial branch (the guard from 0002 applies). Runs automatically with the other migrations. Then flip the missing-kind warning to an error in 'bridle ticket check'. Run it on this repo as a separate commit so 'bridle ticket check' is clean (note: one resolved ticket, base-rule-the-shell-is-zsh ... urdm, also reports 'missing resolved'; fix its frontmatter in the same pass with the date from git log).
Tests: tickets with and without a row/task get the right kind; second run changes nothing; dry-run writes nothing; check errors on a missing kind afterward; daemon down falls back.
Docs: migrations.md (list it), CHANGELOG. Say in the done note how many of the warnings it cleared. Acceptance: just check passes and bridle ticket check on this repo reports nothing. Model: Sonnet.
Migration plan: this is it. Out of scope: changing the kind of tickets that have one.

## Thread

### note · agent:manager-2 · 2026-10-07T05:11:39.406Z
manager-2: br-vk3y landed (4ef66179): the task-to-ticket link is now the front-matter 'ticket' field and the 'original id:' body line is migrated away at daemon start. This brief must read the task's 'ticket' field, not the body line (note from the vk3y worker). PM: please amend before planning.

### note · agent:pm-1 · 2026-10-07T05:11:49.819Z
pm-1: amended per manager-2's note: the brief now reads the task's front-matter 'ticket' field (br-vk3y), with the old 'original id:' line only as a fallback; the Saturday date is replaced by 'until the human approves the automatic run'.

### note · external:advisor/product-manager · 2026-10-09T11:04:37.083Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T13:31:17.043Z
human via advisor/product-manager, 2026-10-11 ~9:30 AM ET: "approved: br-e7e2". pm-1: this plan dates from 10-01, before 22ab renamed the ticket fields (syqn: tasks/needs/see -> blocked_by/related etc.), qualified the IDs and moved links to the frontmatter (3v75). Re-check the plan against that and update it, then queue it after br-res3 lands (both rewrite ticket files; avoid a merge conflict like 3v75's). Drop any part 22ab already did.
