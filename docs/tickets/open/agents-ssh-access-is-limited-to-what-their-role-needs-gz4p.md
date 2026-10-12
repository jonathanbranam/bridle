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

## The end state: two tiers (the human, 2026-10-12)

The human, verbatim (to advisor product-manager): "agents (all agents) should not have access to
SSH into another machine as me. I think there should be, end state, two tiers of SSH connections:

1. Read-only or limited permissions - many agents can have this
2. read-write but with a specific user (not jbranam) - admin agent only"

So, for the design:
- **No agent SSHes as the human's own user (`jbranam`)**, any role. Today's ssh helper (`--machine`,
  `token pair`, the move's stop/take-over over ssh) runs as the human; the design says what replaces it.
- **Tier 1, limited**: many agents; read-only or a fixed set of commands (e.g. a forced-command key or
  a restricted account), enough for status and read-only bridle calls on another machine.
- **Tier 2, read-write**: a separate machine user, not `jbranam`, used only by the machine admin role
  (j6d3/gb97). The design says which bridle work needs tier 2 (setup, take-over, token pairing).
