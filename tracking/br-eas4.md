+++
id = "br-eas4"
title = "bridle serve ignores --project for the repo: say so when --project and the folder disagree"
kind = "bug"
state = "planned"
created_at = "2026-10-12T02:49:11.448Z"
updated_at = "2026-10-15T23:04:03.404472Z"
created_by = "external:advisor/nuc-rename"
watchers = ["external:advisor/nuc-rename"]
ticket = "eas4"
+++

docs/tickets/open/bridle-serve-ignores-project-for-the-repo-say-so-when-projec-eas4.md

## Thread

### note · external:advisor/product-manager · 2026-10-15T22:51:57.876Z
Readied by the human via advisor product-manager, 2026-10-15 ~6:51 PM ET: "Yes ready approve 3-5."

### note · agent:pm-1 · 2026-10-15T22:52:40.147Z
Brief (pm-1). Model: Sonnet. Size: S. Take the ticket's primary fix (refuse, do not resolve the repo from the registry): in `bridle serve` (crates/bridle/src/serve.rs ~41; the project name is passed on as `project: cli.project.clone()`), before anything else, including migrations, when `--project` / `$BRIDLE_PROJECT` is given and the repo from `--repo`/cwd belongs to a different project, refuse with both names: "--project <a>, but this folder is project <b> (<path>); cd to <a>'s clone (<registry path, if known>) or drop --project". Find how a repo's own project name is read (git grep -n "project_name\|fn project_of" crates/bridle/src crates/bridle-daemon/src; the .bridle/config.toml or the state branch) and reuse it; if it can't be read, skip the check rather than block serve. Also make `refuse_if_already_running` (crates/bridle-daemon/src/lib.rs ~1290-1311) name the project and workspace it found in "a daemon is already running for this workspace". Tests: mismatch refuses before migration; matching name serves; no --project serves; already-running error names project and workspace. Docs: docs/design/agent-host/daemon.md (serve), CHANGELOG. Acceptance: just check. Out of scope: resolving the repo from the registry. No migration. Both touch crates/bridle/src/serve.rs-area code; if br-9a7b/others in the queue touch lib.rs, the manager orders them.
