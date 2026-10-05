---
id: y25n
title: "The human's to-do list has an order agents can set: priorities with enough levels to put things at the top"
kind: feature
opened: 2026-10-05
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

The human, verbatim (2026-10-05 ~6:20 AM ET, to the bridle-ui aide): "let's go ahead and ask the bridle to get working on a task ordering system for me. I don't know how bridle Q works. I think I know a little bit how it works, but I think, for me, I just want a priority system added. I think we could just do that: add some priorities. I'm tempted to add a lot of levels of priority, frankly, so that agents can kind of tune it to what's needed, but that is just something basic.

The point is that I want the agent to be able to say, \"This is something that needs to be done. I want you to put these two things at the top of my task list so that I see them.\" That's basically it. The problem, obviously, is that agents will all put their things at the top at some point, and then other things need to go above them. I feel like it's kind of a weird question. I don't know, but it needs to get better because it's not good how it is now."

- The human's to-dos (tasks claimed by human) get an order agents can set: an agent can put specific to-dos at the top so the human sees them first.
- Basic is fine: more priority levels than today's low/normal/high, or something equivalent.
- Open design point the human raised: if every agent puts its things at the top, newer urgent things still need to go above them. Pick a simple answer.
- The UI's to-do list shows that order; the CLI too (`bridle task list --claimed-by human`).
