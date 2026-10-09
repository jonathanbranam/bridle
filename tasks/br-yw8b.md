+++
id = "br-yw8b"
title = "fake-claude spawns skip the pyenv shim: resolve the interpreter once (n4w4 rec 7)"
kind = "chore"
state = "planned"
created_at = "2026-10-09T01:41:32.122Z"
updated_at = "2026-10-09T11:04:40.715866Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
    "external:advisor/product-manager",
]
parent = "br-n4w4"
+++

Ticket: docs/tickets/open/postmortem-bridle-s-own-ps-polling-every-daemon-test-daemons-n4w4.md, Recommendation 7 (approved by the human): each fake agent in the test suite starts bash, bash, python through the pyenv shim (`python3` resolves to a pyenv shim script), which adds exec load and syspolicyd attention per fake agent.

Do: find how the tests start the fake (crates/bridle-claude/tests/fake-claude.py and its shebang/launcher; crates/bridle-daemon/tests/support/mod.rs and any `claude_bin` setting that points at it). Make the launched command the interpreter by absolute path (resolved once per test process, e.g. in a OnceLock via `command -v python3` followed by canonicalising away the shim, or a `python3`-from-PATH lookup that skips a pyenv shims dir; fall back to the current behaviour if nothing better is found) so one fake agent is one exec, not three. If the fake is invoked through a wrapper shell script, replace it with a direct exec of the interpreter with the script as argument.

Files: the test support code that builds the fake's command line, fake-claude.py's shebang only if needed, CHANGELOG.md not needed (test-only).

Acceptance: just check passes; on the task thread show `ps` evidence (process tree of one running fake agent before and after: depth and count of processes) from a one-off test run that you start and own.

Model: Sonnet. Migration: none. Out of scope: replacing the fake, the tracker interval (br-6nzj), real claude.

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.715Z
watching the task
