+++
id = "br-cxjs"
title = "A project move tells neither machine's orchestrator: notices go to the moved project's own orchestrator role, unread"
kind = "fix"
state = "integrated"
created_at = "2026-10-19T12:11:04.037Z"
updated_at = "2026-10-20T22:40:46.678047Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wcxjs",
]
branch = "bridle/wcxjs"
commit = "f608b623478c5662dbfa24d2059a4ad040a7faa9"
summary = "New move_notice.rs (home lookup, Sender trait with a fake for tests, tell_home). project stop tells external:orchestrator and external:aide in <host>-home; project start after a take-over tells external:orchestrator there (from, schedules carried, mail state); project drain also messages the home orchestrator. No <host>-home: output says so and the old target stays. Failed notices are printed, never fail the move. Sends shell out to 'bridle --project <home> send' from the home checkout (stop's own daemon is down). Not tested live. Docs: cli.md, move-a-project.md, CHANGELOG."
ticket = "cxjs"
+++

docs/tickets/open/a-project-move-tells-neither-machine-s-orchestrator-notices-cxjs.md

## Thread

### note · external:advisor/product-manager · 2026-10-19T12:11:13.023Z
Readied by advisor (product-manager): a fix from kx5u trial 2, per the human's model "An orch is for machine".

### note · agent:pm-1 · 2026-10-19T12:11:48.843Z
Brief (pm-1). Model: Sonnet. Build the ask as written (items 1-5), using the cross-project send path of `bridle send --project <host>-home`. Files: the project stop/start/drain code in crates/bridle (the same area as br-jqf9 and br-ec5m), docs/context/move-a-project.md, the move design doc, CHANGELOG. Tests with a fake send: stop notifies OLD-home's orchestrator and aide once; start after take-over notifies NEW-home's orchestrator; drain messages go to the home orchestrator; a missing <host>-home falls back and says so; a failed send never fails the move. Acceptance: just check. Out of scope: reading moved-in.json beyond what start already reads; changing the home project rule (br-7ytg). Same code as br-jqf9 and br-ec5m: run after them, not alongside.

### note · agent:wcxjs · 2026-10-20T03:04:31.562Z
built: ed486c5f (branch bridle/wcxjs)

### note · agent:wcxjs · 2026-10-20T03:04:32.025Z
done: move notices go to <host>-home orchestrator/aide (stop, start, drain); check exit 0, 1920 tests passed; ed486c5f

### note · agent:manager-2 · 2026-10-20T03:04:39.658Z
integrated: f608b623478c5662dbfa24d2059a4ad040a7faa9 (branch bridle/wcxjs)

### note · agent:manager-2 · 2026-10-20T03:04:55.484Z
cleanup: removed agent wcxjs, branch bridle/wcxjs

### note · external:advisor/product-manager · 2026-10-20T22:40:46.678Z
Resolved by advisor (product-manager): f608b623 on main holds the worker's last done sha ed486c5f (source files identical; later edits are from 163f/t7ya). Items 1-5 in move_notice.rs, project_stop/start/drain.rs, move-a-project.md, cli.md.
