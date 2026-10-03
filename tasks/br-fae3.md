+++
id = "br-fae3"
title = "Lean checks B: worker check output to a file, test-count sniff band (qgma 3)"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T13:43:22.107Z"
updated_at = "2026-09-30T14:23:38.221476Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/lean-b"
commit = "467cdcc13bf2be29160beb02d90d867cd57a9d76"
summary = "Most of Lean checks B was already built by br-0e42/br-e882: check_worker is unbound (defaults to just check), role text sends output to a file, land parses the nextest count, bands it (half to double, 0 fails) and records it in <workspace>/last-full-test-count, with integration tests, docs and CHANGELOG. This task added the missing unit tests (band_check, parse_test_count in integrator.rs) and fixed a CHANGELOG punctuation slip."
+++

Implement Shape 3 and the 'test count' and 'worker runs the full suite' sections of docs/tickets/open/lean-checks-skip-on-fast-forward-quiet-output-qgma.md. Check first what is already built. (1) [commands] check_worker in bridle's own .bridle/config.toml back to 'just check'; role text in workflow/base/roles/worker.md and manager.md ('Reading and output'): run the check with output to a file, judge by exit status, read the file tail only on failure, on success read only the nextest 'Summary [...] N tests run' line. (2) Sanity band: land (or CI) records the last full-suite test count on the integration branch; a worker/land result with a count under half or over double is treated as a failure to look into, not a pass. Simplest place to record it: where land already runs the check (integrator.rs run_check); keep it roughly right, a band not an exact match. Tests: band logic unit tests, count parse from a nextest summary line. Docs, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Not daemon start-up. Run after Lean checks A (same file, integrator.rs).

## Thread

### note · agent:lean-b · 2026-09-30T14:23:25.861Z
done: qgma 3 was already built (br-0e42/br-e882); added unit tests for band logic and nextest count parse, fixed a CHANGELOG typo; 6a32e97

### note · agent:manager-2 · 2026-09-30T14:23:29.554Z
integrated: 467cdcc13bf2be29160beb02d90d867cd57a9d76 (branch bridle/lean-b)

### note · agent:manager-2 · 2026-09-30T14:23:38.221Z
cleanup: removed agent lean-b, branch bridle/lean-b
