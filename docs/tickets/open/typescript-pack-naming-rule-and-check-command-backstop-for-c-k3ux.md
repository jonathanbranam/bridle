---
id: br-k3ux
title: "Typescript pack: naming rule and check-command backstop for case-colliding paths (slice 2 of 4fpk)"
kind: feature
created: 2026-11-09
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-4fpk]
related: [br-4fpk]
---

## The ask

Slice 2 of br-4fpk (read its ask and brief; the human wants this for meta-notes-ui, his at-work project, and said so directly on 2026-11-09). Blocked-by br-4fpk (same rules/hooks files). Model: Sonnet.

Build: in the typescript pack (workflow/packs, find the typescript or web pack; add one if none exists only if the pack layout makes it a small step) add a naming rule (PascalCase.tsx for components, kebab-case.ts for the rest, no two paths equal when lowercased with the extension stripped) and a pack-provided check, run by the project's check command, that fails on any two tracked paths that collide case-insensitively. It is the backstop for files written other ways than the Write tool. Reuse the collision logic from slice 1 (`bridle case-guard`), exposed as a subcommand that scans `git ls-files`, rather than a second copy. Look at mu-bgb6's project-local version in meta-notes-ui (a case-collision check in `npm run check`) as the starting point but do not edit that project: it is the human's existing project (rule existing-projects).

Acceptance: a test with a repo containing timelog.ts and TimeLog.tsx fails the check, and one without passes; the rule appears in `bridle prime` for projects using the pack; `just check`. Docs and CHANGELOG; existing projects get the rule when they enable the pack, say so. Out of scope: renaming any project's files; any change to meta-notes-ui.
