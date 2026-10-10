+++
id = "br-bcw6"
title = "cli_e2e sigint and events_stream shutdown tests flake under load"
kind = "bug"
state = "integrated"
created_at = "2026-10-10T12:08:56.477Z"
updated_at = "2026-10-10T19:20:37.366134Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
branch = "bridle/wbcw6"
commit = "a563aa5a5a3f4430832b73c605d888d32a2a625f"
summary = "events_stream shutdown test (crates/bridle-daemon/tests/events_stream_test.rs): the 3 s bounds on running.join() and on the client seeing EOF were wall-clock limits on a shutdown that flushes and pushes the state branch (git subprocesses); measured 2.9 s on a quiet-ish machine, so over 3 s under load. Now both waits are 120 s hang guards on the events (join returns, stream ends). Promptness of ending the stream is still checked load-independently: a stream the server fails to end keeps its connection open, so the EOF wait cannot pass however long the guard. cli_e2e sigint tests: already event-based since br-648a (they wait on process exit with the 180 s HANG_GUARD and assert no time limit), so there was nothing wall-clock-bound to change; signals are installed before daemon.json is written, so an early SIGINT is queued, not lost. I could not reproduce a failure of either test, so the cli_e2e cause is NOT demonstrated; I made no change there. If it recurs, capture the failing assertion text. Runs: 20/20 alone and 10/10 with 16 busy loops added (on a machine already at load 25-40), both tests plus the in-flight sigint test. Docs: none needed (test-only)."
ticket = "bcw6"
+++

Ticket: docs/tickets/open/cli-e2e-sigint-and-events-stream-shutdown-tests-flake-under-bcw6.md (read it; also the br-n96z thread for the diagnosis of the same class, and the n6gy/br-8ff8/br-n96z fixes for the idiom). Goal: the cli_e2e sigint test (crates/bridle/tests/cli_e2e.rs) and the events_stream shutdown test (find with grep events_stream in crates/*/tests) pass reliably at machine load 30-40, without weakening what they check. They put wall-clock limits on how long shutdown takes; wait on the event (process exit, stream end) with a generous load-scaled deadline instead, and keep any assertion that shutdown itself is prompt measured by something load-independent, or explain on the task why a limit must stay. Find the cause first (what is slow under load) before changing a number. Acceptance: just check passes; each test run 20x alone and 10x while a cargo build or another nextest run loads the machine, report counts. Migration: none. Model: Sonnet. Out of scope: other flakes, daemon shutdown behaviour changes.

## Thread

### note · agent:wbcw6 · 2026-10-10T19:03:57.198Z
Done at 002676e4. events_stream shutdown test: its 3 s bounds on join and EOF were wall-clock limits on a shutdown that flushes and pushes the state branch (2.9 s measured on a quiet machine); now 120 s hang guards on the events, and the EOF check still fails if the server doesn't end the stream. cli_e2e sigint tests were already event-based (br-648a, 180 s hang guard); I could not reproduce a failure, so no change there and no cause found. Runs: 20/20 alone, 10/10 with 16 busy loops. just check exit 0, 1455 passed.

### note · agent:wbcw6 · 2026-10-10T19:03:59.463Z
done: events_stream shutdown bounds -> 120s hang guards; cli_e2e sigint already event-based, flake not reproduced (no change); 20/20 alone, 10/10 loaded; check exit 0, 1455 tests; 002676e4

### note · agent:manager-2 · 2026-10-10T19:20:18.714Z
integrated: a563aa5a5a3f4430832b73c605d888d32a2a625f (branch bridle/wbcw6)

### note · agent:manager-2 · 2026-10-10T19:20:37.366Z
cleanup: removed agent wbcw6, branch bridle/wbcw6
