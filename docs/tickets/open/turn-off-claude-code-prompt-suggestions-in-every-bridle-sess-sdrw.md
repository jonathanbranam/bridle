---
id: sdrw
title: Turn off Claude Code prompt suggestions in every bridle session
kind: chore
opened: 2026-10-07
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

Every Claude Code session bridle launches (`bridle session ...` and spawned agents) should run with prompt suggestions off (`"promptSuggestionEnabled": false`, the "Prompt suggestions" toggle in `/config`).

The human, 2026-10-06 ~11 PM ET: "file a ticket to turn that off for all bridle sessions. It causes a big problem when somebody tries to look at the tmux pane, and without reading the ASCII color codes, it looks like there's something I just typed in there. I went ahead and added it myself to my own fig, but I want it as part of bridle"

Why: a suggestion shows as greyed text on the input line. Anyone reading the tmux pane (the human, or bridle's own pane checks, e.g. "pane is running ..., not typing into it") without the colour codes sees what looks like typed input.

Done when: bridle sets it in the settings it passes to every session and agent it launches, so it holds without the human's own config, and it is checked by a test.
