+++
id = "br-hd28"
title = "Count the external and interactive sessions in bridle usage (orchestrator, advisors, aide, the human's own)"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T22:39:55.368Z"
updated_at = "2026-10-12T23:47:49.144942Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:whd28",
]
branch = "bridle/whd28"
commit = "5b8b854ad0343ec546486fd70f100d5b7f6d5666"
summary = """`bridle statusline` (runs in every orchestrator/advisor/aide/human session) now also posts the session's running cost, input/output token totals and an identity (external:orchestrator, external:advisor/<name>, external:aide, else external:unbridled; from BRIDLE_AS/BRIDLE_ADVISOR_NAME) to POST /v1/statusline, best effort, 2 s timeout, concurrent with the counts call. Migration 28 adds identity, running totals and cost/input/output delta columns to interactive_usage; the daemon stores a delta against the session's previous row (counter reset = new total is all new; unchanged report = no row; first report = baseline 0; no session id or missing figure = NULL = "not measured"). Usage and UsageBreakdown gain `external` (grouped by identity, role or model, honouring --since); `bridle usage` prints it in its own section outside the hosted totals, "not measured" where nothing was reported. Caveats: usage counts from when bridle first saw a session; coverage is per project daemon (sessions whose cwd resolves to this daemon), so an account-wide sum needs every project's output. Docs: usage-and-budget.md, storage.md, cli.md, CHANGELOG. Tests: store (deltas, reset, not-measured), session_identity, statusline_test. Not done: reading Claude Code session files, cross-daemon aggregation."""
ticket = "hd28"
+++

docs/tickets/open/count-the-external-and-interactive-sessions-in-bridle-usage-hd28.md

## Thread

### note · agent:pm-1 · 2026-10-12T22:40:32.963Z
Brief (pm-1). Model: Sonnet. Size: M (keep to one branch; if the statusline revival and the usage report both grow, tell the manager before splitting). Blocked by br-9y7v: both handle per-session running totals, so build on its baseline-reset approach. The ask above is the spec and the planner's recommendation (revive the statusline post into `interactive_usage`; deltas per session id). Start: docs/design/usage-and-budget.md, docs/design/storage.md, the statusline hook code, the `interactive_usage` table and the `bridle usage` command (crates/bridle/src, bridle-daemon store), br-gztq gap 6 and br-368g for background. Account-wide totals: remember each project daemon has its own database, so say in the docs what `bridle usage` covers. A session bridle can't measure shows "not measured", never zero. Migration: only a DB migration if the table changes; no project file changes unless the statusline config does (if it does, say how existing projects get it). Files also: docs/design/cli.md, CHANGELOG.md. Out of scope: reading Claude Code's session files, per-project budget changes.

### note · agent:whd28 · 2026-10-12T23:47:21.187Z
built: fa91d885 (branch bridle/whd28)

### note · agent:whd28 · 2026-10-12T23:47:23.652Z
done: bridle usage now counts external sessions (statusline posts per-session running totals + identity; deltas per session; 'not measured' never zero); just check exit 0, 1628 tests, main already merged; fa91d885

### note · agent:manager-2 · 2026-10-12T23:47:29.794Z
integrated: 5b8b854ad0343ec546486fd70f100d5b7f6d5666 (branch bridle/whd28)

### note · agent:manager-2 · 2026-10-12T23:47:49.144Z
cleanup: removed agent whd28, branch bridle/whd28
