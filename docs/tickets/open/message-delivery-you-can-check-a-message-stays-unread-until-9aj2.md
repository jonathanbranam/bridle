---
id: 9aj2
title: "Message delivery you can check: a message stays unread until the session has seen it, a recent-messages command for any principal, and a full delivery audit trail in the event log"
kind: feature
opened: 2026-10-10
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [75h2, xxxq, 3haz]
tasks: [br-9aj2]
---

## The ask

The human, 2026-10-10 ~8:15 AM ET, verbatim (to the aide), after the aide lost a message to a
waiter started wrongly:

> Let's try to improve this and it feels like we may need some improvement in the guarantees of
> our message delivery if this happens. How did you notice it was started incorrectly? How often
> do agents do this? How do you check for missed messages?

> yes file that ticket; as part of that, how are agents "checking for missed messages" today?
> There should be an easy command for an agent to run that they can use to see something like
> "the last 5 messages" or "messages that were delivered in the last N minutes". Default would be
> "for me" based on the role/name; agents should also be able to run that for any agent or human
> role; that might be easy in the event log or something specific with messages; also - message
> delivery and receipt should be in the event log so that we have an audit trail for
> investigations.

## What happened

2026-10-10 08:01Z, the dalek aide started `bridle agent wake external:aide` with a shell `&` and
its output sent to `/dev/null`, against its role file ("never with `&`, never with its output
discarded"). The aide noticed by rereading its own command; nothing in the system caught it. That
waiter got m-8999 (the orchestrator's morning summary for the human, asking the go for br-88d4,
br-751e, br-hdbj), which was marked read at 08:01:06Z, and the output was lost. The aide found the
loss only by listing its messages with their read times through the HTTP API
(`GET /v1/messages?to=external:aide`) and matching them against what it had seen. Later messages
repeated the content, so no harm this time. The same aide did this twice more on 2026-10-09
(handover h-0092); the same warning sits in the aide, advisor and orchestrator role files. How
often other agents do it is not recorded anywhere.

## Facts (2026-10-10)

- A wake waiter marks a message read the moment it hands it over (`workflow/base/roles/aide.md`,
  "Waiting for messages"). If the output is lost, so is the message; the sender is not told.
- `bridle inbox` lists unread messages to the caller; `--all` adds read ones, with no limit, time
  window or other principal. No CLI shows another principal's messages; the human's are read
  through the HTTP API only.
- The event log already has `message.sent` (actor, message, to), `message.delivered` and
  `message.read` (actor, message). The read event doesn't say how the message was read (which
  waiter, session or command), so it can't tell a delivery to a lost waiter from a real read.
  `bridle events --kind message.` returns only a recent window and can't filter by recipient or
  message.

## The ask, itemised

1. **A message stays unread until the session has seen it.** Recommended: a waiter prints the
   message but marks it read only when the same session next runs `bridle agent wake` or
   `bridle inbox` (an implicit acknowledgement), so a waiter whose output is lost loses nothing:
   the next wait delivers it again. Open: how long before an unacknowledged delivery counts as
   lost and is offered again, and whether the sender hears.
2. **One easy command for recent messages**, for example `bridle messages [--for <principal>]
   [--last 5] [--since 30m]`: by default the caller's own (from its role and name), with sent,
   delivered and read times and how each was read. Any agent may run it for any agent or human
   role (read-only; who may read whose bodies is open: maybe headers for others, bodies for
   one's own).
3. **A full audit trail in the event log**: send, delivery and receipt (read or acknowledged),
   each with who, when and the channel (waiter pid and session, `inbox`, the UI, mail), so an
   investigation can trace one message end to end, and `bridle events` can filter by message id
   and by recipient.
4. Refuse `bridle agent wake` run as a shell background job (`&`) or with its output discarded,
   the way the kill guard refuses kills by pattern (rule `no-kill-by-name`).
5. Update the role files' "Waiting for messages" sections: how to check for missed messages.
