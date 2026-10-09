---
id: rztb
title: "Incident: something keeps restarting dalek's gateway outside launchd from a Claude session (pid 88281 since 10-08 9:39 PM); recurring"
kind: incident
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [57nt, bek3, 76td, ppa6]
tasks: []
---

## The ask

The human, verbatim (2026-10-09 ~3:25 PM ET):

> ok, this is a repeated, recurring incident. I NEVER started those gateways. They're being started by some process, I presume it is the bridle upgrade process. This needs to be investigated; I have reported this before.

Earlier reports: [[the-gateway-runs-detached-and-keeps-itself-current-like-the-bek3|bek3]], [[the-gateway-records-its-pid-and-has-stop-and-restart-command-76td|76td]], [[gateway-inherits-bridle-as-bridle-project-from-the-shell-tha-ppa6|ppa6]], [[bridle-gateway-restart-takes-a-launchd-managed-gateway-out-o-57nt|57nt]] (fix landed 2026-10-09 3:06 PM ET), and the 2026-10-08 10:37 AM-6:46 PM outage (a gateway started with `BRIDLE_AS=orchestrator`).

## Facts (checked by the aide, 2026-10-09 ~3:25 PM ET)

- The gateway serving `:7878` on dalek is **pid 88281, ppid 1, started Thu 2026-10-08 9:39:49 PM ET**, env `CLAUDECODE=1`, `CLAUDE_CODE_ENTRYPOINT=cli`: started from a Claude Code session, not by launchd and not by the human. `~/.bridle/gateway.pid` says 88281.
- `gateway.log` 01:39:49Z: "SIGTERM; shutting down", then a new "gateway listening" 46 ms later: a stop-and-start (`bridle gateway restart` or `--detach`), not a crash.
- The same SIGTERM-then-restart appears at 2026-10-07 22:08Z, 23:09Z, 2026-10-08 14:37Z, 22:46Z (twice), 22:51Z, 23:14Z, 23:50Z, 2026-10-09 01:39Z. Which of those were launchd and which were a Claude session isn't in the log: it doesn't record who asked.
- Since the restart the gateway has stayed alive through upgrades by re-executing itself in place ("the bridle binary changed; restarting onto it"), which keeps the pid and the inherited environment.
- launchd's `dev.bridle.gateway` has been failing since ~2:54 PM ET today with "Address already in use" (100+ runs, state "spawn scheduled"), most likely kickstarted by a br-57nt test run, so launchd is supervising nothing.
- The aide's search of `~/.claude/projects` transcripts found no `gateway restart` near 9:39 PM; bridle-spawned agents may keep transcripts elsewhere.

## The ask

1. Find what started pid 88281 (and the other restarts): which session or code path (the upgrade, `just install`, an agent, a worker test) ran `gateway restart`/`--detach` against the live gateway.
2. Fix it so nothing but launchd (or systemd) starts the live gateway, and tests never touch the real one.
3. Log who asked for each gateway stop or start, so the next one can be traced.
4. Then hand the port back to launchd (stop 88281), with the human's OK.
