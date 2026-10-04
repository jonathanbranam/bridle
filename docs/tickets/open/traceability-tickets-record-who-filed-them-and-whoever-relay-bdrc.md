---
id: bdrc
title: "Traceability: tickets record who filed them, and whoever relays an ask watches the tasks made from it"
kind: feature
opened: 2026-10-04
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [xxxq, 9nrt, gtzx]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 5:42 PM ET, to the bridle-ui aide; dictated):

> At any rate, are you watching those tickets? You should be, you should add yourself as a watcher
> if you're not. And we need to follow up on whether that's been implemented properly. But did
> you create the tickets yourself? Or the tasks yourself? And did you, are you listed as the
> creator on the ticket? Like, there needs to be some traceability to which agent was responsible
> for these things. If you want to be notified when something lands, then you should be watching
> it. And that should be, I think, largely automatic. The, just the watching, I mean,

## Context (2026-10-04)

- Tasks record `created by` and `watchers`
  ([[task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq|xxxq]]). The bridle-ui
  aide relayed k3qx, tc7t, bek3 and bnhn. Every task made from them (ui-n6cu, br-tc7t, br-bek3,
  br-bnhn) shows `created by external:orchestrator`, with the orchestrator (and the manager for
  ui-n6cu) as the only watchers. The aide wasn't watching any of them until the human asked; it
  then ran `bridle task watch` on br-bnhn, br-tc7t and br-bek3 by hand.
- Tickets have no creator field. The aide wrote k3qx, bek3 and bnhn with `bridle ticket new` and
  committed them under the human's git identity, with a `Co-Authored-By: Claude` trailer only.
  Nothing on the ticket or the commit names `external:aide`. tc7t was written by the orchestrator
  and committed by the aide.
- The incident the aide filed (ui-wdp3) shows `created by external:aide`.
