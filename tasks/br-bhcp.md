+++
id = "br-bhcp"
title = "u6w9 a: daemon GET /v1/interactions + the interactions API types (ts-rs) for bridle-ui"
kind = "feature"
state = "planned"
created_at = "2026-10-03T21:02:57.106Z"
updated_at = "2026-10-03T21:03:04.434273Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Part 1 of 3 of br-u6w9 (read it and docs/tickets/open/track-the-human-s-time-and-attention-spent-talking-to-agents-u6w9.md, section 'Design'). Human wants this built today; LAND FIRST: the UI's Time page is gated on the ts-rs types.
Build: (1) daemon: GET /v1/interactions?since=<rfc3339> serves this machine's ~/.bridle/prompts.jsonl (written by crates/bridle/src/focus.rs; format in docs/design/agent-host/roles-and-config.md), parsed, bad lines skipped; human or local readers only. Wire type in crates/bridle-api/src/types.rs, plus a client method. (2) gateway: the ts-rs response types for the /api/v1/interactions/* endpoints of the Design section (report, day with per-session intervals and concurrency peak and minutes at 1/2/3+, hours, raw intervals), in crates/bridle-gateway/src/types.rs so the generated TypeScript exists; no handlers yet (task c wires them).
Docs: api.md (daemon endpoint), human-web-ui.md (types), roles-and-config.md, CHANGELOG. Tests: endpoint incl. bad line and since filter. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: gateway collection and interval math (b), handlers (c), UI.
