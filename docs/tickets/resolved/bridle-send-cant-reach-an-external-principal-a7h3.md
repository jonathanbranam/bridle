---
id: a7h3
title: bridle send can't reach an external principal
opened: 2026-09-27
resolved: 2026-09-29
repos: [bridle]
changes: [34e6268, d998a2a, 939b289]
specs: []
needs: []
see: [hj4g]
---

## The question

`bridle send external:orchestrator ...` fails with "no such recipient":
the server resolves only `human` and stored agents (reported by `manager-2`,
message m-0258, 2026-09-27). So nothing can message the orchestrator, and
`bridle inbox` for an external principal is always empty. The manager falls
back to messaging `human`, which the orchestrator then reads on the human's
behalf.

Should external principals be addressable, with their own inbox, and how does
the orchestrator (or anything else outside bridle) get woken when a message
arrives for it?

## Resolution

Resolved by d998a2a (34e6268): `bridle send` and `bridle inbox` work for `external:<name>` principals, so the orchestrator has its own inbox; 939b289 has the advisor message the orchestrator directly rather than via the human's inbox. The answer lives in docs/design/agent-host/principals.md and docs/design/agent-host/messages.md.
