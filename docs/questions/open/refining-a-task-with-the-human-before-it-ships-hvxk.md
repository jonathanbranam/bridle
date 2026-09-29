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

## More from the human: iterating on the implementation, and reviewing it

**Not fully fleshed out.** The human, 2026-09-29: this needs more design work, and they don't
have time for it right now. Don't build from this ticket yet.

The human, verbatim (2026-09-29, via the advisor):

> Something else to think about in this is the idea of whether the work needs a human review
> and what the right way to do that is. There are some tasks that are too vague for me to commit
> to without seeing either the code, maybe, or without being able to actually try out the
> solution. That comes up pretty often in the video game work and in other areas where I've had
> to do a lot of tracked work, like multiple rounds of prototyping with the agent to build a
> version, test the UI, and then give feedback, and then test it again and give feedback.
>
> I'm happy with the general idea of trunk-based development, but when we work with specs,
> changes, and tasks, it is strange to create a new task just to fix a bug in something that was
> shipped but not fully tested, and then go fix a bug in it. It's probably fine if we create
> tasks for that, but I want an association there, or I want something where we can iterate on
> the first approach.
>
> I'm not really sure how to design this the best way with the system, specifically with the Git
> branching, the merging, and the way everything works now. Would we hold up an agent, a worker
> agent, and have me look at the branch and see if I like the result? I'm not sure that's the
> best approach, or is it better to have the agent merge in the work and then have me review it
> on main and then pick it up with a new task and a new agent? I don't know. I think this may be
> what the give-and-take was for, but basically there's some kind of work or some kind of
> ability I want to have at least as an option: to take a more careful, iterative approach with
> the implementation. This goes back to each task having maybe a different workflow,
> potentially.
>
> There are probably a lot of tasks where I can write it up, be clear, ship it off, and just
> check it after it merges. I prefer that approach. That's where I'd like to get to with most,
> but in some cases, again, I need to work with an iterative approach.
>
> There may be tasks where I want the agent to work through the design or the implementation.
> I'd like to be able to iterate on the design and the proposal with an agent. That's one thing,
> and I prefer that the agent's context be more or less focused on what we're doing and not
> distracted by other details. That's a specific thing, but I don't know. I don't want an agent
> sitting there running all the time, even if it's idle.
>
> I think that's where the task comes into play, and this is where the task ticket thing is a
> problem. I want to have a fairly clean context agent that can help me refine a specific task
> until it's ready to go before implementation, and that should be tracked on the task itself.

What this adds to the ask above:

5. **An iterative implementation option**: rounds of build, the human tries it (often UI or
   game work), feedback, build again. Open: review on the branch with the worker held, or merge
   to main and follow up with a new task and agent. "Give-and-take" (take/give, parked in the
   build order) may be meant for this.
6. **A follow-up fix is linked to the task that shipped it**, or iterates on it, rather than
   being an unrelated new task.
7. **Most tasks ship and are checked after merge**, which the human prefers. The iterative
   workflow is an option per task, not the default.
8. **The refine agent doesn't idle.** It has a clean context, is focused on one task, starts
   when needed and doesn't keep running; its work is tracked on the task.

## Refining needs no branch

The human, verbatim (2026-09-29, via the advisor):

> Just another thought to record: that ticket refinement or task refinement, in terms of
> planning, writing a proposal, design, and specs, can all happen without needing a branch
> because it's all either managed in the docs folder or within Bridal. That doesn't need a
> worker agent with a branch, and the design shouldn't require that. If we're moving on to
> building a prototype and reviewing it, then we would need a branch and a work tree for the
> agent to work on.

9. **Refining (plan, proposal, design, specs) needs no branch or worktree**: it lives on the
   task in bridle or in the docs folder, and needs no worker. A branch and worktree come only
   with building (a prototype to review, or the implementation).

This conflicts with `docs/design/specs.md` today ("A task edits `design/specs/<capability>.md`
**in place on its branch**. The plan commit's spec diff is the proposal."), where the spec
proposal is a commit on the task's branch. Settling this ticket means deciding where a spec
change under refinement lives before any branch exists.
