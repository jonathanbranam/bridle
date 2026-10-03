+++
id = "br-25nn"
title = "u6w9 b: gateway collects interactions and human messages every 5 min, stores them, computes intervals"
kind = "feature"
state = "planned"
created_at = "2026-10-03T21:03:00.106Z"
updated_at = "2026-10-03T21:12:42.915635Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Part 2 of 3 of br-u6w9 (read it and docs/tickets/open/track-the-human-s-time-and-attention-spent-talking-to-agents-u6w9.md, sections 'Design' and 'Gaps between writing and reading'). Human wants this built today. Needs task a (daemon GET /v1/interactions, event:prompt|reply lines, the client method).
Build in crates/bridle-gateway: (1) every 5 minutes poll every daemon the gateway knows (local and other machines, the same way its listing reaches them) for interactions and for messages from=human; dedupe by (machine, session, at) / message id; keep the merge in the gateway's own store; report unreachable machines without failing. (2) interval computation, per session: after a prompt, time counts through the agent's turn (waiting counts as the human's attention) and on to the next prompt if it comes within gap (default 10m) of the reply finishing, else the run ends at reply end + tail (default 2m); no reply recorded: a prompt counts to the next prompt capped at gap, the last of a run counts tail, as before. The first prompt of a run gets a lead (default 1m) before it. Human time = union of intervals; concurrency = sessions covering a moment. Config [interactions] gap, tail, lead in the gateway config (config.rs). Days in US Eastern. Agent = the prompt's role.
Pure functions for the interval math, so they test with fixed fixtures. Tests: cap from reply end, waiting counts, lead, no-reply fallback, tail, union, concurrency, dedupe, one unreachable daemon. Docs: human-web-ui.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none (new gateway store). Out of scope: HTTP handlers (c), the UI.

## Thread

### note · agent:pm-1 · 2026-10-03T21:12:42.915Z
Folded the advisor's 'Gaps between writing and reading' additions into this brief (cap from reply end, waiting counts, lead).
