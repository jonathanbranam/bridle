+++
id = "br-xz4f"
title = "Tasks approved by anyone but the orchestrator are never planned: nothing tells the PM a task is open (br-p88z sat 9 hours)"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T22:06:38.716Z"
updated_at = "2026-10-05T15:10:11.991232Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/open-nudge"
commit = "189508b0f2fa2222edd40fcf283038b5acbb4c38"
summary = "Readying a task (task ready) now messages the running project-manager, else the orchestrator, once per settled burst naming each still-open task (skipped when the PM readied it). A task open with no activity for [tasks] open_stale (default 4h, 0 = off) goes back to pending with a thread note and a message to the orchestrator and the readier (readier kept in memory; creator after a restart); tasks with an open question are skipped. New crates/bridle-daemon/src/open_watch.rs, run on the existing settle-wake tick; TaskManager::unready_stale; config + docs + CHANGELOG. Caveat: the staleness clock is updated_at, so any comment on an open task resets it."
ticket = "xz4f"
+++

docs/tickets/open/tasks-approved-by-anyone-but-the-orchestrator-are-never-plan-xz4f.md

Approval: the human, via aide (m-5136, 2026-10-04 ~8:50 PM ET): "yes, I agree with number 1. The daemon sends a message to the PM when a task opens" and, on fix 3, "That should come from the daemon ... That task should go back to pending, needs comments, or whatever the status is. If there's something wrong with it, it should change status." Full quote: the ticket's "The human's decision".

Goal:
- Fix 1: when a task goes pending -> open (`task ready`), the daemon sends a message (not only a wake; it must wait in the inbox for a PM that's down) to the running project-manager, else the orchestrator. Debounce like queue_nudge.rs (several readies in a burst become one message naming every task). Skip it when the PM itself readied the task.
- Fix 3: a task left open and never planned for more than N hours (config, default 4h) goes back to pending by the daemon, with a task comment saying why ("open 4h, never planned"), and a message to whoever readied it and the orchestrator. Exclude tasks with an open question to the human. Checked on an existing periodic tick, not a new loop.
- docs/design/agent-host/ (task states, nudges) and docs/design/cli.md where they describe these.
Acceptance: just check green; tests: ready -> PM gets one message naming the task (and the orchestrator when no PM runs); a burst gives one message; an open task past N hours goes pending with the comment; one with an open question to the human stays open.
Model: sonnet.
Out of scope: fix 2 (PM sweeps) and fix 4 (rule); a new task state.

## Thread

### note · agent:open-nudge · 2026-10-05T15:05:25.997Z
done: ready tells PM (else orchestrator), stale open tasks return to pending; just check exit 0, 1238 tests run (1238 passed, 5 skipped); commit 501bae7c (main already merged)

### note · agent:manager-2 · 2026-10-05T15:05:30.950Z
integrated: 189508b0f2fa2222edd40fcf283038b5acbb4c38 (branch bridle/open-nudge)

### note · agent:manager-2 · 2026-10-05T15:10:11.991Z
cleanup: removed agent open-nudge, branch bridle/open-nudge
