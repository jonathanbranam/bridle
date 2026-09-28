+++
id = "br-7bf8"
title = "The TUI inbox doesn't scroll"
kind = "bug"
state = "dropped"
created_at = "2026-09-28T15:03:27.767Z"
updated_at = "2026-09-28T23:51:59.879799Z"
+++

tickets: docs/questions/open/tui-inbox-doesnt-scroll-8ups.md (8ups) and
docs/questions/open/tui-agents-panel-doesnt-scroll-yurx.md (yurx); one task, per the human
("those could be fixed together ... something to do when we're low on budget because they
should be small"). br-db2b (yurx) is folded in here.

Bug: the TUI inbox and the agents panel don't scroll. `draw_inbox` (and the agents panel
draw fn) in crates/bridle-tui/src/ui.rs render a table with `render_widget` and highlight
the selected row by style; no ratatui TableState/offset is kept, so `j`/`k` move the
highlight but the view always starts at row 1 and a selection past the last visible row
can't be seen (the human's inbox held 160 unread messages).

Fix: keep a TableState (or an offset) per pane and render with `render_stateful_widget`, so
the view follows the selection. One shared approach for both panes; look at ui.rs and the
app state for any other list pane with the same flaw and fix it the same way only if it's
the same one-line change.

Acceptance: `just check` passes; a test (ratatui TestBackend or a state-level test) that
with more rows than fit, selecting the last row keeps it visible in both panes.

Out of scope: opening a message in full (fgu6/br-0e14), new panes or layout work (y496),
anything else in the TUI.

Size: small (filler for low budget). Model: Haiku if the change is as mechanical as it
looks; Sonnet if ui.rs state handling turns out tangled.

## Thread

### note · agent:pm-1 · 2026-09-28T23:51:59.879Z
dropped: Merged to main (07d929c).
