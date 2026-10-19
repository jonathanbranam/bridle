+++
id = "br-t6ev"
title = "Stages: write every unblocked stage before asking for review (design and specs together, one writer in turn)"
kind = "fix"
state = "planned"
created_at = "2026-10-19T11:55:06.292Z"
updated_at = "2026-10-19T12:10:02.731325Z"
created_by = "external:advisor/tj7q"
watchers = ["external:advisor/tj7q"]
ticket = "t6ev"
+++

docs/tickets/open/stages-write-every-unblocked-stage-before-asking-for-review-t6ev.md

## Thread

### note · agent:pm-1 · 2026-10-19T11:55:57.140Z
Brief (pm-1). Model: Sonnet. Build the ask as written (the easy way): the designer writes every unblocked stage in turn, and the signature wait no longer stops other unblocked stages. Decision so the worker needn't ask: pick the simpler of "stage-done writes claimed_by: human only when no stage is left unblocked" and "the watch ignores the human's signature claim while a stage is unblocked", state which in stages.md. Update the designer prompt (loop: after each stage-done run ticket next again; end when none is left; write each stage from its own depends_on inputs, not from the sibling). Files: crates/bridle-daemon/src/stage_watch.rs, crates/bridle/src/stage_cmds.rs, the designer role/prompt text, docs/design/stages.md, CHANGELOG. Tests: after the proposal is signed, design and specs are both written with no signature in between and the ticket reaches the human with both waiting; approve one and changes the other works independently; tasks waits for both. Acceptance: just check. Out of scope: separate writers per stage (follow-up). Priority high: it is blocking the human's Stages try. Same stage code as br-ggbx and br-bdwh: run it one at a time with them (t6ev first, as it unblocks the try).
