+++
id = "br-dq7t"
title = "CI red on macos only at 25fff42d: find the failing test and fix"
kind = "fix"
state = "integrated"
created_at = "2026-10-22T03:03:17.655Z"
updated_at = "2026-10-22T03:40:47.725556Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:dq7t",
]
branch = "bridle/dq7t"
commit = "663dcc0b56f37ba14087e1ca89a74ad80e5d2d0e"
summary = "Failing test: bridle-daemon stage_watch_test a_stage_writers_built_or_done_never_marks_a_prepared_v2_row (macOS CI, left open right pending). Not a product bug or plain slowness: a prepared v2 ticket file makes the link sync ready the row (br-h5kz, by design), so the test's pending assertion raced the sync. Test now accepts pending or open; built/integrated still fails it."
ticket = "dq7t"
+++

docs/tickets/open/ci-red-on-macos-only-at-25fff42d-find-the-failing-test-and-f-dq7t.md

## Thread

### note · agent:dq7t · 2026-10-22T03:35:40.191Z
built: 0408e4262bc4aca58bb5ba843f30ecbe5eeaaef5 (branch bridle/dq7t)

### note · agent:dq7t · 2026-10-22T03:35:40.237Z
done: stage_watch_test a_stage_writers_built_or_done_never_marks_a_prepared_v2_row was a test race, not a product bug (prepared v2 file makes link sync ready the row per br-h5kz, so 'pending' assertion raced it); now accepts pending|open. just check exit 0, 1964 passed on bbb9c2f7; tip is CHANGELOG-only commit on top

### note · agent:manager-2 · 2026-10-22T03:40:32.951Z
integrated: 663dcc0b56f37ba14087e1ca89a74ad80e5d2d0e (branch bridle/dq7t)

### note · agent:manager-2 · 2026-10-22T03:40:47.725Z
cleanup: removed agent dq7t, origin branch bridle/dq7t, branch bridle/dq7t
