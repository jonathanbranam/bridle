---
id: br-3stk
title: "ticket land's integration check has no node_modules in a JS project (biome: command not found on bridle-ui)"
kind: bug
created: 2026-10-17
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-17 ~8 AM ET, to advisor (product-manager): "Yes ready that fix for bridle
hi" (priority high).

`bridle ticket land`'s own check must pass for a JS project (bridle-ui) without
`--checked-commit`: prepare the project's dependencies in the land's check worktree before the
check command runs (see "Fix (to decide)"). Also: land finds the ticket's branch by itself (no
`--branch` needed). Tests; docs (cli.md, the land section of docs/design/agent-host/) and
CHANGELOG. Acceptance: `just check`, and a bridle-ui land without `--checked-commit` passes.

## What happened

2026-10-17 ~8:20 AM ET, bridle-ui: manager-3 ran `bridle ticket land ui-3mxz --branch bridle/devport`. The land's own check failed with `sh: biome: command not found`. The worker's worktree and the clone both have `node_modules/.bin/biome`; the worktree the land checks in does not, so no JS project can land without `--checked-commit`.

Also seen: land couldn't find the ticket's branch by itself (needed `--branch`).

## Workaround

The manager runs `npm run check` in the worker's worktree and lands with `--checked-commit <sha>` (orchestrator told manager-3, m-1522).

## Fix (to decide)

The land check prepares the project's dependencies (for npm: `npm ci`, or link the clone's node_modules) before running the check command, or the project config names a setup step. Files likely: the land/integration check in crates/bridle-daemon and crates/bridle. Acceptance: a bridle-ui land without `--checked-commit` passes its check.
