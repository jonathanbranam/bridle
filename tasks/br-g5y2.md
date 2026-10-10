+++
id = "br-g5y2"
title = "Scheduled messages slice 2: role priming: wait at the maximum timeout and schedule a message for timed wake-ups (hrcn)"
kind = "chore"
state = "integrated"
created_at = "2026-10-08T14:29:41.482Z"
updated_at = "2026-10-10T21:00:02.878670Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:orchestrator",
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
size = "S"
branch = "bridle/wg5y2"
commit = "01bda1b4816801b26c11e4d4f0b2358012a7edc8"
summary = "Added rule workflow/base/rules/scheduled-wakes.md (wait at max timeout; bridle schedule add --at/--cron --message for timed wakes; list/rm own) for project-manager, manager, worker, prototyper, designer, plus one-line pointers in manager, project-manager and worker role text, and a CHANGELOG line. Gap: br-9xze refuses externals (orchestrator, advisor, aide) as schedule targets, so their wait text is unchanged; a follow-up is needed once schedule add allows externals. docs/design/cli.md needed no change."
parent = "br-9xze"
+++

Follows br-9xze (the mechanism: `bridle schedule add/list/rm`); start only after it is integrated AND the daemon runs it. The human's reason, verbatim: "agents, instead of waking themselves up with shorter wake-up timers, start using this functionality. They set their wake-up timer to the maximum and then rely on scheduled message sends to wake themselves up, and that gives them a reminder of what the purpose of the wake-up is."
Goal: role text and rules tell agents to wait at the maximum timeout and, when they need to wake at a time (before a meeting, at 3 AM, in 20 minutes), run `bridle schedule add --to <self> --at ...|--cron ... --message "<why you are waking>"` instead of looping short waits.
Files: workflow/base/roles/*.md that describe waiting or timed wakes (orchestrator Watch section, aide, advisor, manager, project-manager; grep `wake`, `timer`, `sleep`, `wait`), a short new rule workflow/base/rules/scheduled-wakes.md (when to schedule, the message says why, list/rm your own when done), docs/design/cli.md only if a mention is stale. Use the exact flags from the landed br-9xze (`bridle schedule add --help`); do not invent any.
Migration: role text and the rule reach projects via `bridle workflow sync`; no project files change.
Acceptance: just check passes; every role that mentions waiting says what to do for a timed wake. Model: Haiku.

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.176Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T02:59:22.510Z
advisor (product-manager): approved by the human (2026-10-09 ~10:58 PM ET, "Approve"). The human, 2026-10-09 ~10:55 PM ET: "if the machine work finishes up, let's prioritize work that makes sending and receiving messages work better and more reliable, reducing waiter counts like the orc has 5 waiters; I think the scheduled message work is also an important epic to finish up soon". Epic scheduled-messages, ranked after messaging; build it alongside the messaging work, after machine setup.

### note · agent:pm-1 · 2026-10-10T02:59:27.267Z
pm-1: br-9xze decision 6 refuses externals (orchestrator, advisor, aide) as schedule targets/creators; only agents and the human may use it today. Write the priming for roles that can use it (manager, project-manager, workers); for the external roles, say 'when schedule add allows externals' only if a follow-up lands, otherwise leave their wait text alone and note the gap in the done note so a follow-up task can be filed. Do not change the daemon here.

### note · agent:wg5y2 · 2026-10-10T20:38:02.684Z
done: scheduled-wakes rule + role pointers (manager, project-manager, worker); just check exit 0, 1461 tests; 58be80e0. Gap: external roles (orchestrator/advisor/aide) untouched since schedule add refuses them; follow-up needed.

### note · agent:wg5y2 · 2026-10-10T20:50:45.568Z
done (re-checked after merging main at fb18e39f): just check exit 0, 1462 tests; d151aef0

### note · agent:manager-2 · 2026-10-10T20:59:45.274Z
integrated: 01bda1b4816801b26c11e4d4f0b2358012a7edc8 (branch bridle/wg5y2)

### note · agent:manager-2 · 2026-10-10T21:00:02.878Z
cleanup: removed agent wg5y2, branch bridle/wg5y2
