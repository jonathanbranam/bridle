---
id: kc4v
title: context_tokens overcounts turns with more than one API call
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [htp6]
---

## The question

`context-measure` (the first half of
[[govern-agent-context-size-htp6|govern agent context size]]) records an
agent's context size at each turn end as
`input_tokens + cache_read_input_tokens + cache_creation_input_tokens` from
the `result` event's `usage` (`crates/bridle-daemon/src/supervisor.rs`, the
`context_tokens` computation in the turn-end handler, at `ae45ec7`).

Spike 01 says that `usage` is not one call's usage
(`docs/spikes/01-stream-json-findings.md`, row 8):

> `result.usage` is **per turn**, summed over the turn's API calls.

A turn with N tool-call rounds re-reads the cached prompt N times, so the sum
is roughly N times the real context. Observed 2026-09-28 01:08 UTC:
`bridle agents` showed `manager-2` at 942,368 after 11 turns, which can't be
its real size. The context governor's wind-down will act on this number, so
it needs the real size first.

Spike 01 names a source that gives it directly, with no model call:

> `get_context_usage` gives a per-category token breakdown and the
> auto-compact threshold (167,000).

The options, for whoever takes it: query `get_context_usage` at turn end; or
take the last assistant message's `usage` if the stream carries it (spike 01
says assistant events don't carry usage on the wire, so probably not); or keep
the sum but label it as turn input, not context.
