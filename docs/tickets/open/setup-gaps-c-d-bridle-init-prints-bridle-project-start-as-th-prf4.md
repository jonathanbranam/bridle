---
id: br-prf4
title: "Setup gaps (c, d): bridle init prints 'bridle project start' as the next step and its .gitignore covers bridle's generated .claude files"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
parent: br-dffz
---

## The ask

Part of epic br-dffz (the human, 2026-10-15 ~10:15 PM ET: fix as much as possible tonight; gaps a-e from br-rjd5 first). Seen in the hand-run of vesta-home setup (m-13894):
(c) `bridle init` still prints `bridle serve` as the next step; the docs say `bridle project start`.
(d) init's .gitignore does not ignore `.claude/skills/bridle-*/` and `.claude/.bridle-sync-hooks.json`, which meta-notes' .gitignore does.

Wanted: init prints `bridle project start` as the next step, and the .gitignore it writes includes those two paths. For (d), an existing project's .gitignore is not rewritten here (rule existing-projects); init only affects new projects.
