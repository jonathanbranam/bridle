+++
id = "br-cyvf"
title = "Agent renewal hands over through the managed record: the outgoing agent writes it, the replacement gets it and its task (e9yu follow-up, qdw8)"
kind = "bug"
state = "planned"
created_at = "2026-10-05T00:23:35.618Z"
updated_at = "2026-10-05T00:23:51.293377Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
+++

Ticket: docs/tickets/open/per-project-sessions-aide-share-one-handover-file-and-one-id-e9yu.md (the human: "every agent in every project with the proper name has the right handover"); incident br-qdw8 (a renewed worker got no task and adopted the wrong one).

Goal: when the daemon renews a manager or worker for context, the handover goes through the managed record from br-e9yu, and the replacement gets it.
- The context governor's message to the outgoing agent (tick_context_check, crates/bridle-daemon/src/supervisor.rs) tells it to run `bridle handover write --file -` (no file, no vague "handoff note").
- The renewal continuation (supervisor.rs ~2822, send_continuation_note) names the agent's claimed task ID, if any, and gives the newest handover note for that agent's identity (the body, or the exact `bridle handover show <id>`). If no note was written, it says so and points at the task thread.
Acceptance: just check green; tests: a renewed worker with a claimed task and a note gets both in its continuation; one with no note gets the task and "no note".
Model: sonnet. Blocked by br-e9yu (needs its per-identity record).
Out of scope: anything br-e9yu does; the qdw8 polling/early-done fixes awaiting the human.

## Thread

### note · external:aide · 2026-10-05T00:23:51.293Z
watching the task
