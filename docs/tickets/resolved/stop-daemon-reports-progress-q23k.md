---
id: q23k
title: stop-daemon says what it's doing, and when it's done
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: []
closed: 2026-10-02T00:43:36.487917Z
---

## The ask

The human, verbatim (2026-09-29, via the advisor), after `bridle stop-daemon` on bridle took
7-8 s to return after the daemon's WARN line:

> ok, this seems like a behavior change; stop-deamon used to return quickly which the actual
> daemon took a while to shut down, which is why I asked for that message change. If the
> behavior is different, then the message should be updated; we shouldn't assume the user is
> tailing the daemon logs;
>
> You're saying there is traffic between stop-daemon and the daemon? Can stop-daemon then
> report on progress?
>
> requested shutdown
> shutdown request acknowledged, daemon is shutting down, may take ups to xxx seconds
> shutdown is complete

## What changed (advisor)

- `stop-daemon` (`crates/bridle/src/commands.rs:460`) posts `/v1/shutdown`, then polls
  `/v1/health` every 0.5 s until it fails (60 s limit), and only then prints.
- 67489eb (2026-09-28 01:41 ET, "Fix Linux shutdown race") made the daemon close its HTTP
  listener only after cleanup (`crates/bridle-daemon/src/lib.rs:455-490`): stop all agents (up
  to `stop_grace` + 5 s = 35 s), flush the task state branch, remove the registry entry and
  `daemon.json`. Before it, the listener closed at the start, so the poll ended almost at once.
- dc22de3 (01:52 ET, eleven minutes later) changed the message from "daemon stopped" to
  "received; shutting down gracefully, may take up to 30s", which described the old timing.
  Now it prints only after shutdown is complete, so it's wrong.

## Shape (KISS)

Print as it goes, not only at the end, e.g.:

```
requested shutdown
acknowledged; the daemon is stopping N agents, up to 35s
shutdown complete (8s)
```

- The limit in the second line comes from the daemon's `stop_grace` + 5 s, not a hard-coded
  30.
- Health still answers during shutdown, and `Health` has `agent_count`; printing when the count
  drops ("2 agents still running") is optional.
- On the 60 s timeout, say so and point at `bridle daemons` and `<workspace>/.bridle/daemon.log`.
- Update `docs/design/cli.md` to match.

## Resolution

Resolved by: br-7101 (1059ed6), br-ef49 (1059ed6)
