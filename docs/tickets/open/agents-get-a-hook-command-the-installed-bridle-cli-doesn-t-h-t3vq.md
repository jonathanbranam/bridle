---
id: t3vq
title: Agents get a hook command the installed bridle CLI doesn't have
kind: bug
opened: 2026-10-06
repos: [meta-notes]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask


## The ask

Found by the meta-notes orchestrator on the NUC, 2026-10-06 00:20 UTC. A
worker spawned by the meta-notes daemon (task mn-s36r) had every Bash call
refused:

```
PreToolUse:Bash hook error: [bridle kill-guard]: error: unrecognized subcommand 'kill-guard'
```

The daemon's `--settings` for agents now includes the `bridle kill-guard`
PreToolUse hook (br-75h2, adb4d30f, 2026-10-05 16:45 UTC). The `bridle` on
the NUC's `PATH` (`~/.cargo/bin/bridle`, 0.5.0, installed 2026-10-05
10:39 UTC) predates it. Workers spawned an hour earlier ran fine; the
manager, spawned before the change, is unaffected. How a daemon started on
2026-10-01 renders the new hook isn't known.

## Wanted

A daemon never hands agents a hook the installed CLI can't run: check at
spawn (e.g. `bridle --version`, or the subcommand list) and refuse the spawn,
or leave the hook out with a warning; and say so in `bridle status`. A
failing hook should fail open with a clear message rather than block every
Bash call.

## The other direction (2026-10-06 02:10 UTC)

After the human reinstalled bridle (29296901), the skew reversed: the CLI is
newer than the four NUC daemons (running since 10-01 and 10-04). `bridle send
--project <other>` now posts to the caller's own daemon at `/v1/outbox`
(br-3haz), which the old daemons don't have. The 404's empty body surfaces
as `error: unknown:` with nothing else, so it looks like a refusal. The
meta-notes manager couldn't message its worker. Workaround: run the send
from the target project's workspace, so it posts `/v1/messages` directly.
The CLI should fall back to (or at least name) a missing endpoint, and an
empty error body should print the HTTP status.
