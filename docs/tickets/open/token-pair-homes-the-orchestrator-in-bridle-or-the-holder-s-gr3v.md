---
id: br-gr3v
title: "token pair homes the orchestrator in 'bridle' or the holder's first project, never <machine>-home: vesta's orchestrator is homed in dotfiles-local and needs a waiter per daemon"
kind: bug
created: 2026-10-16
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-8usa, br-8uxe, br-rnhp]
---

## The ask

Reported by orchestrator@vesta (m-13905, 2026-10-15 ~9:57 PM ET), "the human asked me to check with you": "should the vesta orchestrator need only ONE waiter? Right now the remote daemons (bridle, bridle-ui, data-contracts, remus-home, track-web) report visitor_home=dotfiles-local for external:orchestrator@vesta even after the human re-ran 'bridle token pair'; this machine's home is now vesta-home. And the local daemons (dotfiles-local, meta-notes, meta-notes-ui, notes) give me direct tokens with no forwarding, so I'm running 5 waiters (vesta-home + those 4)."

Cause (aide, from the code): `home_for` in crates/bridle/src/token_pair.rs (~586) picks the visitor's home as: the same-named project on the holder's machine, else the role's `home` in `TOKEN_ROLES`, which is hard-coded "bridle" for every role, else the holder's first project. vesta has no bridle project, so the orchestrator is homed in its first project, dotfiles-local. `<machine>-home` (br-8uxe) is never considered, so br-8usa's "a principal that moved to <host>-home follows on a re-pair" can't happen for the orchestrator. On dalek it is homed in bridle, not dalek-home, for the same reason.

Ask:
1. The orchestrator's home on a machine is its `<machine>-home` project when that machine has one; re-pairing then moves existing visitors there (br-8usa's re-mint path).
2. Same-machine: today each local project daemon gives the orchestrator a direct token, so it needs one waiter per local daemon. Decide whether local daemons should forward the orchestrator's mail to `<machine>-home`, so one waiter on the home daemon is enough (the role prompt's "one orchestrator per machine" and br-qca7's "listen only on your own daemon" assume this).
