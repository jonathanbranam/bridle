+++
id = "br-6nzj"
title = "Test daemons stop polling at 200 ms; a resource-budget test; log the incident in docs/context/incidents.md (n4w4 recs 1, 2, 9)"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T01:41:03.473Z"
updated_at = "2026-10-09T16:21:21.732431Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
    "external:advisor/product-manager",
]
branch = "bridle/tick6nzj"
commit = "ff9958b7fa4077d89b5c54074952af47434c545e"
summary = "Harness default tracker_interval is now 2 s (production). containment::SNAPSHOTS counts process-table snapshots; new tests/resource_budget_test.rs asserts zero for an idle daemon and <= elapsed/interval+1 with a live agent. incidents.md entry and CHANGELOG line added. No test needed the fast tracker, so no helper was added; context_governor/restart/tasks tests keep their own explicit 200 ms."
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

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.515Z
watching the task

### note · agent:tick6nzj · 2026-10-09T13:09:31.863Z
Timer forks found: tracker (containment::snapshot, now counted); load.rs sysctl/ps only under load_watch (off in tests); upgrade/ci/ports/doc-watch loops are 3600 s in the harness. Whole bridle-daemon suite passed with the 2 s default, so no test needed a fast tracker; I added no with_fast_tracker helper (it would be unused). Sabotage check: with tick_tracker snapshotting with no agents, idle_daemon_takes_no_process_snapshots FAILED; reverted. Wall time: daemon suite 206 s with the 2 s tracker; no before run, machine was not quiet.

### note · agent:tick6nzj · 2026-10-09T13:09:33.880Z
done: harness tracker 2 s, resource-budget test (fails when tick_tracker snapshots with no agents), incident entry, changelog; just check exit 0, 1382 tests run, 1382 passed; ac6a527a

### note · agent:tick6nzj · 2026-10-09T14:05:35.189Z
Merged main (kept both incidents.md entries; tip 19aeee3c). just check not green here: machine load average ~66 (16 cores). Run 1: 1044 pass, governor_test working_agent_is_notified... timed out at 71 s under load (passes alone in 4 s). Run 2: cli_e2e sigint_shuts_down... refused by the load hold (3.8 per core). Neither touches my change. Merge is docs-only conflict; suggest landing on your own check when load drops.

### note · agent:tick6nzj · 2026-10-09T14:05:37.965Z
merged main, incidents.md conflict resolved (both kept), tip 19aeee3c. just check did NOT pass cleanly: host load ~66 caused 2 unrelated failures (governor_test timeout, cli_e2e load hold); both pass or are load-bound. Details on thread. Prior green tip before merge: ac6a527a.

### note · agent:tick6nzj · 2026-10-09T15:54:16.410Z
Orphan-sweep fix: spawn_child_orphan_is_swept_on_stop now uses fast_tracker_overrides() (200 ms); 11/11 solo passes. Other tracker-waiting tests: restart_test already sets its own 200 ms; none else found. Full just check: two runs killed by the 30 min background limit under load 20-30; the third failed on cli_e2e sigint_shuts_down_cleanly (spawn refused by the load hold, 4.1 per core, load avg 51 from other agents' builds), unrelated to this change. Merged main again (tip below); not re-checked since the merge, which only adds fake-claude.py changes.

### note · agent:tick6nzj · 2026-10-09T15:54:31.118Z
fast-tracker fix for lifecycle orphan test done, tip 1f4162b7 (merged main). Full just check could not complete green: host load 20-50 from other builds trips cli_e2e sigint_shuts_down (load hold) and exceeds the 30 min background limit. Please land on your check when quiet; the orphan test passed 11/11 alone.

### note · agent:manager-2 · 2026-10-09T16:21:21.732Z
integrated: ff9958b7fa4077d89b5c54074952af47434c545e (branch bridle/tick6nzj)
