---
id: z485
title: Make the orchestrator non-interactive (headless)
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [r8kv, jttf]
tasks: [br-ab3a]
---

## The ask


Record, don't build yet. The human, verbatim (2026-10-03, via the advisor, a list headed "Ideas to record not build"):

> Make orch non interactive

## Notes

- Today the orchestrator is an interactive `claude` session the human talks to
  (`bridle session orchestrator`). Goes with the split in
  [[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]]: if a triage role
  takes over the human's conversations, the orchestrator could run headless under the daemon.
