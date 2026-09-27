---
id: 7r7m
title: Spike: human takeover round trip
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

From `docs/research/01-agent-runtime.md` §8 @ c192bfc, item 4:

> **Takeover round trip**: headless → interrupt → `claude --resume`
> interactive → exit → headless `--resume`. Confirm context is intact and
> nothing is lost from the stdin queue.

## Why it matters

`bridle take` / `give` (`docs/agent-host.md` §12 item 2) depends on it.

## Notes
