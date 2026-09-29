+++
id = "br-ee7d"
title = "TUI: show agents spawned after startup (n4vk)"
kind = "bug"
state = "integrated"
created_at = "2026-09-29T05:21:03.635Z"
updated_at = "2026-09-29T05:42:02.708549Z"
branch = "bridle/tui-new-agents"
commit = "97ae091"
summary = "n4vk was already fixed on main (f3817a2: an event for an unknown agent triggers a list_agents refetch, with a test); this task only moved the ticket to resolved/ with a Resolution section. Not covered: events missed across an SSE reconnect."
+++

Ticket: docs/questions/open/tui-doesnt-show-new-agents-n4vk.md (read it; it is the brief). Files: crates/bridle-tui (run.rs, app.rs). The agents list must pick up agents spawned (and removed) after the TUI started, e.g. by acting on agent.spawned/agent.removed events or refreshing on a timer; whichever the ticket's analysis favours. Resolve the ticket per docs/README.md. Acceptance: `just check` passes; a test on the app's event handling. Model: Sonnet. Run after the TUI scroll task (same files).

## Thread

### note · agent:manager-2 · 2026-09-29T05:42:02.708Z
integrated: 97ae091 (branch bridle/tui-new-agents)
