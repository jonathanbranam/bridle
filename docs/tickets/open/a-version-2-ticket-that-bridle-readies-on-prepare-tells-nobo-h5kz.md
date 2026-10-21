---
id: br-h5kz
title: "A version 2 ticket that bridle readies on prepare tells nobody: the PM never hears it is open, and open_stale sends it back to pending"
kind: bug
created: 2026-10-21
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-2ae8, br-ewk8, br-tj7q, br-xz4f]
---

Found on br-2ae8, the first real version 2 ticket (Stages MVP step 6, br-ewk8), 2026-10-20, reported
by advisor tj7q for the human. The human: "it does seem like a bug. I believe after 4 hours there will
be another message sent but I'm not sure that's the same situation exactly or not".

What happened: all four stages were signed at 23:07:45Z (7:07 PM ET) and the daemon opened the row
("prepared: every stage is written and every gate signed; ready", 23:07:56Z). Nobody was told. pm-1's
8:00 PM ET nudge (m-17826) listed only br-6wp2, readied later the normal way. Two hours on, br-2ae8
was still `open` with no plan.

Cause (advisor product-manager, from the code):
- The API's ready (`ready_task` in crates/bridle-daemon/src/server.rs) calls `open_watch.readied`
  (the xz4f "open and not yet planned" message to the PM), emits the state-change event, and
  calls `note_task_filed`.
- The version 2 auto-ready in `sync_ticket_links` (crates/bridle-daemon/src/ticket_links.rs) calls
  `TicketManager::ready_task` directly and only writes a note: no message to the PM, no event, no
  filed notice.
- The 4-hour `[tasks] open_stale` timer is not a reminder: it returns the row to `pending` and tells
  the orchestrator and whoever readied it. For a version 2 ticket, the next link sync then readies it
  again (prepared, pending), silently once more, so it can cycle every 4 hours and never be planned.

## The ask

1. A version 2 ticket that bridle readies on prepare tells the PM (else the orchestrator) exactly as
   a ready through the API does: the open-and-not-yet-planned message, the state-change event and
   the filed notice.
2. `open_stale` never returns a prepared version 2 ticket to `pending` (nothing to re-prepare);
   it re-sends the open-and-not-yet-planned message instead, and tells the orchestrator.
3. Tests: prepare a version 2 ticket and see the PM's message and the event; an open, prepared
   version 2 ticket past open_stale stays open and the PM is told again.
4. docs/design/stages.md (and wherever open_stale is described) says so.

## Proposal

Move the side effects of the API's ready into one function (on the daemon side, beside OpenWatch)
that both the API handler and `sync_ticket_links` call, so the two paths can't drift again. In
`OpenWatch::tick`, branch on version 2 and prepared: re-announce instead of `unready_stale`.
