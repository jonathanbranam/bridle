---
id: n4vk
title: The TUI doesn't show agents spawned after it starts
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [y496]
---

## What happened

The human, verbatim (2026-09-29): "I did not see that worker agent in the tui until
restarting it". meta-notes' manager had spawned worker `timeblock-hl` while `bridle tui` was
open; the agents pane didn't show it until the human restarted the TUI.

## Notes

- `run_app` (`crates/bridle-tui/src/run.rs`) calls `list_agents` once at startup. After that,
  `apply_event_to_agents` (`crates/bridle-tui/src/app.rs`) only updates the state of rows it
  already has (`agent.state`) and drops removed ones (`agent.removed`). Its doc comment says
  a new agent "only shows up on the next full `agents` list", but nothing lists again.
- Likely fix: on `agent.spawned` (and on SSE reconnect, which can miss events), re-fetch the
  agents list. Test: a spawn event with no prior row makes the agent appear.
