---
id: c9zm
title: The watcher tells the orchestrator when its own context is getting large
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [d4mz, s8kn]
closed: 2026-10-02T00:43:36.067904Z
---

## What happened

The eighth orchestrator session ran from 23:55 to about 02:35 UTC without once checking its
own context size; the human called the handover. The human, verbatim (2026-09-29): "you
aren't monitoring your own context; that needs to happen. That should be part of the watcher
to remind you or give you a hint."

## Proposal

- `scripts/orchestrator-watch.sh` also exits with `CONTEXT <tokens>` when the orchestrator's
  own session passes a threshold (default 140K, `CONTEXT_WAKE` to override), once per
  threshold crossing, so the orchestrator proposes a handover at the next quiet point.
- The number comes from what Claude Code already reports: the `bridle statusline` hook
  (s8kn) sends the session's context to the daemon's ledger for sessions bridle doesn't host.
  The watcher needs to know which session is the orchestrator's (e.g. the session id written
  by `scripts/claude-orchestrator` at start, or the Remote Control name `bridle-orch-<hostname>`).
  Check that the ledger is actually being fed for this session: `bridle usage --json`'s
  `interactive_today` was empty on 2026-09-29 even with the statusline running.
- The same for the advisor's session, later.

## Progress (2026-09-29)

Built (037c6ee): `scripts/orchestrator-watch.sh` exits with `CONTEXT <tokens>` when the
orchestrator's own session passes 140K, finding the session by the id
`scripts/claude-orchestrator` records. Still open: the same for the advisor's session, and
confirming the ledger is fed for sessions bridle doesn't host (`interactive_today` empty).

## Resolution

Resolved by: br-f025 (bb6bbf9), br-8a3f (c81c4cf), br-5575 (037c6ee), br-47ba (8a1c3d4)
