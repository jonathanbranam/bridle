---
id: yurx
title: The TUI agents panel doesn't scroll
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [8ups, y496]
closed: 2026-09-30T05:12:44Z
---

## What happened

The human, verbatim (2026-09-28): "Scrolling doesn't work in the Agents panel either. I can
arrow down past the last agent, and I can highlight the next one down, but the panel
doesn't scroll."

## Notes

- Same cause as [[tui-inbox-doesnt-scroll-8ups|the inbox]]: `draw_agents`
  (`crates/bridle-tui/src/ui.rs`) renders the table with `render_widget` and highlights
  `app.selected_agent` by row style, with no table state or offset, so the view always
  starts at the first row. One fix (a `TableState` for both, `render_stateful_widget`)
  likely covers both panels.
- Backburner ideas for the TUI from the same conversation:
  [[tui-panels-and-seeing-the-work-y496|panels and seeing the work]].

## Resolution

The renderer copied the `TableState` each frame, discarding the scroll offset. `App` now keeps the offset (`agents_offset`, `inbox_offset`) and `draw_agents`/`draw_inbox` restore and write it back (br-9e67).

Resolved 2026-09-29.
