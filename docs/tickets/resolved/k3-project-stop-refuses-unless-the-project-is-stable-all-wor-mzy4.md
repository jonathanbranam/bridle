---
id: br-mzy4
title: "K3: project stop refuses unless the project is stable (all work landed or dropped, nothing unpushed, no undelivered mail)"
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: [br-3ch7]
related: [br-kx5u, br-kt25]
parent: br-kx5u
resolved: 2026-10-12T17:19:40Z
---

Child of epic br-kx5u (MVP), from the design in br-kt25 ("## Design", child K3), as settled in the human's grilling session 2026-10-12 (answers verbatim on br-kt25's thread).

The human (Q2, ~10:15 AM ET), verbatim: "MVP parking, or MVP project preflight, means that all workers have landed their work, and it's been merged or abandoned, merged or dropped. All is done. The manager has handled all of it, so we're moving only while the project is in a stable state."

## The ask

`bridle project preflight` (K1, K2) gains a "stable project" check, and `bridle project stop` runs it and refuses when it fails:

- no ticket is claimed or built (everything landed or dropped);
- no agent is running;
- no local `bridle/*` or `integrate/*` branch is ahead of the integration branch;
- the integration branch is pushed (origin has it);
- the main checkout is clean, including untracked `review-documents.txt` and `migrations.toml`/`.log`, unless `--leave` names them LEFT in the listing;
- no mail to or from the project is undelivered (the human, ~10:26 AM ET: "I think we just check that and refuse if there are undelivered messages, because it means something's wrong, essentially").

Each failure names what is not stable. No WIP commits and no pushes of agent branches (agents can't push; that is post-MVP, br-t7ya).

## Acceptance

A test project with a claimed ticket, an unpushed branch, a dirty checkout or an undelivered outbox message each fails the check with its name; a clean one passes; `--leave` lets a dirty checkout through and prints it; `just check` passes.
