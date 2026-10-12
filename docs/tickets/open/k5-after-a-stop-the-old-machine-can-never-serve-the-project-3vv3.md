---
id: br-3vv3
title: "K5: after a stop, the old machine can never serve the project (tools-only entry, registry, upgrade tree)"
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-kt25]
parent: br-kx5u
---

Child of epic br-kx5u (MVP), from the design in br-kt25 ("## Design", child K5).

## The ask

After `project stop` succeeds on OLD: add P's clone to `[machine] tools_only` (so OLD can never serve it: `tools_only.rs`), remove P's registry entry, prune the upgrade worktree and `upgrade-*` tree; leave `bridle.db` in place and say so (events and usage can still be read there).

## Acceptance

On OLD after stop: `bridle serve` for P refuses; `.bridle/bridle.db` still exists; running it twice changes nothing; `just check` passes.
