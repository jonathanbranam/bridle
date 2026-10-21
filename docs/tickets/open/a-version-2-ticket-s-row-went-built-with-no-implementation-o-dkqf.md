---
id: br-dkqf
title: "A version 2 ticket's row went built with no implementation: only its stage branches had landed (br-2ae8)"
kind: bug
created: 2026-10-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

br-2ae8 (a version 2 ticket, Stages MVP step 6) showed `built` in tier 1 on 2026-10-21 10:05Z with no implementation: no implementation branch, only its stage branches (`bridle/designer-2ae8-{design,specs,tasks}`), all already on main (the last stage landed in 7e110b43). manager-2 found nothing to land; pm-1 sent it back to planned (10:07Z).

Timeline: all stages signed 23:07:45Z; "prepared ... ready" 23:07:56Z; still `open` 01:28Z (br-h5kz); `built` by 10:05Z. What moved it to `built` is unknown: the last stage branch's landing, the stage designer (designer-2ae8-tasks) reporting done / `ticket built`, or a path br-fyww's guard (which covers only `done_task` -> `integrated`) does not cover.

Wanted: find what set `built` (the row's history/events), and make a stage landing or a stage writer's done never mark a version 2 ticket's row built; only its implementation can. Cover the `built` transition (`built_task` or equivalent in crates/bridle-daemon/src/tasks.rs) the way fyww covered `done`: refuse for a v2 ticket whose build is only stages, and find any caller that does it implicitly (stage_watch.rs, the stage-done / land path).

Check: `just check`, plus a test: land the last stage of a v2 ticket and its row stays open/planned. Docs: docs/design/stages.md. Model: Sonnet. Related: br-2ae8, br-h5kz, br-fyww. Same stage-watch files as br-h5kz and br-43ug: run after them.
