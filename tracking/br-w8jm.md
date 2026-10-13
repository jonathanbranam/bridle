+++
id = "br-w8jm"
title = "ticket ready makes a missing row, accepts a qualified id, and skips dropped rows"
kind = "fix"
state = "planned"
created_at = "2026-10-13T02:02:19.375Z"
updated_at = "2026-10-13T13:07:08.161876Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "w8jm"
+++

docs/tickets/open/ticket-ready-makes-a-missing-row-accepts-a-qualified-id-and-w8jm.md

## Thread

### note · agent:pm-1 · 2026-10-13T13:07:07.098Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ask above is the spec (three snags; the cross-project blocked_by note at the end is a question for the PdM, not part of this). Start: `ready_cmd` in crates/bridle/src/ticket.rs (row lookup, "has no row on this daemon", first-row-wins) and the task-creation handler for `POST /v1/tasks` in bridle-daemon (the `ticket` field). Fixes: (1) `ticket ready <id>` on a ticket file with no row creates the row from the file's title and kind (reuse what `ticket new` does), then readies it; (2) the daemon normalizes a qualified `ticket` (strip its own project prefix; a different prefix stays an error or is kept as is, say which in a comment); (3) `ready_cmd` prefers a pending row and skips dropped ones, and only reports "dropped" when no other row exists. Tests for each of the three as in the Verify line. Docs: docs/design/cli.md, CHANGELOG.md. Migration: none; no change to what projects keep. Out of scope: cross-project queue edges, repairing old duplicate rows.
