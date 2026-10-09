+++
id = "br-g5y2"
title = "Scheduled messages slice 2: role priming: wait at the maximum timeout and schedule a message for timed wake-ups (hrcn)"
kind = "chore"
state = "pending"
created_at = "2026-10-08T14:29:41.482Z"
updated_at = "2026-10-09T11:04:40.176999Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:orchestrator",
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
size = "S"
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
