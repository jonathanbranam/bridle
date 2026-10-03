+++
id = "br-bhcp"
title = "u6w9 a: daemon GET /v1/interactions + the interactions API types (ts-rs) for bridle-ui"
kind = "feature"
state = "planned"
created_at = "2026-10-03T21:02:57.106Z"
updated_at = "2026-10-03T21:21:36.300239Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
summary = "Daemon GET /v1/interactions?since= serves ~/.bridle/prompts.jsonl (Interaction{at,event,session,role,machine,project}; bad lines skipped; no event = prompt; human/local readers only) with client.interactions(); 'bridle focus reply' Stop hook added to both session settings logs event:reply, prompt lines get event:prompt; gateway ts-rs types (InteractionReport, DayReport, HoursReport, IntervalsReport + parts) in bridle-gateway/src/interactions.rs, bindings generated. Docs and CHANGELOG updated. Types are my design from the ticket; task b/c may adjust."
+++

Part 1 of 3 of br-u6w9 (read it and docs/tickets/open/track-the-human-s-time-and-attention-spent-talking-to-agents-u6w9.md, sections 'Design' and 'Gaps between writing and reading'). Human wants this built today; LAND FIRST: the UI's Time page is gated on the ts-rs types.
Build: (0) collection: 'bridle session' adds a Stop hook that appends {at,machine,project,role,session,event:"reply"} to ~/.bridle/prompts.jsonl; prompt lines (crates/bridle/src/focus.rs) get event:"prompt"; old lines without event are prompts. (1) daemon: GET /v1/interactions?since=<rfc3339> serves this machine's ~/.bridle/prompts.jsonl (format in docs/design/agent-host/roles-and-config.md), parsed incl. the event field, bad lines skipped; human or local readers only. Wire type in crates/bridle-api/src/types.rs, plus a client method. (2) gateway: the ts-rs response types for the /api/v1/interactions/* endpoints of the Design section (report, day with per-session intervals and concurrency peak and minutes at 1/2/3+, hours, raw intervals), in crates/bridle-gateway/src/types.rs so the generated TypeScript exists; no handlers yet (task c wires them).
Docs: api.md (daemon endpoint), human-web-ui.md (types), roles-and-config.md (the log, event field, hook), CHANGELOG. Tests: endpoint incl. bad line, old line without event, since filter; the Stop hook appends a reply line. Acceptance: just check passes. Model: Sonnet. Migration: none (old log lines stay valid; the hook comes with the binary's session setup). Out of scope: gateway collection and interval math (b), handlers (c), UI.

## Thread

### note · external:advisor · 2026-10-03T21:12:32.217Z
From the human, via advisor (2026-10-03): fill the gaps between writing and reading. Ticket u6w9, newest section 'Gaps between writing and reading' (pushed). Add to br-bhcp (collection): a Stop hook in 'bridle session' that appends an event:reply line to ~/.bridle/prompts.jsonl (prompt lines get event:prompt; old lines without event are prompts), served by the endpoint. Add to br-25nn (intervals): the gap cap runs from the reply's end, the agent's working time counts, and a 1-minute lead before the first prompt of a run (config 'lead'). Please fold these into the briefs before the workers start.

### note · agent:pm-1 · 2026-10-03T21:12:42.897Z
Folded the advisor's 'Gaps between writing and reading' additions into this brief (Stop hook, event field).

### note · agent:manager-2 · 2026-10-03T21:12:46.522Z
The brief was updated while you work: it now also needs (0) collection: 'bridle session' adds a Stop hook appending {at,machine,project,role,session,event:"reply"} to ~/.bridle/prompts.jsonl; prompt lines get event:"prompt"; old lines without event are prompts; the endpoint parses the event field; tests for old line without event and the hook appending a reply line; docs for the log/event/hook. Re-read 'bridle task show br-bhcp' and include it.

### note · agent:interactions-api · 2026-10-03T21:21:36.300Z
done: GET /v1/interactions + event field + Stop hook (bridle focus reply) + gateway ts-rs types; just check green (1084 tests); 29bed78
