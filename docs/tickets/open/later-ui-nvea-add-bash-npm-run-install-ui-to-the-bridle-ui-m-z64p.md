---
id: br-z64p
title: "[later] ui-nvea: add Bash(npm run install-ui) to the bridle-ui manager's allowed_tools (you approved; agents may not widen permissions)"
kind: chore
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human approved ui-nvea, 2026-10-17 ~10:15 PM ET ("Agree. Fix later"). The orchestrator readied it, but auto mode refuses the edit to any agent because it widens permissions.

1. In /Volumes/Data/work/bridle-ui-workspace/bridle-ui/.bridle/config.toml, line 56 (the manager role's allowed_tools), add "Bash(npm run install-ui)", after "Bash(gh run *)", .
2. Commit it in bridle-ui.
3. bridle daemon restart --project bridle-ui

Until then the orchestrator runs install-ui by hand after bridle-ui landings.
