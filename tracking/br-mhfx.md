+++
id = "br-mhfx"
title = "integration/target is back to 90 GB (788K files): the br-j4bk prune isn't keeping it down, and dead worktrees keep their target/"
kind = "bug"
state = "claimed"
created_at = "2026-10-19T01:52:02.424Z"
updated_at = "2026-10-19T10:00:35.350862Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wmhfx",
]
priority = "high"
priority_at = "2026-10-19T01:52:12.617255Z"
ticket = "mhfx"
+++

docs/tickets/open/integration-target-is-back-to-90-gb-788k-files-the-br-j4bk-p-mhfx.md

## Thread

### note · external:aide · 2026-10-19T01:52:12.617Z
priority: normal -> high

### note · external:aide · 2026-10-19T01:52:13.001Z
Readied by aide (bug; rule ready-without-asking), high: it makes every check on dalek take ~2 hours.

### note · agent:pm-1 · 2026-10-19T01:52:50.762Z
Brief (pm-1). Model: Sonnet. High priority: it makes every check on dalek take ~2 hours, so rank it ahead of other new work. Two parts, investigate the first on this thread before changing code:
1. Why the br-j4bk prune isn't keeping integration/target small (find the prune: grep prune/target in crates/bridle-daemon and the j4bk ticket's resolution; check whether it runs, when, and what it keeps). Fix so debug/deps stays near a clean build (~12K files). Add a test for the prune rule with a fake tree.
2. Worktrees whose branch has landed and whose agent is gone: remove the worktree (or at least its target/), with a log line or event per removal. Pick the simpler; the rule must never touch a worktree with a live agent, unmerged work or uncommitted changes (check git status clean and branch merged into the integration branch). Test with temp git repos.
Docs: the relevant design doc under docs/design/agent-host/ (worktrees/cleanup), CHANGELOG. Acceptance: just check. Out of scope: running fewer checks at once (aide handles separately), the macOS Developer Tools setting, br-p29s.
