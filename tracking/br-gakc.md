+++
id = "br-gakc"
title = "Agent-filed and linked tickets sit uncommitted: commit ticket new/link/set writes, or flag them to the orchestrator"
kind = "bug"
state = "built"
created_at = "2026-10-22T01:00:14.338Z"
updated_at = "2026-10-22T01:52:05.462962Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:gakc",
]
branch = "bridle/gakc"
commit = "22e86647cd95da83a48d348f611e26f875c46bd0"
summary = "ticket new/set/link (crates/bridle/src/ticket.rs commit_ticket_files) commit the file they write when the clone is on the integration branch; commit only, never push; on a worker branch nothing is committed (the worker's commit carries it). Unit test in a temp repo. cli.md and CHANGELOG updated. br-dfy7's stray untracked file lives in the main clone, outside my worktree: not handled, manager should commit-as-dropped or delete it."
ticket = "gakc"
+++

docs/tickets/open/agent-filed-and-linked-tickets-sit-uncommitted-commit-ticket-gakc.md

## Thread

### note · external:orchestrator · 2026-10-22T01:00:42.516Z
orchestrator: committed and readied (bug/chore, rule ready-without-asking). Split from br-6gdp by pm-1.

### note · agent:gakc · 2026-10-22T01:52:05.384Z
built: 22e86647cd95da83a48d348f611e26f875c46bd0 (branch bridle/gakc)

### note · agent:gakc · 2026-10-22T01:52:05.462Z
done: ticket new/set/link commit their file on the integration branch; just check exit 0, 1963 passed; 22e86647. Not done: br-dfy7's stray file is in the main clone (outside my worktree); please commit it as dropped or delete it.
