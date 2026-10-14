---
id: br-cvfu
title: "bridle daemon restart --cancel / --postpone: hold a pending drain while a land runs (pulled forward from 8swr)"
kind: feature
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-8swr, br-y55w]
---

## The ask

The human, 2026-10-14 ~6:15 PM ET (to aide, relayed m-13135), verbatim: "Yes A", to aide's option A on br-8swr: "add now only a small command so the orchestrator can postpone or cancel a pending restart while a land runs (e.g. `bridle daemon restart --cancel`/`--postpone`), the full fix staying in the later epic".

## Why

br-8swr: a drain's quiet point counts a manager waiting on a background `bridle ticket land` as idle, so a restart (self-upgrade every 3 h, or `bridle daemon restart`) kills the land mid-check. It happened four times on 2026-10-14 (05:44Z, 07:50Z, 12:44Z, 15:07Z), ~30-45 min of machine time each. The human chose a manual guard for now, but nothing cancels or postpones a pending drain.

## What's wanted

- `bridle daemon restart --cancel`: cancel a pending drain (a requested restart or a self-upgrade waiting for its quiet point); the daemon keeps running, agents untouched. A self-upgrade's built binary may stay for the next cycle.
- `bridle daemon restart --postpone <dur>` (e.g. `30m`): don't start a drain (or let a pending one reach its quiet point) before then. Covers the self-upgrade timer too, so a land can finish.
- `bridle status` shows a postponement (until when) on its `upgrade` line, as it shows draining today.
- Who may: the orchestrator and the manager (the manager is the one that knows a land is starting). Keep it to the daemon-admin permission the restart already needs, plus the manager role if that's simple.

Docs: docs/design/agent-host/daemon.md ("Restart in place"), docs/design/cli.md, the manager role's landing steps (postpone before a land, or say the orchestrator does), CHANGELOG.

## Out of scope

The full fix (the daemon sees an agent's background work as busy) stays in br-8swr's later epic.

## Acceptance

Tests: a cancel ends a pending drain with no restart; a postpone delays a drain (and a self-upgrade) until it expires; status shows it. `just check`.
