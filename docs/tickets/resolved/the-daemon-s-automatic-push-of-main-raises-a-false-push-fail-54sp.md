---
id: br-54sp
title: The daemon's automatic push of main raises a false 'push failed' alarm when the manager already pushed
kind: bug
created: 2026-10-13
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-16T10:02:58Z
---

## The ask

Twice on 2026-10-12 (06:16Z, and 00:14Z on the 13th, two alarms) the daemon's automatic push of `main` was rejected with "cannot lock ref 'refs/heads/main': is at X but expected Y" and messaged the orchestrator "The landing is on the local branch only; find out why before landing more." Each time local and origin `main` already agreed: manager-2 had pushed the same commits itself. Logged in docs/context/incidents.md (06:16Z entry).

A false alarm that tells agents to stop landing costs an orchestrator check each time and trains everyone to ignore a real one.

Fix: when the automatic push is rejected, re-fetch; if origin's `main` equals or contains local `main`, stay quiet (debug log only). Alarm only when local has commits origin lacks after the re-fetch.

Check: a test with a remote already at the local tip (pushed by someone else between fetch and push) gives no message; one where local is ahead and the push is refused still alarms.

Model: Sonnet.
