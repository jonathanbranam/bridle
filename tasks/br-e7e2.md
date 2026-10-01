+++
id = "br-e7e2"
title = "Migration: backfill ticket kind and two-way task links in every bridle project (v3dk slice B)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T19:46:02.983Z"
updated_at = "2026-10-01T19:46:09.706964Z"
size = "S"
+++

Ticket: docs/tickets/open/tickets-get-a-kind-kinds-are-editable-tickets-and-tasks-link-v3dk.md, section 'Migration'. Design: docs/design/migrations.md. Code: crates/bridle/src/migrate.rs (the MIGRATIONS list; ids are append-only), crates/bridle/src/ticket.rs. Depends on slice A (br-2e6e: the kind field, the 'tasks:' field and the link check) and on the automatic run (br-2718) so it reaches every project.

Goal: add one migration (next id in the list, e.g. 0002-ticket-kind-and-links) that, for every ticket under docs/tickets/** and docs/spikes/** with a bridle frontmatter: (1) adds 'kind:' where missing, taking it from the ticket's task when it has one (the task body's 'original id: <id>' link or the task's ticket path; ask the daemon if up, else read the state branch/tasks) and otherwise a plain recorded default ('chore'; the human can edit with 'ticket set'); (2) adds 'tasks: [...]' listing every task that points at the ticket, so both sides agree. Idempotent; honours --dry-run (report what it would change); refuses, as the others do, on uncommitted changes in the files. Then flip slice A's missing-kind / missing-link warning to an error in 'bridle ticket check'. Not opt-in: it runs automatically.
Tests: tickets with and without a task get the right kind and links; second run changes nothing; dry-run writes nothing; check errors on a missing kind afterward; the daemon being down doesn't fail it (falls back).
Docs: migrations.md (list the migration), CHANGELOG. Acceptance: just check passes, and this repo's own docs/ pass 'bridle ticket check' after running the migration on a scratch copy. Model: Sonnet.
Migration plan: this is it.

PARKED FOR SATURDAY, same terms as br-6b8a and br-2718: it must not land before Sat 2026-10-03 (it is the first migration to rely on the automatic run, and it changes the check every project runs). Build on the branch, pass just check, hand off; the development manager parks it. Do NOT run it on the human's other projects; the automatic run does that after the human approves.
Out of scope: changing the kind of existing tickets beyond the default.
