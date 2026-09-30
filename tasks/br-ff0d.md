+++
id = "br-ff0d"
title = "Focus hours B: hand-written override file with delay (cvaq)"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T12:53:32.479Z"
updated_at = "2026-09-30T13:25:47.290297Z"
branch = "bridle/focus-b"
commit = "717a444fdcfdf0e7aecbebaa344702c1effd83fd"
summary = "Focus override file: ~/.bridle/focus-override.toml (until RFC3339, capped 2h after it takes effect; reason) takes effect focus_override_delay_minutes (top-level config key, default 10; not [focus] since [[focus]] is an array) after the file's mtime; gate silent while active; malformed ignored with a warn log; bridle status prints a focus line (local, not in the JSON wire type). Deny rules Edit/Write of ~/.bridle/focus* and config.toml for all roles (DENY_FOCUS_FILES) and advisor/orchestrator session settings; role text lines. Not done: catch-up summary event. Off without [[focus]]; daemon start-up untouched."
+++

Slice B of cvaq (br-ff33). Read docs/tickets/open/focus-hours-quiet-and-locked-cvaq.md ('Override'). Needs slice A (br-e540) merged. HARD CONSTRAINT: with no [[focus]] configured, everything is off; nobody configures it until the human is back Fri night Oct 2. Do not touch daemon start-up. Goal: ~/.bridle/focus-override.toml written by hand by the human (no CLI command): 'until' (capped, e.g. 2 hours max) and 'reason'; takes effect only after [focus] override_delay (default 10 minutes, a config option); the gate honors it (no nudge or block while active). Each override is an event, shown in bridle status. Agents can't write it: add deny rules for Edit/Write of ~/.bridle/focus* and the [[focus]] config in the settings bridle generates, plus role-text lines (never create or edit it, even when asked). Tests with an injected clock: before/after delay, cap, expiry, malformed file ignored with a log. Docs: cli.md, roles-and-config.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: locked mode (slice C), a remote way to write the file.

## Thread

### note · agent:focus-b · 2026-09-30T13:25:33.254Z
done: focus override file (delay, 2h cap, deny rules, status line, tests, docs); 6bd497e. Note: delay is top-level focus_override_delay_minutes since [focus] can't coexist with [[focus]]; catch-up-summary event not done.

### note · agent:manager-2 · 2026-09-30T13:25:37.944Z
integrated: 717a444fdcfdf0e7aecbebaa344702c1effd83fd (branch bridle/focus-b)

### note · agent:manager-2 · 2026-09-30T13:25:47.290Z
cleanup: removed agent focus-b, branch bridle/focus-b
