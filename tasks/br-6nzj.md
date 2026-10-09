+++
id = "br-6nzj"
title = "Test daemons stop polling at 200 ms; a resource-budget test; log the incident in docs/context/incidents.md (n4w4 recs 1, 2, 9)"
kind = "bug"
state = "pending"
created_at = "2026-10-09T01:41:03.473Z"
updated_at = "2026-10-09T01:41:03.474877Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
]
parent = "br-n4w4"
+++

Ticket: docs/tickets/open/postmortem-bridle-s-own-ps-polling-every-daemon-test-daemons-n4w4.md (read Summary, Root cause 2, Recommendations 1, 2 and 9). The human approved all nine recommendations (aide, m via br-n4w4 thread). br-9z2n (no-fork read) has landed; this is the rest of rec 1, plus recs 2 and 9.

Do:
1. Test harness: crates/bridle-daemon/tests/support/mod.rs:218 `default_overrides()` sets `tracker_interval` to 200 ms, and context_governor_test, restart_test and tasks_test set it too. Make the default long (2 s, the production value) and keep 200 ms only in the tests that need a quick tracker (stop and containment tests, governor tests that wait on a reap): find them by running the suite and by reading what each waits for; give those an explicit `tracker_interval: Duration::from_millis(200)` override via a named helper (e.g. `with_fast_tracker`) so the reason is visible.
2. Resource-budget test (rec 2): in crates/bridle-daemon (a new tests/resource_budget_test.rs, or inside the supervisor unit tests), start a daemon with no agents and let its timer loops run for a few seconds with short intervals; count the child processes forked via an injected command runner / snapshot counter (see tick_tracker_with in supervisor.rs for the existing injection point; extend the same idea to the other `Command::new` callers on timers: load.rs top_consumers, any `git` or `ps` call on a loop -- list the ones you find on the task thread). Assert zero forks from timers for an idle daemon. Add a second case with one fake agent live that asserts the tracker snapshot count is at most (elapsed / interval) + 1, to catch a hot loop.
3. Rec 9: add an entry to docs/context/incidents.md (create it if absent, follow its style if present) for the 2026-10-07/08 ps-polling load hold: 5 lines (what, impact ~26 h of spawn holds, cause, fix commits a1bde105 and 018b9cfd, pointer to ticket n4w4). ASCII only.

Files: crates/bridle-daemon/tests/support/mod.rs and the tests that set tracker_interval, new test file, docs/context/incidents.md, CHANGELOG.md. 

Acceptance: just check passes; the new test fails if you temporarily make tick_tracker snapshot with no agents (say you tried it on the thread); report wall time of `just test` before and after if the machine is quiet, else say it was not.

Model: Sonnet. Migration: none. Out of scope: the load-hold notes (separate task), a cap on concurrent test runs, the fake-claude shim, restarting the other projects' daemons (orchestrator action, rec 8).
