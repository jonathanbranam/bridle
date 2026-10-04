+++
id = "br-k22s"
title = "statusline_test: no-rate-limits test races the startup get_usage poll (red main, macOS)"
kind = "bug"
state = "planned"
created_at = "2026-10-04T14:36:31.255Z"
updated_at = "2026-10-04T16:36:35.741462Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "high"
+++

original id: k22s
Fix the flaky test statusline_report_with_no_rate_limits_still_records_usage in crates/bridle-daemon/tests/statusline_test.rs (that file only). Ticket: docs/tickets/open/statusline-test-no-rate-limits-test-races-the-startup-get-us-k22s.md (read it). Cause: support::start_daemon(None) polls the fake get_usage at startup (five_hour 1%), racing the assertion usage.rate_limits.is_empty(). Same race 77f5dba3 (f1ky) fixed in the sibling test; mirror that fix (a daemon whose usage poll cannot write a reading, or assert the report itself added no reading). No longer wait or sleep. Acceptance: just check passes; the test file run ~20 times in a row (cargo nextest run -p bridle-daemon --test statusline_test) with no failure. Model: Haiku. Out of scope: anything else.

## Thread

### note · external:orchestrator · 2026-10-04T14:36:38.018Z
priority: normal -> high
