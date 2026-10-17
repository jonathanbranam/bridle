+++
id = "br-8kzq"
title = "bridle agent wake says 'timeout 6900 s' but the orchestrator's wait ends at 25 min"
kind = "bug"
state = "built"
created_at = "2026-10-16T21:09:32.161Z"
updated_at = "2026-10-17T04:41:35.828065Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:8kzq",
]
branch = "bridle/8kzq"
commit = "2d1331dc2a65b3134dd3f551662f46da3f8a5cd7"
summary = "CLI wake waits now default to the 6900 s cap: bridle agent wake and bridle orchestrator wait-for-wake send it when given no --timeout (agent.rs wake_wait_secs, orchestrator.rs wait_for_wake), so the printed timeout is the one applied. The daemon's bare /v1/orchestrator/wake default stays 25 min for raw API callers; api.md and cli.md say so. Unit test in agent.rs. CHANGELOG line added."
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

### note · agent:8kzq · 2026-10-17T04:25:31.424Z
built: 2d1331dc2a65b3134dd3f551662f46da3f8a5cd7 (branch bridle/8kzq)

### note · agent:8kzq · 2026-10-17T04:25:31.475Z
done: CLI wake default is now the 6900 s cap (agent wake and orchestrator wait-for-wake); api.md, cli.md, CHANGELOG updated; unit test; just check exit 0, 1785 run, 1785 passed, 6 skipped; commit 2d1331dc (main merged in). Note: j86p touches wake code, so re-merge main after it lands.

### note · agent:manager-2 · 2026-10-17T04:25:33.844Z
j86p has landed. Please merge main again, re-run just check, and message me the sha.

### note · agent:8kzq · 2026-10-17T04:41:35.828Z
done (re-merged main after j86p): d949937c; just check exit 0, 1787 run, 1787 passed, 6 skipped
