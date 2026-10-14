---
id: br-nbgp
title: Internal upgrade to 0.7.2 dropped every agent token on one daemon (vesta/meta-notes); only the human token worked
kind: bug
created: 2026-10-14
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
theme: reliability
---

## The ask

Reported by meta-notes' aide on vesta, 2026-10-14.

## What happened
- 06:49 AM ET (10:49 UTC): meta-notes' daemon on vesta restarted for the internal upgrade to v0.7.2 (`restart requested via POST /v1/restart principal=system`).
- After the restart, every external token in ~/.bridle/credentials.toml for meta-notes got 401 `invalid token`: aide, orchestrator, advisor and mail. Only the human token worked; the daemon rewrote it at start (`wrote human token`).
- The other three vesta daemons (notes, meta-notes-ui, dotfiles-local) did the same upgrade and kept their tokens.
- The aide's waiter exited 6 ("re-arm once it is back"), but re-arming was impossible. The orchestrator's meta-notes waiter was locked out too.

## Fix that worked
The human ran `bridle token pair`, and all four tokens returned 200 again. Output: "machine vesta (role tokens): minted 4, kept 32" and "machine vesta (peer tokens): minted 1 ... peer: minted on the receiver for vesta -> vesta/meta-notes", plus "dalek (peer tokens): minted 1 ... dalek -> vesta/meta-notes". So the meta-notes daemon had lost the principals that pair minted last night (Oct 13, after the nuc -> vesta take-over).

## Possibly related
- meta-notes' bridle.db was replaced at 21:34 on Oct 13 during the take-over (db-backups/2026-10-14-*). Pair ran after that, so the tokens it minted should have been in this db.
- The same start logged: `migration 0006-backfill-ticket-kind skipped, will retry at next start: main is a protected branch of meta-notes`.
- Earlier incidents with the same shape: 90fc1df (a manager's Bash denied after a daemon restart) and 91c0238 (take-over left daemons with no ticket rows).

## Asks
1. An upgrade or restart must keep the principals and tokens.
2. When agent tokens fail and only the human token works, raise an incident naming `bridle token pair` as the fix, instead of every role silently failing with a 401.

