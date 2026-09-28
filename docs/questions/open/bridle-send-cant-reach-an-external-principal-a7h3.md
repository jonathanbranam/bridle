---
id: a7h3
title: bridle send can't reach an external principal
opened: 2026-09-27
repos: [bridle]
changes: []
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
