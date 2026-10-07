+++
id = "br-ksz7"
title = "cli_e2e tests time out waiting for daemon.json under load (60 s), failing landing checks"
kind = "bug"
state = "integrated"
created_at = "2026-10-05T04:02:26.737Z"
updated_at = "2026-10-05T06:16:49.590649Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/e2e-timeout"
commit = "665244850f6e519d08518e410bf2311203d0c463"
summary = "cli_e2e: could not reproduce the 60 s daemon.json timeout (40 busy loops plus a parallel workspace build gave a 26 s run, all passing), so the root cause is not proven. Start-up waits on no lock or build; the wait was already condition-based. A timeout at ~61 s most likely means the daemon died at start-up, since the tests sent its stderr to null and kept polling blind. Changes in crates/bridle/tests/cli_e2e.rs: new wait_for_daemon fails at once if the daemon exited; daemon stderr is inherited so nextest shows why; the hang guard is raised 60 s to 180 s as the last resort. If it recurs, the failure now names the cause."
ticket = "ksz7"
+++

Ticket (the ask; read first): docs/tickets/open/cli-e2e-tests-time-out-waiting-for-daemon-json-under-load-60-ksz7.md
Goal: make crates/bridle cli_e2e's daemon start-up wait robust under load. Six cli_e2e tests failed "timed out waiting for .../.bridle/daemon.json" (cli_e2e.rs:83, ~61 s) in landing checks that ran beside a worker's `just check`. Find what the start-up waits on under load (a build or cargo lock inside the test? daemon start slow? a fixed 60 s?) and fix that; raising the timeout alone is the last resort.
Files: crates/bridle/tests/cli_e2e.rs and the helper that starts the daemon.
Acceptance: just check passes; run cli_e2e while another `cargo build`/nextest runs in parallel and it passes; the done note says the cause.
Model: Sonnet. Out of scope: other tests, daemon start-up changes unless they are the cause.

## Thread

### note · agent:e2e-timeout · 2026-10-05T06:10:34.802Z
done: cause NOT reproduced (see summary); cli_e2e now fails fast if the daemon dies at start-up, inherits its stderr, hang guard 60->180 s. just check exit 0, 1201 tests passed, sha 289ba11a

### note · agent:manager-2 · 2026-10-05T06:10:41.795Z
integrated: 665244850f6e519d08518e410bf2311203d0c463 (branch bridle/e2e-timeout)

### note · agent:manager-2 · 2026-10-05T06:16:49.590Z
cleanup: removed agent e2e-timeout, branch bridle/e2e-timeout
