---
id: k7tm
title: Tickets and tasks, why both, and is that the long-term plan
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [ticket-state-without-moving-files-p2ys, which-docs-live-in-bridle-and-which-in-markdown-hv8e, refining-a-task-with-the-human-before-it-ships-hvxk]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> And all of this should be associated with the ticket or task. I'm still kind of unsure about
> why we have tasks and tickets separately, and if that's the right thing to do or if that's the
> long-term plan. Something to look into as well

## Today (advisor, 2026-09-29, at f5cd08b)

- A ticket is a markdown file in `docs/questions/` or `docs/spikes/` (verbatim source, notes,
  a Resolution naming where the answer landed). Since P0-6 (tskm), every ticket also has a
  bridle task whose body starts `original id: <id>`, and the task is the queue
  (`docs/README.md`). New work can be a task with no ticket.
- So each ticket is written down twice, and the two can drift.
- Related, still open: [[ticket-state-without-moving-files-p2ys|ticket state without moving
  files]] and [[which-docs-live-in-bridle-and-which-in-markdown-hv8e|which docs live in bridle
  and which in markdown]].

## To look into

Whether the long-term plan is one record per piece of work (the task, with its markdown
readable and editable in vim or Obsidian, per p2ys) or keeping both, and what each is for if
both stay.

## The human, again (2026-10-03, via the advisor)

After the advisor filed four raw ideas (8r5x, z485, 2tpm, 67qw) and each got a task straight
away, verbatim:

> Yeah, interesting. I don't know that they need tasks yet. Um, so I kind of want to understand
> that. Why you created tasks for them immediately. When I, I, like I was hoping to just create
> tickets only. Um, the task can stay open. I think it's, uh, it's fine. Uh, since they're
> already... created and uh, I yeah this the delineation between tickets and tasks is still
> actually if, if there isn't a ticket to talk about that then make make a make one of make a
> question uh, I think ticket um, for that that like do we really need both of these things and
> what lives on a ticket versus a task.

Why the tasks were made: the advisor role says to file each ticket "with its `bridle task new`",
`docs/README.md` says every ticket has a task (since tskm), and `bridle ticket new` creates the
task by default (`--no-task` skips it). Nothing distinguishes a raw idea, one that needs
refinement with the human before anyone could build it, from work ready to queue.

Added questions:

- Does a raw idea need a task at all, or only a ticket until it's refined and specified?
- What lives on a ticket and what on a task?
