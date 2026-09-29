---
id: b5br
title: Dropping a claimed task leaves its claim behind
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What happened

`bridle queue` on 2026-09-29 showed, under "Claimed:":

```
br-29f9    feature dropped -    agent:branch-rules Branch rules per project: trunk or dev+release, configurable
```

`agent:branch-rules` claimed br-29f9 at 17:18Z on 2026-09-28. Its work was merged to main
(c533cb0), and pm-1 closed the task with `bridle task drop` ("Merged to main per orchestrator
(c533cb0) ... Closing to clear"). The task is `dropped`, but its row in `claims` is still
there, so the queue lists it as claimed. The agent has since been removed.

The human, verbatim: "it is claimed but dropped, I don't understand" and "yes, file this and
get it fixed. can you clear this from the queue as well?"

## Why

- `TaskManager::done_task` releases a claim before marking the task integrated
  (`crates/bridle-daemon/src/tasks.rs`, `TaskState::Claimed => task = self.release_claim(id)`).
  `drop_task` doesn't: it changes the state and leaves the `claims` row and the in-memory
  claim alone.
- Nothing clears it later. `tick_claim_lease_check` skips a claim whose claimant has no
  `agents` row ("leave the claim as is rather than guessing"), which is the case once the agent
  is `bridle rm`'d. So the stale claim stays in the queue for good.
- Had the agent still existed and gone quiet, the lease check would have called
  `release_claim`, which sets the task's state to `planned`, bringing a dropped task back
  into the queue.
- `bridle release` only releases the caller's own claim, so nobody else (the PM, the
  orchestrator, the human) can clear it today.

## What the human wants

- Fixed: dropping a claimed task releases its claim, as `done` does.
- br-29f9's leftover claim cleared from the queue.

## Notes

- Suggested scope: `drop_task` releases any claim (without the `planned` transition in
  `release_claim`); the lease check never moves a task that isn't `claimed`; drop a `claims`
  row whose task isn't `claimed` when the daemon loads (`open`/`rebuild_from_state_branch`),
  which also clears br-29f9 on the next restart. A test for each.
- br-29f9's work landed, so it should be `integrated` (c533cb0), not `dropped`. Once the fix
  is in: `bridle task reopen br-29f9` then `bridle task done br-29f9 --commit c533cb0`.
