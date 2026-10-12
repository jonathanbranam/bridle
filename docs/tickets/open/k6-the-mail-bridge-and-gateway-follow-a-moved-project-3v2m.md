---
id: br-3v2m
title: "K6: the mail bridge and gateway follow a moved project"
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

Child of epic br-kx5u (MVP), from the design in br-kt25 ("## Design", child K6).

The human, 2026-10-11 (kx5u's ask): "we need the work for mail bridge and gateway to work as well."

## The ask

When `[mail] projects` names P: `project start` on NEW installs the mail unit (`bridle mail install`) and copies the `[mail]` section, and `mail.key` (0600, over ssh) when open mailed questions exist; else it says that replies to mail sent from OLD will fail verification. The bridge on OLD is already unloaded by stop. The gateway reaches P at its new machine through `[projects]` (2wnn); check it.

## Acceptance

After a move between two test homes: NEW's bridge unit exists and answers; OLD's has none; the gateway lists P at NEW; `just check` passes.
