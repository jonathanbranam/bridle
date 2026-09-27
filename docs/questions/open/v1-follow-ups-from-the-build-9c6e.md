---
id: 9c6e
title: Small v1 follow-ups found during the build
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [k4wq]
---

## The question

Which of these small gaps from the v1 build are worth fixing before the next
feature? None is a design question. They're collected here so they aren't
lost, and each can become its own ticket when picked up.

## Fixed

On 2026-09-27, after an audit of the code against the agent-host design:
held messages stranded by an exit, `events?agent=` not resolving names, `rm`
stopping an agent before refusing its dirty worktree, `rm` erasing usage, no
event prune at start, no caller recorded for interrupt, stop and resume, no
events from restart reconciliation, the SIGHUP window at start, an idle
autostarted manager (now `start_prompt`), and reset times missing from
`bridle usage`.

On 2026-09-27, four more small gaps: `Client::events_stream` now reconnects
(reissuing the request with `since` = last seq yielded, after a short
backoff) when the daemon ends the SSE response on a lagging or closed
subscriber; `bridle events` without `--follow` now returns the most recent
500 matching events instead of the oldest, and `--follow` without `--since`
starts at the tail instead of replaying the whole history; `bridle logs`
now shows the command or path for a tool call instead of a raw JSON
fragment, matching `tool.use` events; and `Store::set_session`, which had
no callers, is gone.

## Gaps

- **`bridle daemons` shows no agent counts.** The CLI deferred it because it
  would need to probe every registered daemon. With a short timeout that's
  cheap.
- **Spawn failures after `claude` starts** (bad model name, auth expired)
  surface only as a `crashed` agent with a stderr tail. `spawn` still returns
  201. Either wait for the first `system/init`, or document that `spawn`
  returning isn't readiness.
- **`rm` doesn't check for open files** under the worktree (`lsof +D`), as
  the original design said it would.
- **The `Containment` trait is unused.** Callers use the `ps` functions
  directly. Wire it in when a Linux implementation arrives
  ([[docs/spikes/open/process-containment-on-linux-2mj9|spike 2mj9]]).
- **Without a spawn prompt or a role `start_prompt`, an agent never learns
  its name or worktree**: those facts are prefixed to the first message, and
  there is none.
- **No `token list` or `token revoke`.** An external token can't be rotated.
- **The fake `claude` matches magic words on the last line of a message**,
  because bridle wraps deliveries in an envelope. Tests that send multi-line
  bodies need the magic word last.
