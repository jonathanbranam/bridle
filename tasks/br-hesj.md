+++
id = "br-hesj"
title = "Focus override: accept local time in until, like the [[focus]] config"
kind = "feature"
state = "integrated"
created_at = "2026-10-07T01:50:14.012Z"
updated_at = "2026-10-07T03:01:48.383734Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/hesj-focus-local"
commit = "688330abadc24b76b66fe8d5f8d42317e1eeb505"
summary = "Implemented local time support for focus override until field: supports UTC-aware datetimes (existing), local datetimes interpreted in the local zone (new), and HH:MM strings for next time occurrence (new). Includes comprehensive tests and updated documentation. Check passed: 1273 tests, commit 2455014401de0dbfbb560488fdb8f405fcbefb29."
ticket = "hesj"
+++

Ticket (the human's words, today's behaviour, the change): docs/tickets/open/focus-override-accept-local-time-in-until-like-the-focus-con-hesj.md . The human: "ok, UTC sucks, file a low-prio small ticket to support local timezone same as in the config." Approved to schedule 2026-10-07.
Goal: `until` in ~/.bridle/focus-override.toml accepts local time, read in the machine's local zone exactly like `[[focus]]` periods (checked against DateTime<Local>).
EXACT ACCEPTED FORMS (parse_override in crates/bridle-daemon/src/focus.rs; decided here):
1. Existing, unchanged: a value with an offset or Z (`2026-10-01T22:00:00Z`, `2026-10-01T18:00:00-04:00`), as a TOML datetime or a string.
2. NEW: a TOML local datetime, no offset (`until = 2026-10-01T22:00:00`), or the same as a string: interpreted in the local zone.
3. NEW: a string `"HH:MM"` (24-hour, e.g. `until = "22:00"`): the next time the local clock reads that, i.e. today if still ahead, else tomorrow. (A bare unquoted `22:00` is not valid TOML, so the string form is the one to document.)
4. Anything else is rejected as today (logged as malformed; the file is ignored). A local time that does not exist or is ambiguous (DST gap or overlap) is also rejected with a log line saying so; do not guess.
Docs: docs/design/agent-host/roles-and-config.md focus-override section and its example (it says UTC today); the focus command help text and docs/design/cli.md if they mention the UTC requirement; CHANGELOG entry (read with a limit). If `bridle focus override ...` (or similar) writes this file, check what it writes and that its own `--until` flag text still matches; do not change what it writes.
Tests (in focus.rs's test module, with a fixed "now" and TZ-independent construction, e.g. build expected values with chrono Local the same way the code does so they pass in any zone): each of the four forms; "HH:MM" earlier than now rolls to tomorrow; a malformed value is rejected.
Acceptance: just check passes. Migration: none (new accepted forms only). Model: Haiku. Out of scope: the `[[focus]]` config, any change to how focus periods are evaluated, timezone names.

## Thread

### note · external:orchestrator · 2026-10-07T01:50:14.113Z
From orchestrator: approved by the human, 2026-10-07 ~9:50 PM ET: 'yeah, just go ahead and file it now'.

### note · agent:hesj-focus-local · 2026-10-07T02:59:53.430Z
Done: local time support for focus override; check passed 1273 tests; commit 2455014401de0dbfbb560488fdb8f405fcbefb29.

### note · agent:hesj-focus-local · 2026-10-07T02:59:57.837Z
done: local time support for focus override; 1273 tests passed; 2455014401de0dbfbb560488fdb8f405fcbefb29

### note · agent:manager-2 · 2026-10-07T03:00:04.447Z
integrated: 688330abadc24b76b66fe8d5f8d42317e1eeb505 (branch bridle/hesj-focus-local)

### note · agent:manager-2 · 2026-10-07T03:01:48.383Z
cleanup: removed agent hesj-focus-local, branch bridle/hesj-focus-local
