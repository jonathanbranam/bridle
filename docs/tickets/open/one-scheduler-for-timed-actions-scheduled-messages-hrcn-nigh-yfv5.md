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

## Design options

Focus: interface (the schedule CLI and what an action is), with a little internal architecture
(where the loop lives). Designer dyfv5, 2026-10-10. Priority per the human, 2026-10-10: scheduled
messages are next; the one-daemon-per-machine question (kuw2) is not ready and is not designed for.

### What exists

Built (br-9xze, `schedule.rs`, daemon.md "Scheduled messages"): `bridle schedule add/list/rm`, a
`schedules` table (`sc-xxxx`, created_by, target, body, kind once|cron, tz, next_fire_at, state), a
15 s loop that sends due rows as a system note through the normal message path, DST and missed-firing
rules, per project. The only action is "send body to target". Who: the human for anyone; an agent for
itself; everyone else refused (`schedule_actor_ok`, server.rs). Not built: restarts (cbbn),
maintenance windows (3nyk), anything machine-wide. Today's nearest relatives: `max_uptime` (12 h,
orchestrator only) and the context-driven handover (gq9r); neither is a clock time.

### Point 1: does cbbn wait for the scheduler?

The scheduler's mechanism now exists, so the PM's "ship cbbn first behind its own config" no longer
saves anything: a second clock and second config for the same job is the near-duplicate
`design-principles` warns about.

- **A. cbbn as its own `restart_at` config.** The user edits config, restarts the daemon to change it.
  Cost: a second timer, a second place for time zone and DST rules, and a later fold-in. Falls short:
  one name per action (two ways to say "at 3 AM").
- **B. cbbn as a second action on the schedules table.** `bridle schedule add --cron "0 3 * * *"
  --restart notes-advisor` (agent for itself, or the human for anyone). Cost: one nullable column
  (see Point 3) and a restart routine. Uses the existing loop, zone rules and list/rm.
- **C. Do nothing:** the notes agent keeps being restarted by hand. Honest, but it is the stated need.

**Recommend B.** Ship cbbn as the restart action of the scheduler. The restart routine itself (ask
for handover with the role's instructions (ft3b), wait for the deadline, restart in the pane, defer
when the human was active within N minutes) is the real work of cbbn and is the same under A or B, so
nothing is wasted. Accepts: cbbn now depends on the schedule table's shape, and still waits on its
own needs (4s3z, gq9r, ft3b), which is true for A as well.

**Decided (the human, 2026-10-10 ~11:55 PM ET): A.** "I think cbbn solution is simple: at a specific time, the daemon forces a handover of an agent. In the config I specify something like: at_time = 4:00 am". Details in ticket [[scheduled-nightly-restart-of-an-interactive-session-at-a-clo-cbbn|cbbn]]. Point 3's action column is not needed for cbbn.

### Point 2: per-project first, machine-wide later

**Recommend: per project for messages and restarts; the maintenance window (3nyk) waits on cy2v / kuw2.**
Unchanged from the PM and the human's 2026-10-10 caveat. Nothing here is designed for kuw2.

What would change under one daemon per machine (noted, not built): the loop and the table would move
to the machine daemon (or stay per project and be driven by it); `target` would need a project part
(`project/agent`) or the schedule would carry a project column; message delivery to a target becomes a
call into that project's agent host rather than the in-process message path. The CLI text
(`bridle schedule ...`) and the action names would not change. That is the cost of not designing for
it now, and it is small: today's `target` is a bare principal and the one place that sends is
`schedule.rs`.

### Point 3: what an action is (how cbbn and 3nyk reuse the table and loop)

**Recommend: one nullable `action` column, default `message`.** Rows keep `kind` (when: once|cron)
separate from `action` (what). The loop is unchanged: find due rows, call `fire(row)`, which matches
on action.

