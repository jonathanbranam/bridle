+++
id = "br-fae3"
title = "Lean checks B: worker check output to a file, test-count sniff band (qgma 3)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T13:43:22.107Z"
updated_at = "2026-09-30T13:43:23.985751Z"
+++

Implement Shape 3 and the 'test count' and 'worker runs the full suite' sections of docs/tickets/open/lean-checks-skip-on-fast-forward-quiet-output-qgma.md. Check first what is already built. (1) [commands] check_worker in bridle's own .bridle/config.toml back to 'just check'; role text in workflow/base/roles/worker.md and manager.md ('Reading and output'): run the check with output to a file, judge by exit status, read the file tail only on failure, on success read only the nextest 'Summary [...] N tests run' line. (2) Sanity band: land (or CI) records the last full-suite test count on the integration branch; a worker/land result with a count under half or over double is treated as a failure to look into, not a pass. Simplest place to record it: where land already runs the check (integrator.rs run_check); keep it roughly right, a band not an exact match. Tests: band logic unit tests, count parse from a nextest summary line. Docs, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Not daemon start-up. Run after Lean checks A (same file, integrator.rs).
