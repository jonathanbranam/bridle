---
id: 4s3z
title: bridle session restart says it restarted a session that's still running, and types the relaunch into it
kind: bug
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [h3ar, krz8]
tasks: [br-4s3z]
closed: 2026-10-09T23:11:06Z
---

## The ask

The human, verbatim (2026-10-04, via the aide), approving the aide's restart: "Yeah, please. I thought
you could restart yourself. If you can't restart yourself, that's a real problem. I approve a restart.
Get that done."

## What happened (2026-10-04 ~00:33Z)

The aide (pane %86, launcher pid 58633) ran `bridle session restart aide` on itself, in the background.
It printed "asked aide to write ~/.bridle/handover/aide.md; waiting", then **"restarted aide in pane
%86"** and exited 0. But the session was never stopped: same pid, same `started_at` (13:40:44Z) in
`bridle status`. The relaunch command `bridle --project bridle session aide` was typed into the pane
and arrived in the running Claude session **as a user prompt**, not in a shell.

So `restart` reports success without checking that the old session exited (`stop_session` in
`crates/bridle/src/session.rs` SIGTERMs the launcher's children, waits up to 20 s, then carries on and
sends the keys anyway).

## Fix

- `restart` confirms the old launcher pid is gone before typing the relaunch. If it isn't, it escalates
  (SIGTERM, then SIGKILL, to that exact pid's child) or reports failure, and never types into a live session.
- A session can restart itself: the restart runs detached from the session it kills, or the daemon
  does it, so killing the caller doesn't kill the restart.
- Report "restarted" only after the new session has registered (a new pid in `bridle status`).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
