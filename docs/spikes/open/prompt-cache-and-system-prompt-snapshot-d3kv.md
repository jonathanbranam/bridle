---
id: d3kv
title: Spike: prompt cache and the system-prompt snapshot
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

From `docs/spikes/01-stream-json-findings.md`, Risks and follow-ups @ c192bfc:

> **Prompt cache:** use `--exclude-dynamic-system-prompt-sections` for workers. Check how
> `--system-prompt-snapshot` (default on) interacts with a role file changing between resumes.

## Why it matters

Rule 2 of [[docs/design/usage-and-budget#Designing for fewer tokens|designing for fewer tokens]]
depends on a stable, cached prefix.

## Notes
