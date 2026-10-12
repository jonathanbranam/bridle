---
id: br-ep5z
title: "K10: the MVP runbook: move a project by hand, step by step"
kind: docs
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: [br-mzy4, br-kefh, br-3vv3, br-3v2m, br-d6h6]
related: [br-kx5u, br-kt25]
parent: br-kx5u
---

Child of epic br-kx5u (MVP), from the human's grilling session 2026-10-12 (Q3, verbatim on br-kt25's thread).

The human (~12:15 PM ET), verbatim: "Q3, yes, I agree. Can run that by hand in the order, as long as each of the steps is clear and easy, which they are, I think."

## The ask

A short runbook for moving a project in the MVP, run by hand by the human or the orchestrator, in order: `project preflight --to NEW`; `project drain` (and watching it); `project stop` on OLD; `project take-over` and `project start` on NEW; the checks after (health, tokens both ways, mail bridge, gateway); restarting the interactive sessions on NEW from their handover notes; how to back out at each step. One page in `docs/` (the operating docs), linked from docs/README.md. Written after K3, K4, K5, K6 and K9 land, against the real commands. The one-command `bridle project move` (K7) is V1, not this.

## Acceptance

The human can follow it for the first data-contracts move with no other doc open.
