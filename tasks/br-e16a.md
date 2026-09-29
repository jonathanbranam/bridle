+++
id = "br-e16a"
title = "Budget: the human's thresholds win over Claude's allowed_warning (kv7d)"
kind = "bug"
state = "planned"
created_at = "2026-09-29T00:46:10.185Z"
updated_at = "2026-09-29T00:46:47.893824Z"
+++

Decided (human, 2026-09-28): my settings override Claude warnings. Ticket docs/questions/open/allowed-warning-overrides-the-override-kv7d.md has the incident. Goal: Claude Code's allowed_warning rate-limit status (sent near 90% five_hour) is information only; the configured thresholds (hold_at, wind_down_at, stop_at, overrides) alone decide the governor state. rejected still forces Paused. Do: in crates/bridle-daemon/src/governor.rs near line 765 stop raising state to WindingDown on allowed_warning (keep the rejected arm); fix the comment in crates/bridle-daemon/src/server.rs near lines 405 and 435 so bridle budget still shows the warning but doesn't say it forces wind-down; update the governor test near line 1245 that pins the old behaviour and add one showing a raised override at 91% with allowed_warning stays normal; update Seeing the windows in docs/design/usage-and-budget.md. Acceptance: just check passes. Model: Sonnet (the governor must be right). Out of scope: any other threshold or override change. Touches governor.rs and server.rs, not the task store or cli.rs, so it can run beside the others.
