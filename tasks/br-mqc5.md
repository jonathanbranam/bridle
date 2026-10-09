+++
id = "br-mqc5"
title = "Fix flaky upgrade_test a_drain_starting_during_a_spawn_restarts_promptly (HTTP race after restart)"
kind = "bug"
state = "dropped"
created_at = "2026-10-09T20:33:29.456Z"
updated_at = "2026-10-09T20:35:03.979394Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
+++

CI red on main f8b6b5b5 (ubuntu): crates/bridle-daemon/tests/upgrade_test.rs:395 messages_to hits a server already shutting down (Unreachable). Test race, added by br-b6mu. Fix test only: read the held message before the restart is requested, or via the store handle, or retry messages_to briefly; keep the elapsed < 6s assertion. Acceptance: just check passes.

## Thread

### note · agent:manager-2 · 2026-10-09T20:33:35.786Z
Thanks. Filed as br-mqc5 (test-only fix, your proposal). Please do it on your existing branch bridle/rztb: merge main first, fix the race, keep the <6s assertion, write a task summary on br-mqc5, just check green, then message me. The task may take ~5 min to settle; start the work meanwhile.

### note · agent:pm-1 · 2026-10-09T20:34:05.957Z
pm-1 addendum: Model Haiku (test-only). File: crates/bridle-daemon/tests/upgrade_test.rs (line ~395, test added by br-b6mu). Out of scope: any daemon code change. Migration: none.

### note · external:orchestrator · 2026-10-09T20:35:03.979Z
dropped: duplicate of br-ngya (same CI flake, run 37986250994); worker rztb does it as br-ngya
