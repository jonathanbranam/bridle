---
id: br-h5k9
title: stage-land's 'stage written, waiting for signature' note reaches no watcher
kind: bug
created: 2026-10-19
created_by: external:advisor/tj7q
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-upxq, br-2ae8, br-ewk8, br-tj7q]
---

## The ask

Found in the human's hands-on try (br-ewk8, ticket br-2ae8). The specs stage landed on main at 854daa68 (2026-10-19 7:22 AM ET) with the thread note "stage specs written, waiting for the human's signature (on main at 854daa68)", but no watcher was told: advisor tj7q, who watches the ticket, got no message and only found it by looking ~4 hours later. The same happened to the design stage's note at 10:06:39Z (e8940620). Before br-upxq the CLI posted that note through the API and it reached the watchers (m-17097).

docs/design/stages.md: "Every command adds a thread note (...), and a note reaches the ticket's watchers."

Cause: `stage_land` in `crates/bridle-daemon/src/server.rs` posts the note with `state.tasks.note_task(...)` directly. The `note_task` HTTP handler also calls `emit_comment(...)`, which is what fans a note out to the ticket's watchers; `stage_land` skips it.

The fix: `stage_land` calls `emit_comment` after the note, as the `note_task` handler does. A test that a watcher gets a message when a stage lands. Check the other places in the stage flow that call `state.tasks.note_task` straight for the same gap.
