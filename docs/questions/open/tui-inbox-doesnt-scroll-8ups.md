---
id: 8ups
title: The TUI inbox doesn't scroll
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [fgu6]
---

## What happened

The human, verbatim (2026-09-28): "File a bug - the tui Inbox doesn't seem to scroll; not
urgent".

At the time the human's inbox held 160 unread messages (150 notes, 10 questions), far more
than the inbox pane shows.

## Notes

- `draw_inbox` (`crates/bridle-tui/src/ui.rs`) renders the table with `render_widget` and
  highlights `app.selected_message` by row style. No table state or offset is kept, so
  `j`/`k` move the highlight but the view always starts at the first row; once the
  selection passes the pane's last visible row it can't be seen.
- Related: [[tui-inbox-open-a-message-in-full-fgu6|reading a message in full]].
