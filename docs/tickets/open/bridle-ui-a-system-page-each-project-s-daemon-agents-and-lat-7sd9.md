---
id: 7sd9
title: "bridle-ui: a System page, each project's daemon, agents and (later) running servers"
kind: feature
opened: 2026-10-04
repos: [bridle-ui, bridle]
changes: []
specs: []
needs: []
see: [s6cj, essy]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 evening, to the bridle-ui aide; dictated, "Bridal" is bridle; one ask
split into [[bridle-ui-a-tasks-page-every-project-s-open-tasks-with-statu-s6cj|s6cj]], the tasks, and this ticket, the rest of the system):

> Okay, I want to queue up some more work. I know we have a few open things waiting on me, but
> here's some additional work that I'd like to get started pretty soon. So, I really want to start
> using the Bridal UI to keep track of what's happening across the system. We're getting there.
> We're making some good moves in the right direction. But what I want to see next is the tickets
> and statuses. So primarily, I'm sorry, tickets and tasks, but primarily tasks. So I want to see a
> list of tasks organized by project. In particular, I want to see, you know, open tasks, closed
> tasks, I don't want to see on the list. There should be a way to find them in the future, and
> they should still link, like a closed task should still open. And, of course, when we're
> rendering a task, we should see all of the links between tasks, so we should be able to follow
> those links once that lands. But yeah, I want to see all the tasks. I want to see what status it's
> in, everything organized. You know, I'm on a mobile phone, so Kanban style isn't the best
> necessarily for mobile. I think a vertical scroll is better than a horizontal scroll, like for a
> Kanban board. You know, TBD, actually, I mean, feel free to come up with something. I'm not sure
> how something like, I can't think of what it's called, but some of those, the one kind of primary
> Kanban player, I'm not sure how they do it on mobile. But, yeah, I want to see, you know, tickets
> that are being worked, what who's working on them, like if they're assigned to a manager, if
> they're assigned to... a worker, what the worker's name is, and all that information should be
> attached to the ticket. And I want to be able to, that could be static today, if that's easier.
> We could do a static version first, but I want to be able to see those things moving in the
> system live at some point in the future. And yeah, I want to see status on everything else in the
> system by project. So, you know, bridal agents, bridal demons, at some point the, in the future,
> this is, this wouldn't be a first version, but I should be able to see, like, the servers that
> are running, so for a project that sets up, you know, servers, like track web, I should be able
> to see the names of the names and ports, IPs and ports of the servers that are running. That can
> come later because that's project specific. But yeah, think about everything that's in Bridal,
> what it can do, and how you can render that. Yeah, so make some tickets for that. Try to resolve
> any questions and get that tick that work moving. And queued up for after this other work lands.

## What the human asked for

- By project: bridle's daemons and agents, and "everything else in the system": "think about
  everything that's in Bridal, what it can do, and how you can render that."
- Later, not the first version: the servers a project runs (track-web's dev servers), with names,
  IPs and ports.
- Static first, live later, as for the Tasks page.

## Context

- The daemon already serves `GET /v1/status` (daemon pid, version, start time, CI, incidents,
  budget, rate limits, sessions, upgrade state), `GET /v1/agents` (name, role, state, task,
  branch), and a port registry (`/v1/ports`) that may cover the servers.
- The gateway design limits it today: "no agents, events or any agent control. The gateway
  exposes only the v1 actions and refuses the rest, so a stolen session can answer and check off,
  not run work" (`docs/design/human-web-ui.md`, section 2). Read-only views of agents and status
  keep that property (no control), but the doc needs updating to the human's ask.
- Queued after the work already in flight, as for the Tasks page.
