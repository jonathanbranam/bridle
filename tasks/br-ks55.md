+++
id = "br-ks55"
title = "Only one full test run at a time per machine: just check takes a machine-wide lock (n4w4 rec 6)"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T01:41:27.525Z"
updated_at = "2026-10-11T00:24:25.282586Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
    "external:advisor/product-manager",
]
branch = "bridle/wks55"
commit = "c58c07fd17723361fe35605236c664a327898e5f"
summary = """Added scripts/test-lock.sh: a machine-wide lock (symlink ~/.bridle/test.lock whose target is "pid time worktree"; atomic ln -s, no flock) that the nextest recipes in the justfile (test, test-live, test-contract, check-affected's nextest call) run through, so concurrent runs queue. Waiters print "waiting for the test lock held by <worktree> since <time> (pid N)" and poll every 5 s; a dead holder pid is taken over; released by EXIT/INT/TERM trap; command exit status passes through; BRIDLE_TEST_NOLOCK=1 skips it. BRIDLE_TEST_LOCK / BRIDLE_TEST_LOCK_POLL override path/poll (used for the demo). Docs: CLAUDE.md line, CHANGELOG. Demo: A held the lock 8 s and exited 3; B started 1 s later, printed the waiting line, then ran after A ended (A rc=3, B rc=0). Stale: started a holder, kill -9 by its pid; the next run printed "held by dead pid 98020; taking it over", ran, and removed the lock. Caveat: killing the lock script with -9 orphans its child command, which keeps running without the lock. just check: first run failed on a timing test (resource_budget_test tracker_with_a_live_agent_stays_within_its_interval) under load average 60 from other agents; unrelated; second run green, 1478 passed."""
parent = "br-n4w4"
+++

Ticket: docs/tickets/open/postmortem-bridle-s-own-ps-polling-every-daemon-test-daemons-n4w4.md, Recommendation 6 (approved by the human). Today N worktrees each run `just check` with 8 nextest threads, so N times 8 test daemons run at once; that multiplied the load (Root cause 3).

Do: make the nextest-running recipes in the justfile (check, test, check-affected's nextest call) take a machine-wide lock first, so a second concurrent run waits (printing "waiting for the test lock held by <worktree path> since <time>") instead of running in parallel. Mechanism: an atomic directory or file lock under the user's temp or ~/.bridle (e.g. `mkdir ~/.bridle/test.lock` with a pid file inside; a waiter polls every 5 s; a lock whose pid is not alive is stale and is taken over; released by a trap on exit, Ctrl-C and failure). Must work in zsh/bash on macOS and Linux (WSL2) with no new tool install (no flock on macOS). Implement as a small script scripts/test-lock.sh invoked by the recipes (`scripts/test-lock.sh cargo nextest run ...`), run under bash, not inline justfile shell. Exit status of the wrapped command is passed through. An env var BRIDLE_TEST_NOLOCK=1 skips the lock (for CI and emergencies).

Files: justfile, scripts/test-lock.sh (new, executable), CLAUDE.md (one line under Commands: concurrent runs queue), CHANGELOG.md.

Acceptance: just check passes; demonstrate on the task thread two concurrent `just test -E 'test(none_such)'` style runs where the second waits and then proceeds; a stale-lock takeover test by hand (kill -9 the holder by its pid, which you started) and say what you saw. Do not use pkill/killall/pgrep (rule no-kill-by-name).

Model: Sonnet. Migration: none (repo scripts only; reaches worktrees with a merge of main). Out of scope: the nextest thread count (set in br-7fr6), CI, a fair queue order (first to notice wins is fine).

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.665Z
watching the task

### note · agent:wks55 · 2026-10-11T00:24:09.946Z
Added scripts/test-lock.sh: a machine-wide lock (symlink ~/.bridle/test.lock whose target is "pid time worktree"; atomic ln -s, no flock) that the nextest recipes in the justfile (test, test-live, test-contract, check-affected's nextest call) run through, so concurrent runs queue. Waiters print "waiting for the test lock held by <worktree> since <time> (pid N)" and poll every 5 s; a dead holder pid is taken over; released by EXIT/INT/TERM trap; command exit status passes through; BRIDLE_TEST_NOLOCK=1 skips it. BRIDLE_TEST_LOCK / BRIDLE_TEST_LOCK_POLL override path/poll (used for the demo). Docs: CLAUDE.md line, CHANGELOG. Demo: A held the lock 8 s and exited 3; B started 1 s later, printed the waiting line, then ran after A ended (A rc=3, B rc=0). Stale: started a holder, kill -9 by its pid; the next run printed "held by dead pid 98020; taking it over", ran, and removed the lock. Caveat: killing the lock script with -9 orphans its child command, which keeps running without the lock. just check: first run failed on a timing test (resource_budget_test tracker_with_a_live_agent_stays_within_its_interval) under load average 60 from other agents; unrelated; second run green, 1478 passed.

### note · agent:wks55 · 2026-10-11T00:24:12.395Z
done: machine-wide test lock via scripts/test-lock.sh, demo on thread; just check exit 0, 1478 tests; 0f3e9351

### note · agent:manager-2 · 2026-10-11T00:24:19.303Z
integrated: c58c07fd17723361fe35605236c664a327898e5f (branch bridle/wks55)

### note · agent:manager-2 · 2026-10-11T00:24:25.282Z
cleanup: removed nothing; kept agent wks55 (background job pid 19345)
