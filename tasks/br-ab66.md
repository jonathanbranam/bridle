+++
id = "br-ab66"
title = "Renewed worker gets no first message and sits idle (htp6b gap)"
kind = "bug"
state = "open"
created_at = "2026-09-28T15:58:22.990Z"
updated_at = "2026-09-28T15:58:22.990Z"
+++

Small, queue after j479 (br-8638). Flagged by the orchestrator: after an automatic context renewal (htp6b), the fresh process gets no prompt and sits idle until something else prompts it -- p2-3-bridle-sync sat idle 8 minutes today.

Root cause: crates/bridle-daemon/src/supervisor.rs renew_inner() (~1849-1983) spawns a brand-new session (Session::New) but, unlike spawn() (~700-765), never sends a first message/turn. spawn() computes `first_message = req.prompt.or(role.start_prompt.clone())` and sends it via self.send(...) as a Note, which is what actually starts the process's first turn (a fresh claude process just waits on stdin otherwise). renew_inner only replays already-pending messages (~1967-1976) via messages_for_agent(&agent.id, &[Pending]) -- if there are none (the common case: the agent renewed itself mid-task, nothing new was sent to it), nothing gets written and the process idles.

Before a renewal, the context governor already sends the agent a "Context handoff:" notice asking it to commit WIP and "send a short handoff note on where you are" (supervisor.rs ~416-419) -- so by the time renew actually runs, the agent's own handoff note is usually already sitting in the message history/task thread.

KISS fix: after renew_inner spawns and registers the fresh process, send it a synthetic first message (same self.send(...) as a Note, same as spawn's first_message) along the lines of: "You were renewed for context (you were at ~N tokens). Continue from your task's thread and your own last handoff note -- check `bridle inbox`/the task body for where you left off." This should happen whether or not there were already-pending messages (send it after the pending replay, or before -- pick whichever reads more naturally as the first thing the fresh process sees). Reuse the pattern spawn() already has for waiting on readiness (~780-789) if useful, though that is optional polish.

Acceptance: just check passes; add a test that renew() results in a message being sent to the renewed agent even when there were no pending messages beforehand (mirror however the existing spawn()/start_prompt tests check that a first message was sent).

Out of scope: changing the context-handoff notice text itself, and anything about resume() after a daemon restart (separate code path, not reported as idling the same way -- check briefly whether resume() has the same gap, but only fix renew unless it is a one-line addition to also cover resume).

Model: Sonnet — this is supervisor.rs internals with an existing, similar pattern to follow, but touches async spawn/messaging plumbing.
