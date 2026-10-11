---
id: br-gz4p
title: Agents' SSH access is limited to what their role needs
kind: feature
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-cp5p]
parent: br-3827
theme: security
---

## The ask

The human, 2026-10-11 ~1:20 PM ET (to advisor product-manager), verbatim: "we need a theme/epic for security + hardening; future work in that realm is: * agents need limited access to SSH * I think switch to PAT for github to reduce permissions as well * split admin responsibilities that orch has to a separate machine admin role (related: orch + admin become per-machine) * related is identifying agents that make changes" (the first item).

## What's wanted

Agents get only the SSH access their role needs. Today an agent can run `ssh` as the user, with
the user's keys and agent, to any host those keys reach (the NUC, the PC, GitHub).

A design pass first: what each role needs (workers likely none; the orchestrator or a machine
admin role may need the other machines), how it is enforced (Claude Code permission deny rules,
the containment profile, a restricted key or ssh config per role, no forwarded agent), and what
breaks (git over SSH for pushes: see the PAT ticket under the same epic). Then the human reviews.
