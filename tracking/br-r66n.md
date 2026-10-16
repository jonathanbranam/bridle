+++
id = "br-r66n"
title = "A reopened non-incident ticket can't be planned: reopened is a dead end"
kind = "bug"
state = "integrated"
created_at = "2026-10-13T13:07:23.575Z"
updated_at = "2026-10-16T06:23:09.823649Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:r66n",
]
branch = "bridle/r66n"
commit = "441fba2f4b4de136ab0779139358760d85868e53"
summary = "plan_task (tasks.rs) now accepts open or reopened for every kind (was reopened only for incidents); drop already accepted reopened and now has a test. api.md, cli.md and CHANGELOG updated; incidents.md needed no change. just check green, 1740 tests."
ticket = "r66n"
+++

docs/tickets/open/a-reopened-non-incident-ticket-can-t-be-planned-reopened-is-r66n.md

## Thread

### note · external:advisor/product-manager · 2026-10-15T22:52:37.503Z
Readied by the human via advisor product-manager, 2026-10-15 ~6:55 PM ET: "Ready more tasks" (advisor picked this one).

### note · agent:pm-1 · 2026-10-15T22:53:20.628Z
Brief (pm-1). Model: Sonnet. Size: S. Fix per the ticket body. crates/bridle-daemon/src/tasks.rs ~862 (`plan_task` accepts `reopened` only for incidents) and the drop path: let `ticket plan` take a reopened non-incident ticket back to planned (its brief is still on the thread; reopening already cleared the claim, check that) and let `drop` accept reopened. Check `ticket ready`/`claim` handling of reopened for the same dead end (git grep -n "Reopened" crates/). Incidents keep their current behaviour. Tests: reopened feature/fix/bug can be planned and dropped; incident unchanged; an unreopened state still refuses as before. Docs in step: docs/design/agent-host api.md, docs/design/cli.md, docs/context/incidents.md (as the ticket says), CHANGELOG. Acceptance: just check. Out of scope: new states or changes to what reopen does.

### note · agent:r66n · 2026-10-16T06:04:08.347Z
built: 88b171a4 (branch bridle/r66n)

### note · agent:r66n · 2026-10-16T06:04:08.396Z
done: plan accepts reopened for any kind, drop tested, docs+CHANGELOG; check exit 0, 1740 passed; 88b171a4

### note · agent:manager-2 · 2026-10-16T06:23:09.823Z
integrated: 441fba2f4b4de136ab0779139358760d85868e53 (branch bridle/r66n)
