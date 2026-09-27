---
id: 65bf
title: Spike: hook mid-turn injection and its latency
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [akjw]
---

## What to find out

From `docs/research/01-agent-runtime.md` §8 @ c192bfc, item 3:

> **Hook mid-turn injection**: `PostToolUse` `additionalContext` from an
> `http` hook reaches a `-p` worker, and latency stays under ~20 ms when there
> is nothing to deliver.

From `docs/spikes/01-stream-json-findings.md`, Risks and follow-ups @ c192bfc:

> **Hook latency** (~0.9 s each) matters if bridle uses `PostToolUse` hooks for message delivery on every
> tool call. Measure with a matcher-scoped hook and compare with `--replay-user-messages`-acked stdin
> injection, which S3 shows already reaches the agent mid-turn.

## Why it matters

The original delivery model assumed hooks. Delivery now uses stdin
([[docs/design/coordination#How agents actually hear things (Claude Code integration)|how agents hear things]]),
so this matters only if hooks come back for status or injection, e.g. for
agents bridle doesn't host.

## Notes
