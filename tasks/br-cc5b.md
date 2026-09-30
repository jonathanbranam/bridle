+++
id = "br-cc5b"
title = "Token counts in config accept k and M, defaults 150k/180k/200k (by23)"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T14:24:28.527Z"
updated_at = "2026-09-30T14:34:57.554178Z"
branch = "bridle/token-counts"
commit = "6fb42f3fdf2fc1d8c3238ad05d0802d9eda02f13"
summary = "Most of by23 was already built (TokenCount deserializer, parse_token_count, 150k/180k/200k defaults, docs). This change tightens parse_token_count (rejects sign, exponent, inf/nan, garbage, overflow; rounds) and adds tests: bad values name the key, an old integer-only config loads, mixed forms. CHANGELOG line added. No docs needed changes."
+++

Implement docs/tickets/open/token-counts-accept-k-and-m-by23.md (read it; verify what is already built first). Every token count in config (note_tokens, plan_tokens, handover_tokens, RawOrchestrator in crates/bridle-daemon/src/config.rs; any other token-count key you find) accepts an integer or a string like "150k" / "1.5M" (case-insensitive, k=1000, M=1000000); anything else is a config error naming the key. One parser beside parse_duration. Defaults change to 150k/180k/200k in code, tests and design docs; update examples in docs/design/agent-host/orchestrator-supervision.md and roles-and-config.md. Tests: integers and strings parse, bad values name the key, existing configs still load (an old integer-only config must not break serve). Keep it small and heavily tested: config parsing is on the daemon start-up path and must land by Thu 2026-10-01 10:00 ET, else it waits until the human is back (Fri night). CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: any other config keys or behavior.

## Thread

### note · agent:token-counts · 2026-09-30T14:34:43.791Z
done: by23 was mostly built already; tightened parser (rejects inf/nan/sign/exponent/overflow) + tests for key-naming and old integer configs, CHANGELOG; just check green (915 passed); f930f47

### note · agent:manager-2 · 2026-09-30T14:34:48.009Z
integrated: 6fb42f3fdf2fc1d8c3238ad05d0802d9eda02f13 (branch bridle/token-counts)

### note · agent:manager-2 · 2026-09-30T14:34:57.554Z
cleanup: removed agent token-counts, branch bridle/token-counts
