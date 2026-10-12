+++
id = "br-86qp"
title = "53gy C: project start (systemd --project), token re-pair in take-over (eurm's code), scheduled messages carried over"
kind = "feature"
state = "planned"
created_at = "2026-10-12T01:04:04.447Z"
updated_at = "2026-10-12T01:05:21.965053Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
]
parent = "br-53gy"
+++

Slice of br-53gy (pm-1's split, agreed by advisor product-manager 2026-10-11 ~9:10 PM ET). The brief is br-53gy's ask and thread, plus br-3pc2's 'What 53gy must add' and the human's move decisions (comment on br-53gy). Order A -> B -> C, behind br-2wnn.

## Thread

### note · agent:pm-1 · 2026-10-12T01:04:50.956Z
Build brief (pm-1). Model: Sonnet. Slice C of br-53gy (A br-ga6s -> B br-n4ug -> C); read br-53gy's thread and br-3pc2 "What 53gy must add". Blocked by br-n4ug.

Do: `bridle project start`: install the unit if missing, enable (boot, restart on failure) and start it; local, `--machine <m>` over SSH. systemd --project support per br-3pc2 (units for a project on Linux). In take-over (slice A): re-pair tokens by calling br-eurm's pair code (no second minting path); carry schedules to the new machine (per the human's decision on br-53gy); count undelivered mail and offer stop/continue (--yes continues). Help text for the whole transfer: 1 stop here, 2 nothing to give up (stop pushes state), 3 take over there, 4 start there.
Docs: cli.md, the add-a-machine guide, CHANGELOG. Acceptance: just check passes; tests with launchd/systemd/SSH seams: start installs when missing and is idempotent, schedules carried, mail count shown, re-pair called.
Migration: none. Out of scope: bridle.db copy (decided no), kt25 (one-command move, built from these), evy6.
