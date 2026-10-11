---
id: stx8
title: "A task's state says what's really happening: held and built-awaiting-landing are states, not 'planned' with a note"
kind: feature
opened: 2026-10-06
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [kzw2]
tasks: [br-stx8]
---

## The ask

The human, 2026-10-05 ~10:05 PM ET, verbatim (to bridle's aide, looking at the UI):

> I do not understand why there are 24 planned tickets, and I don't see any open tickets in the UI that are being worked on. I just cannot understand what's going on with our system. Doesn't a ticket being planned mean that it's in the queue to be worked on? I think a bunch of them say "hold" or something. If the tickets are put on hold, then change the status. God damn it.

What the aide found (2026-10-06 02:05 UTC): 24 tasks in `planned`, only 2 of them being worked. "Planned" covers three different situations, and nothing in the state tells them apart:

- **Built, waiting to land** (work done on a branch, not merged; the human's to-do br-a3b9 is to review them): br-6b8a, br-2718, br-2672, br-8b98, br-79c3, br-88d4, br-751e.
- **Held** (a decision is pending; the hold is only text in the thread or an open question): br-vk3y (until the human reviews kzw2), br-1ddd (until k8jn), the review slices br-91b3, br-6ba3, br-46fe, br-cf00, br-f610 (low priority).
- **Actually queued**: br-ckvz, br-58c9, br-bdrc, br-gztq, br-dxcw (being worked), br-ukpm (being worked), br-9966, br-e7e2, br-7172, br-8c25.

The ask: a task's state says what is really happening, so the human can see it at a glance in the UI and the CLI:
1. A held task has a held state (with who holds it, why, and until what), not `planned` with a note. Holding and releasing are commands that set it.
2. A task built on a branch and waiting to land (or waiting for the human's review) is not `planned`: it has a state that says so.
3. `planned` then means only "in the queue, a worker can start it".
4. A migration moves today's tasks into the right state.

See kzw2 (task data the system acts on is structured fields, not text): the same principle.

## The human's input on the states (2026-10-11 ~12:10 AM ET, to advisor product-manager)

pm-1's proposed design (2026-10-06) is on the br-stx8 thread, not approved. The human, reading it,
verbatim:

> I definitely want 'built' because integration can take a while and I want to clearly know how long
> tasks stay in built.
>
> if a built task is sent back to the worker for additional changes, is there any state transition? I
> think that should be a state. We're missing a state for while the manager is reviewing and
> checking the built task before it is integrated or sent back for changed.

And on claimed -> integrated (pm-1 allowed it): today `task done` takes any state but
dropped/integrated (`tasks.rs`); claimed -> integrated is used by the human's to-dos and incidents
(no commit) and by direct commits to main. The human: "I think it makes sense, just asking".

So the design needs, beyond pm-1's `held` and `built`:

- **A review state** between `built` and `integrated`: the manager is checking and landing it
  (time in it = how long review and the landing check take).
- **A sent-back state**: changes requested, waiting for the worker (time in it = how long rework
  waits), then back to claimed when a worker picks it up.
- So the time in each part of the flow can be read off the transitions: waiting to be reviewed
  (`built`), being reviewed (review state), waiting for rework (sent-back state).

Overlaps to settle in the design:

- **v2va slice A (br-6ba3)** plans an `in_review` state "between claimed and integrated" with
  `bridle task ready-for-review`. That is the same territory as `built` + the review state; one
  set of names must come out of the two. storage.md also names `in_review` and `accepted` as
  designed, not built.
- **22ab section 10** (ticket states) may supersede task states once tasks become tickets.

**The principle** (the human, 2026-10-11 ~12:13 AM ET, verbatim): "basically, I want to be sure,
every time a ticket/task passes to a new agent or is being treated differently, that there is a state
transition that we can trace and understand the flow." The design is checked against it: every
hand-off between agents (worker, manager, reviewer, human) and every change in how the work is
treated (waiting, being reviewed, sent back, held) is its own state with a recorded transition.
