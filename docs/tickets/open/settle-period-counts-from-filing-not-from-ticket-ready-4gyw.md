---
id: br-4gyw
title: Settle period counts from filing, not from ticket ready
kind: bug
created: 2026-11-01
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-11-01 via advisor (relayed by orchestrator@remus, m-0557), verbatim: "ok, there should be a 5 min wait on all tickets still. tell orch; that is important". Rank it high: it is the human's explicit, "important" ask.

The `[tasks] settle` hold should start (or restart) when a ticket is readied (`bridle ticket ready`, pending -> open), not only when it was filed. Today a ticket filed days ago is plannable the instant it is readied, so the human's follow-up edits to the ask miss the hold.

Seen on glassbox gl-hr6t (bridle v0.7.6, `settle = "5m"`): filed 2026-10-31 12:08 UTC, readied 2026-11-01 ~12:33 UTC, the manager ran `bridle ticket plan` at once with no settle error, and the human changed the ask at 12:34 and 12:35 UTC. Tickets readied seconds after filing held correctly, which fits a hold that counts from filing only.

Fix: `settle_until` is also set to now + settle when the ticket is readied (pending -> open), in addition to the existing restarts (creation, a human comment or edit, a relay of the human's words). Find where settle_until is set (daemon store/tasks, `ticket ready` handler) and update docs/design (the tasks/settle section) and CHANGELOG. No migration: settle_until is derived at runtime from existing rows; say so in the CHANGELOG.

Acceptance: a test that files a ticket, back-dates its filing past `settle`, readies it, and checks `ticket plan` is refused until `settle` has passed since ready, and that a comment after ready restarts it; `just check`. Model: Sonnet. Out of scope: changing the default `settle` value; a to-do or a `--for-human` row (created planned) is not held by this. Interim: glassbox and remus-home carry a project rule settle-after-ready; orchestrator@remus removes it when this ships.
