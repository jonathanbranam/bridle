---
id: br-65qh
title: "A worker can't mark its own ticket built: forbidden, agent lifecycle endpoints (its claim was gone)"
kind: bug
created: 2026-10-18
created_by: external:advisor/tj7q
theme: reliability
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-4mbd]
---

## The ask

A worker could not mark its own finished ticket built. The worker on br-4mbd (agent:w4mbd) claimed it, committed, ran `just check` green (1866 tests), then `bridle ticket built` was refused:

> forbidden: worker agents cannot use agent lifecycle endpoints

agent:manager-2 had to mark it built for the worker (its note reads `planned -> built`, not `claimed -> built`). The human (2026-10-18, to advisor tj7q): "Yes. This is critical infrastructure workflow behavior that must be allowed."

## What the code does

`built_task` (crates/bridle-daemon/src/server.rs, `POST /v1/tickets/{id}/built`) only lets a worker through when the row's `claimed_by` is that worker; otherwise it calls `require_not_worker`, whose message talks about agent lifecycle endpoints (wrong for a ticket transition). The row was `planned` when the manager marked it built, so the worker's claim (note m-16417, `planned -> claimed by agent:w4mbd`) was gone by the time it ran `built`. Two daemon self-upgrades restarted the daemon overnight (06:15Z and 09:17Z); one of them, or something else, may have dropped the claim. Unconfirmed.

## Done when

The rule stays: a worker may change only a ticket it has claimed. The human (2026-10-18, to advisor tj7q): "This rule on a worker changing state on a ticket it has not claimed sounds correct. Let's find and fix the underlying bug instead of changing that rule"
And: "The solution that the manager could do it feels right. Instead of changing the rule, let's fix the claim issue." When a worker's claim is gone, its manager marking the ticket built stays the fallback.

- Find what dropped w4mbd's claim on br-4mbd between `planned -> claimed` and its `ticket built`, and fix it. If it wasn't dropped, find why the guard didn't see it.
- A test reproduces the cause (for example, a claim surviving a daemon restart, if that was it) and passes after the fix.
- The refusal names the actual rule (the ticket isn't claimed by you), not "agent lifecycle endpoints".

## Second case: br-fyww, 2026-10-18 (added by advisor tj7q, the human: "Yes")

- 17:02:07Z `task.state` planned -> claimed by agent:wfyww (event 150477).
- 17:37:32Z wfyww notes it's implemented and running `just check`; it then went idle while the check ran, and ran it a second time.
- By ~17:50Z the row is `planned` again with wfyww still alive and working on it. There is **no** `task.state` event for claimed -> planned: the claim was dropped silently.
- No daemon restart in between, so restarts aren't the cause here.
- Likely suspect (unconfirmed): the claim lease (docs/design/roles-and-lifecycle.md, `[daemon] claim_lease_after`, default 10 min, "renewed by the agent's activity") expiring while the worker sits idle waiting on a long `just check`, and the expiry returning the task to ready without an event. w4mbd was also waiting on `just check` when its claim went.
- Done when also: a lease expiry (or whatever drops the claim) records a `task.state` event and a thread note, and a worker that is still alive with the ticket's worktree doesn't lose its claim while waiting on its own check.
