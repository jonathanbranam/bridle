+++
id = "br-cc5b"
title = "Token counts in config accept k and M, defaults 150k/180k/200k (by23)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T14:24:28.527Z"
updated_at = "2026-09-30T14:24:31.909133Z"
+++

Implement docs/tickets/open/token-counts-accept-k-and-m-by23.md (read it; verify what is already built first). Every token count in config (note_tokens, plan_tokens, handover_tokens, RawOrchestrator in crates/bridle-daemon/src/config.rs; any other token-count key you find) accepts an integer or a string like "150k" / "1.5M" (case-insensitive, k=1000, M=1000000); anything else is a config error naming the key. One parser beside parse_duration. Defaults change to 150k/180k/200k in code, tests and design docs; update examples in docs/design/agent-host/orchestrator-supervision.md and roles-and-config.md. Tests: integers and strings parse, bad values name the key, existing configs still load (an old integer-only config must not break serve). Keep it small and heavily tested: config parsing is on the daemon start-up path and must land by Thu 2026-10-01 10:00 ET, else it waits until the human is back (Fri night). CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: any other config keys or behavior.
