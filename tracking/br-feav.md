+++
id = "br-feav"
title = "Docs: document the built-but-undocumented routes, events, packs and commands (items 12-18 of 9x55)"
kind = "docs"
state = "claimed"
created_at = "2026-11-21T16:10:51.450Z"
updated_at = "2026-11-21T18:08:15.250755Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wfeav",
]
summary = 'Docs only. api.md: added routes ready (+tasks twin), stage-land, push, restart/hold, sessions/keep, review/now|add, usage/history, and event rows for load.hold.*, schedule.fired/missed_fired, restart.failed, push.failed, worktree.removed, task.settle_skipped, orchestrator.context and session.* (checked against the constants in bridle-api types.rs; each points to the doc that has the fields). cli.md: `usage --agent` and a "Dev recipes" section (test-lock, specs-check, gateway-types, deny). human-web-ui.md: core routes and the done|drop|answer|reply actions. workflow-layers.md: packs list fixed (mobile, python, typescript, vim, web) with a line each; the example config no longer names web-ui/monorepo. Caveat: `project.held` looked like an event but is a meta key, so it is not listed. No code looked wrong.'
ticket = "feav"
+++

docs/tickets/open/docs-feav.md

## Thread

### note · external:orchestrator · 2026-11-21T16:11:36.729Z
readied by external:orchestrator
