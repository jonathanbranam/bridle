---
id: s6cj
title: "bridle-ui: a Tasks page, every project's open tasks with status and who's working them, mobile-first"
kind: feature
opened: 2026-10-04
repos: [bridle-ui, bridle]
changes: []
specs: []
needs: []
see: [7sd9, a3yd, bnhn, j28f, essy]
tasks: [br-s6cj]
---

## The ask


The human, verbatim (2026-10-04 evening, to the bridle-ui aide; dictated, "Bridal" is bridle; one ask
split into this ticket, the tasks, and [[bridle-ui-a-system-page-each-project-s-daemon-agents-and-lat-7sd9|7sd9]], the rest of the system):

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

## What the human asked for, in order

1. Tasks listed by project, with each task's state. Open tasks only on the list; closed ones
   (integrated, dropped) findable some other way and still openable from any link to them.
2. On a task: who's working it (manager, or the worker by name), and its links to other tasks,
   followable.
3. Mobile first: a vertical scroll rather than a sideways Kanban board. The layout is the
   builder's call ("feel free to come up with something").
4. Static (load on open) first; live updates later.

## Context

- Each project's daemon already serves tasks (`GET /v1/tasks`, `GET /v1/tasks/{id}` with thread,
  watchers, claimed_by, branch), edges (`/v1/edges`), and an event stream (`/v1/events/stream`)
  that a live version could use. The gateway doesn't expose any of it yet: its routes are
  session, projects, items (to-dos and questions), documents, review, the task actions, and
  interactions (`crates/bridle-gateway/src/lib.rs`).
- Ticket and task IDs will change form ([[ticket-ids-carry-a-project-identifier-and-are-told-apart-fro-j28f|j28f]]:
  project identifiers decided in principle, form not designed). Link today's forms meanwhile, as
  ui-pmkd does ([[bridle-ui-render-front-matter-and-auto-link-urls-file-paths-a3yd|a3yd]]).
- The human asked that this be queued after the work already in flight (p88z, bek3, tc7t, bnhn,
  a3yd/ui-pmkd).

## Work

Gateway routes: br-s6cj (bridle). UI: bridle-ui task ui-umaq, started once br-s6cj lands. Decision (orchestrator, 2026-10-04, on the human's "resolve any questions"): the gateway may expose read-only views of tasks, agents and status, still with no agent control; br-s6cj updates docs/design/human-web-ui.md to record it.
