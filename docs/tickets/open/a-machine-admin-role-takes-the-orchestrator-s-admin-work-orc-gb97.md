---
id: br-gb97
title: A machine admin role takes the orchestrator's admin work; orchestrator and admin are per machine
kind: feature
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kuw2, br-ma8e, br-j2vq]
parent: br-3827
theme: security
---

## The ask

The human, 2026-10-11 ~1:20 PM ET (to advisor product-manager), verbatim: "we need a theme/epic for security + hardening; future work in that realm is: * agents need limited access to SSH * I think switch to PAT for github to reduce permissions as well * split admin responsibilities that orch has to a separate machine admin role (related: orch + admin become per-machine) * related is identifying agents that make changes" (the third item).

## What's wanted

A machine admin role takes the machine-wide admin work the orchestrator does today, so the
orchestrator only steers the work and holds fewer permissions. The orchestrator and the admin are
each one per machine.

A design pass first: inventory what the orchestrator does that is admin (daemon restarts and
upgrades, installs, signing, launching sessions and windows, disk and process checks, tokens,
scheduled maintenance) versus steering (queue, workers, landings); what permissions each role then
needs (see the SSH and PAT tickets under the same epic, and j2vq on stop/rm); whether the admin is a
session or part of the machine daemon (kuw2). Related: ma8e, the epic one orchestrator per machine.
Then the human reviews.
