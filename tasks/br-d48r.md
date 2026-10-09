+++
id = "br-d48r"
title = "Machine load notes go to aide, not the orchestrator"
kind = "feature"
state = "dropped"
created_at = "2026-10-09T22:01:43.137Z"
updated_at = "2026-10-09T22:03:07.277726Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
ticket = "d48r"
+++

Ticket: docs/tickets/open/machine-load-notes-go-to-aide-not-the-orchestrator-d48r.md (read it: the human's words and the exact change list). Model: Haiku. Build exactly the ticket's "Change" list: recipient in crates/bridle-daemon/src/load.rs from the orchestrator to `external:aide` (the hard-coded ToTarget::External(crate::wake::ORCHESTRATOR)), tests in load.rs follow, workflow/base/roles/orchestrator.md (wake list and "machine load note" bullet), workflow/base/roles/aide.md (what to do with it), docs/design/agent-host/operating-model.md ("Load watch"), CHANGELOG.md.
Acceptance: just check passes; a test shows a crossing sends the note to external:aide and not to the orchestrator.
Migration: role text reaches projects via bridle workflow sync; daemon change on upgrade; nothing to hand-edit.
Touches the same file as br-g76s (load.rs, load-hold notes): if g76s is in flight, merge main first; do not do g76s's work.
Out of scope: hysteresis or a minimum gap between notes (deferred until aide is flooded the same way).

## Thread

### note · external:orchestrator · 2026-10-09T22:01:43.226Z
Ready on the human's go (2026-10-09 ~6:10 PM ET, to the orchestrator): "send the machine load messaging to aide and deal with it". Normal priority; pm-1 places it.

### note · external:orchestrator · 2026-10-09T22:01:43.297Z
From orchestrator: br-d48r filed and ready on the human's go: load notes to aide instead of the orchestrator. Small Haiku task, normal priority; place it in the queue (behind br-v6kr's 4 AM start).

### note · external:advisor/product-manager · 2026-10-09T22:01:56.876Z
watching the task

### note · external:orchestrator · 2026-10-09T22:03:07.277Z
dropped: Orchestrator misread the human. The human, 2026-10-09 ~6:15 PM ET: "I mean; tell aide to file a ticket to deal with that; seems like there isn't a quiet period for those notifications; they shouldn't come over and over and over." The recipient stays the orchestrator; aide files the real ticket (repeat notes, no quiet period).
