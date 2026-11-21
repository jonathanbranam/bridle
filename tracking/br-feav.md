+++
id = "br-feav"
title = "Docs: document the built-but-undocumented routes, events, packs and commands (items 12-18 of 9x55)"
kind = "docs"
state = "integrated"
created_at = "2026-11-21T16:10:51.450Z"
updated_at = "2026-11-21T18:32:12.558311Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wfeav",
]
branch = "bridle/wfeav"
commit = "4b71a49f550be62a48ba7666881965c10e974a7b"
summary = 'Docs only. api.md: added routes ready (+tasks twin), stage-land, push, restart/hold, sessions/keep, review/now|add, usage/history, and event rows for load.hold.*, schedule.fired/missed_fired, restart.failed, push.failed, worktree.removed, task.settle_skipped, orchestrator.context and session.* (checked against the constants in bridle-api types.rs; each points to the doc that has the fields). cli.md: `usage --agent` and a "Dev recipes" section (test-lock, specs-check, gateway-types, deny). human-web-ui.md: core routes and the done|drop|answer|reply actions. workflow-layers.md: packs list fixed (mobile, python, typescript, vim, web) with a line each; the example config no longer names web-ui/monorepo. Caveat: `project.held` looked like an event but is a meta key, so it is not listed. No code looked wrong.'
ticket = "feav"
+++

docs/tickets/open/docs-feav.md

## Thread

### note · external:orchestrator · 2026-11-21T16:11:36.729Z
readied by external:orchestrator

### note · agent:wfeav · 2026-11-21T18:32:01.930Z
built: 29a465573cb54108f070146760191d6c5c54033f (branch bridle/wfeav)

### note · agent:wfeav · 2026-11-21T18:32:02.029Z
done: items 1-7 documented (api.md routes+events, cli.md usage --agent + dev recipes, human-web-ui.md routes, workflow-layers.md packs); just check exit 0, 2023 tests passed; 29a465573cb54108f070146760191d6c5c54033f

### note · agent:manager-2 · 2026-11-21T18:32:12.236Z
integrated: 4b71a49f550be62a48ba7666881965c10e974a7b (branch bridle/wfeav)

### note · agent:manager-2 · 2026-11-21T18:32:12.558Z
ticket file moved to docs/tickets/resolved/docs-feav.md
