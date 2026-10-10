---
id: ve74
title: A handover request reaches the new session after the old one already handed over
kind: bug
opened: 2026-10-10
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [5j35, cbbn, gtzx]
tasks: [br-ve74]
---

## The ask

The human, 2026-10-10 ~7:15 PM ET, verbatim (to the aide):

> I'm just recording this for investigation. I've seen at least two agents say that they received a
> message about their handover needing to happen in the new session, in the handover session, and
> they said they've ignored it since it was since they had just handed over. So that's something to
> low priority investigate. Ask the product manager to put it in the roadmap somewhere.

Low priority (the human).

## One instance (the aide, 2026-10-10)

The aide's own restart today:

- 17:23:03Z m-9411 (system): "Your context is 200k tokens ... run `bridle session restart aide`".
- 17:23:08Z m-9412 (from `external:aide`, sent by `bridle session restart`): "The human is
  restarting this session. Write a handover note ...".
- 17:23:33Z the old session wrote handover h-0099 and the restart followed.
- 17:23:45Z m-9412 was marked read, 12 s **after** the handover: by the new session's first
  `bridle inbox`. So the old session handed over without reading the request, and the request
  then went to the new session, which had nothing to hand over.

## Questions for the investigation

1. Should a handover request expire, or be marked handled, once the session it was sent to has
   handed over (or been replaced)? Today it is addressed to the identity, and identities outlive
   sessions.
2. Does br-3zhx (landed 2026-10-10: a waiter's message stays unread until the same session sees
   it) make this more likely, by keeping such a message unread across the restart?
3. Do the 200k context notes (m-9411 and the like) have the same problem?
