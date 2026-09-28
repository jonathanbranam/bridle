---
id: fgu6
title: TUI inbox: open a message to read it in full
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [8ups]
---

## The ask

The human, verbatim (2026-09-28):

> File request - inbox tui I can't read the entire note body; should be ENTER to view the
> entire note and then r to reply or esc or enter to dismiss

## Notes

- The inbox table (`draw_inbox`, `crates/bridle-tui/src/ui.rs`) puts each message's
  body in a single-line cell, so long bodies are cut off. `r` already starts a reply to
  the selected message (`App::start_reply`, `crates/bridle-tui/src/app.rs`).
- Related: [[tui-inbox-doesnt-scroll-8ups|the inbox doesn't scroll]].
