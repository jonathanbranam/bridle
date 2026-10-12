+++
id = "br-eurm"
title = "bridle token pair isn't one command and done: it mints visitors with no home, so replies don't come back"
kind = "bug"
state = "pending"
created_at = "2026-10-12T00:03:22.445Z"
updated_at = "2026-10-12T00:14:51.754150Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "eurm"
+++

docs/tickets/open/bridle-token-pair-isn-t-one-command-and-done-it-mints-visito-eurm.md

## Thread

### note · external:aide · 2026-10-12T00:13:27.353Z
From the human, via aide (2026-10-11 ~8:20 PM ET), verbatim: "then i should revoke and recreate all tokens with a home. it that going to be a single command? Or what? That is what I really want." Added to the ask: the same one command must also fix existing pairings: re-mint every visitor token that has no home (made before n7cg, or by pair) with its home, for all projects on both machines, and update the holder side credentials, so the human never revokes and re-creates tokens by hand. (pair --rotate is the obvious place.)

### note · external:aide · 2026-10-12T00:14:27.305Z
From the human, via aide (2026-10-11 ~8:25 PM ET), verbatim: "OMG it is SOOOO hard. I just want a list of projects and which machines they are on. Then I run \"pair everything\" and it all gets paired assuming I have tokens." So the target is: one list of projects and the machine each lives on (see evy6, one placement record), and one command ("pair everything") that reads it and mints, re-mints and installs every token both ways, with homes, given the human's own tokens. No per-project, per-role or per-direction steps.

### note · external:advisor/product-manager · 2026-10-12T00:14:46.544Z
PdM (advisor product-manager): recommended shape for the human's 'pair everything' (aide m-10816). The list of projects and their machines already exists: [projects] + [machines] in ~/.bridle/config.toml (used by discovery, the gateway and cli.md). So eurm need not wait for evy6: one command (e.g. 'bridle token pair --all', or bare 'bridle token pair' with no projects named) reads that list, and for every project on every other machine mints, re-mints (no home, or stale machine name) and installs every token both ways with homes, using the human's own tokens over SSH. Default home: the same-named project on the holder machine, else the project the role lives in (bridle for the orchestrator). Idempotent: run it again any time, after a rename or a new machine. evy6 later replaces the source of the list, not the command. Waiting on the human to ready.

### note · external:aide · 2026-10-12T00:14:51.754Z
From the human, via aide (2026-10-11 ~8:30 PM ET), verbatim: "either that or just get rid of all the tokens b/c they don't provide any actual security. It could be one token per machine". So an acceptable alternative to "pair everything": drop the per-project, per-role visitor tokens for machine-to-machine traffic and use one token per machine. Compare n63z (the human, 2026-10-05: "This whole thing smacks of security theater") and 3827 (security and hardening); the PdM should decide which way with that in view, and bring the choice to the human.
