+++
id = "br-25nn"
title = "u6w9 b: gateway collects interactions and human messages every 5 min, stores them, computes intervals"
kind = "feature"
state = "planned"
created_at = "2026-10-03T21:03:00.106Z"
updated_at = "2026-10-03T21:03:04.451553Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Part 2 of 3 of br-u6w9 (read it and docs/tickets/open/track-the-human-s-time-and-attention-spent-talking-to-agents-u6w9.md, section 'Design'). Human wants this built today. Needs task a (daemon GET /v1/interactions and the client method).
Build in crates/bridle-gateway: (1) every 5 minutes poll every daemon the gateway knows (local and other machines, the same way its listing reaches them) for interactions and for messages from=human; dedupe by (machine, session, at) / message id; keep the merge in the gateway's own store; report unreachable machines without failing. (2) interval computation: per session, a prompt counts the time to the next prompt in that session capped at gap (default 10m); the last of a run counts tail (default 2m); human time = union of intervals; concurrency = sessions covering a moment. Config [interactions] gap, tail in the gateway config (config.rs). Days in US Eastern. Agent = the prompt's role.
Pure functions for the interval math, so they test with fixed fixtures. Tests: cap, tail, union, concurrency, dedupe, one unreachable daemon. Docs: human-web-ui.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none (new gateway store). Out of scope: HTTP handlers (c), the UI.
