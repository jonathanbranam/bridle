---
id: 3nqf
title: Is the driver Opus by default?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [xcpy, xypj]
---

## The question

From `docs/design.md` §15 @ c192bfc, item 12:

> **Is the driver Opus by default?** Max 5x has a separate weekly Opus
> window. A long-lived Opus driver may be the largest single consumer. An
> alternative is a Sonnet driver that escalates to Opus for planning and
> architecture turns only. The usage baseline (§11.5) should decide this.

## Why it matters

The budget is one subscription with no overage
([[docs/design/usage-and-budget|usage and budget]]).

## Notes

- The driver is now split into an external orchestrator (the human's own
  agent) and a bridle-hosted manager
  ([[docs/design/roles-and-lifecycle|roles]]). The built-in manager role
  defaults to `sonnet` ([[docs/design/agent-host/roles-and-config|roles and config]]).
  The question applies to both.
