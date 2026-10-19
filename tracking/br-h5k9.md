+++
id = "br-h5k9"
title = "stage-land's 'stage written, waiting for signature' note reaches no watcher"
kind = "bug"
state = "built"
created_at = "2026-10-19T21:00:43.249Z"
updated_at = "2026-10-19T23:51:17.065646Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wh5k9",
]
branch = "bridle/wh5k9"
commit = "14957c0e11eb67c018f7075447354703c764a073"
summary = "stage_land (server.rs) now calls emit_comment after its note_task, as the note_task handler does, so watchers and the claimant are told when a stage lands. The other note_task call sites are not stage flow (send, submit, conflicts, done/incident, landing report); no gap fixed there. Test: a_stage_landing_tells_the_tickets_watchers in principal_wake_test.rs."
ticket = "h5k9"
+++

docs/tickets/open/stage-land-s-stage-written-waiting-for-signature-note-reache-h5k9.md

## Thread

### note · agent:pm-1 · 2026-10-19T21:01:28.534Z
Brief (pm-1). Model: Haiku to Sonnet (Sonnet if the test setup is involved). Do the ask as written: stage_land in crates/bridle-daemon/src/server.rs calls emit_comment after its note, as the note_task handler does; check the other stage-flow call sites of state.tasks.note_task for the same gap and fix those found. Test: a watcher gets a message when a stage lands. Docs: CHANGELOG. Acceptance: just check. Priority high: signature requests are silently lost in the human's Stages try.

### note · agent:wh5k9 · 2026-10-19T23:51:16.967Z
built: 14957c0e11eb67c018f7075447354703c764a073 (branch bridle/wh5k9)

### note · agent:wh5k9 · 2026-10-19T23:51:17.065Z
done: stage_land now calls emit_comment so watchers are told; no other stage-flow gap; just check exit 0, 1902 tests passed; 14957c0e11eb67c018f7075447354703c764a073
