+++
id = "br-yfv5"
title = "One scheduler for timed actions: scheduled messages (hrcn), nightly session restarts (cbbn), maintenance windows for upgrades and reboots (3nyk)"
kind = "feature"
state = "pending"
created_at = "2026-10-05T10:47:07.760Z"
updated_at = "2026-10-05T10:47:20.780204Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
ticket = "yfv5"
+++

docs/tickets/open/one-scheduler-for-timed-actions-scheduled-messages-hrcn-nigh-yfv5.md

submitted by external:orchestrator@nuc

The human, 2026-10-05 (NUC orchestrator): 'There's a ticket on bridle somewhere about having different types of scheduled message sends, I think, that can occur either on some form of cron or at specific points in the future. That's something that the notes agent is going to use a lot to ensure that they wake up in enough time before a meeting or something they need to send me a reminder for. We've also discussed the ability to reboot all agents for a machine upgrade in the middle of the night. These are all related things that probably should be solved in the same way.'

Design ticket (no task until the human refines it, like hrcn). Related asks that should share one mechanism:
- hrcn: scheduled messages, one-time or cron, set by an agent for itself or by the human for any agent. Notes use: wake the notes agent ahead of a meeting or a reminder it must send.
- br-cbbn: restart an interactive session at a clock time (notes agent at 3 AM), with handover first and the role's handover instructions (br-ft3b).
- 3nyk: pause work before a planned reboot, hand over, resume after.
- Machine upgrades: build and install bridle, restart every daemon on the machine, resume agents, at night (today on the NUC it's by hand: temp worktree, just install, then 'bridle daemon restart' per project; self_upgrade only works where the bridle project's own daemon runs).

Shape to consider: one schedule record (daemon DB, survives restarts; human time zone with DST; one-time or cron; what to do on a missed firing) with an action: send a message (hrcn), hand over and restart a session (cbbn), or a maintenance window (pause, hand over, then upgrade/reboot/restart daemons, resume; 3nyk). One CLI (bridle schedule add/list/rm) for agents and the human. Machine-wide actions need a home above one project's daemon (cy2v, one watcher for every project).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T10:47:07.761Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T10:47:20.780Z
Triage (pm-1): accept as a design ticket (minted, uncommitted; commit on main). No build task: it needs the human's refinement first, as the submitter says, so the task stays pending. My recommendation (also in the ticket): ship br-cbbn first as its own small restart_at config, then fold it into the scheduler; per-project messages and restarts before the machine-wide maintenance window (waits on cy2v). Related: hrcn, 3nyk, br-ft3b.
