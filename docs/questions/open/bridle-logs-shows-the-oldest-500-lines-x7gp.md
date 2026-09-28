---
id: x7gp
title: bridle logs and events show the oldest 500 lines, so `tail` shows stale output
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The problem

`bridle logs <agent>` with no `--since` renders transcript lines 1 to 500
(`GET /v1/agents/{id}/transcript`, `limit` defaults to 500,
`crates/bridle-daemon/src/server.rs:467`). Once a transcript is longer than
that, `bridle logs <agent> | tail` shows output hours old, not the agent's
current turn. `bridle events --since N` pages the same way.

What it cost the orchestrator on 2026-09-28, at the fourth session's start:

- `bridle logs manager-2 | tail` showed manager-2 reviewing a `context-measure`
  worktree that had already been removed. Its transcript was about 2,440 lines,
  and its real current activity (reviewing htp6b) was only visible with
  `--since 2380`.
- The watcher was started from `bridle events --since 0 | max(seq)`, which is
  500, not the latest seq (9329). It woke at once on day-old exits.

Managers use `bridle logs <worker>` to check on workers ("idle isn't always
idle", w8bz), so they can misread a worker in the same way.

## Options

- `bridle logs` with no `--since` shows the latest lines (a tail), like
  `--follow` would start from; `--since 0` keeps the old behaviour.
- A `--tail N` flag, and a way to get the latest event seq
  (`bridle events --latest`, or the seq in `bridle status --json`).
