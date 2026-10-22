---
id: br-gakc
title: "Agent-filed and linked tickets sit uncommitted: commit ticket new/link/set writes, or flag them to the orchestrator"
kind: bug
created: 2026-10-22
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-6gdp]
---

## The ask

Part 2 of 3 of the orchestrator's finding (br-6gdp; the human, 2026-10-21: "Why do we have files which aren't committed? that is a red flag that something is being lost somewhere").

`bridle ticket new/link/set` by an agent writes a ticket file or frontmatter in the main clone and leaves it uncommitted, and nothing tells anyone. Seen on bridle: `blocked_by` edits on dkqf, cgw3, remg, q7my, df8w, r8jf (from the PM's `ticket link`) sat uncommitted; br-dfy7 (filed by manager-2, row later dropped) was never committed and is still an untracked file in docs/tickets/open/; br-t7uq waited for the orchestrator to commit it (the PM cannot commit, and `ticket ready` refuses an uncommitted file).

Fix (the fixer decides, keep it small): the daemon commits ticket files its own commands write (new, link, set, edit) on the integration branch in the owner's clone, as it already commits state elsewhere; OR a daemon check flags uncommitted ticket files to the orchestrator (an incident-free note). Prefer the commit: it also removes the PM's "please commit it" round-trip. Respect the one-pusher rule: commit only, never push from a non-owner clone.

Also: br-dfy7's stray file: commit it as dropped or delete it (the fixer's call; note it on br-dfy7).

Acceptance: `ticket new/link/set` by an agent leaves no uncommitted ticket file; `ticket ready` right after `ticket new` by the same agent works; test in a temp repo. `just check`. Docs: docs/design/cli.md, rule `tickets` text if it says agents cannot commit. Model: Sonnet.
