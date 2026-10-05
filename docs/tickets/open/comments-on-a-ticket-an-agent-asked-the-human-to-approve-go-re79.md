---
id: re79
title: Comments on a ticket an agent asked the human to approve go back to that agent; a pending ticket gets a direct Approve
kind: feature
opened: 2026-10-05
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [g6v4, x8jt, wjhp, ehv6]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 ~9 PM ET, to the bridle-ui aide; dictated; the end of a longer message
filed as [[how-agents-send-the-human-status-updates-versus-things-to-ac-g6v4|g6v4]]):

> When I follow that link and review the ticket, I have a few options:
> - I'm going to read the ticket.
> - I could just directly approve it. A ticket that's pending or needs approval should have that
>   ability for me to mark it approved directly instead of through a comment.
> - I can use the comment system to send it back to that agent.
>
>
> Actually, that's an interesting issue there. If an agent's asking me to approve a ticket, my
> comments on that ticket should be sent back to that agent, not to a different agent. Let's make
> sure to file that and keep track of that as a change.

## Context

- Comments on a document under review go to that document's **document-reviewer** agent, not to
  whoever asked: "the pending threads go as one batch to that document's agent (role
  `document-reviewer`, in the main checkout; named `doc-<id>` when the file stem ends in a ticket
  ID ...): spawned if there is none" (`docs/design/agent-host/daemon.md`, around line 113).
- Tickets have no approval state or field today; approvals are quoted into the ticket or task body
  by whoever relays them.
- One sentence of the dictation ("Take filter into this") is left out: the human said it was a transcription error.

## The human's idea (not a decision)

The human, verbatim (2026-10-04 ~9:10 PM ET, to the bridle-ui aide):

> I want to check the design of how review comments would go to that agent. Can you explain that
> to me? I don't have an answer exactly yet, but I'm thinking that there should be something in the
> front matter that says review comments get filtered to this specific agent. That's my thinking,
> but what does the design say?

Fact: the daemon's document watcher (`crates/bridle-daemon/src/doc_watch.rs`) reads no front matter.
The receiving agent is fixed by the file name (`doc-<id>` / `doc-<slug>-<hash>`, role
`document-reviewer`).
