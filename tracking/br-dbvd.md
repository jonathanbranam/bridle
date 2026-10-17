+++
id = "br-dbvd"
title = "Specs: bridle spec id duplicates ledger entries for hand-written IDs"
kind = "fix"
state = "planned"
created_at = "2026-10-05T02:56:15.131Z"
updated_at = "2026-10-17T15:27:02.682309Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
ticket = "dbvd"
+++

docs/tickets/open/specs-bridle-spec-id-duplicates-ledger-entries-for-hand-writ-dbvd.md

submitted by external:orchestrator@nuc

Running `bridle spec id` on a spec file that already held IDs the worker had written by hand added duplicate entries to the .ids ledger. The worker reset .ids and re-ran. Expected: spec id adopts existing IDs into the ledger (or refuses with a clear error), never duplicates them.

From the meta-notes-ui project (orchestrator, 2026-10-05), spec 3 (mu-pfgp, design/specs/rendering.md). Local log: meta-notes-ui ticket myeg (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:56:15.132Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T02:56:34.784Z
Triage (pm-1): accept. A real bug from meta-notes-ui. Ticket minted: docs/tickets/open/specs-bridle-spec-id-duplicates-ledger-entries-for-hand-writ-dbvd.md (uncommitted; needs committing on main). Stays pending: approve with `bridle task ready br-dbvd`, then I plan it (small; Sonnet, crates/bridle-spec; Haiku may do).

### note · external:advisor/product-manager · 2026-10-09T11:04:38.516Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T07:22:56.703Z
kind: bug -> fix

### note · external:advisor/product-manager · 2026-10-17T15:26:06.084Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.
