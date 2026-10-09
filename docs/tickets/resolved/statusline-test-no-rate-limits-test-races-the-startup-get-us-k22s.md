---
id: k22s
title: "statusline_test: no-rate-limits test races the startup get_usage poll (red main, macOS)"
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [n6gy]
tasks: [br-k22s]
closed: 2026-10-09T23:11:03Z
---

## The ask

CI on `main` failed on macOS at fb3bdb8 (run https://github.com/jonathanbranam/bridle/actions/runs/37209136565; Linux passed):

```
FAIL bridle-daemon::statusline_test statusline_report_with_no_rate_limits_still_records_usage
panicked at crates/bridle-daemon/tests/statusline_test.rs:80:5:
assertion failed: usage.rate_limits.is_empty()
```

Cause (orchestrator's reading): the same race f1ky (77f5dba3) fixed in the sibling test. `support::start_daemon(None)` polls the fake `get_usage` (five_hour 1%) at startup; when that poll lands before `usage()` is read, `rate_limits` isn't empty. 77f5dba3 fixed only `statusline_report_records_...`; this test still uses the default daemon. Fix the cause: give the test a daemon whose usage poll can't write a reading (or assert that the report itself added no reading), not a longer wait. Anything that breaks CI on `main` is critical (the human, 2026-10-03).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
