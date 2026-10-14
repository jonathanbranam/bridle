---
id: br-5n4f
title: projects start / take-over don't re-create a project's tokens after a fresh database, and say nothing (split from wfcx)
kind: fix
created: 2026-10-14
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-wfcx, br-xqwy]
theme: multi-machine
---

Split from br-wfcx (landed 44ab4a48 without it; its builder listed it as out of scope). Theme multi-machine.

The human asked for this on wfcx, verbatim (to the vesta aide, 2026-10-13 ~9:50 PM ET): "please incude it yes".

## What happened (vesta, 2026-10-13, from the aide's note on br-wfcx)

After the take-overs of the host rename (nuc -> vesta), the daemons came back with fresh databases (old ones kept in each workspace's .bridle/db-backups/2026-10-14-*) and no project tokens except `human`. GET /v1/status with each token in ~/.bridle/credentials.toml at ~9:45 PM ET:

- meta-notes: the aide, advisor, orchestrator and mail tokens accepted (200).
- notes, dotfiles-local, meta-notes-ui: every token refused (401), so the orchestrator's waiters and the aides on those projects were cut off.

`bridle projects start --project notes`, run again, printed only "daemon unit bridle-notes.service enabled and started / notes: daemon is up" and still 401s. Its help says it "re-creates the project's tokens on every machine (the `token pair` code)" after a take-over; it didn't, or only on the first start, which hit the remus Tailscale SSH hang. credentials.toml was last written at 21:21, before the take-overs; daemon.log shows only "wrote human token". The way through was `bridle token pair`.

## The ask

`projects start` (and take-over) re-create the project's tokens reliably, on every run that finds them missing, and say what they did (minted, kept, per machine). When they can't (an unreachable machine, ssh timeout), they fail loudly, name the machine and print the `bridle token pair` command to finish. Test with a fresh database: start re-creates the role tokens; a fake unreachable machine gives the named failure. `just check`. Docs: cli.md, CHANGELOG. Size S-M.
