+++
id = "br-dkqf"
title = "A version 2 ticket's row went built with no implementation: only its stage branches had landed (br-2ae8)"
kind = "bug"
state = "integrated"
created_at = "2026-10-21T10:07:43.686Z"
updated_at = "2026-10-21T11:26:26.673908Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:dkqf",
]
branch = "bridle/dkqf"
commit = "d6f83a6125a6d0847179868e59a456365fe6e99e"
summary = "built_task (tasks.rs) now refuses a version 2 ticket that is unprepared, and built_task/done_task refuse a bridle/designer-* branch on a v2 ticket even when all stages are signed (new stage_watch::is_v2). Cause of br-2ae8: the old guard covered only done and unprepared tickets; a stage writer's built passed once prepared. Caveat: the exact historical caller is not provable from events; stage-land never touched the row. Test in stage_watch_test.rs; docs/design/stages.md updated."
ticket = "dkqf"
+++

docs/tickets/open/a-version-2-ticket-s-row-went-built-with-no-implementation-o-dkqf.md

submitted by external:orchestrator

## The ask

Filed by the orchestrator, 2026-10-21 ~6:10 AM ET, at pm-1's request.

br-2ae8 (a version 2 ticket, Stages MVP step 6) showed `built` in tier 1 on 2026-10-21 10:05Z with no implementation: no implementation branch, only its stage branches (`bridle/designer-2ae8-{design,specs,tasks}`), all already on main (the last stage landed in 7e110b43). manager-2 found nothing to land; pm-1 sent it back to planned (10:07Z).

Timeline from its thread: all stages signed 23:07:45Z; bridle "prepared ... ready" 23:07:56Z; still `open` 01:28Z (br-h5kz); `built` by 10:05Z. What moved it to `built` is unknown: the last stage branch's landing, the stage designer (designer-2ae8-tasks) reporting done, or the br-fyww stage-landing path.

Wanted: find what set `built` (the row's history), and make a stage landing or a stage writer's done never mark a version 2 ticket's row built; only its implementation can. A test: land the last stage of a v2 ticket and its row stays open/planned.

Model: Sonnet. Related: br-2ae8, br-h5kz, br-fyww.

## Thread

### note · external:orchestrator · 2026-10-21T10:07:43.686Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-21T10:08:18.740Z
orchestrator: readied (bug).

### note · agent:dkqf · 2026-10-21T11:26:04.782Z
built: e5dff769 (branch bridle/dkqf)

### note · agent:dkqf · 2026-10-21T11:26:05.289Z
done: stage writer's built/done refused on v2 tickets (designer branch or unprepared); check exit 0, 1959 tests; e5dff769 (last commit only adds CHANGELOG line after the green check)

### note · agent:manager-2 · 2026-10-21T11:26:20.278Z
integrated: d6f83a6125a6d0847179868e59a456365fe6e99e (branch bridle/dkqf)

### note · agent:manager-2 · 2026-10-21T11:26:26.673Z
cleanup: removed origin branch bridle/dkqf; kept agent dkqf (background job pid 82889)
