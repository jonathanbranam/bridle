---
id: br-jp52
title: Every project daemon on a machine follows the installed bridle binary, not just the self-upgrading one
kind: bug
created: 2026-10-12
created_by: external:advisor/product-manager
theme: reliability
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kyjg, br-36dp, br-kuw2]
---

Found 2026-10-12 by the orchestrator while fixing br-kyjg's ops part, verbatim: "bridle-ui, data-contracts and track-web daemons on dalek were all on 0.6.1, because only bridle's own .bridle/config.toml sets self_upgrade = true, so the other daemons never restart onto the shared installed binary." The skew caused br-kyjg (a 0.6.3 CLI calling a route the 0.6.1 daemon lacked).

## The problem

On a machine there is one installed `bridle` binary but one daemon per project. Only a daemon with `self_upgrade` on (here bridle's, which builds and installs from main) restarts onto a new binary; every other project's daemon keeps running the old one until someone restarts it by hand. The CLI, the gateway and the agents use the new binary at once, so they talk to old daemons. br-36dp (machine-wide `[daemon] self_upgrade` in ~/.bridle/config.toml) exists but isn't set on dalek, and on a dev machine "build from main" is right for bridle's daemon only.

## The ask

Every daemon on a machine runs the installed binary within a short time of it changing, without a human step:

- A daemon notices the installed binary changed (a newer version or build than its own) and restarts onto it at its next safe point (the same drain as self-upgrade), whether or not it builds or fetches upgrades itself.
- Or the daemon that installed it restarts the others (one machine-wide step); the planner picks, KISS.
- `bridle status` (and the orchestrator's view) shows a daemon running an older binary than the installed one.

Related: br-kyjg (the named error for an unmatched route), br-36dp (machine-wide self_upgrade), br-kuw2 (machine daemon).
