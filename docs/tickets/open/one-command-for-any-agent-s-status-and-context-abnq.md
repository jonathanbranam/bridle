---
id: abnq
title: One command for any agent's status and context
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-abnq]
---

## The ask

## The ask

The human, 2026-10-05 (NUC): "this should be an easy and direct command to find the context of any agent that we can run. It should just return status and context information about that agent, so we're tracking the context."

Today workers and managers show CONTEXT in `bridle agents`, but interactive sessions (advisor, aide, orchestrator) appear only in `bridle status --json` under .sessions (identity, pid, pane, tokens, started_at, last_activity). Finding the notes advisor at 227K took --help reading, jq and tmux capture-pane.

## What's wanted

- One command, e.g. `bridle context [<agent|session>]` (or `bridle agent status <name>` covering sessions): state, pid, uptime, last activity, context tokens and where they sit against the 150k/200k/250k/300k steps (see gq9r), and whether a handover is planned.
- With no argument: one line per agent and session in the project.
- Sessions also listed in the plain `bridle status` and `bridle agents` tables.

Pick one command name, keep it a read of what the daemon already records. Update docs/design/cli.md. Verify: just check, plus a CLI test with a fake session record. No migration (read-only).

Source: orchestrator@nuc, 2026-10-05.
