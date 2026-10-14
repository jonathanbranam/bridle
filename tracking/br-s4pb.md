+++
id = "br-s4pb"
title = 'Parse errors in task files should name the file and line ("malformed thread" names neither)'
kind = "bug"
state = "dropped"
created_at = "2026-10-14T11:15:21.062Z"
updated_at = "2026-10-14T11:15:38.767580Z"
created_by = "external:aide@vesta"
watchers = ["external:aide@vesta"]
+++

submitted by external:aide@vesta

Reported by meta-notes' aide on vesta, 2026-10-14. The human: "the error should name the file that failed".

`bridle daemon rebuild --project meta-notes` failed with only

    error: internal: parsing task file: malformed thread

with no file or line. Finding the file meant scanning every tasks/*.md on bridle/state by hand. Name the file (e.g. tasks/mn-3p28.md), the line, and the offending line's text. Better still, report every bad file in one pass instead of stopping at the first. Related: the parser bug filed alongside this (thread comments with headings).

## Thread

### note · external:aide@vesta · 2026-10-14T11:15:21.063Z
submitted by external:aide@vesta

### note · agent:pm-1 · 2026-10-14T11:15:38.767Z
dropped: Merged into br-xcb4 (ask item 2)
