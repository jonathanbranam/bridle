+++
id = "br-a34f"
title = "Task watchers 1/4: persistent per-principal wake cursor (task changes are lost between waits)"
kind = "bug"
state = "dropped"
created_at = "2026-10-03T02:07:01.118Z"
updated_at = "2026-10-03T02:30:37.722390Z"
size = "M"
+++

Ticket: docs/tickets/open/task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq.md (read 'Today' and item 4). Code: crates/bridle-daemon/src/principal_wake.rs (wait() takes since_seq = the newest event at call start, so task changes between waits or across a daemon restart are never reported); how the orchestrator's wait-for-wake keeps its persistent cursor (find it; REUSE that mechanism and its storage); docs/design/agent-host/messages.md, api.md, storage.md.
Goal: each principal has a persistent event cursor for task wakes. wait() starts from the stored cursor (first ever wait: the newest event now, so there is no flood), and the cursor advances ONLY when a wake that reports task reasons is delivered (to the newest event seq examined for that wake); a timeout with nothing leaves it. So a change that lands while no wait is running is reported at the next wait at once, and a daemon restart loses nothing. Keep today's reason shape and the 'created or claimed' rule (watchers come later); do not change message wakes (unread messages already wake at once).
Storage: if a new table/column is needed, add it with the project's usual migration so existing databases upgrade cleanly; a missing or unreadable cursor must degrade to 'newest event now', never an error at daemon start-up. No schema change at all if the orchestrator's mechanism can be reused as is.
Tests: a task change between two waits is reported by the second; reported once (not again on the third); survives a store reopen (restart); a first wait doesn't flood with old events; timeout leaves the cursor; two principals have independent cursors. Docs: messages.md/api.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: as above, none beyond an automatic table if needed. OVERLAP: br-2672 (parked, branch bridle/..., human review br-a3b9) moves the orchestrator's wakes onto 'agent wake': this task lands first on main; br-2672 must then use this cursor when it lands (note it in the summary). Out of scope: created_by, watchers, change details, wake-vs-message.

## Thread

### note · agent:manager-2 · 2026-10-03T02:29:52.697Z
Pause: the human changed the xxxq design (notifications are messages, the bookmark is read state), so this task may be superseded; pm-1 is re-checking. Keep reading, but do not write code or commit until I message you again. Reply with a one-line note of what you've done so far.

### note · agent:pm-1 · 2026-10-03T02:30:14.153Z
dropped: Superseded by the human's decision (xxxq, 2026-10-03): task changes become messages, and read state is the bookmark, so a separate persistent wake cursor is not built. The lost-changes gap closes when br-7605 lands. Nothing from this task is to be merged.

### note · agent:manager-2 · 2026-10-03T02:30:37.722Z
br-a34f is dropped (superseded). Stop; do nothing further.
