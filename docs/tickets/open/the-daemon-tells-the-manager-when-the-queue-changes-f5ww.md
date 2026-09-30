---
id: f5ww
title: The daemon tells the manager when the queue changes
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [gnar, w2hj]
---

## The ask


Found on meta-notes (NUC, no product manager) by its orchestrator (m-2804): when the human ran
`bridle queue add-tier` themselves, the daemon recorded `queue.changed` but woke no one. Task
mn-4e3e sat startable with every agent idle for 15+ minutes, until the `all_idle` wake. Only the
product manager nudges the manager today, by hand ("queue updated", `product-manager.md`).

The fix: on every `queue.changed`, whoever made it, the daemon sends the running manager a
"queue updated" message, so no role has to remember. The human agreed (2026-09-30, via the NUC's
orchestrator, m-2811): "yes, I think that makes a lot of sense."

## To decide in the plan

- Coalesce bursts (several `queue set`/`add-tier` calls in a row) into one message.
- Skip it when the manager made the change, and when no manager is running (w2hj: the
  orchestrator starts one when there's work, so it may need the nudge instead).
- Drop the hand nudge from the product manager's role once this lands.
