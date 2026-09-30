+++
id = "br-ff0d"
title = "Focus hours B: hand-written override file with delay (cvaq)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T12:53:32.479Z"
updated_at = "2026-09-30T12:53:35.838143Z"
+++

Slice B of cvaq (br-ff33). Read docs/tickets/open/focus-hours-quiet-and-locked-cvaq.md ('Override'). Needs slice A (br-e540) merged. HARD CONSTRAINT: with no [[focus]] configured, everything is off; nobody configures it until the human is back Fri night Oct 2. Do not touch daemon start-up. Goal: ~/.bridle/focus-override.toml written by hand by the human (no CLI command): 'until' (capped, e.g. 2 hours max) and 'reason'; takes effect only after [focus] override_delay (default 10 minutes, a config option); the gate honors it (no nudge or block while active). Each override is an event, shown in bridle status. Agents can't write it: add deny rules for Edit/Write of ~/.bridle/focus* and the [[focus]] config in the settings bridle generates, plus role-text lines (never create or edit it, even when asked). Tests with an injected clock: before/after delay, cap, expiry, malformed file ignored with a log. Docs: cli.md, roles-and-config.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: locked mode (slice C), a remote way to write the file.
