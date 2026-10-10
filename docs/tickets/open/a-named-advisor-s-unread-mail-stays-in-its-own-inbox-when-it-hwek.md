---
id: hwek
title: A named advisor's unread mail stays in its own inbox when its session ends or restarts (gtzx P4, pulled forward)
kind: bug
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [gtzx, 9aj2, jttf, e35h]
tasks: [br-hwek]
---

## The ask

The human, 2026-10-10 ~10:40 AM ET, verbatim (to advisor product-manager), after a message from
the NUC's notes advisor to the PdM was lost during the PdM's restart:

> Yes, I think we have a lost message here. File this in the incident notes immediately, and
> let's be sure we understand exactly what happened. I haven't looked at the incidents, but we
> should tag things somehow in there about message delivery. Give it a tag, and then, whatever
> tickets come out of this, add them to the epic on message delivery.

And earlier, 2026-10-03/04, on mail to a named advisor that isn't running (ticket
[[seats-every-role-is-a-named-tracked-seat-that-outlives-its-s-gtzx|gtzx]]):

> We have to read them and deal with them later, so it's important they're not lost. They should
> probably just stay where they are, and we should have a note that this advisor, this role,
> this name has been shut down.

## What happened

Incident log entry "2026-10-10 14:36: a named advisor's restart stranded its unread mail" in
`docs/context/incidents.md` (category `messaging`). In short: m-9300 (the human's request,
relayed from the NUC's notes advisor) reached `external:advisor/product-manager` 0.2 s before the
PdM's old session was recorded as ended. `Sessions::emit_ended` (`crates/bridle-daemon/src/
sessions.rs`) moved it, with five other unread messages, to the shared `external:advisor` inbox,
marked "(originally for advisor/product-manager)". The next PdM session reads only its own inbox,
and no main advisor is running, so nobody saw them. The sender was told "delivered".

## The fix (gtzx P4, mail half only)

Pulled out of gtzx so it doesn't wait for the whole seats review:

1. **Unread mail stays in the named advisor's own inbox when its session ends.** Drop the move in
   `emit_ended`.
2. **Mail sent to a named advisor that isn't running stays in its own inbox** too, instead of
   the send-time fallback to `external:advisor` (`server.rs`, `fell_back`). The next session of
   that name gets it from its first `bridle inbox` or wake.
3. **The sender learns the recipient isn't running**, as now ("<name> isn't running"), but the
   wording says the message waits in its inbox.
4. **Recover what is stranded:** move the existing unread "(originally for advisor/<name>)"
   messages in the shared inbox back to their named inbox (a one-off, or done at the named
   session's next start), on every daemon.
5. Update `docs/design/agent-host/principals.md` (the fallback paragraph) and cli.md's `send`
   wording, and the test `named_advisor_addressing_and_delivery_fallbacks`.

Out of scope: the rest of gtzx (seats table, retire, names for every role), and the waiter
acknowledgement (9aj2 item 1, br-3zhx), which fixes a different loss.

## Open

- A name used once and never again (a one-off advisor) keeps unread mail forever. Acceptable:
  the human wants it kept, and `bridle inbox` for that name shows it.
