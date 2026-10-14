+++
id = "br-kf7t"
title = "project start refuses a brand-new project (no owner), and take-over refuses it too (no bridle/state on origin)"
kind = "bug"
state = "planned"
created_at = "2026-10-14T17:50:53.536Z"
updated_at = "2026-10-14T17:51:46.402074Z"
created_by = "external:advisor/wsl2"
watchers = ["external:advisor/wsl2"]
ticket = "kf7t"
+++

docs/tickets/open/project-start-refuses-a-brand-new-project-no-owner-and-take-kf7t.md

## Thread

### note · agent:pm-1 · 2026-10-14T17:51:46.055Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body (What's wanted, Why, file pointers) is the spec; it blocks the home-project setup on remus (br-8uxe), so keep it near the top. Files: crates/bridle/src/project_start.rs (owner check ~339; br-sh8y also edits this file, so this runs after sh8y), crates/bridle-daemon/src/state_branch.rs (TakeOverError::NotFetched ~918: split "branch absent on origin" from "origin unreachable"; check with ls-remote), crates/bridle-daemon/src/lib.rs (serve's first-run creation ~561-572: reuse it, don't duplicate), docs/design/cli.md, docs/context/adding-a-project.md, CHANGELOG.md. Design: start on a project with no bridle/state anywhere (local none, origin reachable and has none) creates the branch and claims this machine as owner exactly as serve does, then installs and starts as usual; still refuses when bridle/state exists on origin with another owner, or origin is unreachable. take-over on a project with no bridle/state on origin says "new project: run `bridle project start`". Never bypass the one-pusher hook; the new branch is pushed by the owner only. Tests: new project start succeeds (local bare repo as origin); existing other owner still refused; take-over message for absent branch vs unreachable origin. Acceptance: just check. No migration. Out of scope: workers/config for home projects (8uxe).
