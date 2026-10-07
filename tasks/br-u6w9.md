+++
id = "br-u6w9"
title = "Human interaction time: daemon serves the prompt log; gateway collects across machines and reports (u6w9)"
kind = "feature"
state = "pending"
created_at = "2026-10-03T21:01:56.091Z"
updated_at = "2026-10-05T15:24:32.211937Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
ticket = "u6w9"
+++

Ticket: docs/tickets/open/track-the-human-s-time-and-attention-spent-talking-to-agents-u6w9.md, section "Design" (read the whole ticket; the human approved building it today, 2026-10-03: "make reasonable design decisions and ask for it to be implemented today").
Goal (bridle side; the UI is a separate bridle-ui task): (1) Daemon: `GET /v1/interactions?since=<rfc3339>` serves this machine's `~/.bridle/prompts.jsonl` lines (parsed; tolerate bad lines), human or local readers only. (2) Gateway: every 5 minutes poll every daemon it knows (local and other machines, the same way its listing reaches them) for interactions and for messages `from = human`; dedupe by (machine, session, at) / message id; keep the merge in the gateway's own store; report unreachable machines without failing. (3) Gateway computes intervals: per session, a prompt counts the time to the next prompt in that session capped at `gap` (default 10m), the last of a run counts `tail` (default 2m); human time = union of intervals; concurrency = sessions covering a moment. Config `[interactions] gap, tail` in the gateway config. Days in US Eastern. (4) API under /api/v1 with ts-rs types: `/interactions/report?from&to&group=project|agent|machine&bucket=day|week`, `/interactions/day?date=` (per-session intervals, concurrency: peak and minutes at 1/2/3+), `/interactions/hours?from&to&days=weekday|weekend|mon,...`, and raw intervals. Agent = the prompt's `role`.
Tests: interval rules (cap, tail, union, concurrency) with fixed fixtures; dedupe; the endpoint; one unreachable daemon. Docs: human-web-ui.md, api.md (daemon endpoint), roles-and-config.md (the log), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none (new endpoint, gateway store). Out of scope: CLI reads, anything outside bridle, the UI.

## Thread

### note · agent:pm-1 · 2026-10-03T21:03:12.164Z
Split into br-bhcp (a: daemon endpoint + ts-rs types, lands first), br-25nn (b: gateway collect/store/intervals), br-59vt (c: handlers). Chained by dependency edges; queued as tiers 1-3, ahead of everything else. Human approved building today.

### note · system · 2026-10-05T15:24:32.205Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.
