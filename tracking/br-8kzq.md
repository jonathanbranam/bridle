+++
id = "br-8kzq"
title = "bridle agent wake says 'timeout 6900 s' but the orchestrator's wait ends at 25 min"
kind = "bug"
state = "claimed"
created_at = "2026-10-16T21:09:32.161Z"
updated_at = "2026-10-17T03:57:00.199424Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:8kzq",
]
ticket = "8kzq"
+++

docs/tickets/open/bridle-agent-wake-says-timeout-6900-s-but-the-orchestrator-s-8kzq.md

## Thread

### note · external:aide · 2026-10-16T21:41:43.035Z
The human, 2026-10-16 ~5:35 PM ET (via aide): "Yes that looks good. I think the default wait could be higher. If things are working, why wake up agents more often?"

### note · agent:pm-1 · 2026-10-16T21:42:55.010Z
Brief (pm-1): the ask is the brief; the human wants the default wait higher ("if things are working, why wake up agents more often?"). Do: the daemon's default (wake::POLL_TIMEOUT, crates/bridle-daemon/src/wake.rs:34, clamp_timeout in server.rs ~878) and the CLI agree on 6900 s: simplest is the CLI sends its 6900 s default (or the daemon default becomes 6900). Update api.md and cli.md. Test: a wake with no --timeout is sent/applied as 6900. Acceptance: just check. Model: Haiku. Small; touches wake code only.

### note · agent:manager-2 · 2026-10-17T03:57:00.199Z
Heads up: bridle/j86p touches wake.rs and server.rs near the wake timeout and is landing now. After it lands, merge main again before reporting done.
