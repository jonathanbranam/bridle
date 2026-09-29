+++
id = "br-1fdb"
title = "Track each orchestrator session's starting context and growth (ct8m step 6)"
kind = "chore"
state = "planned"
created_at = "2026-09-29T20:34:11.070Z"
updated_at = "2026-09-29T20:38:02.679527Z"
size = "S"
+++

Ticket: docs/questions/open/lean-starting-context-for-every-agent-ct8m.md step 6; the supervisor in crates/bridle-daemon/src/orchestrator.rs already reads the session's context tokens (fx7x slice 2, docs/design/agent-host/orchestrator-supervision.md section 6). Goal: so 'how long can the orchestrator run' has a number. The supervisor appends an event (orchestrator.context) at most once per 10 minutes per session id when the reading changed, and once at the first reading, carrying session id, tokens, window size and uptime; no new table (events are pruned at 30 days already). Add a small read command only if trivial: bridle orchestrator context (last N events: start, peak, tokens per hour); otherwise document the events query (bridle events --kind ...) in cli.md. Files: orchestrator.rs, the events kind list wherever kinds are declared (grep for orchestrator.incident), cli.md, the design file's status, CHANGELOG. Acceptance: just check passes; tests with the fake clock: first reading logs, unchanged reading within 10 min does not, a lower reading (compact) logs. Model: Haiku. Out of scope: state-file shrink (step 5), thresholds (built), TUI display.

## Thread

### note · agent:manager-2 · 2026-09-29T20:38:02.679Z
Reprioritised: br-9e71 must go first on orchestrator.rs. Stop now: commit what you have as 'WIP' on your branch (needn't pass check), message me 'wip: <sha>', and end. Don't write a summary.
