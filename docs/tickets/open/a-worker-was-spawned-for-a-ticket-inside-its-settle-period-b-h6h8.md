---
id: br-h6h8
title: A worker was spawned for a ticket inside its settle period, before it was planned (dotfiles-local dl-v27w)
kind: bug
created: 2026-10-16
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Reported by orchestrator@vesta, 2026-10-15 (m-13904, carried from meta-notes handovers h-0044/h-0045):
on dotfiles-local, the manager started a worker for dl-v27w as soon as the ticket was readied,
before its settle hold ended and before it was planned and queued.

Per roles-and-config.md (`[tasks] settle`, default 10m) and coordination.md "Settling", a ticket
waits after creation / a human comment or edit / a relay of the human's words before any work
starts on it: claiming, planning, or `agent spawn --ticket`, in any state. And a manager should
only spawn for a planned, queued task.

To find out: dotfiles-local's events and the task's thread for dl-v27w (ready time, settle
window, the spawn call). Was the spawn made without `--ticket` (so the gate never saw it), with
a `skip-settle`, or did the daemon's gate let it through? Also whether the daemon refuses a
`--ticket` spawn for a task that isn't planned, or leaves that to the manager's prompt.

Wanted: whichever gate is missing is enforced by the daemon (a spawn naming a task in its settle
period, or not planned, is refused with the reason), with a test; or, if the spawn bypassed the
ticket link, the manager role/skill is fixed so spawns always name the ticket.
