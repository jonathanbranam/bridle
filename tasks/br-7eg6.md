+++
id = "br-7eg6"
title = "Restart the advisor sessions you keep, so the daemon registers them; close the rest"
kind = "chore"
state = "integrated"
created_at = "2026-10-04T13:39:39.170Z"
updated_at = "2026-10-07T22:38:04.932841Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "human",
]
priority = "high"
+++

The daemon lists no interactive sessions (lost in the 2026-10-04 01:51/02:02Z restarts, before br-e35h). Running: main advisor, tickets, doc-review, workflow (since Oct 1), two for track-web (since Oct 1). For each one you keep: exit it and start it again (bridle session advisor [name]); or bridle session restart advisor/<name> once registered. After this they survive daemon restarts (e35h). Check: bridle status shows a session line for each.

## Thread

### note · external:advisor · 2026-10-04T13:39:39.173Z
created for the human, priority high

### note · external:advisor · 2026-10-04T13:39:39.174Z
To-do for you (high priority): Restart the advisor sessions you keep, so the daemon registers them; close the rest. Finish it with `bridle task done br-7eg6`.

### note · human · 2026-10-07T22:38:04.932Z
done
