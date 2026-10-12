---
id: br-wayb
title: "Ticket links from bridle link don't open: bare ids (and file ids with a different row id) 404 on the daemon"
kind: fix
created: 2026-10-12
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-12 ~12:05 AM ET (to advisor product-manager): "After the ticket to task change, ticket links you share don't open: http://dalek.tailbc91f5.ts.net:7878/p/bridle/tickets/7d62". Then of `.../tickets/10c0` and `.../tickets/btdn`: "No ticket 10c0", "neither of those open". And: "I have to take the id, then add br- to the front of it;"

## What the advisor found

- `bridle link btdn` (bare id) prints `.../p/bridle/tickets/btdn`. The UI calls the gateway's `/api/v1/projects/bridle/tickets/btdn`, the gateway passes the id to the daemon's `get_task` as is, and the daemon answers 404: it only finds project-qualified ids (`GET /v1/tasks/br-btdn` is 200, `/v1/tasks/btdn` is 404, checked on dalek's daemon, port 7401). The UI then shows "No ticket <id>".
- `bridle link br-btdn` prints `.../tickets/br-btdn`, which should open.
- A second case: a ticket whose file id differs from its row's id. 7d62's file is `br-7d62` but its row is `br-10c0` (made with `--from-task`), so `bridle link 7d62` points at an id the daemon has no row for, qualified or not.

## Fix (smallest)

1. `bridle link` always prints the row's project-qualified id: expand a bare id, and map a ticket file's id to its row's id when they differ.
2. The gateway's ticket detail (and the actions under `/v1/tickets/{id}`) expand a bare id to the project's prefix before asking the daemon, so links already sent (and hand-typed ones) open too.
3. Optional, if cheap: the detail looks up a ticket file id that has a different row id.

Acceptance: `.../tickets/btdn`, `.../tickets/br-btdn`, `.../tickets/7d62` and `.../tickets/br-10c0` all open the right ticket; a test for each in bridle-gateway; `just check` green.
