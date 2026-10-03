+++
id = "br-7ee4"
title = "Other projects submit bridle work directly: 'bridle ticket submit' files an open task marked with its submitter (93xm)"
kind = "feature"
state = "integrated"
created_at = "2026-10-01T23:54:53.934Z"
updated_at = "2026-10-02T00:17:12.318266Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
branch = "bridle/ticket-submit"
commit = "a1bb7e089501b51a2125fee4305b0d310d9a3f01"
summary = "Added 'bridle ticket submit -k <kind> <title> [--body|--body-file]' over new POST /v1/tasks/submit: any principal files an OPEN task whose body's first line and first thread note say 'submitted by <principal>'; the product manager (else external:orchestrator) gets one inbox message. Dropping a submission with a reason messages the submitter. Visitors (name@machine) are now refused plan/claim/drop/edit, and may comment only on their own submissions (previously unguarded). Tests in tests/ticket_submit_test.rs; docs: cli.md, principals.md, 93xm Built, CHANGELOG."
+++

Ticket: docs/tickets/open/other-projects-submit-bridle-tickets-directly-for-triage-93xm.md (the human's ask is verbatim there). Design decision made by the PM, KISS: an OPEN task is already 'proposed': only the PM plans tasks, so no new state or table. Code: 'bridle task new' and the task-create route in crates/bridle-daemon, principal permissions (docs/design/agent-host/principals.md), crates/bridle/src/ticket.rs for the CLI shape.

Goal: any principal with a token on the daemon, visitors included (external:advisor@nuc, external:orchestrator@nuc), can run 'bridle ticket submit --project bridle --kind <kind> "<title>" --body-file -' and get back a task id. It creates an OPEN task (never planned, never queued) whose thread's first note and first body line record the submitter ('submitted by <principal>') and the PM/orchestrator is notified by one inbox message to the product-manager role (or the orchestrator when there is no PM) pointing at the task. The PM triages: accept (turn it into a ticket with 'ticket new' and plan it), merge (comment, drop with reason naming the ticket), or decline (drop with a reason; the daemon sends the reason to the submitter as a message, if that is not already what drop does, add it). Submitters may not plan, claim, drop or edit others' tasks (check the permission table; add only 'create open task' for visitors if it is missing, and a comment on their own submission).
Docs: principals.md permissions, cli.md, a short section in the 93xm ticket ('Built'), CHANGELOG. Tests: a visitor token submits and gets an open task with the submitter recorded; it cannot plan or claim it; the PM is messaged; decline-by-drop notifies the submitter.
Acceptance: just check passes. Model: Sonnet.
Migration plan: none (no project files).
SAFETY: no start-up or orchestrator-path change; lands normally. Out of scope: automatic dedupe, a separate proposed state, submissions of ticket files.

## Thread

### note · agent:ticket-submit · 2026-10-02T00:15:27.037Z
done: ticket submit + visitor guards; just check green (984 tests); fa32fed

### note · agent:manager-2 · 2026-10-02T00:15:33.224Z
One fix: in supervisor.rs you inserted note_submission/note_to_submitter between the doc comment 'Resolve or drop while active: ...' and the function it belongs to, so that comment now documents note_submission. Move your two functions above that comment (or below its function). Re-run just check, then message me the new sha.

### note · agent:ticket-submit · 2026-10-02T00:16:57.819Z
fixed doc comment placement; just check green (984 tests); f767991

### note · agent:manager-2 · 2026-10-02T00:17:00.215Z
integrated: a1bb7e089501b51a2125fee4305b0d310d9a3f01 (branch bridle/ticket-submit)

### note · agent:manager-2 · 2026-10-02T00:17:12.318Z
cleanup: removed agent ticket-submit, branch bridle/ticket-submit
