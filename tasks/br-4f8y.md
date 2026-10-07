+++
id = "br-4f8y"
title = "agent rm fails when git no longer knows the agent's worktree (pruned record, directory left)"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T13:39:52.020Z"
updated_at = "2026-10-04T16:38:15.127793Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/rm-pruned-wt"
commit = "7d707f34e67e294c6b707392d315007fbdb2efa7"
summary = "worktree::remove now checks git's worktree list; an unregistered (pruned-record) directory is deleted directly, refusing without --force if anything besides the .git pointer is in it. New delete_branch_if_exists makes a missing branch a no-op in agent rm and remove_pair. Test, CHANGELOG, agents.md updated. Without --force a pruned full checkout still needs --force (no index to compare)."
ticket = "4f8y"
+++

Bug, small. Ticket: docs/tickets/open/agent-rm-fails-when-git-no-longer-knows-the-agent-s-worktree-4f8y.md (read it; the human's error output is there). 'bridle agent rm <name> --delete-branch' fails with 'git worktree remove ... is not a working tree' when git's worktree record was pruned but the directory (with a .git file pointing at a missing .git/worktrees/<name>) is left. Fix in crates/bridle-daemon/src/worktree.rs (the remove path): when the path isn't a registered worktree, remove the leftover directory (refuse without --force if it holds files git can't account for, as the ticket says; keep it simple), then carry on removing the agent and the (possibly missing) branch without erroring. Tests: pruned-record worktree dir removed; a missing branch doesn't error; unaccounted files refuse without --force. Docs: CHANGELOG, and the rm text in cli.md if it describes this. Touches worktree.rs like br-x3xk: land after it, rebase. Acceptance: just check passes. Model: Sonnet (Haiku if mechanical). Migration: none.

## Thread

### note · agent:rm-pruned-wt · 2026-10-04T16:35:11.165Z
done: agent rm handles pruned worktree records and missing branches; just check green (1155 tests; main merged after, docs-only); 1b7483a6

### note · agent:manager-2 · 2026-10-04T16:35:15.641Z
integrated: 7d707f34e67e294c6b707392d315007fbdb2efa7 (branch bridle/rm-pruned-wt)

### note · agent:manager-2 · 2026-10-04T16:38:15.127Z
cleanup: removed agent rm-pruned-wt, branch bridle/rm-pruned-wt
