+++
id = "br-ee78"
title = "A message an agent receives is read: no separate mark-read step, no unread for agents"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T02:28:55.666Z"
updated_at = "2026-10-03T02:47:21.164011Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/read-on-receipt"
commit = "21bd096de47a9e6d81f35516611f4bc9d02af8a0"
summary = "A message that reaches an agent's context is now read automatically (rmpq). Headless agents: the stdin ack sets delivered then read (supervisor.rs). `GET /v1/wake` (principal_wake.rs) returns the unread messages in full (`messages` on the reason) and marks them read via the new atomic `Store::take_unread_messages`, for non-human callers only, so a second wake never repeats them and none is lost; the human's wake and reads are unchanged. The orchestrator's wake (wake.rs, server.rs) marks the message behind each `message` wake read as it answers. `GET /v1/messages` gained `id` and `mark_read` (honoured for agent/external principals on their own messages); `bridle inbox` and `inbox show` send them. `POST .../unread` is 403 for non-humans. `--mark-read` still works; the advisor prompt no longer asks for it. No schema change. OVERLAP: br-2672 (parked) also touches orchestrator wakes (`orchestrator_wake` handler in server.rs); whichever lands second reconciles. Docs: messages.md, cli.md, api.md, CHANGELOG. Not touched: daemon start-up/restart/upgrade."
+++

original id: rmpq
Ticket: docs/tickets/open/a-message-an-agent-receives-is-read-no-separate-mark-read-st-rmpq.md (read it all: the human's words, 'Today', 'Decided'). Docs: docs/design/agent-host/messages.md (states pending -> written -> delivered -> read), cli.md, api.md. Code: the message store and delivery in crates/bridle-daemon (the stdin delivery/ack of headless agents, the principal wake in principal_wake.rs and the orchestrator wake in wake.rs), 'bridle inbox' and 'bridle agent wake' in crates/bridle, bridle-api types.
Goal (decided by the human 2026-10-03): a message that reaches an agent's context is read automatically. (1) Headless agents: marked read when delivered into the conversation (the stdin ack), i.e. 'delivered' becomes 'read' for agent recipients. (2) Interactive sessions (orchestrator, advisors): the wake returns the messages themselves (text, from, id, not just ids) and marks them read; 'bridle inbox' run by a non-human principal marks what it LISTS read (not messages it filters out; 'inbox show' of one message marks that one read). (3) Agents cannot mark a message unread: 'bridle inbox unread' refuses for non-human principals with a clear error. (4) The human keeps read/unread exactly as today (inbox as a task list; their reads are explicit). (5) Drop '--mark-read' from the role prompts (workflow/base/roles/*, e.g. the advisor loop and the orchestrator) and any role text that tells agents to mark messages read; keep the --mark-read flag working for compatibility (no-op for agents is fine). Careful with the wake: a message must be marked read only after it is returned to the caller, in the same call (no race that returns it twice or loses it); unread messages still ring the wake. Tests: headless delivery marks read; wake returns text and marks read, a second wake doesn't repeat them; non-human inbox marks what it lists; agent 'unread' refused; human 'unread' still works; human reading unchanged. Existing already-delivered-unread messages need no migration. Docs: messages.md, cli.md, api.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none (state semantics only; no schema change). Out of scope: task-change messages (a separate task), fbfy delivery-system question, the human's inbox UI. OVERLAP: br-2672 (parked) touches orchestrator wakes; whichever lands second reconciles. This task goes AHEAD of the watcher tasks.

## Thread

### note · agent:read-on-receipt · 2026-10-03T02:47:05.485Z
done: messages an agent receives are read on receipt (headless ack, wake returns text + marks read atomically, non-human inbox marks what it lists, unread is human-only); just check green, 1060 tests (last full count 1003); main merged; br-2672 overlap in summary; 4620902

### note · agent:manager-2 · 2026-10-03T02:47:10.890Z
integrated: 21bd096de47a9e6d81f35516611f4bc9d02af8a0 (branch bridle/read-on-receipt)

### note · agent:manager-2 · 2026-10-03T02:47:21.164Z
cleanup: removed agent read-on-receipt, branch bridle/read-on-receipt
