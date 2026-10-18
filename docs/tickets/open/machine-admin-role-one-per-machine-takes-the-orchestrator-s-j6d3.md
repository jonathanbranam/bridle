---
id: br-j6d3
title: "Machine admin role: one per machine, takes the orchestrator's admin work"
kind: epic
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-z485]
related: []
theme: agents-and-cli
---

## The ask

The human, 2026-10-11 ~6:35 PM ET (to advisor product-manager), on what matters most next: "Per-machine role support. Both current orch and an admin role in a following epic."

## Outcome

A machine admin role holds the machine-wide admin work, so the orchestrator only steers the work;
both are one per machine. Follows the epic one orchestrator per machine.

## Done when (PdM draft)

The admin role's design is approved (gb97); the admin runs one per machine and does the
machine-wide work (restarts, upgrades, installs, signing, sessions, checks); the orchestrator no
longer needs those permissions.

## Child tickets

gb97 (moved here from the security epic 3827, which keeps it as related: the admin split also
narrows the orchestrator's permissions). Related: kuw2 (machine daemon), j2vq.
