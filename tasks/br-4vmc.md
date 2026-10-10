+++
id = "br-4vmc"
title = "Flaky lifecycle_test spawn_child_orphan_is_swept_on_stop times out waiting for agent.orphans_killed (macOS CI once, local land check once)"
kind = "bug"
state = "open"
created_at = "2026-10-10T02:02:03.952Z"
updated_at = "2026-10-10T02:02:06.066611Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
+++

Seen twice in a day: macOS CI run 38000780827 (b798c8bc) and a local landing check (br-8ff8), both 'timed out waiting for event agent.orphans_killed' at crates/bridle-daemon/tests/support/mod.rs:346 after ~62-66s (HANG_GUARD), under machine load. Test: crates/bridle-daemon/tests/lifecycle_test.rs:434. Passes on re-run. Find out whether the sweep ran but the event came after stop returned, or never ran; make the test wait on the event plus orphan-pid liveness (kill -0) as a second signal, or fix product code if the sweep skips the event when the child is already gone (say so on the thread before changing product code). Do not raise HANG_GUARD. Acceptance: just check passes; run the one test ~20 times in a loop under some load and report the count.
