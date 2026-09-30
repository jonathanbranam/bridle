+++
id = "br-e22f"
title = "xe5c: rename docs/questions to docs/tickets (bridle; meta-notes uses tickets/)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T03:20:35.033Z"
updated_at = "2026-09-30T03:21:11.978090Z"
size = "S"
+++

See docs/questions/open/questions-become-tickets-and-a-docs-folder-plan-xe5c.md. The name is the human's decision; the wider docs-folder layout waits for a planning session with the human.

## Thread

### note · agent:pm-1 · 2026-09-30T03:21:11.955Z
PM brief: rename docs/questions/ (open/, resolved/) to docs/tickets/ with git mv (history kept), then fix every reference: grep -rn 'docs/questions' across docs/, workflow/ (roles, rules, skills), CLAUDE.md, scripts/, crates/ (code and tests that read or write ticket paths, e.g. task import/ticket links, doctor), CHANGELOG stays as history except a new line. Wiki links [[...]] and 'ticket: docs/questions/...' lines in existing task bodies are records: leave old task bodies; if code resolves such paths, make it accept both or fall back to docs/tickets. Update docs/README.md conventions and the ticket-filing rules. Do NOT restructure other docs/ folders (the human will plan that in a session). Runs ALONE: one worker, nothing else landing, so rebase and merge right away. Acceptance: just check passes; grep for docs/questions shows only historical records. Model: Sonnet. Out of scope: the docs-folder planning.
