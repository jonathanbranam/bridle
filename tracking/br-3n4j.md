+++
id = "br-3n4j"
title = "The orchestrator holds and asks about a ticket without putting the reason or the question on its thread"
kind = "fix"
state = "planned"
created_at = "2026-10-19T12:15:34.875Z"
updated_at = "2026-10-19T12:30:04.317164Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "3n4j"
+++

docs/tickets/open/the-orchestrator-holds-and-asks-about-a-ticket-without-putti-3n4j.md

## Thread

### note · external:advisor/product-manager · 2026-10-19T12:15:44.000Z
Readied by advisor (product-manager): the human asked for it (via track-web's aide); one clear fix.

### note · agent:pm-1 · 2026-10-19T12:16:20.318Z
Brief (pm-1). Model: Sonnet (item 3 may need code). Build the ask and Proposal as written: orchestrator.md (and aide and project-manager roles if they have the same gap) say a hold or question about a ticket goes with `--task <id>` / `bridle ticket ask` on the ticket's own project (`--project <p>`); add the one sentence to workflow/base/rules/talk-on-the-task.md that a hold reason and a question must state the issue itself. Item 3: try `bridle send --project <p> --task <id>` from a <host>-home context in a test or by reading the send code (crates/bridle, crates/bridle-daemon send route); if it does not land on the other project's ticket thread, fix it, else say on this thread what the orchestrator should run. Acceptance: just check (workflow text may be asserted by tests; update them). Same files as br-vsp9 and br-57kc (orchestrator.md): run after them. Priority: normal-high, it affects track-web's trial visibility.
