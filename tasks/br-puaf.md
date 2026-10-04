+++
id = "br-puaf"
title = "Usage readings go stale while agents work, and a stale reading at low usage holds the workforce"
kind = "bug"
state = "planned"
created_at = "2026-10-04T15:18:54.499Z"
updated_at = "2026-10-04T17:43:41.938672Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
summary = "Governor usage poll now tries one HTTPS GET of the OAuth usage endpoint (via system curl, token on stdin; token from ~/.claude/.credentials.json or macOS keychain), new usage_http.rs. Only if that fails it starts a throwaway claude -p probe for that poll and kills it after; the resident probe and the ask-a-working-agent path are gone. Failures log at warn; each poll logs one info line (source, elapsed_ms, model_calls=0). Staleness slides: allowed age is max_staleness at hold_at, rising linearly to 6x at <=half of hold_at (also used for the (stale) flag in bridle budget). No config default changed. Caveat: curl is a runtime dependency (reqwest has no TLS here); the endpoint/token location is undocumented and untested against the live API. Tests: HTTP-ok (local server), HTTP-fail-then-probe, sliding staleness unit test; existing governor tests now script the probe in the repo cwd. Docs: usage-and-budget.md, daemon.md, cli.md, CHANGELOG."
+++

original id: puaf
Build per the Decided section of docs/tickets/open/usage-readings-go-stale-while-agents-work-and-a-stale-readin-puaf.md (read it and the task thread first; the human rejected the long-lived probe, option A). Summary: (1) usage poll tries direct HTTP with the OAuth token first; (2) only if HTTP fails, run the existing probe for that poll, started and exited each time (every 5 min, nothing left running); (3) staleness slides by usage: a stale reading matters only near the limits, not at 20-30%; (4) probe/HTTP failures logged at warn level; (5) no model call anywhere, and per-poll cost visible (bridle usage or a log line per poll). Default config changes only; ~/.bridle/config.toml is the human's, do not touch it. Files: budget/usage code in crates/bridle-daemon, config defaults, docs/design/agent-host/ and cli.md kept in step. Acceptance: just check passes; tests with the fake for HTTP-ok, HTTP-fail-then-probe, and sliding staleness. Model: Sonnet. Out of scope: one machine-wide source (that is kuw2, not scheduled).

## Thread

### note · external:orchestrator · 2026-10-04T15:19:27.972Z
priority: normal -> high

### note · external:orchestrator · 2026-10-04T15:19:28.007Z
The human, 2026-10-04 via aide: 'This is a noisy alert that shouldn't be happening ... If we're at 20% or 30% usage, I don't care if the [reading] is probably an hour old. If we're at 80% usage, then the sampling makes sense.' Readied by the orchestrator (bug fix). Default config changes only; ~/.bridle/config.toml is the human's.

### note · external:aide · 2026-10-04T15:28:29.205Z
The human, 2026-10-04, via aide, on the stopgap of raising max_staleness to 45m in ~/.bridle/config.toml: "Yes, I'm making a stop gap staleness for 45 minutes. I can't make that change right now, though, so we can just deal with it until later. [...] Yes, I approve that." They'll make the edit themselves later. They also want to keep track of what the probe costs (aide's answer: get_usage makes no model call, per spike 01 S8).

### note · external:orchestrator · 2026-10-04T15:28:50.880Z
Added via aide, 2026-10-04: the human wants to see what the usage probe costs. If the build changes the probe, keep it at no model call and make the cost visible (bridle usage, or a log line per poll).

### note · external:aide · 2026-10-04T15:31:17.089Z
The human, 2026-10-04, via aide, after hearing an idle probe is ~150 MB per daemon: "Let's pause that, then. That's way too much memory, especially, and that's always on, always going, and scales with every project. I do not like that solution. Let's find a better one." Option A (always-on probe) is rejected; the ticket now lists the candidates still open (direct HTTP, one machine-wide source, spawn per poll, sliding staleness).

### note · external:orchestrator · 2026-10-04T15:31:35.481Z
The human, 2026-10-04 via aide, on option A (long-lived probe per daemon): 'Let's pause that, then. That's way too much memory ... I do not like that solution. Let's find a better one.' Don't build A. Research first: direct HTTP with the OAuth token, one machine-wide source, spawn per poll. Warn-level probe-failure logging is unaffected.

### note · external:aide · 2026-10-04T15:36:51.317Z
The human, 2026-10-04, via aide: "Yeah, can we implement the HTTP call and then keep the probe implementation as a fallback? If the HTTP call is failing, then use the probe. Don't leave it running. Start it every 5 minutes." On one per machine: "Ideally, this would be one machine, not one per project, so that's an enhancement to consider." That part is in kuw2. Ticket updated under 'Decided'.

### note · external:orchestrator · 2026-10-04T15:37:03.930Z
The human, 2026-10-04 via aide: 'Yeah, can we implement the HTTP call and then keep the probe implementation as a fallback? If the HTTP call is failing, then use the probe. Don't leave it running. Start it every 5 minutes.' Build per the ticket's Decided section.

### note · agent:usage-http · 2026-10-04T17:43:41.938Z
done: usage polls try HTTP (curl, OAuth token) first, throwaway probe only on failure, sliding staleness, warn logs + per-poll info line (model_calls=0); just check green (1160 passed, no reruns); 5cb987db
