---
id: br-3827
title: "Security and hardening: least access for agents, scoped credentials, a machine admin role, traceable changes"
kind: epic
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
theme: security
---

## The ask

The human, 2026-10-11 ~1:20 PM ET (to advisor product-manager), verbatim: "we need a theme/epic for security + hardening; future work in that realm is: * agents need limited access to SSH * I think switch to PAT for github to reduce permissions as well * split admin responsibilities that orch has to a separate machine admin role (related: orch + admin become per-machine) * related is identifying agents that make changes"

## Outcome

Agents and roles hold only the access their work needs, and every change can be traced to the
agent that made it. Theme `security` (new): access, credentials, permissions, provenance.

## Done when (PdM draft, for the human to amend)

1. Agents reach SSH only as their role needs (workers none by default; named exceptions), enforced,
   not just written in a rule.
2. Bridle's GitHub access uses fine-grained personal access tokens scoped per repo and permission,
   not a broad key or token.
3. A machine admin role holds the machine-wide admin work the orchestrator does today; the
   orchestrator keeps only steering the work. Both are one per machine.
4. Every commit and change names the agent (and machine) that made it, and the human can look it up.

## Child tickets

- Agents' SSH access is limited (new).
- GitHub access by fine-grained PAT (new).
- A machine admin role: gb97, moved to its own epic j6d3 (the human, 2026-10-11: "an admin role in a following epic"); related here.
- Identifying agents that make changes: 2bzw (how strong provenance should be), xccp (a git identity per machine).

Priority: low; it comes after many other epics (the human, 2026-10-11 ~1:25 PM ET: "security is low priority, comes after many other epics"). Future work. Each child needs a design pass
before build.
