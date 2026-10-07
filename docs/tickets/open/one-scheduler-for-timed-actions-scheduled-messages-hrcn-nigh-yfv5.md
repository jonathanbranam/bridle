---
id: yfv5
title: "One scheduler for timed actions: scheduled messages (hrcn), nightly session restarts (cbbn), maintenance windows for upgrades and reboots (3nyk)"
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-yfv5]
---

## The ask

## Status

A design ticket. No build task until the human refines it (like hrcn). The task br-yfv5 stays pending.

## The ask

The human, 2026-10-05 (NUC orchestrator): "There's a ticket on bridle somewhere about having different types of scheduled message sends, I think, that can occur either on some form of cron or at specific points in the future. That's something that the notes agent is going to use a lot to ensure that they wake up in enough time before a meeting or something they need to send me a reminder for. We've also discussed the ability to reboot all agents for a machine upgrade in the middle of the night. These are all related things that probably should be solved in the same way."

Related asks that should share one mechanism:

- hrcn: scheduled messages, one-time or cron, set by an agent for itself or by the human for any agent (notes: wake the notes agent ahead of a meeting).
- cbbn: restart an interactive session at a clock time (notes agent at 3 AM), handover first, using the role's handover instructions (ft3b).
- 3nyk: pause work before a planned reboot, hand over, resume after.
- Machine upgrades: build and install bridle, restart every daemon on the machine, resume agents, at night (today on the NUC it is by hand; self_upgrade only works where the bridle project's own daemon runs).

## Shape to consider

One schedule record (daemon DB, survives restarts; human time zone with DST; one-time or cron; what to do on a missed firing) with an action: send a message (hrcn), hand over and restart a session (cbbn), or a maintenance window (3nyk). One CLI (bridle schedule add/list/rm) for agents and the human. Machine-wide actions need a home above one project's daemon (cy2v, one watcher for every project).

## For the PM / human to settle

- Does cbbn wait for this scheduler, or ship first as a small restart_at config (the notes agent needs it soon) and become one action of the scheduler later? PM recommendation: ship cbbn first, simple, behind its own config; fold it in when the scheduler is designed. Keep the restart action reusable.
- Split: per-project scheduler for messages and restarts first; the machine-wide maintenance window waits on cy2v.

Source: orchestrator@nuc, 2026-10-05.
