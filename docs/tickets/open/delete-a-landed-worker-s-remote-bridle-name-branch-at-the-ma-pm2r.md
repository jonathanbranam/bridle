---
id: br-pm2r
title: Delete a landed worker's remote bridle/<name> branch at the manager's cleanup step (from t7ya, part 2)
kind: feature
created: 2026-10-19
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-t7ya]
related: [br-t7ya]
---

## The ask

Part 2 of br-t7ya (read it for the human's words and approval: "pushing branches to remote and cleaning them up afterwards", no pull requests). Part 1 (workers push their own branch) is br-t7ya itself and must land first.

At the same cleanup step that removes the local `bridle/<name>` branch after a landing (grep "cleanup: removed agent" in crates/bridle-daemon), also delete the remote branch on origin: only `bridle/*`, only when the ticket is integrated or dropped and the branch's commits are on the integration branch; a plain `git push origin --delete`, never anything else; a failure (no remote, no permission, already gone) is logged and never fails the cleanup. Which agent runs the delete follows part 1's decision on who may push. Model: Sonnet. Tests with a local bare repo as origin: a landed branch is deleted on origin; an unmerged branch, a non-bridle branch and the integration branch are never deleted. Docs: operating-model.md, CHANGELOG. Acceptance: just check. Out of scope: pull requests; local leftover branches (br-wnuf).
