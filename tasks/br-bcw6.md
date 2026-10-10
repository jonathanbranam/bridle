+++
id = "br-bcw6"
title = "cli_e2e sigint and events_stream shutdown tests flake under load"
kind = "bug"
state = "planned"
created_at = "2026-10-10T12:08:56.477Z"
updated_at = "2026-10-10T12:09:10.068124Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "bcw6"
+++

Ticket: docs/tickets/open/cli-e2e-sigint-and-events-stream-shutdown-tests-flake-under-bcw6.md (read it; also the br-n96z thread for the diagnosis of the same class, and the n6gy/br-8ff8/br-n96z fixes for the idiom). Goal: the cli_e2e sigint test (crates/bridle/tests/cli_e2e.rs) and the events_stream shutdown test (find with grep events_stream in crates/*/tests) pass reliably at machine load 30-40, without weakening what they check. They put wall-clock limits on how long shutdown takes; wait on the event (process exit, stream end) with a generous load-scaled deadline instead, and keep any assertion that shutdown itself is prompt measured by something load-independent, or explain on the task why a limit must stay. Find the cause first (what is slow under load) before changing a number. Acceptance: just check passes; each test run 20x alone and 10x while a cargo build or another nextest run loads the machine, report counts. Migration: none. Model: Sonnet. Out of scope: other flakes, daemon shutdown behaviour changes.
