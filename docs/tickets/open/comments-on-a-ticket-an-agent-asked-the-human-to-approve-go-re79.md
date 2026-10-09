---
id: re79
title: Comments on a ticket an agent asked the human to approve go back to that agent; a pending ticket gets a direct Approve
kind: feature
opened: 2026-10-05
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [g6v4, x8jt, wjhp, ehv6, 4cgx]
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

## Refined (the human, 2026-10-09 ~7:20 PM ET)

The human, verbatim (to advisor product-manager): "Refine re79 and put it in the proper epic - a
document can have a reviewer frontmatter that indicates which agent is sent messages when
comments are added. Goes alongsie 4cgx they are related."

So the idea above is now the ask:

- A document's front matter may carry `reviewer: <agent>`. When comments are added, the daemon's
  document watcher (`doc_watch`) sends the batch to that agent instead of the default
  `doc-<id>` document-reviewer. With no `reviewer:`, today's behaviour stays.
- The agent that asks the human to approve a ticket sets `reviewer:` to itself, so the human's
  comments go back to the asker (the original ask).
- Naming the agent follows 4cgx's mention rules: a role on the same project by default
  (`orchestrator`, `advisor/product-manager`), external roles detected and sent to `external:`,
  an agent by name; the cross-project form is 4cgx's open question. An unknown or gone agent is
  reported, and the batch falls back to the default reviewer.
- Not in this ticket: the direct Approve button on a pending ticket (the first half of the ask).
  It belongs with readiness on the ticket (22ab step 2, br-bpku) and reviews (v2va).

Epic `comment-routing` (theme `human-ui`) with 4cgx. Low priority, like all comment work (the
human, 2026-10-09).
