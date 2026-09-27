---
id: qb97
title: Workers as Agent subagents or separate claude sessions?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: [docs/agent-host.md]
needs: []
see: []
---

## The question

From `docs/design.md` §15 @ c192bfc, item 2:

> **Workers as `Agent` subagents or separate `claude` sessions?** *Mostly
> answered* by [`research/01`](docs/research/01-agent-runtime.md): separate headless
> `claude -p` sessions that bridle spawns, with 7 spikes (§8 there) to run
> before committing.

And from §6.4:

> **To verify before building on it:** whether hooks fire inside `Agent`-tool
> subagents with enough identity (session/agent id) to tell them apart, or whether
> workers must be separate `claude` sessions started by `bridle spawn` (tmux +
> worktree). The design supports both, since everything goes through the store.
> Which is the default depends on the answer.

## Why it matters

It sets the agent runtime everything else is built on.

## Notes

See [[docs/design/coordination#How agents actually hear things (Claude Code integration)|how agents hear things]].

## Resolution

Separate headless `claude -p` stream-json sessions that bridle owns.

- [research/01](docs/research/01-agent-runtime.md) found that hooks do fire in
  subagents and carry `agent_id`, and recommended bridle-owned headless workers.
- Spike 01 verified the stream-json host:
  [findings](docs/spikes/01-stream-json-findings.md).
- [[docs/agent-host|agent-host.md]] designs the daemon, API and agent host on it,
  and `crates/bridle-claude` implements the client.
- The remaining spikes from research/01 §8 are filed under `docs/spikes/open/`.
