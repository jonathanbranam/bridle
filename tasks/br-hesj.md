+++
id = "br-hesj"
title = "Focus override: accept local time in until, like the [[focus]] config"
kind = "feature"
state = "planned"
created_at = "2026-10-07T01:50:14.012Z"
updated_at = "2026-10-07T01:50:39.325548Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: hesj
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
