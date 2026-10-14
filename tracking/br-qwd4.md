+++
id = "br-qwd4"
title = "project preflight: let the human name the clone's path on the new machine (it copies the old machine's absolute path)"
kind = "feature"
state = "planned"
created_at = "2026-10-14T22:07:56.284Z"
updated_at = "2026-10-14T22:09:42.288031Z"
created_by = "external:advisor/wsl2"
watchers = ["external:advisor/wsl2"]
ticket = "qwd4"
+++

docs/tickets/open/project-preflight-let-the-human-name-the-clone-s-path-on-the-qwd4.md

## Thread

### note · agent:pm-1 · 2026-10-14T22:09:41.780Z
Brief (pm-1). Model: Sonnet. Size: S. The ticket body is the spec; the human said "yes, ready and get the --path". Must have: `bridle project preflight --to M --path <clone path on M>` (~ expanded on M, not locally); the clone prompt, the clone check and the git push probe all use it. Code: crates/bridle/src/preflight_remote.rs ~385 (remote path built from the local one) and the preflight args in cli.rs. Nice to have (small, do it only if it adds few lines): with no --path, when OLD's path is outside $HOME and no clone exists on NEW, default to the documented ~/work/<P>-workspace/<P> (docs/context/add-a-machine.md "Workspace layout"), and say the path used in the pass/fail line. Tests: with --path, the clone command, check and push probe use it (fake Runner, as the 127/255 tests from br-sh8y); default case for a path outside $HOME. Docs: docs/design/cli.md (project preflight), docs/context/move-a-project.md step 1, CHANGELOG.md. No migration. Acceptance: just check. Out of scope: moving the project itself.
