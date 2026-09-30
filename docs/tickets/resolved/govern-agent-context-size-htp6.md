---
id: htp6
title: Govern agent context size, and renew agents in place
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [tx3f]
closed: 2026-09-30T05:12:44Z
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

## Resolution

Implemented context governance with three pieces:

1. **Threshold configuration** in `crates/bridle-daemon/src/config.rs`: each
   role has a `wind_down_at` threshold (seconds of agent context).

2. **Governance logic** in `crates/bridle-daemon/src/governor.rs` and
   `src/supervisor.rs`: the supervisor monitors context size and winds agents
   down as they approach the threshold, persisting state to bridle before
   stopping.

3. **Renew on demand** via `bridle renew <agent>` in `crates/bridle/src/cli.rs`
   and `commands.rs`: spawns a new agent in the same worktree and branch with
   `--resume`, re-reading the persisted message history. Verified end-to-end
   in `crates/bridle-daemon/tests/renew_test.rs`.

Resolved 2026-09-28.
