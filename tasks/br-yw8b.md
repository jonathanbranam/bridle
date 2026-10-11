+++
id = "br-yw8b"
title = "fake-claude spawns skip the pyenv shim: resolve the interpreter once (n4w4 rec 7)"
kind = "chore"
state = "planned"
created_at = "2026-10-09T01:41:32.122Z"
updated_at = "2026-10-11T01:50:00.625909Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
    "external:advisor/product-manager",
]
summary = "Test fake-claude now launches as one exec of the real interpreter. fake_claude_path() (crates/bridle-daemon/tests/support/mod.rs, crates/bridle/tests/cli_e2e.rs) asks `python3` once per test process (OnceLock) for sys.executable and runs a content-hashed copy of fake-claude.py, written under $TMPDIR/bridle-fake-claude/, whose shebang names that interpreter; falls back to the script itself if python3 doesn't answer. The existing sh wrappers exec this path so they benefit too. Not changed: bridle-claude/tests/process_test.rs (no daemon, few spawns). ps evidence: the pyenv shim chain execs through, so a running agent shows one process either way (python3 .../fake-claude.py); the saving is at startup (env -> shim bash -> pyenv bash -> python becomes a single python exec), which a snapshot ps can't show. just check green (1478 tests) on the commit before the main merge; the first run hit one load-flaky unit test (sessions::the_hard_limit_has_no_override..., passes alone, unrelated). Main merge brought 4 files (scripts/test-lock.sh etc.); check not re-run after it."
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

### note · agent:wyw8b · 2026-10-11T00:31:11.933Z
done: fake-claude launched via real interpreter (one exec, not shim chain); check exit 0, 1478 tests, on 3b7dd595~1 (main merged after as 3b7dd595, check not re-run); ps note in summary

### note · agent:wyw8b · 2026-10-11T01:50:00.625Z
update: main (br-rhba) merged into the branch; just check exit 0, 1480 tests, on c2dafee6; ready to land
