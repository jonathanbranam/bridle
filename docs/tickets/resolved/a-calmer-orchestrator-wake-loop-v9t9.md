---
id: v9t9
title: A calmer orchestrator wake loop: restart the waiter first, 25-minute poll, 15-minute alert
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [a-leaner-orchestrator-pdmd, incident-notices-that-retract-themselves-nc7r]
closed: 2026-10-02T00:43:36.592312Z
---

## The ask

On 2026-09-30 at 01:36Z the human got m-2136, "The orchestrator has no wake command running
(`bridle wait-for-wake`) since 01:34:39 UTC": the orchestrator was busy with a burst of advisor
messages and restarted its waiter a few minutes later. The human, verbatim (2026-09-29, via the
advisor):

> tune that alert - I don't want to get a bunch of spurious alerts like that; only alert me if
> it is 2x past the time

> it checks every 2 min about whether the orch has a wake command running? That seems
> aggressive. Even at 4min that feels aggressive. Why check so much?

> I think the main thing is if the orch isn't waking up for 20-30 min, we should do something
> about it; do we know the last time it woke up? I mean, it woke up and was really busy, what is
> the wake from bridle doing? I though the orch had to have its own wakeup? Is this a hearbeat
> from bridle to check the orch is alive?

> that all sounds good, except the 25 to 30min worries me a little; if nothing is happening, a
> wake should happen at 25, right? so we're giving 5 extra minutes after that? and what are we
> waiting for? For orch to start another agent to wake itself? Is that not the first thing it
> does?

> sure, that sounds good, but raise to 10min to 15min; no need to be pushy here.

## Today (at 02d1faf)

- `wait-for-wake` is a long poll that returns `nothing` after 5 minutes
  (`crates/bridle-daemon/src/wake.rs`, `POLL_TIMEOUT`), so an idle orchestrator takes a turn every
  5 minutes just to restart it, spending context.
- `workflow/base/roles/orchestrator.md` ("Watch, don't poll by hand"): "read what it printed,
  act, and run it again". The waiter is restarted *after* the work, so a busy orchestrator has
  no waiter for minutes.
- `[orchestrator] waiter_grace` (default 2 m, `config.rs`): no waiter for longer than that sends
  the human a notice. That's what fired.
- The orchestrator's own 30-minute heartbeat: the role file no longer mentions it (pdmd), but the
  startup steps built into the binary still do (`crates/bridle/src/commands.rs`,
  `ORCHESTRATOR_STARTUP_STEPS`: "Start the watcher ..., plus a 30-minute heartbeat").
- When the orchestrator last woke isn't recorded anywhere visible: no event, nothing in
  `bridle status`.

## Shape

1. **Restart first.** The orchestrator's first action on any wake is to start `bridle
   wait-for-wake` again, then act. Wakes queue, so nothing is lost; the gap becomes seconds.
   Update the role file.
2. **Poll timeout 5 m -> 25 m.** An idle orchestrator wakes about twice an hour, not twelve times.
3. **`waiter_grace` default 2 m -> 15 m.** The notice then means the orchestrator really forgot
   (or one very long stretch of work). This replaces the advisor's earlier request (m-2238) to
   set it to 4 m.
4. **Last wake is visible:** `bridle status` shows when the orchestrator's last wake was
   delivered (and whether a waiter is open).
5. **No separate heartbeat.** The 25-minute poll does its job; drop "plus a 30-minute heartbeat"
   from the startup steps and the heartbeat line from the handover steps.

## Resolution

Resolved by: br-ee04 (3baf657)
