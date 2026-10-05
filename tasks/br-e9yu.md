+++
id = "br-e9yu"
title = "Per-project sessions (aide) share one handover file and one identity across projects; key them by project"
kind = "bug"
state = "planned"
created_at = "2026-10-05T00:21:30.006Z"
updated_at = "2026-10-05T00:22:49.318860Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
+++

original id: e9yu
Ticket: docs/tickets/open/per-project-sessions-aide-share-one-handover-file-and-one-id-e9yu.md (read it; it quotes the human, including the clarification).

Approval: the human, via the bridle-ui aide (m-0259, m-0268, 2026-10-04 ~8:25 PM ET), critical: "File is critical, but fix that". And: "That file should be based on the ID of the agent, so the named agents get a different file. I don't even know why it's a file. The orchestrator's handover is some kind of note in the system."

Goal: aides and advisors (named advisors included) hand over through the same handover record the orchestrator uses (the `handovers` table; types.rs `Handover` has role and project), not a file under ~/.bridle/handover/. Each project's daemon keeps its own, so two projects' aides never collide, and each named advisor is its own role.
- Who may write: today `bridle handover write` / POST /v1/handovers accepts only human and external:orchestrator. Allow external:aide and external:advisor[/<name>]; `role` is the writer's identity (aide, advisor, advisor/doc-review), never taken from the request body. Keep the wire type unless a field is truly needed (if so, types.rs and all clients together).
- Reading: `bridle session aide` / `bridle session advisor [name]` (crates/bridle/src/session.rs ~203, ~226, take_handover) fetch the newest note for that role from the project's daemon and put it in the opening prompt (the body, or "run `bridle handover show <id>`"), instead of reading a file.
- Prompting: the daemon's context-limit message to sessions (crates/bridle-daemon/src/sessions.rs, handover_note() at ~125 and its callers) tells the session to run `bridle handover write --file -` instead of writing to a path. Drop the file path.
- Old files: leave existing ~/.bridle/handover/*.md alone (don't read or move them); one line in the doc notes they are obsolete.
- CHECK, and fix if keyed by identity alone across projects, the same collision in the session registry (sessions.rs register/lookup), `bridle session restart aide`, and refuse_if_running("aide", &project). Write what you found per item on the task thread, fixed or already safe, with the line.
- Docs: docs/design/agent-host/orchestrator-supervision.md (section 7 and the file path at ~228/~244), docs/design/cli.md (handover write/show/list), workflow/base/roles/aide.md and advisor.md where they describe handing over, storage.md if the table changes.
Acceptance: just check green; tests: aide and a named advisor can write a handover and get role = their identity; a worker can't; session start for aide picks up the newest aide note and not an advisor's; the context-limit message names the command, not a file.
Model: sonnet.
Out of scope: seats (gtzx); the waiter work (h3ar/75h2); the orchestrator's own handover flow beyond what sharing the code needs.

## Thread

### note · external:orchestrator · 2026-10-05T00:22:19.616Z
From orchestrator: br-e9yu is CRITICAL (the human, via aide: 'File is critical, but fix that'), planned with a full brief (aides and advisors hand over through the handovers record, per the human's clarification). Put it in its own tier at the very front of the queue now.

### note · external:orchestrator · 2026-10-05T00:22:19.644Z
From orchestrator: br-e9yu is critical (the human's word): every project's aide shares one handover file. Brief on the task, sonnet. Give it the next free worker slot, ahead of everything except landing what's already done (push p88z, land bnhn). Two workers max.

### note · external:aide · 2026-10-05T00:22:27.199Z
watching the task

### note · external:aide · 2026-10-05T00:22:49.318Z
The human, verbatim (via bridle-ui's aide): "There should be a write command for an agent to write a handover, and it should be fully managed. Nobody should be ready to file. This should all be managed by the system, and the system then can ensure that every agent in every project with the proper name has the right handover and that there's no confusion about anything."
