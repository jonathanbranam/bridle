+++
id = "br-b6e1"
title = "Pane tags are unique: tagging a pane clears the same tag from every other pane (butk gap)"
kind = "bug"
state = "planned"
created_at = "2026-10-01T23:20:47.516Z"
updated_at = "2026-10-01T23:20:49.522754Z"
size = "S"
+++

Ticket: docs/tickets/open/tag-a-tmux-pane-from-bridle-butk.md (lines ~29-32: 'Tagging a pane clears the same tag from any other pane ... It says so when it does (moved orchestrator from %39)'). The code falls short: 'bridle pane tag' (commands/workflow.rs pane(); shared helper from br-7e5a, session.rs tag_pane) only sets @bridle on the current pane, so a stale tag on an old pane leaves two candidates (the human hit this: jttf decision 4).
Goal: tagging a pane first clears that tag value from every other pane (tmux list-panes -a with the @bridle value, then set -p -u on the others), prints 'moved <tag> from <pane>' when it did, and both the CLI and the advisor session use the one helper. Failure to clear is a warning, never fails the tag. Do it in the helper br-7e5a leaves (rebase on it, or wait for it to land).
Tests with the stub tmux in crates/bridle/tests/pane_test.rs: second pane with the same tag loses it; message printed; no other pane: no message; list failure still tags.
Docs: butk ticket 'Built' note if missing, CHANGELOG. Acceptance: just check passes. Model: Haiku.
Migration plan: none. SAFETY: tmux tagging only, no start-up or relaunch change; lands normally. Out of scope: hand-tagged panes with other values, panes per window.
