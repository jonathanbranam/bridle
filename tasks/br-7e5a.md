+++
id = "br-7e5a"
title = "Interactive sessions D: advisor sessions tag their tmux pane with their identifier (jttf 4, advisors only)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T22:51:22.652Z"
updated_at = "2026-10-01T22:51:25.267836Z"
size = "S"
+++

original id: jttf
Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md (decision 4); docs/tickets/open/tag-a-tmux-pane-from-bridle-butk.md (built: 'bridle pane tag <name>', which moves the tag so tags set through bridle are unique). Code: crates/bridle/src/pane*.rs / commands, bridle session in crates/bridle/src/session.rs (advisor branch), tests crates/bridle/tests/pane_test.rs and session_test.rs.

Goal: when 'bridle session advisor [<name>]' starts inside tmux, it tags its own pane with its identifier (advisor or advisor/<name>, in whatever form 'bridle pane tag' accepts; reuse that code path) before launching claude. Outside tmux or if tagging fails: log and carry on, never fail the session. Because tagging moves the tag, a stale tag on an old pane is cleared by the new session; say so in docs.
Docs: the pane/session docs, CHANGELOG. Tests: with the stub tmux used in pane_test.rs, the advisor session tags its pane; no tmux means no error.
SAFETY: advisors only; do NOT touch 'bridle session orchestrator' or the relaunch path (the orchestrator's tagging is parked for Saturday with the rest of that path). Lands normally.
Acceptance: just check passes. Model: Haiku.
Migration plan: none.
Out of scope: stale-tag cleanup of hand-tagged panes, panes-per-window, restart.
