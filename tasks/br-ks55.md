+++
id = "br-ks55"
title = "Only one full test run at a time per machine: just check takes a machine-wide lock (n4w4 rec 6)"
kind = "feature"
state = "planned"
created_at = "2026-10-09T01:41:27.525Z"
updated_at = "2026-10-09T11:04:40.665895Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
    "external:advisor/product-manager",
]
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
