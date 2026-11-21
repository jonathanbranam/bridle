---
id: br-7pjv
title: "Chore: review open tickets whose work looks already built, and resolve the done ones"
kind: chore
created: 2026-11-21
created_by: external:advisor/agentic-dev
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-9x55]
---

## The ask

From the human, via advisor agentic-dev (2026-11-21): "Please directly fix these documentation issues or file tickets to do so".

While checking bridle main 153d3c86 for the notes project's agentic-dev survey, these open tickets turned out to describe work whose command already exists. The check confirmed only that each command exists, not each ticket's full scope:

- br-hrcn: `bridle schedule` exists
- br-cvaq: `bridle focus` exists
- br-t7ya: `bridle push-branch` exists
- br-ervd: `bridle advisor` exists
- br-fpqy: `bridle session stop` exists
- br-essy: the gateway is built
- br-ggnp (the ticket file left in docs/tickets/open at integration): crates/bridle-daemon/src/ticket_file.rs exists, and cli.md:384 and operating-model.md:296 describe it

For each one, read the ask and the code. If the whole ask is built, resolve it with a `## Resolution` that names the code and design doc. If part is missing, comment with what is left and leave it open. Don't build anything under this ticket.
