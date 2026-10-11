---
id: br-stx8
title: "A task's state says what's really happening: held and built-awaiting-landing are states, not 'planned' with a note"
kind: feature
created: 2026-10-06
repos: [bridle, bridle-ui]
changes: []
specs: []
blocked_by: [br-72t9]
related: [br-kzw2]
resolved: 2026-10-11T22:41:06Z
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

## Approved design (the human, 2026-10-11 ~12:00 PM ET)

The human, verbatim (to advisor product-manager): "approve current design of stx8 as designed; we
can deal with the review cycle questions later; let's land that first and refine".

So pm-1's design of 2026-10-06 (below, copied from the br-stx8 thread) is approved as written, with
its three open questions at pm-1's recommendations: Q1 a held ticket stays in `bridle queue` under
its tier, labelled held; Q2 `send-back` is a new command (reopen stays for dropped/integrated); Q3
`built` is a state in the normal worker flow too (worker finishes -> built -> manager lands ->
integrated).

Since 22ab step 7 the commands are under `bridle ticket` (`ticket hold`, `ticket unhold`,
`ticket built`, `ticket send-back`), with `task` as the hidden alias like the rest.

**Deferred, a later refinement** (the human's input below): the review state between `built` and
`integrated`, the sent-back state, and settling names with v2va slice A's `in_review` (br-6ba3).
They come after this lands, checked against the human's principle below.

### pm-1's design (2026-10-06)

1. STATE NAMES (task `state` field, wire string = lowercase)
Existing, unchanged: pending, open, planned, claimed, dropped, integrated, reopened.
New (exactly two): `held`, `built`.

2. HOW EACH WORKS
- planned: in the queue; a worker may claim it if deps are met and no open question. Nothing else.
- claimed: unchanged (a worker holds the lease). Returns to `planned` on release or lease expiry.
- held: a person has paused it. Not claimable, not spawnable, shown by `bridle queue` as "held" (never "startable"), does not block its dependents' edges any differently than planned (it is unmet until integrated). Fields on the task: `held_by` (principal), `held_reason` (text, required), `held_until` (free text, optional, e.g. "human reviews kzw2"), `held_at`. Set only from `planned` (a claimed task must be released first).
- built: work is finished on a branch and is waiting to land or for review. Fields: `branch` and `commit` (these already exist; both required to enter). Not claimable, not in the queue's startable list, shown as "built". Does NOT resolve `blocks` edges (only integrated does). Left only by landing (`task done`) or by being sent back.
- integrated, dropped, reopened: unchanged.

3. COMMANDS (exact)
- `bridle task hold <id> --reason TEXT [--until TEXT]`: planned -> held. Records a thread note and a `task.held` event.
- `bridle task unhold <id>`: held -> planned. Thread note, `task.unheld` event. (Not "release": that already means giving up a claim.)
- `bridle task built <id> --branch NAME --commit SHA`: claimed -> built (the worker's finish when the work is not landed yet; releases the lease), and planned -> built (for parked work, by the manager/orchestrator/human). Thread note, `task.built` event.
- `bridle task done <id> --commit SHA [--branch NAME]`: built -> integrated (today also allowed from claimed/planned; unchanged).
- `bridle task send-back <id> --reason TEXT`: built -> planned (review failed; keeps the branch field for the next worker). Thread note, `task.sent_back` event.
- `bridle task drop`, `reopen`: unchanged; drop is allowed from held and built too.

4. TRANSITIONS (the whole workflow; anything else is a 409)
pending -> open -> planned
planned -> claimed (claim) | held (hold) | built (built) | dropped
claimed -> planned (release/expiry) | built (built) | integrated (done) | dropped
held    -> planned (unhold) | dropped
built   -> integrated (done) | planned (send-back) | dropped
integrated/dropped -> reopened -> (as open)
Who may: hold/unhold: human, manager, PM, orchestrator. built: the claiming worker, or manager/orchestrator/human for parked work. done/send-back: manager, orchestrator, human (rule work-flow: only their merge accepts).

5. MIGRATION (automatic at daemon start, idempotent, in storage.md): every task in `planned` with `branch` and `commit` set and not integrated -> `built`. No guessing for held; the orchestrator holds br-vk3y, br-1ddd, br-91b3, br-6ba3, br-46fe, br-cf00, br-f610 with `task hold` afterwards. Rerun is a no-op.

6. OPEN QUESTIONS FOR THE HUMAN (my recommendations in brackets)
Q1: a held task still shows in `bridle queue` under its tier, labelled held? [yes, so the queue stays one list]
Q2: Is `send-back` a new command, or reuse `task reopen`? [new command; reopen is for dropped/integrated]
Q3: Should `built` be a state even for the normal worker flow (worker finishes -> built -> manager merges -> integrated)? [yes: that is the "waiting to land" state the manager reads]

Tell me when approved and I'll rewrite the br-stx8 brief with these exact names (and split UI/TUI polish into a second task).
  [note] external:advisor/product-manager (2026-10-09T11:04:39.266+00:00): watching the task
  [note] external:advisor/product-manager (2026-10-10T17:54:59.632+00:00): advisor/product-manager (PdM), human via advisor (2026-10-10 ~2:20 PM ET), verbatim: "The task states work needs to be, I think, possibly superseded by, or it is definitely related to, the tasks as tickets work, and those two need to be in that epic and in the same theme. ... that needs to go with that, and that decision needs to be sequenced after that work. The first thing is to fix that: do all the ticket task work first because they're dependent and interrelated. That's a nice one. It'll be really nice to have, but it can come later." So: stx8 moves into epic everything-is-a-ticket (theme tickets-and-release), last, after all the ticket work. Blocked by br-72t9 (the last ticket step filed so far); the PdM re-points the block as later steps of 22ab's plan are filed. Still HELD for a proposed design and the human's approval; the design may find stx8 superseded by the ticket states of 22ab.
  [note] external:advisor/product-manager (2026-10-10T17:55:35.315+00:00): advisor/product-manager: correction, the human's words above were at ~1:55 PM ET, not 2:20.
  [note] external:advisor/product-manager (2026-10-11T04:12:48.340+00:00): advisor (product-manager): the human's input on pm-1's design (2026-10-11 ~1:45 AM ET) is in ticket stx8, section 'The human's input on the states': keep built; add a review state (manager checking) and a sent-back state (waiting for rework); settle names together with v2va slice A's in_review (br-6ba3).
  [note] external:advisor/product-manager (2026-10-11T04:14:01.010+00:00): Correction from advisor (product-manager): the human's input in my earlier comment was dated '~1:45 AM ET'; it was ~12:10 AM ET 2026-10-11 (the principle ~12:13 AM ET). Ticket stx8 fixed in aa70230b.

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
