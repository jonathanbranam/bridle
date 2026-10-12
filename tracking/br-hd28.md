+++
id = "br-hd28"
title = "Count the external and interactive sessions in bridle usage (orchestrator, advisors, aide, the human's own)"
kind = "feature"
state = "planned"
created_at = "2026-10-12T22:39:55.368Z"
updated_at = "2026-10-12T22:40:34.569837Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "hd28"
+++

docs/tickets/open/count-the-external-and-interactive-sessions-in-bridle-usage-hd28.md

## Thread

### note · agent:pm-1 · 2026-10-12T22:40:32.963Z
Brief (pm-1). Model: Sonnet. Size: M (keep to one branch; if the statusline revival and the usage report both grow, tell the manager before splitting). Blocked by br-9y7v: both handle per-session running totals, so build on its baseline-reset approach. The ask above is the spec and the planner's recommendation (revive the statusline post into `interactive_usage`; deltas per session id). Start: docs/design/usage-and-budget.md, docs/design/storage.md, the statusline hook code, the `interactive_usage` table and the `bridle usage` command (crates/bridle/src, bridle-daemon store), br-gztq gap 6 and br-368g for background. Account-wide totals: remember each project daemon has its own database, so say in the docs what `bridle usage` covers. A session bridle can't measure shows "not measured", never zero. Migration: only a DB migration if the table changes; no project file changes unless the statusline config does (if it does, say how existing projects get it). Files also: docs/design/cli.md, CHANGELOG.md. Out of scope: reading Claude Code's session files, per-project budget changes.
