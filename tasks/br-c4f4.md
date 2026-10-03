+++
id = "br-c4f4"
title = "Agent wake gets its own 2h cap; advisor waits 90 minutes, not 5 (789x)"
kind = "chore"
state = "integrated"
created_at = "2026-10-03T00:00:32.650Z"
updated_at = "2026-10-03T01:13:55.924468Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/wake-cap"
commit = "e22e2d8b248e6ddf0264071a586e257a9d3af78f"
summary = "Both wake routes now clamp timeout_secs with one shared wake::clamp_timeout to wake::MAX_WAKE_TIMEOUT (6900 s, 1 h 55 min). Agent wake defaults to the cap; the orchestrator wake keeps its 25 min default (POLL_TIMEOUT) and gains a timeout_secs query parameter (OrchestratorWakeQuery) plus `--timeout` on `bridle orchestrator wait-for-wake` (and --mail, clamped to 6900 in the CLI). The client has no total HTTP timeout (plain reqwest::Client::new), so a long wait doesn't die client-side; no change needed there. Advisor role prompt uses --timeout 5400 as a fallback only. Tests: a unit test of the clamp, and a CLI parse test. Docs: api.md, cli.md, CHANGELOG. The orchestrator role prompt now tells the orchestrator to choose its own --timeout (long when quiet, shorter when busy, max 6900)."
+++

original id: 789x
Ticket: docs/tickets/open/advisor-wake-waits-about-90-minutes-not-5-789x.md (read it; human's words verbatim; the human approved the cap raise directly on 2026-10-03). Code: crates/bridle-daemon/src/server.rs (~540, the agent wake handler clamps timeout_secs to wake::POLL_TIMEOUT), the wake module, bridle-api client and the CLI 'bridle agent wake' (crates/bridle). Docs: docs/design/agent-host/api.md and cli.md; role: workflow/base/roles/advisor.md ('Waiting for messages').
Goal: (1) Agent wake (the per-agent/advisor wake) gets its OWN maximum of 2 hours; the orchestrator's wake::POLL_TIMEOUT (25 min) is unchanged and still governs the orchestrator wake. (2) Check the client side (bridle-api client and the CLI) has no shorter request/HTTP timeout than the wait; fix if it does, so a 90-minute wait doesn't die client-side. The CLI's own --timeout cap, if any, matches the new max. (3) The advisor role prompt uses --timeout 5400 (90 min), no shell loop, named and unnamed forms; say the timeout is only a fallback (a message ends the wait at once). Tests: a timeout above 25 min is honoured up to the new cap and clamped above it; the orchestrator wake still clamps at 25 min; client does not time out early (fake server or a unit check of the configured timeout). Docs: api.md, cli.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: other wake behaviour, the orchestrator wake.
OVERNIGHT: small daemon change, human approved. Keep the change confined to the wake clamp and client timeout; do not touch daemon start-up, restart or upgrade paths. It must not interrupt the gateway work: lower priority than the gateway tasks.

## Thread

### note · external:advisor · 2026-10-03T00:15:11.347Z
From the human, via advisor (2026-10-03): the agent wake cap is exactly 6900 s (1 h 55 min), not 2 hours: Claude Code kills background tasks at 2 h and the human wants no process killed. Clamp above 6900; callers pick any timeout up to it. Recorded in 789x.

### note · external:advisor · 2026-10-03T00:29:48.737Z
From the human, via advisor (2026-10-03), added scope: 'bridle orchestrator wait-for-wake' (and --mail) gets a --timeout flag, max 6900 s like agent wake, so the orchestrator picks its own (long when quiet, shorter when busy or unstable). Today it has no flag and the daemon answers 'nothing' at 25 min; GET /v1/orchestrator/wake needs a timeout_secs parameter. The orchestrator role prompt says to choose. waiter_grace (15 m) is unaffected: it counts an open request as waiting. Recorded in 789x.

### note · external:advisor · 2026-10-03T00:38:24.444Z
From advisor: overlap with br-2672 (parked branch, lands Sat after review): it turns wait-for-wake into a thin alias of agent wake. Keep the --timeout change easy to carry over (one clamp constant shared by both routes).

### note · agent:wake-cap · 2026-10-03T01:11:24.726Z
done: wake cap 6900 s via one shared clamp (both routes), wait-for-wake/--mail --timeout, advisor prompt 5400; just check passed (1046 tests); 1681313. Not done: orchestrator role prompt doesn't yet say to choose a timeout.

### note · agent:manager-2 · 2026-10-03T01:11:26.946Z
Please finish the last item: the advisor note says the orchestrator role prompt (workflow/base/roles/orchestrator.md) must say to choose its own wait-for-wake --timeout (long when quiet, shorter when busy or unstable, max 6900). Add one or two lines, merge main, run just check, message me sha and result.

### note · agent:wake-cap · 2026-10-03T01:12:41.710Z
done: orchestrator role now says to choose --timeout (max 6900); main already merged; just check passed (1046 tests); 60134f3

### note · agent:manager-2 · 2026-10-03T01:12:45.162Z
Two things: (1) update the task summary (bridle task summary br-c4f4): it still says the orchestrator prompt is not done; (2) main moved, merge it, run just check, message me sha and result.

### note · agent:wake-cap · 2026-10-03T01:13:43.886Z
done: summary updated; main merged; just check passed (1046 tests); 766363f

### note · agent:manager-2 · 2026-10-03T01:13:46.048Z
integrated: e22e2d8b248e6ddf0264071a586e257a9d3af78f (branch bridle/wake-cap)

### note · agent:manager-2 · 2026-10-03T01:13:55.924Z
cleanup: removed agent wake-cap, branch bridle/wake-cap
