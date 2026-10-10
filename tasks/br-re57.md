+++
id = "br-re57"
title = "Benchmark design pass (designer): options and a recommendation in ticket v6kr"
kind = "research"
state = "open"
created_at = "2026-10-10T13:47:41.662Z"
updated_at = "2026-10-10T13:47:43.573204Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
]
size = "M"
parent = "br-v6kr"
+++

Role: designer (workflow/base/roles/designer.md). Ticket: docs/tickets/open/a-system-architect-role-and-measuring-bridle-s-own-resource-v6kr.md. Read all of it, especially '## The human's plan for the benchmark (2026-10-10)' and '## Plan (PdM, 2026-10-10)'. Write '## Design options' into the ticket covering the human's seven points: idle vs. busy; no fixed scenarios yet; a log of what happened during the run (event log export?); the same script every time; script committed and merged before any run; shape and size of the output and where it lives; permanent retention (git, a state-like branch that can be purged without rewriting history, or a dated folder on dalek in the bridle workspace parent, later Dropbox). Also cover: the run interrupts no other work and survives daemon restarts and landings (incident br-vt9k; br-37r9), the measures in the ticket's 'Memory: which measure', and how long a run needs to be (the human: ~30 min idle is enough). Build nothing. Report on the task; the PdM sends the design to the human via the aide.
