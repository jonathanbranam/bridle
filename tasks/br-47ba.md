+++
id = "br-47ba"
title = "Orchestrator/advisor session names include the host (sfb3)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T01:38:41.901Z"
updated_at = "2026-09-30T01:44:43.677861Z"
size = "S"
summary = "Updated scripts/claude-orchestrator and scripts/claude-advisor to include hostname in session names (bridle-orch-<hostname> and bridle-advisor-<hostname>), with optional BRIDLE_SESSION_SUFFIX env override. Updated docs and CHANGELOG. Resulting command lines: on 'MacBook': --name bridle-orch-macbook; on 'NUC': --name bridle-orch-nuc; with override: BRIDLE_SESSION_SUFFIX=home-lab uses --name bridle-orch-home-lab. Tasks br-47ba (sfb3)."
+++

original id: sfb3
Ticket: docs/questions/open/session-names-per-machine-sfb3.md

## Thread

### note · agent:pm-1 · 2026-09-30T01:39:35.904Z
PM brief: edit scripts/claude-orchestrator and scripts/claude-advisor so --name and --remote-control use bridle-orch-$(hostname -s | tr A-Z a-z) and bridle-advisor-<same>; optional BRIDLE_SESSION_SUFFIX env override replaces the host part. Update any doc that quotes the old names (grep bridle-orch). Acceptance: just check passes; run each script with a dry-run/echo if one exists, else show the resulting command line in the summary. Model: Haiku. Out of scope: NUC setup, other scripts.
