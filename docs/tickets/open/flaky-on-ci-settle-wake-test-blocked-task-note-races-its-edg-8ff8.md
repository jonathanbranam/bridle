---
id: 8ff8
title: "Flaky on CI: settle_wake_test blocked-task note races its edge setup"
kind: bug
opened: 2026-10-09
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

CI on main went red on ubuntu at d9f8505b (2026-10-09 ~23:06Z, a docs-only commit; the next commit was green):
`bridle-daemon::settle_wake_test::a_task_blocked_by_a_dependency_makes_no_note` failed at
`crates/bridle-daemon/tests/settle_wake_test.rs:120`:

    left:  ["re-82hs is now startable", "re-kyrw is now startable"]
    right: ["re-82hs is now startable"]

Cause (orchestrator's reading): a race in the test, not the daemon. The daemon runs with a 1 s settle, and the
test plans both tasks *then* adds the `blocks` edge. On a slow runner, planning the second task plus the
edge request takes over 1 s, so the "blocked" task settles while it is still unblocked and gets its note.

Fix (test only): make the edge exist before the blocked task can settle, e.g. create the blocked task, add the
edge, then plan it (if the API allows an edge on an open task), or give this test a settle long enough
(say 5 s, with the sleep scaled to match) that setup can't outrun it. Keep the assertion as it is.
Check the other tests in the file for the same setup-vs-settle race and fix them the same way.

Model: Haiku. Critical: a flaky test on main (the human's rule, 2026-10-03). Verify: `just check`; the test
passes 20 runs in a row (`cargo nextest run -p bridle-daemon --test settle_wake_test` in a loop).
Not yet: the macOS-only `spawn_child_orphan_is_swept_on_stop` timeout in the same hour (b798c8bc), which
passed on re-run; it is the second time (see tr22). If it fails again it gets its own ticket.
