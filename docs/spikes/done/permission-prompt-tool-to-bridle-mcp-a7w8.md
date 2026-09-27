---
id: a7w8
title: Spike: --permission-prompt-tool to a bridle MCP server
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

From `docs/research/01-agent-runtime.md` §8 @ c192bfc, item 2:

> **`--permission-prompt-tool` → bridle MCP**: confirm the request and
> response shape, and that a slow answer (minutes, while a human decides)
> doesn't time out the turn.

## Why it matters

Turning permission prompts into questions to the manager or human needs a
channel that tolerates a slow human.

## Notes

The [[docs/proposal/build-order|build order]] plans this through claude's `can_use_tool`
control requests (`--permission-prompts host`) instead of an MCP tool. The
spike should cover whichever mechanism is chosen, including the slow-answer
timeout.
