---
id: br-r8jf
title: "Three small robustness fixes: pair-mint --replace-direct order, stop_check ps in async, sync never removes a [mail] entry"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Minor findings from the orchestrator's verification, one branch.

1. `pair-mint --replace-direct` revokes the old token before it mints, so a failed mint leaves no token. Mint first, revoke after success. Test the failed-mint case.
2. br-mt4v's `stop_check` runs `ps` synchronously inside the async drain. Wrap it in spawn_blocking (CLAUDE.md conventions).
3. After br-b97n, `projects sync` never removes a [mail] projects entry. Decide whether that is right (an entry for a project removed from [projects] stays forever): if it is a leak, remove entries whose project is gone and are not hand-added; if intended, say so in cli.md and close this item.

Acceptance: just check. Model: Sonnet. Out of scope: other token changes.
