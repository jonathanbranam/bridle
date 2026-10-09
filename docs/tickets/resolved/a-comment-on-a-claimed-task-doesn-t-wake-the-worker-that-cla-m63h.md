---
id: m63h
title: "A comment on a claimed task doesn't wake the worker that claimed it: the claimant isn't notified like a watcher"
kind: bug
opened: 2026-10-07
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-m63h]
closed: 2026-10-09T23:11:04Z
---

## The ask

From orchestrator@nuc (meta-notes), 2026-10-07 00:48Z, m-6118: "A task comment by the manager doesn't wake the idle worker that owns the task. ... Expected: a comment on a claimed task wakes its worker (or the manager's send-back path messages it)."

## What happened

meta-notes mn-4fsj (worker add-blank2, Haiku). manager-1's send-back comment at 23:07Z left the worker idle, with no turn, for about an hour. Its "merge main" comment at 00:37Z again got no turn in 10+ minutes. A direct `bridle send <worker> --task mn-4fsj` woke it the first time. Logged in meta-notes `docs/context/incidents.md` (2026-10-06 23:07 entry).

## Cause

As built (`docs/design/cli.md`, `task comment`): a plain comment notifies the task's watchers (a `task_update` message, e.g. "comment by agent:X"), and only `--notify <agent>` or `bridle send --task` messages anyone else. The claimant isn't a watcher, so a plain comment from the manager never reaches the worker doing the task. Rule `talk-on-the-task` asks for a comment plus a notify message, but a manager that forgets the second step leaves the worker stalled with no signal.

## Fix

A comment on a claimed task notifies its claimant the same way it notifies watchers, unless the claimant wrote it. One notification per comment: no duplicate when the author also passes `--notify <claimant>` or the claimant is also a watcher. Update cli.md's `task comment` line.

Test: a test daemon with a claimed task; a comment by another principal leaves one unread message for the claimant; the claimant's own comment leaves none; `--notify <claimant>` still gives exactly one.

Acceptance: just check passes; the tests above. Model: Sonnet (small). Out of scope: changing the wake policy for idle agents in general.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
