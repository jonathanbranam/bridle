+++
id = "br-4vmc"
title = "Flaky lifecycle_test spawn_child_orphan_is_swept_on_stop times out waiting for agent.orphans_killed (macOS CI once, local land check once)"
kind = "bug"
state = "integrated"
created_at = "2026-10-10T02:02:03.952Z"
updated_at = "2026-10-10T11:25:13.268088Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
branch = "bridle/w4vmc"
commit = "f49d8ebeba8f998fa424cefdc6ff20ed59067ef2"
summary = "Could not reproduce the spawn_child_orphan_is_swept_on_stop flake (20/20 alone, also under 8 CPU burners). The event is emitted inline by sweep_and_emit before stop returns, so a 60 s miss means the sweep found nothing (terminated==0), not a late event; speculative cause: a ps-fallback snapshot makes Tracker::update drop native-format entries. No product change. Test now polls for the child's death (kill -0, also fixing a zombie race in the old single assert) before waiting for the event, so a failure says which half broke. just check was NOT green: unrelated load flakes (run 1: upgrade_test a_daemon_raised_wake..., a_drain_starting_during_a_spawn... both pass alone; run 2: governor_test working_agent_is_notified... known n96z)."
+++

Seen twice in a day: macOS CI run 38000780827 (b798c8bc) and a local landing check (br-8ff8), both 'timed out waiting for event agent.orphans_killed' at crates/bridle-daemon/tests/support/mod.rs:346 after ~62-66s (HANG_GUARD), under machine load. Test: crates/bridle-daemon/tests/lifecycle_test.rs:434. Passes on re-run. Find out whether the sweep ran but the event came after stop returned, or never ran; make the test wait on the event plus orphan-pid liveness (kill -0) as a second signal, or fix product code if the sweep skips the event when the child is already gone (say so on the thread before changing product code). Do not raise HANG_GUARD. Acceptance: just check passes; run the one test ~20 times in a loop under some load and report the count.

## Thread

### note · agent:pm-1 · 2026-10-10T02:26:17.799Z
pm-1 addendum: Model Sonnet (investigation first). Out of scope: other tests, raising HANG_GUARD. Migration: none.

### note · agent:w4vmc · 2026-10-10T06:04:04.855Z
Investigation: could not reproduce (20/20 pass alone under 8 CPU burners). From code: the event is emitted inline by sweep_and_emit in stop() and finish_agent(), before stop returns, so a 60s miss means the sweep reported terminated==0 (child not in tracker.seen, or snapshot failed), not a late event. One speculative cause: tracker.seen keeps native-format start strings; a ps-fallback snapshot would make update()'s retain drop the entry. Not confirmed, so no product change. Test change only: wait for child pid death (poll kill -0; also fixes a zombie race in the old single assert) before the event, so a failure says which half broke.

### note · agent:w4vmc · 2026-10-10T07:31:12.543Z
done with caveat: test-only fix, root cause not reproduced (see thread); just check NOT green, unrelated load flakes (upgrade_test x2, then governor_test n96z); focused test 20/20; 2627ed53

### note · agent:manager-2 · 2026-10-10T11:25:13.268Z
integrated: f49d8ebeba8f998fa424cefdc6ff20ed59067ef2 (branch bridle/w4vmc)
