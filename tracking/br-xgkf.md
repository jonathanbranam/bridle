+++
id = "br-xgkf"
title = "project setup gaps: failed init commit, declined aide token not remembered, next: hints beyond init"
kind = "fix"
state = "planned"
created_at = "2026-10-16T10:03:34.933Z"
updated_at = "2026-10-16T21:08:40.738489Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:xgkf",
]
branch = "bridle/xgkf"
commit = "f6926f95c92cd68ebea59e14b93b7ab11ef9edf9"
summary = "project_setup.rs: (1) check_init now also asks git whether .bridle/config.toml is uncommitted; if so init is todo and apply commits the existing files (no re-scaffold). (2) A declined aide token is written to the clone's git config (bridle.setup.aide-token=declined); the check shows skip (not asked again) unless the token exists. Chose git config over a file or the committed .bridle/config.toml: per machine, no new file, doesn't dirty the tree. (3) print_next_for finds the clone (--repo, cwd, running daemon) and prints next: after projects add, project start (local only), token pair --projects P (single project, not dry-run) and mail install. Tests: init_status, aide_status, git-backed uncommitted/marker test. Docs: cli.md, CHANGELOG. Not verified end to end in a throwaway workspace beyond unit tests."
ticket = "xgkf"
+++

docs/tickets/open/project-setup-gaps-failed-init-commit-declined-aide-token-no-xgkf.md

## Thread

### note · agent:xgkf · 2026-10-16T11:12:31.973Z
built: ec458ec4d751cf147abdc455543a8124ebdf188e (branch bridle/xgkf)

### note · agent:xgkf · 2026-10-16T11:12:32.148Z
done: uncommitted init is unmet, aide decline remembered (git config), next: hints on 4 more commands; just check exit 0, 1758 tests; ec458ec4

### note · agent:manager-2 · 2026-10-16T11:44:09.881Z
integrated: f6926f95c92cd68ebea59e14b93b7ab11ef9edf9 (branch bridle/xgkf)

### note · agent:manager-2 · 2026-10-16T11:44:36.954Z
cleanup: removed agent xgkf, branch bridle/xgkf

### note · external:advisor/product-manager · 2026-10-16T21:08:26.653Z
advisor (product-manager), verifying f6926f95: items 1 and 2 met, with tests. The next: hints are wired, but the ask said 'tests for each' and nothing tests the hints after projects add, project start, token pair and mail install. Reopened for those tests.

### note · agent:pm-1 · 2026-10-16T21:08:38.198Z
Brief for the remainder (pm-1): tests only. Add a test for the "next:" hint printed after each of: projects add, project start, token pair, mail install (the hint text and that it is computed from the inspector's state). Fix any hint a test shows is wrong. Acceptance: just check. Model: Haiku is fine; Sonnet if the commands are hard to drive in tests.
