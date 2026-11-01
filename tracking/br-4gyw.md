+++
id = "br-4gyw"
title = "Settle period counts from filing, not from ticket ready"
kind = "bug"
state = "claimed"
created_at = "2026-11-01T12:37:50.529Z"
updated_at = "2026-11-01T13:51:00.991739Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w4gyw",
]
ticket = "4gyw"
+++

docs/tickets/open/settle-period-counts-from-filing-not-from-ticket-ready-4gyw.md

submitted by external:orchestrator

## The ask

The `[tasks] settle` hold should start (or restart) when a ticket is readied (`bridle ticket ready`, pending -> open), not when it was filed. Today a ticket filed days ago is plannable the instant it is readied, so the human's follow-up edits to the ask miss the hold.

## Seen

glassbox gl-hr6t (bridle v0.7.6, `settle = "5m"`):
- filed 2026-10-31 12:08 UTC
- readied 2026-11-01 ~12:33 UTC
- the manager ran `bridle ticket plan` at once, with no settle error
- the human changed the ask at 12:34 and 12:35 UTC

Tickets readied seconds after filing (gl-q57j, gl-fpcu) held correctly, which fits a hold that counts from filing.

## Want

The settle clock starts at ready (pending -> open), or restarts there. `bridle ticket plan` refuses until `settle` has passed since ready.

## Approval

The human, 2026-11-01 via advisor (relayed by orchestrator@remus, m-0557), verbatim: "ok, there should be a 5 min wait on all tickets still. tell orch; that is important"

Interim: glassbox and remus-home carry a project rule settle-after-ready (managers wait 5 minutes after ready); orchestrator@remus removes it when this ships.

## Verify

A test: file a ticket, back-date its filing past `settle`, ready it, and check that `ticket plan` is refused until `settle` has passed since ready.

## Thread

### note · external:orchestrator · 2026-11-01T12:37:50.530Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-11-01T12:38:19.316Z
Readied by orchestrator@dalek: bug kind, the human called it important (quote in the body).

### note · external:orchestrator · 2026-11-01T12:38:48.141Z
The human, 2026-11-01 ~7:40 AM ET, via orchestrator@remus (m-0559), verbatim: "I want a 5 min wait on all tickets. I frequently have additions and agents will write tickets quickly, which is a great behavior. I don't want tickets to be stuck waiting for my approval if I walk away from the computer. ... The 5 min settle time allows me to amend a ticket before it starts being written, but for the ticket to move ahead without stalling for me."
