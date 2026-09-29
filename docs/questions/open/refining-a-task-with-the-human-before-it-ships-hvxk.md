---
id: hvxk
title: Refining a task with the human before it ships, as much as the task needs
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [product-briefs-of-how-bridle-works-today-8awb, tickets-and-tasks-why-both-k7tm]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> I wanted to send a message to talk about how I want specs to work ultimately. I don't know
> how they work yet, but long term, I should be able to have something similar to the OpenSpec
> workflow where I can work with an agent to refine the proposal and design, and be clear on the
> specs before shipping it off to be done.
>
> But the amount of time I have or want to spend on a specific task is going to depend on both
> the task and the project, and my availability. There may be some tasks that I want to ship off
> and say, "Do your best, implement this, and not that critical. Make assumptions and finish it,
> following project guidance." Other times, I need to round trip on a task, and I want to
> actually work with the agent to be clear on this proposal.
>
> Usually, with the design, I actually tend to have a lot of technical requirements in my
> proposal and OpenSpec to be too rigid on that front, so I want to be a little looser on that.
>
> I think, in general, I do want to have a record of the design decisions that are made by the
> agent. I think I'd also like, when I'm talking to the agent, for it to be focused on my problem
> and not just a generic agent. The main concern I have is around context. I usually work with a
> clean context when working with OpenSpec to keep things focused and clear.
>
> And all of this should be associated with the ticket or task.

## What the design says today (advisor, 2026-09-29, at f5cd08b)

- `docs/design/specs.md`: no OpenSpec change folders. A task edits
  `design/specs/<capability>.md` in place on its branch; "the plan commit's spec diff is the
  proposal". `proposal.md`/`design.md`/`tasks.md` become the task record: the body holds why and
  what, the plan section holds decisions and alternatives.
- `docs/design/gates.md`: the plan gate is the manager by default, and the human only for
  protected requirements, a new capability or an arch-revision. It's set per project, not per
  task, and has no "round trip with the human" mode.
- Built so far is the spec tooling (`spec check/id/export/import/coverage`,
  `docs/design/spec-flow.md`). Nothing is built for the human to refine a task with an agent.

## What the ask adds

1. **The human picks the involvement per task**: from "ship it, make assumptions, follow
   project guidance" to "round trip with me on the proposal before building". It depends on the
   task, the project and the human's availability.
2. **A refine session**: the human works with an agent on the proposal and specs, in a clean
   context focused on this task (not a generic or long-running agent), and the result is
   written to the task.
3. **Looser on technical design** than OpenSpec's proposal/design: less rigid technical
   requirements up front.
4. **A record of the design decisions the agent makes**, on the task, even when the human
   wasn't involved.
