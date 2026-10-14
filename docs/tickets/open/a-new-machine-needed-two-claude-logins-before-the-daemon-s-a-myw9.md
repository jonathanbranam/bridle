---
id: br-myw9
title: A new machine needed two claude logins before the daemon's agents were authenticated
kind: bug
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
theme: multi-machine
---

## The ask

The human, 2026-10-14 (relayed by the orchestrator on remus, m-12833), verbatim:

> I had to login twice the first time on a new machine to get the background workers
> authenticated; not sure why.

## What happened

On remus (remus-home, a new machine), manager a-hj6iz crashed after the daemon restart at
17:51Z with "claude is not logged in in this daemon's session" (`NOT_LOGGED_IN_REASON`,
`crates/bridle-daemon/src/supervisor.rs`). The human logged in; a fresh manager (a-rs7np)
started at ~18:12Z. The human reports two logins in total were needed.

## What's unknown

Why the first login didn't take for the daemon's agents: a different keychain/session than the
daemon's (launchd vs. the login shell), a login before the daemon's restart, or a credential
written where spawned agents don't read it. Needs a look at how `serve.rs`'s login check and the
spawned `claude` find credentials on a fresh machine, and a note in the add-a-machine steps once
known.

## Not critical

remus works now. Worst case if not fixed: each new machine's first setup costs the human one extra
login and a crashed manager.
