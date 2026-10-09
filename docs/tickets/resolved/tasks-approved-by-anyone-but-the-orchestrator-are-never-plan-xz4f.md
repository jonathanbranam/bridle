---
id: xz4f
title: "Tasks approved by anyone but the orchestrator are never planned: nothing tells the PM a task is open (br-p88z sat 9 hours)"
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-xz4f]
closed: 2026-10-09T23:11:03Z
---

## The ask

The human, verbatim (2026-10-04 ~6:05 PM ET, via the aide), on the report that br-p88z never started:

> Investigate this report and file as an incident with your findings. I think our queue is broken. Or PM is not working right

The report: "Its task, br-p88z, never started. The advisor filed it at 9:18 AM this morning at
normal priority, and nobody ever planned it. Nothing in the record says why. It most likely sat in
the queue behind other normal work."

## Findings (event log, pm-1's inbox, the code)

**It didn't sit behind other work: it never reached the queue.** p88z was created `pending` at
13:18:34Z and opened by `external:advisor` at 13:18:36Z. It stayed `open`, never `planned`, until
the human raised it at 22:00Z. Only planned tasks enter the queue, so the manager never saw it.

**pm-1 plans only what someone tells it about.** Every task pm-1 planned on 2026-10-04 was planned
within seconds to minutes of an orchestrator message naming it (jmpf, fc9a, x3xk, 4f8y, m7mp, kae5,
wjhp, k22s, puaf, 3397, qpr7, krz8, 2mtr, bek3, tc7t, bnhn, 58c9, bdrc). jrm2 and ehv6, opened by
the doc-review advisor, were planned only after the orchestrator messaged pm-1 about ehv6. Nobody
messaged pm-1 or the orchestrator about p88z: pm-1's inbox has nothing about it from 13:00 to
16:37Z.

**Nothing wakes the PM when a task opens.** `crates/bridle-daemon/src/queue_nudge.rs` nudges the
**manager** (else the orchestrator) when the *queue* is edited (`set_queue`), not when a task goes
`pending -> open`. The PM role (`workflow/base/roles/project-manager.md`) says "Triage. Read the open
tickets ... and what the human and the orchestrator send you", but pm-1 is idle between messages and
has no sweep of open, unplanned tasks.

**Timing made it worse.** p88z opened between two daemon restarts (13:18:14Z upgrade, 13:27:51Z).
pm-1 was also unable to plan for ~1.5 h later that day (incident "15:01: pm-1 couldn't plan for 1.5
hours"), but that doesn't explain p88z: it was never named to pm-1 at all.

**It's happening to others now.** Open, never planned, opened by an advisor and never named to the
PM: br-anmx (12:47Z, gateway hash-password echoes the password) and br-ckvz (21:48Z). The
September `open` tasks (the k7tm candidates) are different: they wait on the human's br-cb18.

So the human's two guesses: the queue isn't broken; **the PM works as built, but the design has a
gap**. An advisor (or anyone but the orchestrator) can approve a task, and no one is told.

## Fix options

1. **The daemon tells the PM when a task becomes `open`** (debounced like the queue nudge; to the
   running `project-manager`, else the orchestrator). This closes the gap at the source. Recommended.
2. **The PM sweeps open, unplanned tasks** on every wake, and the governor or a timer wakes it
   periodically (for example hourly) when any exist.
3. **A staleness alarm**: `bridle status` and the orchestrator's wake list tasks `open` and
   unplanned for more than N hours (excluding ones with an open question to the human).
   Recommended alongside 1, as the backstop.
4. **A rule**: whoever runs `task ready` also messages the orchestrator. Cheapest, but it relies on
   every role remembering, which is what failed here.

## The human's decision (2026-10-04 ~8:50 PM ET, via the aide)

> For XZ for F, yes, I agree with number 1. The daemon sends a message to the PM when a task opens.
> I guess the task is already pending, so isn't there a status for ready? I don't know how to
> handle that, but when a task is ready, a message goes to the PM. If the PM is dead and wakes up,
> they should get that message, right? That should be fine.
>
> Fix 3 is good too. That should come from the daemon. I don't know what an alarm is or where
> that's sent exactly, but that shouldn't happen. That task should go back to pending, needs
> comments, or whatever the status is. If there's something wrong with it, it should change status.

So:

- **Fix 1, approved.** "Ready" is `task ready`, which moves `pending -> open`; the daemon sends the
  PM a message at that transition. A message (not only a wake), so a PM that's down gets it when it
  comes back.
- **Fix 3, approved and changed.** Not a notice in `bridle status`: the daemon itself acts on a
  task left `open` and unplanned for more than N hours, by changing its state (back to `pending`, or
  a needs-attention state) so the problem shows on the task. Which state is for the design.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
