---
id: rk7k
title: "bridle-ui: a 'Send to an agent' button on a task: pick the agent, type a message, send"
kind: feature
opened: 2026-10-05
repos: [bridle-ui, bridle]
changes: []
specs: []
needs: []
see: [n2q9, s6cj, 7sd9, ma8e, essy]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 evening, to the bridle-ui aide; dictated; one message, split into
[[bridle-ui-show-every-item-s-id-selectable-with-a-copy-icon-t-n2q9|n2q9]] and this ticket):

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

## What the human asked for

- On a task: a "forward" button (a mobile gesture could come later), opening a box to type what
  they need done, with a dropdown of who to send it to.
- Default recipients: that task's project's agents plus the orchestrator. How it should look
  across projects is open ("Something to think about").

## Context

- The gateway sends nothing to agents today. Its design: "no agents, events or any agent control.
  The gateway exposes only the v1 actions and refuses the rest, so a stolen session can answer and
  check off, not run work" (`docs/design/human-web-ui.md`, section 2). Read-only views were
  allowed tonight for s6cj/7sd9. Sending a message is a write, so it's a further change to that
  rule, which the human's ask here makes.
- The daemon already takes messages (`bridle send`, `POST` to an agent's messages,
  `/v1/agents/{id}/messages`), with the sender's principal recorded. A message from the UI would
  be from `human`.
- "The orchestrator is still a problem anyway": one orchestrator serves every project on the
  machine ([[one-orchestrator-and-advisor-or-one-per-project-ma8e|ma8e]]).
- It needs the Tasks page (s6cj) and agent list (7sd9) to exist, so it comes after them.
