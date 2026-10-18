+++
id = "br-ewhy"
title = "Stages MVP 4: bridle spawns a designer for each unblocked stage; timer to the product manager"
kind = "feature"
state = "planned"
created_at = "2026-10-17T21:41:29.528Z"
updated_at = "2026-10-18T02:30:17.435041Z"
created_by = "external:advisor/tj7q"
watchers = ["external:advisor/tj7q"]
priority = "high"
priority_at = "2026-10-17T22:04:05.788409Z"
ticket = "ewhy"
+++

docs/tickets/open/stages-mvp-4-bridle-spawns-a-designer-for-each-unblocked-sta-ewhy.md

## Thread

### note · external:advisor/tj7q · 2026-10-17T22:03:57.964Z
The human, 2026-10-17 ~5:15 PM ET, to advisor (tj7q): "Yes approved. Let's go" (shared understanding of epic tj7q, and ready steps 1-5 so the build can start today; the human wants the MVP done today to use tomorrow). Readied by human via advisor (tj7q).

### note · external:advisor/tj7q · 2026-10-17T22:04:05.788Z
priority: normal -> high

### note · external:advisor/tj7q · 2026-10-18T02:30:17.435Z
advisor (tj7q), for whoever builds this: ussh's worker reports (dd5a6f8f) that "claimed by human" is a `claimed_by` frontmatter line, because row claims need `planned` and a version 2 ticket preparing is `pending`; and that 9mqh's `ticket new --claim` likely conflicts for the same reason. This step spawns only on an unclaimed stage, so it must read the same claim that `--claim` and the human's review write. Check that `--claim` and `claimed_by` agree before relying on either.
