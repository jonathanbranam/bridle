---
id: m7mp
title: Interactive roles (aide, advisor, orchestrator) get their resolved rules, project rules included, at startup
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [98xt]
tasks: [br-m7mp]
closed: 2026-10-09T23:11:00Z
---

## The ask

From the NUC orchestrator, relaying the human (2026-10-04), via the meta-notes daemon (m-0338), verbatim:

> 1) Project rules for the roles that need them. The notes project has a project rule, .bridle/rules/worked-on-log.md (frontmatter roles: [advisor, orchestrator, manager, worker]). The human meant it for the interactive roles in that repo: the aide and the advisors (workers are fine too). But in the notes project, bridle prime advisor, prime orchestrator and prime manager don't include it; only bridle prime worker does. (I only checked bridle prime output, not what bridle session advisor or the daemon actually inject, and read no docs.) Ask: a clear, supported way for a project to add rules or guidance to specific roles (aide, advisor, any role) as a project-level override, delivered at their startup. It may exist already; if so, these roles aren't getting it.

What's known (orchestrator, not verified on the NUC): daemon-spawned agents (manager, worker) get their role's resolved rules, the project's `.bridle/rules/` included, in the system prompt since br-2242, so `bridle prime manager` not showing it may not mean the manager lacks it. The interactive roles (orchestrator, advisor, aide) start from `bridle prime <role>`, which prints no rules: that gap is recorded in [[role-and-rule-files-misstate-how-rules-reach-agents-orchestr-98xt|98xt]] ("The orchestrator and advisor get none"). This ticket is the build ask: those roles get their resolved rules, project rules included, at startup.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
