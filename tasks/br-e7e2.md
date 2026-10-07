+++
id = "br-e7e2"
title = "Migration: backfill ticket kind and two-way task links in every bridle project (v3dk slice B)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T19:46:02.983Z"
updated_at = "2026-10-07T05:11:49.819689Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
+++

Ticket: docs/tickets/open/tickets-get-a-kind-kinds-are-editable-tickets-and-tasks-link-v3dk.md, section 'Migration'. Design: docs/design/migrations.md. Code: crates/bridle/src/migrate.rs (the MIGRATIONS list; ids are append-only), crates/bridle/src/ticket.rs. Depends on slice A (br-2e6e: the kind field, the 'tasks:' field and the link check) and on the automatic run (br-2718) so it reaches every project. Also builds on br-vk3y (landed, 4ef66179): a task's link to its ticket is now the front-matter field `ticket` (shown as "ticket" in `bridle task show` and the API), and the daemon migrates the old `original id: <id>` first body line into that field at start-up.

Goal: add one migration (next id in the list, e.g. 0002-ticket-kind-and-links) that, for every ticket under docs/tickets/** and docs/spikes/** with a bridle frontmatter: (1) adds 'kind:' where missing, taking it from the ticket's task when it has one (find the task by its `ticket` field equal to the ticket id, or by the task's ticket path; ask the daemon if up, else read the state branch/tasks; for a task that still has an `original id:` first body line because the daemon has not migrated it yet, read the line as a fallback) and otherwise a plain recorded default ('chore'; the human can edit with 'ticket set'); (2) adds 'tasks: [...]' listing every task whose `ticket` field points at the ticket, so both sides agree. Idempotent; honours --dry-run (report what it would change); refuses, as the others do, on uncommitted changes in the files. Then flip slice A's missing-kind / missing-link warning to an error in 'bridle ticket check'. Not opt-in: it runs automatically.
Tests: tickets with and without a task get the right kind and links (tasks fixtures use the `ticket` field; one fixture still has the old body line, to cover the fallback); second run changes nothing; dry-run writes nothing; check errors on a missing kind afterward; the daemon being down doesn't fail it (falls back).
Docs: migrations.md (list the migration), CHANGELOG. Acceptance: just check passes, and this repo's own docs/ pass 'bridle ticket check' after running the migration on a scratch copy. Say in the done note how many of the 48 `ticket check` warnings it clears. Model: Sonnet.
Migration plan: this is it.

PARKED, same terms as br-6b8a and br-2718: it must not land until the human approves the automatic run (it is the first migration to rely on it, and it changes the check every project runs). Build on the branch, pass just check, hand off; the development manager parks it. Do NOT run it on the human's other projects; the automatic run does that after the human approves.
Out of scope: changing the kind of existing tickets beyond the default.

## Thread

### note · agent:manager-2 · 2026-10-07T05:11:39.406Z
manager-2: br-vk3y landed (4ef66179): the task-to-ticket link is now the front-matter 'ticket' field and the 'original id:' body line is migrated away at daemon start. This brief must read the task's 'ticket' field, not the body line (note from the vk3y worker). PM: please amend before planning.

### note · agent:pm-1 · 2026-10-07T05:11:49.819Z
pm-1: amended per manager-2's note: the brief now reads the task's front-matter 'ticket' field (br-vk3y), with the old 'original id:' line only as a fallback; the Saturday date is replaced by 'until the human approves the automatic run'.
