---
id: gcvj
title: Replace Remote Control with bridle's own way to reach agents, so every agent can run in the background
kind: research
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [jttf, z485, r8kv, hrcn]
tasks: []
---

## The ask


Research (no build yet). The human, verbatim (2026-10-03, via the advisor):

> Start a research ticket and use subagents to do research on whether we could replace remote
> control with our own solution and what it would take to build so that all agents could run in
> the background.

Context: the human reaches interactive sessions (orchestrator, advisors) from the phone through
Claude Code's Remote Control, which needs a live interactive `claude` in a terminal pane. Headless
agents (managers, workers, pm) run in the background under the daemon (stream-json). If bridle
could carry the human's conversation with any agent itself (gateway / web UI / phone), every
agent could be a background agent.

## Findings

(Research in progress by an advisor's subagent, 2026-10-03.)
