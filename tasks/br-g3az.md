+++
id = "br-g3az"
title = "Status line token setup in the docs writes an empty file: token create needs --print now"
kind = "chore"
state = "planned"
created_at = "2026-10-09T11:23:39.131Z"
updated_at = "2026-10-09T11:24:45.101684Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
ticket = "g3az"
+++

Ticket: docs/tickets/open/status-line-token-setup-in-the-docs-writes-an-empty-file-tok-g3az.md (read it: the facts). The human approved the fix ("yes fix the docs.", 2026-10-09 ~7:45 AM ET, via aide). Small.

Do:
1. Make the statusline read the token robustly: the function that reads ~/.bridle/statusline.token (path from crates/bridle-api/src/discovery.rs:104-108; the reader is in crates/bridle/src/commands/orchestrator.rs near the tests at ~470-495) takes the LAST non-empty line, trimmed, instead of the whole file. That makes `token create statusline --print > file` work even though --print also writes a `principal ...` line first. Add tests: file with one line, with `principal ...` line then token, trailing blank lines, empty file (no token, as today).
2. Fix the documented command in docs/design/cli.md lines ~813 and ~817 (the `bridle statusline` entry) to the form that works now: `bridle --project <name> token create statusline --print > ~/.bridle/statusline.token && chmod 600 ~/.bridle/statusline.token`, and the bare-URL form likewise with `--print`. Say in one sentence that the file may hold a `principal ...` line before the token and that the last non-empty line is used. Check `bridle token create --help` for the exact current flags and use them. Also fix the help text of `bridle statusline` (clap doc comment in crates/bridle/src/cli.rs or commands/) and any docs/cli/ topic text if one mentions the old command (grep `token create statusline`; leave docs/tickets/resolved/ as it is, a dated record).
3. CHANGELOG.md entry on top.

Acceptance: just check passes; run the real flow once by hand on a scratch HOME (set HOME to a temp dir you create; do not touch the real ~/.bridle): create a token with the documented command and show that `bridle statusline` picks it up (or the unit test equivalent if a daemon is not at hand). Model: Haiku. Migration: none for files; the human's existing empty ~/.bridle/statusline.token needs regenerating with the new command (the orchestrator tells them; not this task). Out of scope: changing what `token create` prints.

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:23:55.752Z
watching the task

### note · external:advisor/product-manager · 2026-10-09T11:23:59.907Z
PdM (advisor product-manager): the human approved this fix ("yes fix the docs.", 2026-10-09 ~7:45 AM ET, relayed by the bridle aide). Theme: docs upkeep, no design gate; ready to plan.
