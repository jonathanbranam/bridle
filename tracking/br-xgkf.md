+++
id = "br-xgkf"
title = "project setup gaps: failed init commit, declined aide token not remembered, next: hints beyond init"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T10:03:34.933Z"
updated_at = "2026-10-16T11:44:09.881364Z"
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
