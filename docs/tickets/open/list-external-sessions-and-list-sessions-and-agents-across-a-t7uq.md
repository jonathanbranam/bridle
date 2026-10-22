---
id: br-t7uq
title: List external sessions, and list sessions and agents across all projects (--all-projects)
kind: feature
created: 2026-10-22
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human asked for this directly (2026-10-21, to the orchestrator: "Yes, please file a ticket for that ... ideally both commands would take a param to list across all projects"), so it is readied without a design question.

Two gaps in listing who is running:

1. No command lists external sessions (aide, advisors, orchestrator, mail, statusline). Today the only views are the `session` lines in `bridle status` (one project) and `bridle recipients` (every known identity, running or not). Add `bridle session list`: the running external sessions, with name, machine, context, uptime and last activity (what the `bridle status` session lines show).
2. `bridle session list` and `bridle agent list` work on one project's daemon. Give both `--all-projects`, listing across every project in the local registry with a project column. Without the flag, both keep today's one-project behaviour.

Context: the human was shutting down the aides and looking for idle agents and sessions to stop; answering "which external sessions are running where" took `bridle status` once per project.

## Acceptance

- `bridle session list` prints the running external sessions on the current project.
- `bridle session list --all-projects` and `bridle agent list --all-projects` list every registry project's sessions or agents, with a project column. A daemon that can't be reached is shown as such, not silently skipped.
- `--json` works for both. `just check` passes; tests for the fan-out with one unreachable daemon.
- Update docs/design/cli.md.

## Out of scope

Other machines' daemons (vesta, remus): local registry only, for now.

Model: Sonnet. Files: crates/bridle/src/cli.rs, commands/ (session and agent), the registry reader in bridle-api. Related: br-163f (CLI regroup, runs alone in crates/bridle/src): run this after it or before it, never alongside.
