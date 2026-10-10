---
id: xxq8
title: List the projects an identity holds tokens for, without printing the tokens
kind: feature
opened: 2026-10-10
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

The orchestrator role says: at every start, check a waiter is open for each project you hold an orchestrator token for, "listed under [orchestrator]" in ~/.bridle/credentials.toml. Reading that file is (rightly) refused by Claude Code auto mode as credential exploration (orchestrator start, 2026-10-10 ~01:05Z). The orchestrator fell back to the project list in its handover note, which can go stale.

Ask: a command that prints the project names (and machine, if any) an identity has tokens for, never the token values; e.g. `bridle token projects [--as orchestrator]`, or have `bridle status --all-projects` show waiter_open per project. Then the role text points at it instead of the file.

Not critical: the handover note covers it today. Filed by orchestrator; scheduling is the PdM's.
