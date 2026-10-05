---
id: 5wdu
title: "bridle-ui: stable URLs that open any ticket or task by ID, wherever its file lives"
kind: feature
opened: 2026-10-05
repos: [bridle-ui, bridle]
changes: []
specs: []
needs: []
see: [yfjc, a3yd, j28f, s6cj, k3qx]
tasks: []
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

## Context

- Today a document URL is `/document?project=<p>&path=<repo-relative path>` (k3qx, ui-n6cu). A
  ticket's path changes from `docs/tickets/open/` to `docs/tickets/resolved/` when it's resolved,
  so a link to an open ticket breaks after resolution.
- br-a3yd (planned) extends the gateway's `POST /api/v1/projects/{project}/links/resolve` (from
  br-bnhn) to resolve a bare ticket ID to its file, open/ then resolved/, and a ticket-made task
  ID (`br-<id>`) to its ticket. Other task IDs resolve to nothing, because there's no task view
  yet; that comes with the Tasks page (s6cj: br-s6cj, ui-umaq).
- Wanted here: URL forms that take only an ID (and the project, for tickets, until
  [[ticket-ids-carry-a-project-identifier-and-are-told-apart-fro-j28f|j28f]] makes ticket IDs
  global) and open the ticket or task, so agents can build links from an ID alone.
