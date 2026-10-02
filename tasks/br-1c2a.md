+++
id = "br-1c2a"
title = "Overnight periods must say +1d, and bridle doctor validates the machine config"
kind = "feature"
state = "integrated"
created_at = "2026-10-02T11:47:48.911Z"
updated_at = "2026-10-02T11:57:40.828873Z"
branch = "bridle/config-1d"
commit = "8e5bd04e30e7020a6f5e6c3816ae9d0c5d3fdf0b"
summary = "A [[focus]]/[[budget.schedule]] end may be written HH:MM+1d (parse_time_of_day strips it; matching unchanged). One validator, overnight_problem in config.rs, errors for end<start without +1d (except 00:00) with the fix text, and for +1d on an end not before start. Daemon load only logs a warning (stays lenient, 3xr4 meaning). New machine_config_problems feeds a 'machine config' doctor check (fails, missing file ok). Docs: roles-and-config, cli, usage-and-budget, CHANGELOG."
+++

original id: r5s3
Ticket: docs/tickets/open/overnight-periods-must-say-1d-and-bridle-doctor-validates-th-r5s3.md (read it all; the human's decision is verbatim at the top). Code: in_window and the focus/budget schedule parsing in crates/bridle-daemon/src/config.rs (3xr4 landed at 761d545: end < start already means next day), crates/bridle/src/focus.rs, 'bridle doctor' (crates/bridle/src/commands/ doctor), docs/design/agent-host/roles-and-config.md, the focus docs.

Goal: (1) A period's end may be written 'HH:MM+1d' ('06:00+1d') in [[focus]] and [[budget.schedule]]; the parser accepts it and it means that time on the day after the start day. (2) STRICTNESS LIVES IN VALIDATION, NOT IN LOADING: a block with end before start and no '+1d' (e.g. night 23:00-08:00) is reported as an error by the new validator with the block's name and the fix ('night: end 08:00 is before start 23:00; write "08:00+1d"'), but the daemon's config load must keep accepting it with the 3xr4 meaning (next day) and only log a warning. An invalid or surprising block in ~/.bridle/config.toml must NEVER stop the daemon starting or the budget governor evaluating; the human is away and the daemon must keep starting on today's file. (3) end = '00:00' means midnight at the end of the start day and needs no +1d (so 21:30-00:00 stays valid); start == end stays empty (the days = [] presets untouched). (4) 'bridle doctor' gains a machine-config check: parse every section of ~/.bridle/config.toml ([[focus]], [[budget.schedule]], and the rest it already knows how to load), run the overnight validation above, and report each bad block with its fix; a missing file is fine. Doctor exits non-zero on these errors like its other checks. Reuse one validation function that the daemon-load warning also calls. (5) Docs: roles-and-config.md (the format, the 00:00 rule, 'run bridle doctor after any config change'), cli.md for doctor, CHANGELOG with a release note that a block like budget 'night' 23:00-08:00 should be written '08:00+1d' (the human edits their own ~/.bridle/config.toml; never edit it) and that bridle doctor names it.
Tests: '+1d' parses and matches the morning after, for both focus and budget callers; end<start without +1d: validator error text names block and fix, daemon-load still accepts with the old meaning and warns; 00:00 end valid; start==end empty; doctor reports each bad block and exits non-zero, and passes on a clean file and a missing file; the human's current three blocks give exactly one doctor error (night).
Acceptance: just check passes. Model: Sonnet. Migration plan: no automatic migration (the file is the human's); the release note plus 'bridle doctor' naming the block is the plan, and the lenient load means nothing breaks meanwhile.
Open point to confirm with the human later, built as proposed: end = '00:00' is the same day's midnight. Out of scope: making the daemon refuse an invalid block; per-project focus (mfgb). Lands normally: validation and parsing only, daemon start-up unchanged.

## Thread

### note · agent:config-1d · 2026-10-02T11:57:20.607Z
done: HH:MM+1d ends, one overnight validator (doctor errors, daemon load warns only), doctor machine-config check, docs+CHANGELOG; just check green (1003 tests); 559d895

### note · agent:manager-2 · 2026-10-02T11:57:31.339Z
integrated: 8e5bd04e30e7020a6f5e6c3816ae9d0c5d3fdf0b (branch bridle/config-1d)

### note · agent:manager-2 · 2026-10-02T11:57:40.828Z
cleanup: removed agent config-1d, branch bridle/config-1d
