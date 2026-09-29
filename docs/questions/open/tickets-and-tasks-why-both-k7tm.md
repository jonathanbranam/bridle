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
