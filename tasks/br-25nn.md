+++
id = "br-25nn"
title = "u6w9 b: gateway collects interactions and human messages every 5 min, stores them, computes intervals"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T21:03:00.106Z"
updated_at = "2026-10-03T21:58:19.703915Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/interactions-collect"
commit = "62488ad84c8932becd4d4760708b5f7920acfd50"
summary = "Gateway collects interactions: collect.rs polls every 5 min (one daemon per machine for the prompt log via GET /v1/interactions?since=, every project daemon for from=human messages, modelled as point prompts in session message:<recipient>), merges into an append-only JSONL store <home>/gateway-interactions.jsonl deduped by key, reports unreachable machines/projects. intervals.rs: pure intervals/union/human_time/concurrency per the brief (gap from reply end, waiting counts, lead, tail; no-reply fallback: next prompt if within gap else tail). Config [interactions] gap/tail/lead (defaults 10m/2m/1m). bridle gateway spawns the collector (not serve(), so tests don't poll real daemons). Caveats: Eastern-day bucketing left to handlers (c); handlers need the Store, which gateway.rs currently doesn't pass to serve."
+++

Part 2 of 3 of br-u6w9 (read it and docs/tickets/open/track-the-human-s-time-and-attention-spent-talking-to-agents-u6w9.md, sections 'Design' and 'Gaps between writing and reading'). Human wants this built today. Needs task a (daemon GET /v1/interactions, event:prompt|reply lines, the client method).
Build in crates/bridle-gateway: (1) every 5 minutes poll every daemon the gateway knows (local and other machines, the same way its listing reaches them) for interactions and for messages from=human; dedupe by (machine, session, at) / message id; keep the merge in the gateway's own store; report unreachable machines without failing. (2) interval computation, per session: after a prompt, time counts through the agent's turn (waiting counts as the human's attention) and on to the next prompt if it comes within gap (default 10m) of the reply finishing, else the run ends at reply end + tail (default 2m); no reply recorded: a prompt counts to the next prompt capped at gap, the last of a run counts tail, as before. The first prompt of a run gets a lead (default 1m) before it. Human time = union of intervals; concurrency = sessions covering a moment. Config [interactions] gap, tail, lead in the gateway config (config.rs). Days in US Eastern. Agent = the prompt's role.
Pure functions for the interval math, so they test with fixed fixtures. Tests: cap from reply end, waiting counts, lead, no-reply fallback, tail, union, concurrency, dedupe, one unreachable daemon. Docs: human-web-ui.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none (new gateway store). Out of scope: HTTP handlers (c), the UI.

## Thread

### note · agent:pm-1 · 2026-10-03T21:12:42.915Z
Folded the advisor's 'Gaps between writing and reading' additions into this brief (cap from reply end, waiting counts, lead).

### note · agent:interactions-collect · 2026-10-03T21:50:10.411Z
done: gateway collector (5-min poll, append-only store, dedupe, unreachable reported), pure interval/union/concurrency math, [interactions] gap/tail/lead config, docs+CHANGELOG; just check green (1096 tests); f8ecc9e. Note for c: Store is created in crates/bridle/src/gateway.rs and not yet passed to serve(); Eastern-day bucketing is left to the handlers.

### note · agent:manager-2 · 2026-10-03T21:58:19.703Z
integrated: 62488ad84c8932becd4d4760708b5f7920acfd50 (branch bridle/interactions-collect)
