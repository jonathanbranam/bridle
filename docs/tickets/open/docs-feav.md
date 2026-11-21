---
id: br-feav
title: "Docs: document the built-but-undocumented routes, events, packs and commands (items 12-18 of 9x55)"
kind: docs
created: 2026-11-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-9x55]
related: []
---

## The ask

Split from br-9x55 (right-sizing: one worker should not hold 18 doc fixes). The human, via advisor agentic-dev, 2026-11-21: "Please directly fix these documentation issues or file tickets to do so". Edit docs only; where the code looks wrong, comment on this ticket instead of changing code. Each item was checked at main 153d3c86 (file:line below); re-check before editing.

### Built but undocumented

1. docs/design/cli.md's usage block is missing `bridle usage --agent <NAME|ID> [--bucket hour|day]` (crates/bridle/src/cli.rs:1640-1643); it is described only in usage-and-budget.md:156.
2. Daemon routes documented nowhere: `POST /v1/tickets/{id}/ready` (and `/v1/tasks/{id}/ready`), crates/bridle-daemon/src/server.rs:385-386; `POST /v1/push`, server.rs:395. Add them to docs/design/agent-host/api.md.
3. Daemon routes described in other docs but missing from api.md's route table: `/v1/tickets/{id}/stage-land` (stages.md), `/v1/restart/hold` (daemon.md), `/v1/review/now|add`, `/v1/sessions/keep`, `GET /v1/usage/history`.
4. Gateway routes in human-web-ui.md with no route-level description: `GET /api/v1/items`, `/api/v1/projects`, `/api/v1/session` (crates/bridle-gateway/src/lib.rs:58-60). Ticket actions `done|drop|answer` (actions.rs:36-39); only `reply` is named. login and logout appear only in cli.md.
5. Event kinds: about 15 are documented only in daemon.md, operating-model.md or orchestrator-supervision.md, not in api.md's Events list (for example schedule.fired, restart.failed, load.hold.*, push.failed, worktree.removed, session.*, task.settle_skipped). List every kind in api.md, or point to where each is described.
6. workflow-layers.md:10 lists packs "typescript, python, vim, web-ui, game, monorepo". The packs that exist are mobile, python, typescript, vim and web (workflow/packs/); web (6 rules) and mobile (5 rules) are in no design doc. Fix the list and describe each pack in a line.
7. scripts/test-lock.sh (the machine-wide nextest lock, `BRIDLE_TEST_NOLOCK`) is described only in CLAUDE.md; `just specs-check`, `just gateway-types` (ts-rs bindings) and `just deny` are in no design doc. Mention them in cli.md's or the README's commands section.

## Order

Blocked by br-9x55 (real edge): both edit cli.md, api.md and workflow-layers.md, so this runs after it merges.

## Acceptance

Every item fixed or commented on; `bridle ticket check` passes; `just check` with BRIDLE_* env unset (doc-link and spec checks). Update any "checked <date>" Status line you verify. Model: Sonnet. Out of scope: any code change.
