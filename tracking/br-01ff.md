+++
id = "br-01ff"
title = "Tickets through the bridle binary in every project: new, frontmatter, check, resolve; add closed: (7gk7)"
kind = "feature"
state = "pending"
created_at = "2026-09-30T19:48:27.574Z"
updated_at = "2026-10-11T21:30:44.570677Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
size = "M"
ticket = "pq79"
+++



## Thread

### note · system · 2026-10-05T15:24:32.139Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.

### note · external:advisor/product-manager · 2026-10-09T11:04:36.829Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T13:34:51.118Z
Assessment (the human asked, 2026-10-11; details in the ticket, "Assessment (2026-10-11)" and "What is left"):

The code ask is done: ticket new/resolve (br-1e47), set/check and the tickets rule for every project (br-ae32), cleanup (br-9e4e), then reshaped by 22ab (syqn field renames, qualified IDs, ticket new making the row, ticket ready, ticket link). The `closed:` ask is done as `resolved:` (syqn renamed closed -> resolved; `ticket resolve` stamps it, `closed:` still read). Other projects read the rule from bridle's workflow/; `ticket check` passes in bridle-ui and track-web.

Left, a docs chore (size S): (1) docs/README.md still cites workflow-instructions/ticket-conventions.md and the check-tickets.py loop, a stale questions/ tree, and says resolving a ticket closes its task (it doesn't); (2) docs/spikes/open and done are ticket-shaped but outside the binary (old fields, bare IDs, done/ not resolved/); recommended: move them to docs/tickets as kind research, or stop calling spikes tickets. The human picks. Other projects' old field names wait on their owners running migration 0002 (xebc), not this task.

Title no longer fits; proposed in the report to the advisor.

advisor (product-manager) via subagent
