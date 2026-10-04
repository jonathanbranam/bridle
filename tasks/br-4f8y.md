+++
id = "br-4f8y"
title = "agent rm fails when git no longer knows the agent's worktree (pruned record, directory left)"
kind = "bug"
state = "planned"
created_at = "2026-10-04T13:39:52.020Z"
updated_at = "2026-10-04T13:40:39.075598Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: 4f8y
Bug, small. Ticket: docs/tickets/open/agent-rm-fails-when-git-no-longer-knows-the-agent-s-worktree-4f8y.md (read it; the human's error output is there). 'bridle agent rm <name> --delete-branch' fails with 'git worktree remove ... is not a working tree' when git's worktree record was pruned but the directory (with a .git file pointing at a missing .git/worktrees/<name>) is left. Fix in crates/bridle-daemon/src/worktree.rs (the remove path): when the path isn't a registered worktree, remove the leftover directory (refuse without --force if it holds files git can't account for, as the ticket says; keep it simple), then carry on removing the agent and the (possibly missing) branch without erroring. Tests: pruned-record worktree dir removed; a missing branch doesn't error; unaccounted files refuse without --force. Docs: CHANGELOG, and the rm text in cli.md if it describes this. Touches worktree.rs like br-x3xk: land after it, rebase. Acceptance: just check passes. Model: Sonnet (Haiku if mechanical). Migration: none.
