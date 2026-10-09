+++
id = "br-ngya"
title = "Flaky on Linux CI: upgrade_test a_drain_starting_during_a_spawn_restarts_promptly reads messages after the daemon stopped serving"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T20:34:03.507Z"
updated_at = "2026-10-09T20:43:52.965213Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
priority = "critical"
priority_at = "2026-10-09T20:34:19.557207Z"
branch = "bridle/rztb"
commit = "0809dd1e3c4165391eef436818ac28e4c83fbaf8"
summary = "Test-only: a_drain_starting_during_a_spawn_restarts_promptly (upgrade_test.rs) now counts the held 'hello' message in .bridle/bridle.db via rusqlite (as renew_test does) instead of an HTTP messages_to call racing the restart's shutdown; the <6s assertion is kept. Diagnosis confirmed from the CI log (Unreachable on the post-restart request). 20/20 loop passes; just check green (1398 tests)."
ticket = "ngya"
+++

Ticket: docs/tickets/open/flaky-on-linux-ci-upgrade-test-a-drain-starting-during-a-spa-ngya.md (read it). Model: Sonnet. Critical: a flaky test on main.

Failing run: https://github.com/jonathanbranam/bridle/actions/runs/37986250994 (ubuntu only). crates/bridle-daemon/tests/upgrade_test.rs, a_drain_starting_during_a_spawn_restarts_promptly (from br-b6mu, 3323d2f4): after restart_requested() the test calls messages_to over HTTP, which races the daemon's shutdown (Unreachable).

Do: make the test deterministic without an HTTP call after the restart is requested (e.g. read the held message from the store in the test's temp dir, or check it by a route that doesn't race the shutdown). Keep the timing assertion. Test-only change unless you find the diagnosis wrong; if so, say so on the thread before changing product code.
Acceptance: just check passes; run the one test ~20 times in a loop (cargo nextest run -p bridle-daemon --test upgrade_test a_drain_starting -- with a shell loop) and report the count.
Out of scope: other upgrade_test tests (h7gt already fixed one).

## Thread

### note · external:orchestrator · 2026-10-09T20:34:19.557Z
priority: normal -> critical

### note · external:orchestrator · 2026-10-09T20:34:25.717Z
From orchestrator: main is RED (run 37986250994, ubuntu): flaky upgrade_test a_drain_starting_during_a_spawn_restarts_promptly (from br-b6mu). br-ngya is planned, critical, brief on the task: take the next worker slot now (rztb's slot is free; Sonnet). Merge nothing else until main is green. br-rztb marked done, thanks.

### note · agent:manager-2 · 2026-10-09T20:34:38.811Z
br-ngya (critical, same flake as br-mqc5) supersedes br-mqc5: do it as br-ngya on your branch. Read its brief (bridle task show br-ngya): also run the one test ~20 times in a loop and report the count. Write the summary on br-ngya, then message me.

### note · agent:rztb · 2026-10-09T20:40:56.899Z
done: test reads held message from the db; 20/20 loop passes; just check exit 0 (1398 tests); 2eb1dfdc on bridle/rztb (main merged before the fix)

### note · agent:manager-2 · 2026-10-09T20:41:03.643Z
main moved; merge main into bridle/rztb again (docs-only is fine, no re-check needed if only docs/tickets changed), make sure the br-ngya task summary is written (bridle task summary), then message me the new tip.

### note · agent:rztb · 2026-10-09T20:41:10.364Z
main merged; tip 68b6fe56 (fix commit is HEAD^2's tip; merge brought docs/tickets only, no re-check). Summary is written on br-ngya.

### note · agent:manager-2 · 2026-10-09T20:43:41.206Z
integrated: 0809dd1e3c4165391eef436818ac28e4c83fbaf8 (branch bridle/rztb)

### note · agent:manager-2 · 2026-10-09T20:43:52.965Z
cleanup: removed agent rztb, branch bridle/rztb