| action  | `target` means | `body` means          | firing does                                        |
|---------|----------------|-----------------------|----------------------------------------------------|
| message | principal      | the message           | send as a system note (today)                      |
| restart | agent name     | optional extra note   | handover, wait, restart in the pane (cbbn)         |
| window  | scope (project)| the wind-down note    | hold new turns, ask for handover, resume after (3nyk) |

CLI: `bridle schedule add` keeps its flags; a restart is `--restart` (replaces the message text) and a
window would be `--window 20m` later. One verb, one list: `schedule list` shows an ACTION column.
Rejected: a table per action, or a trait/plugin layer for actions (YAGNI: three known actions, a
`match` is enough; `modularity` is kept by each action's routine living in its own function/module,
called from `fire`).

Reuse detail for later slices:
- cbbn: the restart routine is a function of (agent, deadline); the schedule just calls it. A
  "defer when the human is mid-conversation" skip re-arms the row for N minutes later without
  marking it fired. Missed-firing rule: a restart missed by hours should be skipped, not run late
  (a stale 3 AM restart at noon is unwanted); needs a per-action "max lateness" (message: unlimited
  as today, restart: e.g. 1 h). Decide when cbbn is built.
- 3nyk: a window is a start and an end, so two rows (or one row with a duration). Per project it
  holds work and asks the orchestrator for a handover; it is only useful machine-wide, so it waits on
  kuw2/cy2v. After a boot (4r3k) the missed-firing rule resumes agents. Do not build it before then.

Quiet hours (a stated "Not done" of slice 1) are not part of this; a schedule is an explicit human or
agent request, so it fires through quiet hours. Recommend leaving it as is and saying so in docs.

### Point 4: schedules refuse external principals (known gap)

Today an `external` caller (orchestrator, advisor, aide: the long-lived interactive sessions that most
need reminders; the notes aide is the first user) gets 403 "only agents and the human use schedules".

- **A. Leave it.** The aide asks the human or a spawned agent. Falls short: the user's side first;
  the primary scheduling user (notes) is exactly an external.
- **B. Externals follow the agent rule:** may add for themselves (target must equal their own
  principal) and list/rm only their own. Change is the one match in `schedule_actor_ok` plus the
  self-target compare (`bare()` already handles the `agent:` prefix; externals need the same identity
  compare). Cost: tiny. Caveat: identities are shared (every project's aide is `external:aide`; two
  unnamed advisors are both `external:advisor`, per the no-kill-by-name rule), so one external can
  see and remove the other's schedules, and the message wakes whichever session reads the inbox.
  Acceptable for the same human's own sessions.
- **C. Externals may target anyone.** Rejected: widens who can wake whom; nobody asked.

**Recommend B**, as its own small ticket (it does not depend on the action column). Peers, visitors
and system stay refused.

**Point 4 decided (the human, 2026-10-11 ~12:30 AM ET): B.** "those internal agents barely need it; the purpose of this is specifically for the extern agent. Yes, create a ticket and fix that and get it done tonight." Ticket [[externals-orchestrator-advisors-aides-can-schedule-messages-3k7d|3k7d]].

### Recommendation in one list

1. cbbn is the `restart` action on the schedules table, not a separate config (B).
2. Per project now; the maintenance window waits on kuw2/cy2v; note in docs what moves under kuw2.
3. Add a nullable `action` column (default `message`) when the first non-message action is built; no
   action framework.
4. Let externals schedule for themselves (B); build it first, it is small and unblocks the notes aide.
5. Order: externals-self (small) -> cbbn restart action (after 4s3z, gq9r, ft3b) -> 3nyk window
   (after kuw2/cy2v and 4r3k). Hrcn's "human schedules for any agent" is already built.

### Rejected, so nobody re-proposes them

- A separate `restart_at` config (Point 1 A): second clock, second DST rule set.
- A table or plugin layer per action: no need with three actions.
- Designing the machine-wide scheduler now: the human said the architecture is not ready.
- Blocking on quiet hours: not asked for.

Migration: none for this design; the `action` column is a later additive schema step, done with cbbn.
