---
id: br-d4bz
title: daemon restart reports 'did not come back within 95s' when it came back in seconds
kind: bug
created: 2026-11-17
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

`bridle daemon restart` must report success when the daemon came back, and not "the daemon did not come back within 95s" for a daemon that was healthy within seconds.

Seen twice on remus with the flume daemon (127.0.0.1:7412, also bound to its tailnet address), 2026-11-17 04:40 UTC and 13:36 UTC (reported by orchestrator@remus): `BRIDLE_PROJECT=flume bridle daemon restart` printed "restarting at <sha>: 1 agent to resume" and then "error: the daemon did not come back within 95s; see .../daemon.log". The log shows it back within 2 s (13:36:37 restart requested, 13:36:39 state branch at start), and `bridle status` worked at once. Impact: a false failure makes an operator dig into a healthy daemon or restart it again.

## Likely cause (pm-1 read the code; the worker confirms with a test)

crates/bridle/src/commands/daemon.rs, `wait_for_restart` (about line 76): success needs `down` to be seen first (a health poll that fails), then a health poll that succeeds. The polls are 500 ms apart. If the daemon goes down and comes back between two polls (a fast restart, which is the good case), `health()` never fails, `down` stays false, and the loop runs to the deadline (stop_limit_secs + 60 = 95 s) and reports a failure. The submitter's guess (a mismatched address or identity) is the other candidate; the log timings fit the first better.

## Fix

Detect a restart without having to catch the down window: for example compare something that changes per daemon process (a start time or pid or boot id from the health or status reply; check what the health response carries, and add it to the wire only if nothing suitable exists: then crates/bridle-api/src/types.rs, all clients and the daemon together) before and after, and return "the daemon is back" once it differs and answers. Keep the existing down-then-up path working and the error (with the log hint) for a daemon that really does not return. Keep it simple: the smallest change that closes the race.

No project-file change, no migration. Docs: docs/design/cli.md where `daemon restart` is described, one line if behaviour changes.

## Acceptance

Tests next to the existing `wait_for_restart` tests: a daemon that restarts between two polls (never seen down) reports back; one that goes down and up reports back; one that never returns errors after the limit with the log hint. `just check` with BRIDLE_* env unset. Model: Sonnet.
