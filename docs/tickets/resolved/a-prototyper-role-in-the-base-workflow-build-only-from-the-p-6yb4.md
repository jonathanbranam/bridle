---
id: 6yb4
title: "A prototyper role in the base workflow: build only from the prototype prompt's constraints"
kind: feature
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [refining-a-task-with-the-human-before-it-ships-hvxk]
tasks: [br-a4ea]
closed: 2026-10-09T23:11:05Z
---

## The ask


The human, verbatim (2026-10-02, via the advisor; "Bridal" is bridle). The web UI is
[[a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy|essy]], which needs this:

> a couple things to get in the project plan. One is that I think we're about ready to start
> needing a, I think we do need a web UI for Bridal for the human. In particular, I want to see
> tasks assigned to me, things that I need to do, decisions I need to make, and have those in a way
> I can easily run through them and check them off. The TUI is still really, really useful, so
> let's just keep it as well. But we need to start with a web UI for some of this, and next to
> that, similar to that, I think with within the same work, or as a sorry as as a dependency of
> that, before we start with that, I want to set up a prototype agent to be a standard agent that
> comes with the standard workflow pack. And there's a lot of important details with a prototype
> agent. Because as I've used these before, and when I tell the agent to build a prototype, the
> agent will go and read all the code in the code base, and read the existing design documents,
> and build a solution that is constrained by the existing design, and therefore is not very
> novel. And then I get, like I asked for three prototypes, and they all are very similar. The
> agent doesn't like rethink the design when I ask him to, and they are concerned about like how
> the database works, how the API works, all sorts of things that for a prototype of the user
> interface doesn't matter at all. We should be ignoring all of that when we build a prototype. Or
> to be more clear, we should be only following what's in the prototype prompt and not going off
> and discovering other things about the system. So that's something I want as like a really
> strong guidance to the agent that builds a prototype. I think it's a different role than a
> worker and you know each project is probably gonna have to figure out where prototypes live. I
> don't we can't really decide that up front, but that the basic instructions to the worker should
> be something that are shareable and reusable.
>
> I think it would be something along the lines of, you know, you're building a prototype and,
> you know, you should focus on the constraints specified in the prototype. So if the constraint
> says, don't worry about the current implementation, then don't go look at the current
> implementation. But if the constraint says, you know, this should work with our existing
> database schema, then, then the agent's permitted to go check on the database schema.

## The problem

Asked for a prototype, an agent reads the codebase and the design docs first, so what it builds
is shaped by the existing design. Asked for three, it builds three near-copies. It worries about
the database, the API and other parts that don't matter to, say, a UI prototype, and it doesn't
rethink the design when asked to.

## The ask

1. **A prototyper role**, separate from the worker, shipped in the standard workflow
   (`workflow/base/roles/`, beside worker, manager and the rest) so every project gets it.
2. **Strong guidance: the prototype prompt is the whole brief.** The agent builds from what the
   prompt says and doesn't go discovering the rest of the system. It reads only what the prompt's
   constraints name. "Don't worry about the current implementation": don't look at it. "Must
   work with our existing database schema": it may read the schema, and only that.
3. **Several prototypes must really differ.** Asked for more than one, each takes a different
   approach, not variations on one design.
4. **Where prototypes live is the project's choice** (a folder, a branch, a separate repo); the
   base role doesn't decide it. The instructions are shared and reusable; a project adds its own
   through `.bridle/roles/`.

## Notes (advisor)

- Instructions alone may not hold an agent back from exploring; the role could also start in an
  empty or sparse worktree, or with read access limited to what the prompt names. To weigh in the
  design.
- hvxk (refining a task with the human) mentions building a prototype to review as one step.

## Built

`workflow/base/roles/prototyper.md`, a built-in `prototyper` role (worker defaults), `bridle prime
prototyper`, and the project's `.bridle/roles/prototyper.md` appended to its prompt (br-a4ea).
Instructions only. Follow-up idea, not built (YAGNI): start the role in an empty or sparse
worktree, or limit its reads to what the prompt names.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
