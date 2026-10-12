+++
id = "br-n4ug"
title = "53gy B: project stop with confirmed pushes and released mark, pre-stop listing, old units unloaded, schema guard; take-over requires the mark"
kind = "feature"
state = "open"
created_at = "2026-10-12T01:04:04.403Z"
updated_at = "2026-10-12T01:04:43.500778Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
]
parent = "br-53gy"
+++

Slice of br-53gy (pm-1's split, agreed by advisor product-manager 2026-10-11 ~9:10 PM ET). The brief is br-53gy's ask and thread, plus br-3pc2's 'What 53gy must add' and the human's move decisions (comment on br-53gy). Order A -> B -> C, behind br-2wnn.

## Thread

### note · agent:pm-1 · 2026-10-12T01:04:43.500Z
Build brief (pm-1). Model: Sonnet. Slice B of br-53gy (A br-ga6s -> B -> C br-86qp); read br-53gy's thread and br-3pc2 "What 53gy must add". Blocked by br-ga6s (same files).

Do: `bridle project stop`, one command, the same on every machine and OS (launchd, systemd): stops the project's agents, its daemon and its boot/restart unit (unload, leave the file installed), then pushes the state branch and the integration branch's direct commits and confirms origin has them. Any failure: say so plainly, exit non-zero, leave the unit as it was, never report success. Pre-stop listing of what is lost or left behind (per the human's decisions on br-53gy), with a confirm and a --yes flag. Write a 'released' mark in owner.toml as stop's last pushed commit; take-over (slice A's seam) then requires it, with --force for a dead machine. Unload the old machine's units per br-3pc2. Schema guard per br-3pc2. `--machine <m>` over SSH like token pair.
Docs: cli.md transfer section (what happens to ownership at each step, in the human's order), daemon.md, CHANGELOG. Acceptance: just check passes; tests: failed push exits non-zero and changes nothing, mark written after confirmed push, take-over refuses without it and --force overrides. Use existing launchd/systemd/SSH seams, no real ones.
Migration: none to files; owner.toml gains the mark field, absent means not released (take-over refuses, --force). Out of scope: start, re-pair, schedules, bridle.db.
