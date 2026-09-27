---
id: k4wq
title: A human surface beyond the CLI?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [u6wk]
---

## The question

From `docs/design.md` §15 @ c192bfc, item 5:

> **A human surface beyond the CLI?** `bridle review` and `inbox --human` in
> the terminal first. A TUI or local web board (like SwarmForge's cockpit)
> could come later, with push notifications for questions if the harness
> supports them.

## Why it matters

The human's attention is meant to go on decisions and acceptance
([[docs/proposal/goals-and-non-goals|goals]]). How they see pending decisions
decides how fast those arrive.

## Notes

- `docs/agent-host.md` §12: a TUI on `bridle-api` (item 4), and an MCP server
  at `/mcp` (item 5) that would need bridle behind public HTTPS to reach it from
  claude.ai or mobile. Until then `claude remote-control` on the orchestrator
  covers it.
- The human's mobile-only case: [[answering-hitl-questions-from-mobile-u6wk|answering HITL questions from mobile]].
