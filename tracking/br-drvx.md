+++
id = "br-drvx"
title = "Statusline reports from sessions outside any bridle project are counted in a machine-level bucket (ktgd decision 2)"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T23:05:27.092Z"
updated_at = "2026-10-14T08:36:33.239221Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:drvx",
]
branch = "bridle/drvx"
commit = "42d4a2ab1f93af29a0952334b1a9b7273869ddb0"
summary = "Statusline reports from a session with no bridle project (no daemon for its cwd, no BRIDLE_URL/BRIDLE_PROJECT) now post to one fallback daemon per machine: the first live project in the registry by name (discovery::machine_fallback_daemon), no new setting so existing machines need no edit. An unbridled session is recorded as identity 'external:unbridled/outside any project', its own bucket in that project's 'bridle usage' external sessions. Counts on the line still skip the fallback; the 2 s timeout and silent failure are unchanged. Tests: fallback pick (discovery) and a post to a fake server (orchestrator.rs). Docs: usage-and-budget.md, cli.md, CHANGELOG. Caveat: the bucket lands in whichever project sorts first, so it moves if that project stops; no machine-wide sum was added."
ticket = "drvx"
+++

docs/tickets/open/statusline-reports-from-sessions-outside-any-bridle-project-drvx.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:35.293Z
Brief (pm-1). Model: Sonnet. Size: S-M. Blocked by br-mksa only because both edit the statusline report path; start after it merges (I add the edge). The ask and ktgd decision 2 are the spec. Start: `bridle statusline` in crates/bridle/src (how it finds the project's daemon and drops the rest) and the discovery code in bridle-api. Simplest design: when no project daemon is found for the session's directory, post to one per-machine daemon chosen by a rule you document (e.g. a `[machine]` setting naming a project, defaulting to the first project in the registry), recorded with project = "outside any project". Never block or slow the status line (short timeout, fail silent as today). Test: a session in a non-bridle directory is recorded and shows under the bucket in `bridle usage`. Docs: usage-and-budget.md, cli.md, CHANGELOG.md. If this adds a config key, say how existing machines get a sane default without edits. Out of scope: desktop app and claude.ai sessions.

### note · agent:drvx · 2026-10-14T07:04:26.102Z
built: 34fcd4ea4b64af7786563b7e9ed63f3c2e0c1e1e (branch bridle/drvx)

### note · agent:drvx · 2026-10-14T07:04:26.147Z
done: statusline sessions outside any bridle project now post to a per-machine fallback daemon (first live project by name) as 'external:unbridled/outside any project'; just check exit 0, 1662 tests passed (6 skipped); 34fcd4ea4b64af7786563b7e9ed63f3c2e0c1e1e

### note · agent:manager-2 · 2026-10-14T08:36:12.912Z
integrated: 42d4a2ab1f93af29a0952334b1a9b7273869ddb0 (branch bridle/drvx)

### note · agent:manager-2 · 2026-10-14T08:36:33.239Z
cleanup: removed agent drvx, branch bridle/drvx
