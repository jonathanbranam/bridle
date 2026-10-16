+++
id = "br-eas4"
title = "bridle serve ignores --project for the repo: say so when --project and the folder disagree"
kind = "bug"
state = "integrated"
created_at = "2026-10-12T02:49:11.448Z"
updated_at = "2026-10-16T02:52:33.477520Z"
created_by = "external:advisor/nuc-rename"
watchers = [
    "external:advisor/nuc-rename",
    "agent:eas4",
]
branch = "bridle/eas4"
commit = "d2c02581d80103137a2a23a1575e25025d504114"
summary = """`bridle serve --project X` now refuses, before anything else (migrations included), when the repo from --repo/cwd belongs to a different project: "--project X, but this folder is project Y (<path>); cd to X's clone (<registry path if known>) or drop --project" (crates/bridle/src/serve.rs, refuse_project_mismatch). The repo's project is read with project::from_cwd; if unreadable, no check. refuse_if_already_running now names project and workspace. Tests: mismatch_error unit test, already-running unit test. Docs: daemon.md, CHANGELOG. Not covered: an end-to-end test of the refusal ordering before migrations (it is the first call in run())."""
ticket = "eas4"
+++

docs/tickets/open/bridle-serve-ignores-project-for-the-repo-say-so-when-projec-eas4.md

## Thread

### note · external:advisor/product-manager · 2026-10-15T22:51:57.876Z
Readied by the human via advisor product-manager, 2026-10-15 ~6:51 PM ET: "Yes ready approve 3-5."

### note · agent:pm-1 · 2026-10-15T22:52:40.147Z
Brief (pm-1). Model: Sonnet. Size: S. Take the ticket's primary fix (refuse, do not resolve the repo from the registry): in `bridle serve` (crates/bridle/src/serve.rs ~41; the project name is passed on as `project: cli.project.clone()`), before anything else, including migrations, when `--project` / `$BRIDLE_PROJECT` is given and the repo from `--repo`/cwd belongs to a different project, refuse with both names: "--project <a>, but this folder is project <b> (<path>); cd to <a>'s clone (<registry path, if known>) or drop --project". Find how a repo's own project name is read (git grep -n "project_name\|fn project_of" crates/bridle/src crates/bridle-daemon/src; the .bridle/config.toml or the state branch) and reuse it; if it can't be read, skip the check rather than block serve. Also make `refuse_if_already_running` (crates/bridle-daemon/src/lib.rs ~1290-1311) name the project and workspace it found in "a daemon is already running for this workspace". Tests: mismatch refuses before migration; matching name serves; no --project serves; already-running error names project and workspace. Docs: docs/design/agent-host/daemon.md (serve), CHANGELOG. Acceptance: just check. Out of scope: resolving the repo from the registry. No migration. Both touch crates/bridle/src/serve.rs-area code; if br-9a7b/others in the queue touch lib.rs, the manager orders them.

### note · agent:eas4 · 2026-10-16T02:02:46.132Z
built: e3450ccc035a5dd6e8954956a611f4a0eaa87299 (branch bridle/eas4)

### note · agent:eas4 · 2026-10-16T02:02:46.185Z
done: serve refuses --project/folder mismatch, already-running names project+workspace; check green 1726 passed, ran on e3450ccc035a5dd6e8954956a611f4a0eaa87299 (merge of main on top 736ee92fbef48d623fa536b6b46aec4e18abdf71 added only ticket docs)

### note · agent:manager-2 · 2026-10-16T02:52:33.477Z
integrated: d2c02581d80103137a2a23a1575e25025d504114 (branch bridle/eas4)
