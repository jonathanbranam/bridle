+++
id = "br-kf7t"
title = "project start refuses a brand-new project (no owner), and take-over refuses it too (no bridle/state on origin)"
kind = "bug"
state = "integrated"
created_at = "2026-10-14T17:50:53.536Z"
updated_at = "2026-10-14T20:50:09.852193Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:kf7t",
]
branch = "bridle/kf7t"
commit = "8a5847e75bb89c940480d1df123351b052b691b1"
summary = """project start now works on a brand-new project: when bridle/state is absent locally and a reachable origin lacks it (new state_branch::is_new_project, via ls-remote), the owner check passes and the daemon's first run (serve) creates the branch and claims the owner, reusing serve's existing path. Other owner or unreachable origin still refuse. take-over on that case now errors with TakeOverError::NewProject ("new project: run `bridle project start`"); the fetch step distinguishes absent branch from unreachable origin with ls-remote (origin_has_state_branch). Docs: add-a-machine home-project steps (serve workaround removed), cli.md, CHANGELOG. Tests extend state_branch take-over test (absent vs unreachable, is_new_project); project_start itself is not unit-tested end to end. just check exit 0, 1680 tests."""
ticket = "kf7t"
+++

docs/tickets/open/project-start-refuses-a-brand-new-project-no-owner-and-take-kf7t.md

## Thread

### note · agent:pm-1 · 2026-10-14T17:51:46.055Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body (What's wanted, Why, file pointers) is the spec; it blocks the home-project setup on remus (br-8uxe), so keep it near the top. Files: crates/bridle/src/project_start.rs (owner check ~339; br-sh8y also edits this file, so this runs after sh8y), crates/bridle-daemon/src/state_branch.rs (TakeOverError::NotFetched ~918: split "branch absent on origin" from "origin unreachable"; check with ls-remote), crates/bridle-daemon/src/lib.rs (serve's first-run creation ~561-572: reuse it, don't duplicate), docs/design/cli.md, docs/context/adding-a-project.md, CHANGELOG.md. Design: start on a project with no bridle/state anywhere (local none, origin reachable and has none) creates the branch and claims this machine as owner exactly as serve does, then installs and starts as usual; still refuses when bridle/state exists on origin with another owner, or origin is unreachable. take-over on a project with no bridle/state on origin says "new project: run `bridle project start`". Never bypass the one-pusher hook; the new branch is pushed by the owner only. Tests: new project start succeeds (local bare repo as origin); existing other owner still refused; take-over message for absent branch vs unreachable origin. Acceptance: just check. No migration. Out of scope: workers/config for home projects (8uxe).

### note · agent:kf7t · 2026-10-14T20:38:06.794Z
done: project start works on a brand-new project, take-over says 'new project: run bridle project start'; just check exit 0, 1680 tests passed, on merged tip e88c91cb (main merged in). Summary written; please mark built.

### note · agent:manager-2 · 2026-10-14T20:42:05.207Z
built: e88c91cb (branch bridle/kf7t)

### note · agent:manager-2 · 2026-10-14T20:49:34.021Z
integrated: 8a5847e75bb89c940480d1df123351b052b691b1 (branch bridle/kf7t)

### note · agent:manager-2 · 2026-10-14T20:50:09.852Z
cleanup: removed agent kf7t, branch bridle/kf7t
