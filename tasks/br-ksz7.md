+++
id = "br-ksz7"
title = "cli_e2e tests time out waiting for daemon.json under load (60 s), failing landing checks"
kind = "bug"
state = "planned"
created_at = "2026-10-05T04:02:26.737Z"
updated_at = "2026-10-05T04:02:48.187877Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: ksz7
Ticket (the ask; read first): docs/tickets/open/cli-e2e-tests-time-out-waiting-for-daemon-json-under-load-60-ksz7.md
Goal: make crates/bridle cli_e2e's daemon start-up wait robust under load. Six cli_e2e tests failed "timed out waiting for .../.bridle/daemon.json" (cli_e2e.rs:83, ~61 s) in landing checks that ran beside a worker's `just check`. Find what the start-up waits on under load (a build or cargo lock inside the test? daemon start slow? a fixed 60 s?) and fix that; raising the timeout alone is the last resort.
Files: crates/bridle/tests/cli_e2e.rs and the helper that starts the daemon.
Acceptance: just check passes; run cli_e2e while another `cargo build`/nextest runs in parallel and it passes; the done note says the cause.
Model: Sonnet. Out of scope: other tests, daemon start-up changes unless they are the cause.
