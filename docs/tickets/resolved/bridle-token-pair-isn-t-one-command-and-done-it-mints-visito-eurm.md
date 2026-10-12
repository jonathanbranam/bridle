---
id: br-eurm
title: "bridle token pair isn't one command and done: it mints visitors with no home, so replies don't come back"
kind: bug
created: 2026-10-12
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-sk7p, br-n7cg, br-r2nr, br-v7ug, br-n63z]
theme: multi-machine
parent: br-kx5u
resolved: 2026-10-12T01:36:33Z
---

## The ask

The human, 2026-10-11 ~8:00 PM ET (relayed by advisor nuc-rename, m-10810), verbatim:

> pair is supposed to mint both tokens both ways. I should be able to run it once and be done ... 1. set up machine 2. set up ssh 3. run pair and list projects to pair 4. it works

What advisor nuc-rename found (2026-10-11):

- `bridle token pair` mints every visitor with no home: `crates/bridle/src/commands/misc.rs:883-887`, `PairMint` sends `home: None`. Without a home, a reply to e.g. `external:orchestrator@remus` stays in an inbox on the dalek daemon instead of being forwarded to remus (`docs/design/agent-host/principals.md`, "Replies come home").
- sk7p's approved ask required replies to come back: "a message from human@nuc is answered to human@nuc, and shows in the human's inbox on the NUC" (`docs/tickets/resolved/pair-machines-token-setup-over-ssh-sk7p.md:65-66`).
- Cause: pair landed 2026-10-09 (br-8c25, br-jw9e); visitor homes (`--home`) landed a day later, 2026-10-10 (br-n7cg). pair was never updated, and `docs/design/specs/token-pairing.md` doesn't mention home.
- Result: the docs workaround is "revoke, then `token create --machine X --home P` by hand" (ticket r2nr step 2 says don't use pair). This blocks remus setup (v7ug) and the NUC->vesta rename (r2nr) from being one command.
- Open design point for whoever plans it: which project on the holder machine is each visitor's home (the same-named project there, the project the role lives in, e.g. bridle for orchestrator, or a `--home` default).
- Also check: after the rename, `pair --rotate` should re-mint the role@vesta visitors with homes, so r2nr step 2 can use pair.

Done when: on a new machine, after setting it up and SSH, one `bridle token pair` run naming the projects mints every token both ways with homes, and messages and replies flow both ways with no hand-made tokens.
