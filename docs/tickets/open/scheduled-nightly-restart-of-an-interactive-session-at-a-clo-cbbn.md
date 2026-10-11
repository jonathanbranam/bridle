---
id: cbbn
title: Scheduled nightly restart of an interactive session at a clock time (e.g. 3 AM)
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-cbbn]
---

## The ask

## The ask

The human, 2026-10-05: "Whatever agent I use to take all of my notes, I want it to just restart in the middle of the night. I think it feels a little risky to trust that system to work and the handover to work. ... maybe 3 am is better so that it's always restarted before the morning." And: "I think the orchestrator on Dalek has a rule that it gets refreshed every 12 hours. I don't want to add that rule to other agents unnecessarily."

Today max_uptime (12h) applies to the orchestrator only; gq9r covers context-driven handover, not a clock time.

## What's wanted

Opt-in per-session (or per-role/project) config such as restart_at = "03:00" in the human's time zone. At that time the daemon asks the session for its handover (using the role's handover instructions, ticket ft3b), waits with a deadline, restarts the session in its pane, and the new session opens with the note. Skip or defer when the human is mid-conversation (last_activity within N minutes). Other agents unaffected. First user: the notes project's advisor.

Needs: 4s3z (a session can be restarted reliably), gq9r, ft3b. Do not start before those merge; a task edge to be added at planning. Config is new and optional, so no migration. Document in docs/design/agent-host/ and config docs. Verify: just check, plus a daemon test with a fake clock: fires once, defers when active, restarts after the handover deadline.

Source: orchestrator@nuc, 2026-10-05. Sonnet; likely medium size, split if it grows.

## Decided: a plain config time, not a scheduler action (the human, 2026-10-10 ~11:55 PM ET)

The human, verbatim (to advisor product-manager):

> I think cbbn solution is simple: at a specific time, the daemon forces a handover of an agent. In
> the config I specify something like:
>
> at_time = 4:00 am

So this is option A of yfv5 Point 1 (its own config), not B (a `restart` action on the schedules
table). What that means for the build (advisor product-manager's reading; the human may correct it):

- **Config:** then, also verbatim:

  > project = "notes"
  > agent = "advisor"

  So a machine-wide entry naming the project and the agent, in `~/.bridle/config.toml`, one per
  session to restart:

  ```toml
  [[nightly_restart]]
  project = "notes"
  agent = "advisor"        # a role; "advisor/<name>" for a named advisor
  at_time = "4:00 am"
  ```

  Accept `"4:00 am"` and `"04:00"`. The time is the machine's local time (the human's zone). The
  daemon of that project acts on its own entries. No entry, no restart: other agents are
  unaffected. The orchestrator keeps its own `max_uptime`.
- **What happens at the time:** the daemon does what `bridle session restart <role>` does with
  `--handover` (the default): asks the session for its handover note, waits up to the handover
  deadline, then restarts the session in its pane; the new session opens with the note. "Forces":
  no deferring because the human was active, and if no note comes by the deadline the session is
  restarted anyway (said in the daemon log and the morning list). Once per day; a daemon that was
  down at the time does not catch up.
- **Dropped from the earlier brief:** the per-role handover instructions (ft3b) are not needed
  first: the existing handover request is enough. 4s3z and gq9r have landed. So nothing blocks it.
- Verify: just check, plus a daemon test with a fake clock: fires once at the time, not again that
  day, restarts after the deadline when no note comes, does nothing with no entry.
