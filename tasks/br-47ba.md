+++
id = "br-47ba"
title = "Orchestrator/advisor session names include the host (sfb3)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T01:38:41.901Z"
updated_at = "2026-09-30T01:39:35.934521Z"
size = "S"
+++

original id: sfb3
Ticket: docs/questions/open/session-names-per-machine-sfb3.md

## Thread

### note · agent:pm-1 · 2026-09-30T01:39:35.904Z
PM brief: edit scripts/claude-orchestrator and scripts/claude-advisor so --name and --remote-control use bridle-orch-$(hostname -s | tr A-Z a-z) and bridle-advisor-<same>; optional BRIDLE_SESSION_SUFFIX env override replaces the host part. Update any doc that quotes the old names (grep bridle-orch). Acceptance: just check passes; run each script with a dry-run/echo if one exists, else show the resulting command line in the summary. Model: Haiku. Out of scope: NUC setup, other scripts.
