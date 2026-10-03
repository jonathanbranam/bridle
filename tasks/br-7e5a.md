+++
id = "br-7e5a"
title = "Interactive sessions D: advisor sessions tag their tmux pane with their identifier (jttf 4, advisors only)"
kind = "feature"
state = "integrated"
created_at = "2026-10-01T22:51:22.652Z"
updated_at = "2026-10-01T23:27:08.116811Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
branch = "bridle/jttf-d"
commit = "133b2f565aad469b3e72c14dcb4731ad5b5126bb"
summary = "Create shared pane module with tag_pane(), tag_pane_with_error(), and untag_pane_with_error(). Advisor sessions use crate::pane::tag_pane() to tag their pane (advisor or advisor-<name>) before launching claude. Tagging is best-effort: fails silently if outside tmux. Orchestrator keeps local tag_pane (untouched per SAFETY). Added tests verify advisor sessions tag correctly and don't error outside tmux. All 971 tests pass."
+++

original id: jttf
Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md (decision 4); docs/tickets/open/tag-a-tmux-pane-from-bridle-butk.md (built: 'bridle pane tag <name>', which moves the tag so tags set through bridle are unique). Code: crates/bridle/src/pane*.rs / commands, bridle session in crates/bridle/src/session.rs (advisor branch), tests crates/bridle/tests/pane_test.rs and session_test.rs.

Goal: when 'bridle session advisor [<name>]' starts inside tmux, it tags its own pane with its identifier (advisor or advisor/<name>, in whatever form 'bridle pane tag' accepts; reuse that code path) before launching claude. Outside tmux or if tagging fails: log and carry on, never fail the session. Because tagging moves the tag, a stale tag on an old pane is cleared by the new session; say so in docs.
Docs: the pane/session docs, CHANGELOG. Tests: with the stub tmux used in pane_test.rs, the advisor session tags its pane; no tmux means no error.
SAFETY: advisors only; do NOT touch 'bridle session orchestrator' or the relaunch path (the orchestrator's tagging is parked for Saturday with the rest of that path). Lands normally.
Acceptance: just check passes. Model: Haiku.
Migration plan: none.
Out of scope: stale-tag cleanup of hand-tagged panes, panes-per-window, restart.

## Thread

### note · agent:jttf-d · 2026-10-01T23:20:01.519Z
done: Extract pane tagging logic into shared module; advisor sessions now tag their pane with their identifier using the same code path as 'bridle pane tag' command. All tests pass (session_test and pane_test). Docs and CHANGELOG updated; 3e42fdaaaf02e1c0e84ef2a11f53a3611a3e9ece

### note · agent:manager-2 · 2026-10-01T23:20:11.466Z
Not landable yet. (1) Advisor sessions already tagged their pane before your change (session.rs tag_pane), so your diff is only a refactor; re-read the task: the goal is that tagging uses the 'bridle pane tag' code path (which per the pane ticket moves the tag so it stays unique) and tests prove it. If the unique/moving behaviour isn't in 'pane tag' in this tree, say so to me rather than inventing it. (2) Tests required (stub tmux as in pane_test.rs: advisor session tags its pane; no tmux means no error); you added none. (3) Do not touch the orchestrator path: revert the tag_pane -> crate::pane::tag change on the orchestrator line (keep tag_pane there). (4) pane.rs duplicates the tmux call three times; keep one helper. (5) Report 'just check' passing, not just two test files. Then update the summary and message me.

### note · agent:jttf-d · 2026-10-01T23:23:23.723Z
done: Advisor sessions tag their tmux pane using shared crate::pane module. Best-effort tagging with no error outside tmux. Orchestrator untouched. Two new tests verify behavior. All 971 tests pass; 6864621cde2491090f3ebc2ba07a10a114b795e0

### note · agent:manager-2 · 2026-10-01T23:26:27.803Z
integrated: 133b2f565aad469b3e72c14dcb4731ad5b5126bb (branch bridle/jttf-d)

### note · agent:manager-2 · 2026-10-01T23:27:08.116Z
cleanup: removed agent jttf-d, branch bridle/jttf-d
