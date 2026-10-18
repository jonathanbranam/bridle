---
id: br-4n3v
title: A worker's ticket claim disappears mid-work, so its 'ticket built' is refused (2fqg's cause, still happening)
kind: bug
created: 2026-10-18
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-2fqg]
---

## What happened

- br-r965: wr965 was spawned on it and the queue showed it claimed at 06:02Z. After the 06:15Z self-upgrade restart the row was back to planned/startable, and the resume note told wr965 'You have no claimed task'. manager-2 landed it anyway.
- br-4mbd: w4mbd, spawned ~08:45Z, no restart before it finished. At 09:10Z its 'bridle ticket built' was refused with 'forbidden: worker agents cannot use agent lifecycle endpoints', and it noted 'the ticket shows planned, not claimed'. manager-2 marked it built by hand (thread of br-4mbd).

br-2fqg (landed cff0bb04) found the refusal comes from an unclaimed planned ticket and had worker.md claim first. The claim still goes missing after the worker starts, so the refusal is back.

## The ask

Find what drops a worker's claim on its ticket (restart resume, a stale-claim sweep, the land of another ticket, the manager's queue edits, or the PdM's re-plan) and stop it; a claim should last until the ticket is built or the worker is removed. If a claim is released on purpose, the worker should be told, and 'ticket built' from the worker that was spawned on the ticket should still work.

## Verify

A worker spawned with --ticket keeps the claim across a daemon restart and through other tickets landing, and its own 'ticket built' succeeds; a test for each.
