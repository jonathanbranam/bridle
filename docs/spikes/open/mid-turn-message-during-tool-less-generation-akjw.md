---
id: akjw
title: Spike: mid-turn messages during tool-less generation
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

From `docs/spikes/01-stream-json-findings.md`, Risks and follow-ups @ c192bfc:

> **Mid-turn semantics** need a follow-up: a mid-turn message during a turn with no tool calls (pure
> generation), several mid-turn messages, and what `still_queued` / `cancelled` actually track.
> `msg_lifecycle_v1` may expose lifecycle events that weren't enabled here; the `initialize` control
> request (seen in the binary) may be how it's enabled.

From `docs/agent-host.md` §13 @ c192bfc, item 2:

> **Does a mid-turn message during tool-less generation fold in or wait?**
> This needs a follow-up spike (§4.3).

## Why it matters

v1 delivers messages through stdin ([[docs/design/agent-host/messages|messages]]), so delivery
timing depends on it.

## Notes
