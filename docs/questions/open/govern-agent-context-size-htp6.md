---
id: htp6
title: Govern agent context size, and renew agents in place
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [tx3f]
---

## The question

The human's words, 2026-09-27:

> If we're running million context window agents, they should start winding
> down whenever they reach 200K+ tokens roughly and persist everything to
> bridle and then restart a new agent in the same working tree. The agent is
> ephemeral but the work tree/branch and bridle manage the behavior do an
> agent. This includes the manager and anything bridle-managed. The
> orchestrator should also keep a watch on his own context. Or this should
> all be built into bridle to watch and govern context window growth.
>
> Large windows cause problems both for cost and also for agent reasoning.
> Reasoning starts to break down around 250-300k tokens. Ideally, context for
> workers should stay well under 200k.

Queued as two tasks: measure each agent's context size (`context-measure`,
done on its branch 2026-09-27), then govern it. For the second: the threshold
per role, what the handoff persists and where (a message now, the task record
after P0), how bridle respawns an agent in the same worktree and branch
(`spawn` can only create a new worktree today), and `bridle renew <agent>` to
do it on demand.
