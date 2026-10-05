+++
id = "br-abnq"
title = "One command for any agent's status and context"
kind = "feature"
state = "pending"
created_at = "2026-10-05T10:25:03.830Z"
updated_at = "2026-10-05T10:25:22.495065Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

original id: abnq
docs/tickets/open/one-command-for-any-agent-s-status-and-context-abnq.md

submitted by external:orchestrator@nuc

The human, 2026-10-05 (NUC): 'this should be an easy and direct command to find the context of any agent that we can run. It should just return status and context information about that agent, so we're tracking the context.'

Today: workers and managers show CONTEXT in 'bridle agents', but interactive sessions (advisor, aide, orchestrator) only appear in 'bridle status --json' under .sessions (identity, pid, pane, tokens, started_at, last_activity). The plain 'bridle status' table doesn't show them. Finding the notes advisor at 227K took --help reading, jq and tmux capture-pane.

Ask: e.g. 'bridle context [<agent|session>]' (or 'bridle agent status <name>' covering sessions too, e.g. advisor/notes): state, pid, uptime, last activity, context tokens and where they sit against the 200K/250K/300K steps (gq9r), and whether a handover is planned. Without an argument, one line per agent and session in the project. Also list sessions in the plain 'bridle status' / 'bridle agents' tables.

## Thread

### note · external:orchestrator@nuc · 2026-10-05T10:25:03.833Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T10:25:22.495Z
Triage (pm-1): accept (the human's own ask). Ticket minted (uncommitted; commit on main). Stays pending until approved with `bridle task ready br-abnq`; then I plan it (Sonnet).
