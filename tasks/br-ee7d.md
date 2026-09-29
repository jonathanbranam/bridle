+++
id = "br-ee7d"
title = "TUI: show agents spawned after startup (n4vk)"
kind = "bug"
state = "planned"
created_at = "2026-09-29T05:21:03.635Z"
updated_at = "2026-09-29T05:21:05.277276Z"
+++

Ticket: docs/questions/open/tui-doesnt-show-new-agents-n4vk.md (read it; it is the brief). Files: crates/bridle-tui (run.rs, app.rs). The agents list must pick up agents spawned (and removed) after the TUI started, e.g. by acting on agent.spawned/agent.removed events or refreshing on a timer; whichever the ticket's analysis favours. Resolve the ticket per docs/README.md. Acceptance: `just check` passes; a test on the app's event handling. Model: Sonnet. Run after the TUI scroll task (same files).
