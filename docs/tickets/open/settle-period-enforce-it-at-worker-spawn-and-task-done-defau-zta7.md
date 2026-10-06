---
id: zta7
title: "Settle period: enforce it at worker spawn and task done, default 10m"
kind: bug
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

From the human, via advisor (notes project, message m-0103, 2026-10-06), verbatim:

"Yes I know it's 5 but I figured it wasn't working here at all which is wasn't. Yes increase the default to 10 for all projects. And verify there is a config setting for this. Yes, ask orch to investigate and identify how this should work and be enforced across machines and projects. Only something identified as a critical fix can bypass the wait."

## What happened (meta-notes, 2026-10-06, UTC)

- 14:48:13 the orchestrator filed mn-cys2 and ran `bridle task ready`; settling until 14:53.
- 14:48 the orchestrator told manager-1 "ready ... it can go next". The manager (which had just seen "settling until 10:53 AM, holding") ran `bridle spawn worker --name add-under-tasks` on it at once.
- 14:50:19 the worker reported done; 14:55:42 the manager merged and ran `bridle task done`: the task went `open -> integrated` without ever being planned or claimed. No skip-settle note.
- The human's two corrections arrived during the window (relayed by the advisor), and the worker's first commit missed the second because it was already done.

## Cause (orchestrator's read of the code at 28d25f8c)

The settle check lives only in `is_ready` / the queue's startable flag / `claim` (`crates/bridle-daemon/src/tasks.rs` `settle_until`, coordination.md "Settling"). Two paths go around it:

1. `bridle spawn worker` (manager) takes a free-text prompt and checks no task state, so a worker can start on an `open`, unclaimed, settling task.
2. `bridle task done` accepts a task straight from `open`, never planned or claimed.

Also: the clock restarts only on the human principal's thread entries. Relayed "From the human, via advisor" comments are authored by `external:advisor`, so they don't restart it.

Config: `[tasks] settle` exists per project (`config.rs` `tasks_settle`, `DEFAULT_SETTLE` = 5m). meta-notes, meta-notes-ui and notes don't set it.

## The ask

- `DEFAULT_SETTLE` = 10 minutes (the human's go); update coordination.md and the config docs.
- Enforce at every start path, not only claim: refuse a worker spawn for (or `task done` / integrate of) a task that hasn't gone through plan and claim, or that is still settling. The design question for the build: tie a worker spawn to a claimed task id (`--task`), so the daemon can check it.
- Skip-settle only for a fix identified as critical (the human). Today `skip-settle` is allowed for the human, orchestrator and PM for "the human asked" or "urgent downtime fix"; narrow its wording and role text to "critical fix", reason required.
- Decide whether a relayed human comment (`human via <agent>`, rule human-via-agent) restarts the clock. Recommendation: yes, when the note starts "From the human, via" (cheap; the relay convention already marks it).
- Across machines and projects: the default lives in the binary, so every daemon gets it on upgrade; projects override with `[tasks] settle`. Nothing per machine.

## Verify

Daemon tests: spawn or done on a settling / unclaimed task is refused with "settling until"; after settle + plan + claim it works; a skip-settle note lets it through; default reads 10m.
