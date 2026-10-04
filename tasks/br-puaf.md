+++
id = "br-puaf"
title = "Usage readings go stale while agents work, and a stale reading at low usage holds the workforce"
kind = "bug"
state = "open"
created_at = "2026-10-04T15:18:54.499Z"
updated_at = "2026-10-04T15:31:35.481357Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
+++

original id: puaf
docs/tickets/open/usage-readings-go-stale-while-agents-work-and-a-stale-readin-puaf.md

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
