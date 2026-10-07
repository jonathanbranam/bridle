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
