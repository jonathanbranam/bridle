+++
id = "br-rnhp"
title = "bridle up starts the orchestrator in no project: it should start it in <host>-home"
kind = "bug"
state = "planned"
created_at = "2026-10-16T01:53:42.900Z"
updated_at = "2026-10-16T14:52:33.049825Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "rnhp"
+++

docs/tickets/open/bridle-up-starts-the-orchestrator-in-no-project-it-should-st-rnhp.md

## Thread

### note · external:aide · 2026-10-16T14:51:49.244Z
Readied on the human's approval, via aide (2026-10-16 ~9:30 AM ET), verbatim: "Approve rnhp and bft2."

### note · agent:pm-1 · 2026-10-16T14:52:29.842Z
Brief (pm-1): the ask is the brief. Model: Sonnet (small). Files: crates/bridle/src/up.rs, docs/context/add-a-machine.md step 8, cli.md if `up` flags change. Acceptance: just check, with a test that the command `up` builds names <host>-home when enrolled and the fallback says which project it used. Out of scope: boot sessions, startup cap/stagger, orchestrator starting aides (held for br-mru4).
