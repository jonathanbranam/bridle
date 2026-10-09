---
id: yfjc
title: Every agent talking to the human links each ticket and task it names to the bridle UI, from a configured base URL
kind: feature
opened: 2026-10-05
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [5wdu, j28f, g6v4, jmpf]
tasks: [br-yfjc]
closed: 2026-10-09T23:11:07Z
---

## The ask


The human, verbatim (2026-10-04 ~9:20 PM ET, to the bridle-ui aide, after opening ticket links the aide
sent as `http://dalek.tailbc91f5.ts.net:7878/document?project=bridle&path=docs%2Ftickets%2Fopen%2F<file>.md`;
dictated; one message, split into [[bridle-ui-stable-urls-that-open-any-ticket-or-task-by-id-whe-5wdu|5wdu]] and [[every-agent-talking-to-the-human-links-each-ticket-and-task-yfjc|yfjc]]):

> Yeah, that's a good point about the ticket going from open to resolved. The document viewer
> needs to be smarter to be able to directly open tickets and find them from URL parameters or
> anywhere. Let's file that as something to work on.
>
> We should be able to construct URLs that directly open any task or any ticket, just by ID. The
> ticket ID is unique per project. Task IDs are globally unique, and we're trying to fix the issue:
> the ticket ID is not being globally unique. As far as the other set of IDs, we'll get that fixed
> too.
>
> Yeah, it works beautifully. The first time it came up, I logged in to the Claude AI browser, and
> then after that it stayed logged in. I can also hit the external browser, and it will open Safari
> or whatever. That's a fine backup. Yeah, I would love this. I want this across every session.
>
> Please start doing this, and please write a ticket and ask to get that scheduled so that I have
> links every time an agent that's talking to me refers to a ticket or a task link here. I don't
> see it. I think we probably don't have the work to open a task directly in the UI yet, but we
> could just still start working on those instructions anyway. It will work later once we get that
> implemented.
>
> The address to open, like the URL, should be project configuration or bridle configuration, so
> we can set it up to be whatever we want. I can set it up to be the Tailscale address, and then we
> can change it that way. We can change it to public URLs once, if that's working at some point,
> but for now, I'll just set it up as the Tailscale address. Thanks. That's going to be incredibly
> useful.

## What the human asked for

- Every agent session that talks to the human (aides, advisors, the orchestrator, and any agent
  whose text reaches the human) wraps each ticket and task ID it mentions in a link to the bridle
  UI.
- The base address is configuration ("project configuration or bridle configuration"). The human
  will set it to the Tailscale address now and may change it to a public URL later.
- Start the instructions now, even though task links won't open anything until the UI can open a
  task by ID ([[bridle-ui-stable-urls-that-open-any-ticket-or-task-by-id-whe-5wdu|5wdu]]).

## Context

- The gateway's machine config is the `[gateway]` section of `~/.bridle/config.toml` (bind
  address, login). On dalek the gateway answers at `http://dalek.tailbc91f5.ts.net:7878` and
  `http://100.100.189.100:7878` (Tailscale).
- The human tested it on 2026-10-04: links open in the Claude app's browser (login once, then it
  stays logged in), or in Safari.
- The bridle-ui aide started doing this by hand on 2026-10-04 at the human's request.

## Priority (the human, 2026-10-04 ~10:15 PM ET, via bridle's aide)

> I also want there to be a ticket that updates instructions so that every agent creates links on
> every ticket to the vital UI page for that ticket. I want that done as soon as possible. If you
> have the ticket, you can just read it and start following instructions right now.

("vital UI" is dictation for "bridle UI".) This ticket is that one: as soon as possible. Until the rule
lands, each agent that talks to the human links tickets by hand, in the form
`http://dalek.tailbc91f5.ts.net:7878/document?project=<project>&path=docs%2Ftickets%2Fopen%2F<file>.md`
(the full file name, checked; `resolved` in place of `open` once resolved).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
