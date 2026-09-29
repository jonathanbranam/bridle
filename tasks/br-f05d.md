+++
id = "br-f05d"
title = "Token counts in config accept k and M; defaults 150k/180k/200k (by23)"
kind = "feature"
state = "planned"
created_at = "2026-09-29T20:37:13.711Z"
updated_at = "2026-09-29T21:35:10.888637Z"
size = "S"
summary = "Implement token count parsing with k/M suffixes: added parse_token_count() parser supporting '150k' and '1.5M' formats (case-insensitive) for [orchestrator] note_tokens, plan_tokens, handover_tokens; updated defaults from 150K/210K/255K to 150K/180K/200K; updated examples in orchestrator-supervision.md and roles-and-config.md; added unit tests for parser and config parsing."
+++

original id: by23
Ticket: docs/questions/open/token-counts-accept-k-and-m-by23.md. note_tokens/plan_tokens/handover_tokens accept 150000, "150k" or "1.5M" (case-insensitive; k=1e3, M=1e6); anything else is a config error naming the key. One parser beside parse_duration in crates/bridle-daemon/src/config.rs. Update examples in orchestrator-supervision.md and roles-and-config.md. XS. Acceptance: just check passes; unit tests for the parser and a config using 150k.

## Thread

### note · external:advisor · 2026-09-29T20:40:10.521Z
Also (the human, 2026-09-29): change the defaults to note 150k / plan 180k / handover 200k (from 150K/210K/255K) in config.rs, its tests and the design docs. Ticket by23 updated.
