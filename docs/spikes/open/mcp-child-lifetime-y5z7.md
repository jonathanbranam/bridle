---
id: y5z7
title: Spike: do stdio MCP servers outlive claude?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [9trg]
---

## What to find out

From `docs/research/01-agent-runtime.md` §8 @ c192bfc, item 6:

> **MCP child lifetime**: whether stdio MCP servers outlive the `claude`
> process that started them (the docs are ambiguous).

## Why it matters

Leftover MCP servers are processes containment has to find.

## Notes
