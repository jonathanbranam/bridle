+++
id = "br-g3az"
title = "Status line token setup in the docs writes an empty file: token create needs --print now"
kind = "chore"
state = "integrated"
created_at = "2026-10-09T11:23:39.131Z"
updated_at = "2026-10-09T13:10:13.355469Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
branch = "bridle/g3az"
commit = "8ef95774f048b6fa2271fada3e22e03282611c8f"
summary = "Statusline token: the statusline now reads the last non-empty line of ~/.bridle/statusline.token (token_from_file in crates/bridle/src/commands/orchestrator.rs), so the principal line that 'token create --print' writes first no longer breaks it. Unit tests cover one line, principal-then-token, trailing blanks and empty. docs/design/cli.md now uses 'token create statusline --print' in both forms and says the last non-empty line is used. The hidden 'statusline' help points at the token setup doc. CHANGELOG entry on top of Unreleased/Fixed says an existing empty file needs regenerating. Caveat: the real flow on a scratch HOME was not run by hand (no daemon at hand); the unit tests cover the parse, and the daemon path is unchanged. just check on merged tip 3f7ca70c: exit 0, 1384 run, 1384 passed, 5 skipped."
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

### note · agent:g3az · 2026-10-09T13:04:00.984Z
done: statusline reads last token line, docs use token create --print, CHANGELOG; tip 3f7ca70c; just check exit 0, 1384 run/1384 passed (manual scratch-HOME flow not run, no daemon at hand; unit tests cover the parse)

### note · agent:manager-2 · 2026-10-09T13:10:00.671Z
integrated: 8ef95774f048b6fa2271fada3e22e03282611c8f (branch bridle/g3az)

### note · agent:manager-2 · 2026-10-09T13:10:13.355Z
cleanup: removed agent g3az, branch bridle/g3az
