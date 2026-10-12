---
id: br-yt5v
title: "K4 follow-up: the project drain's hold survives a daemon restart"
kind: fix
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-kefh, br-ep5z]
parent: br-kx5u
---

Child of epic br-kx5u (MVP). Found by the K10 runbook (br-ep5z, worker wep5z, 2026-10-12): "drain hold in memory".

K4 (br-kefh) holds new work for the moving project, and the human's Q5 answer (verbatim on br-kt25) is that "the hold does not expire for the MVP". The hold lives only in the daemon's memory, so a daemon restart (a self-upgrade, which happens several times a day; or a crash) lifts it silently mid-drain, and new work can start on a project that is moving.

## The ask

The project hold survives a daemon restart: it is stored (for example in the database's meta table) and restored at start, and lifted only by `project drain --undo` or by start on the new machine. `bridle status` shows it.

## Acceptance

Hold, restart the daemon (test harness), and a claim or spawn is still refused; `--undo` lifts it; `just check` passes.
