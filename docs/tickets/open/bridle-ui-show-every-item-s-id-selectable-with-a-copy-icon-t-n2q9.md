---
id: n2q9
title: "bridle-ui: show every item's ID, selectable, with a copy icon (to-dos first)"
kind: feature
opened: 2026-10-05
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [rk7k, j28f, a3yd, s6cj]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 evening, to the bridle-ui aide; dictated; one message, split into this
ticket and [[bridle-ui-a-send-to-an-agent-button-on-a-task-pick-the-agent-rk7k|rk7k]]):

> Another thing for the UI: when I look at to-dos that are assigned to me, I don't see the ID in
> there, and that ID is really useful. The message ID is useful to send to an agent so I can copy
> and paste it. Because I'd like to be able to highlight the ID, maybe even with a copy button.
> Let's just start with the ID. I don't know. The copy button as an icon, just an icon next to it,
> would be fine. I can easily paste any message to an agent and ask it something about the
> message. That should also be true for documents or anything with an ID: the ID of the thing
> should be visible and selectable so I can reference them in tickets, notes, and comments to the
> agents.
>
> The next step here would be that, for a task, there would be a button or some interaction, like
> a mobile one, but we could just start with a regular button. I could hit a forward kind of
> button and say, "Send this to an agent," and then I could type directly in there what I need
> done.
>
> I would need a dropdown, I guess, for which agent to send to, or pre-text plus the dropdown.
> That's pretty interesting. Something to think about is how exactly that UI should look with
> different agents across different projects. I think, by default, the easiest thing is whatever
> project this is on, plus the agents, plus the orchestrator for that box.
>
> The orchestrator is still a problem anyway. Plus, the agents I can pick from a dropdown, then
> type the message, and then hit send, and it gets sent to them.

## Context

- The to-do list (`src/Items.tsx` at c5c0a0a) has each to-do's and question's `task_id` (the
  gateway's `Todo` and `Decision` types carry it) but uses it only as a React key and in the
  action calls; it isn't shown.
- "Anything with an ID": to-dos and questions now; documents (path, ticket ID), comment threads
  (`c<n>` IDs from ehv6); later tasks and agents on the Tasks and System pages (s6cj, 7sd9).
- The human's order: the ID visible and selectable first; a copy icon beside it is welcome.
