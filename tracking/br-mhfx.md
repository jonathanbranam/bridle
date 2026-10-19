+++
id = "br-mhfx"
title = "integration/target is back to 90 GB (788K files): the br-j4bk prune isn't keeping it down, and dead worktrees keep their target/"
kind = "bug"
state = "integrated"
created_at = "2026-10-19T01:52:02.424Z"
updated_at = "2026-10-19T10:29:33.172344Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wmhfx",
]
priority = "high"
priority_at = "2026-10-19T01:52:12.617255Z"
branch = "bridle/wmhfx"
commit = "958d0052024eadf614d2db6b488f1ce2cd362413"
summary = "The j4bk prune only ran after a successful warm build, and bridle's config has warm_build commented out, so it never ran. Now WarmBuild::prune_after_land runs after every land under the landing lock (server.rs), over 30K deps entries (PRUNE_ENTRIES), independent of warm_build. Part 2: DiskMonitor tick (hourly) calls worktree::sweep_landed: removes wt/<name> with no agent record, branch merged, clean status, no open files; log line plus worktree.removed event {path, branch}; branch kept. Chose whole-worktree removal (git worktree remove, non-force). Tests: prune on fake tree, sweep on temp git repos. Docs: operating-model.md disk monitor; CHANGELOG Fixed. Caveat: any agent row (even terminal) protects its worktree."
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

### note · agent:wmhfx · 2026-10-19T10:01:33.120Z
Finding (part 1). The j4bk prune is `warm_build::prune_stale`, and its only call site is the end of a *successful warm build*, which runs after each land only when `[integration] warm_build` is set. In bridle's own `.bridle/config.toml` that line is commented out (`# warm_build = ...`), so no warm build runs and the prune has never run. Meanwhile the landing checks build in `integration/target` and add stale hashed artifacts each time. Fix: prune after every land (under the landing lock, so no check is building), independent of warm_build, at a lower threshold (30K deps entries) so it stays near a clean build, not just under the 100K clone limit. Part 2: an hourly sweep (in the disk monitor tick) removes `wt/<name>` worktrees that have no agent record, whose branch is merged, and that are clean and have no open files; one event per removal.

### note · agent:wmhfx · 2026-10-19T10:19:08.289Z
built: ba513f33 (branch bridle/wmhfx)

### note · agent:wmhfx · 2026-10-19T10:19:08.362Z
done: prune now runs after every land (warm_build was off, so it never ran); disk tick removes landed agentless clean worktrees; tip ba513f33d5a409cc369c2331b41eace0fc854997, just check exit 0, 1887 tests passed

### note · agent:manager-2 · 2026-10-19T10:29:20.775Z
integrated: 958d0052024eadf614d2db6b488f1ce2cd362413 (branch bridle/wmhfx)

### note · agent:manager-2 · 2026-10-19T10:29:33.172Z
cleanup: removed agent wmhfx, branch bridle/wmhfx
