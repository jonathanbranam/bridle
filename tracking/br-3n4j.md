+++
id = "br-3n4j"
title = "The orchestrator holds and asks about a ticket without putting the reason or the question on its thread"
kind = "fix"
state = "integrated"
created_at = "2026-10-19T12:15:34.875Z"
updated_at = "2026-10-19T23:01:54.260643Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w3n4j",
]
branch = "bridle/w3n4j"
commit = "f3dd98af03d6ac6f8c040c25cc2296802bda92fc"
summary = """Orchestrator role text now says a question or hold about one ticket goes on its thread in the ticket's own project (bridle ticket ask/hold --project <p>), not a plain send; aide and project-manager got a one-line equivalent; talk-on-the-task gained a sentence that a hold reason and question state the issue itself. Item 3: no code change needed. 'bridle send --project <p> --task <id>' writes the other project's thread only with a direct local token (BRIDLE_AS + credentials entry); via the outbox it refuses --task (misc.rs send). The orchestrator should run 'bridle ticket ask --project <p> <id> "..." --to external:aide' and 'bridle ticket hold --project <p> <id> --reason ...'."""
ticket = "3n4j"
+++

docs/tickets/open/the-orchestrator-holds-and-asks-about-a-ticket-without-putti-3n4j.md

## Thread

### note · external:advisor/product-manager · 2026-10-19T12:15:44.000Z
Readied by advisor (product-manager): the human asked for it (via track-web's aide); one clear fix.

### note · agent:pm-1 · 2026-10-19T12:16:20.318Z
Brief (pm-1). Model: Sonnet (item 3 may need code). Build the ask and Proposal as written: orchestrator.md (and aide and project-manager roles if they have the same gap) say a hold or question about a ticket goes with `--task <id>` / `bridle ticket ask` on the ticket's own project (`--project <p>`); add the one sentence to workflow/base/rules/talk-on-the-task.md that a hold reason and a question must state the issue itself. Item 3: try `bridle send --project <p> --task <id>` from a <host>-home context in a test or by reading the send code (crates/bridle, crates/bridle-daemon send route); if it does not land on the other project's ticket thread, fix it, else say on this thread what the orchestrator should run. Acceptance: just check (workflow text may be asserted by tests; update them). Same files as br-vsp9 and br-57kc (orchestrator.md): run after them. Priority: normal-high, it affects track-web's trial visibility.

### note · agent:w3n4j · 2026-10-19T23:01:32.896Z
built: 62035c09727205d942b42388d9d208ee1e0661e5 (branch bridle/w3n4j)

### note · agent:w3n4j · 2026-10-19T23:01:32.977Z
done: role text + talk-on-the-task sentence; item 3 needs no code (send --task works cross-project only with a direct token, else use ticket ask/hold --project); just check exit 0, 1900 tests passed, main already merged; 62035c09727205d942b42388d9d208ee1e0661e5

### note · agent:manager-2 · 2026-10-19T23:01:39.801Z
integrated: f3dd98af03d6ac6f8c040c25cc2296802bda92fc (branch bridle/w3n4j)

### note · agent:manager-2 · 2026-10-19T23:01:54.260Z
cleanup: removed agent w3n4j, branch bridle/w3n4j
