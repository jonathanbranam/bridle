+++
id = "br-cxjs"
title = "A project move tells neither machine's orchestrator: notices go to the moved project's own orchestrator role, unread"
kind = "fix"
state = "claimed"
created_at = "2026-10-19T12:11:04.037Z"
updated_at = "2026-10-20T02:45:53.563714Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wcxjs",
]
ticket = "cxjs"
+++

docs/tickets/open/a-project-move-tells-neither-machine-s-orchestrator-notices-cxjs.md

## Thread

### note · external:advisor/product-manager · 2026-10-19T12:11:13.023Z
Readied by advisor (product-manager): a fix from kx5u trial 2, per the human's model "An orch is for machine".

### note · agent:pm-1 · 2026-10-19T12:11:48.843Z
Brief (pm-1). Model: Sonnet. Build the ask as written (items 1-5), using the cross-project send path of `bridle send --project <host>-home`. Files: the project stop/start/drain code in crates/bridle (the same area as br-jqf9 and br-ec5m), docs/context/move-a-project.md, the move design doc, CHANGELOG. Tests with a fake send: stop notifies OLD-home's orchestrator and aide once; start after take-over notifies NEW-home's orchestrator; drain messages go to the home orchestrator; a missing <host>-home falls back and says so; a failed send never fails the move. Acceptance: just check. Out of scope: reading moved-in.json beyond what start already reads; changing the home project rule (br-7ytg). Same code as br-jqf9 and br-ec5m: run after them, not alongside.
