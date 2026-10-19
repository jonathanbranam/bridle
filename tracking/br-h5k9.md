+++
id = "br-h5k9"
title = "stage-land's 'stage written, waiting for signature' note reaches no watcher"
kind = "bug"
state = "open"
created_at = "2026-10-19T21:00:43.249Z"
updated_at = "2026-10-19T21:01:28.534603Z"
created_by = "external:advisor/tj7q"
watchers = ["external:advisor/tj7q"]
ticket = "h5k9"
+++

docs/tickets/open/stage-land-s-stage-written-waiting-for-signature-note-reache-h5k9.md

## Thread

### note · agent:pm-1 · 2026-10-19T21:01:28.534Z
Brief (pm-1). Model: Haiku to Sonnet (Sonnet if the test setup is involved). Do the ask as written: stage_land in crates/bridle-daemon/src/server.rs calls emit_comment after its note, as the note_task handler does; check the other stage-flow call sites of state.tasks.note_task for the same gap and fix those found. Test: a watcher gets a message when a stage lands. Docs: CHANGELOG. Acceptance: just check. Priority high: signature requests are silently lost in the human's Stages try.
