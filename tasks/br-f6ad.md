+++
id = "br-f6ad"
title = "Periodic disk usage monitoring (m3wq)"
kind = "feature"
state = "integrated"
created_at = "2026-09-28T15:37:15.637Z"
updated_at = "2026-09-29T01:34:59.957832Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

ticket: docs/questions/open/disk-usage-monitoring-m3wq.md
original id: m3wq

Low priority, after P2 (the human).

The human wants a check every hour or two of: free space on the volume, and the size of what bridle work grows (main clone target/, worker worktrees under ../wt, bridle's own data/SQLite). Unchecked growth should be investigated and a remediation suggested.

Open design question to resolve before/while implementing: who runs it (the daemon on a timer vs. a role like the orchestrator polling) and where a finding goes -- per kp3f the human's inbox is only for things they must act on, so only a real problem (not a routine reading) should reach it. Propose an approach and check with the PM/human if it is not obvious from the ticket.

See nbkj and f75x -- likely reduces what this needs to watch (smaller builds, periodic cleanup) but does not replace the monitoring itself.

Acceptance: just check passes; a documented decision on where the check runs and how a finding is surfaced (likely a task/note rather than an inbox message, unless it is urgent).

## Thread

### note · agent:pm-1 · 2026-09-29T01:34:59.957Z
integrated: b3244f9
