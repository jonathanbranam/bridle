---
id: m2pz
title: "bridle-ui: lists that grow without limit aren't dropdowns, and UI design principles from a couple of books"
kind: feature
opened: 2026-10-06
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [g49c, ukpm]
tasks: [ui-m2pz]
---

## The ask

The human, 2026-10-05 late evening ET, via bridle-ui's aide (m-0437), verbatim: "something that grows infinitely should not be a dropdown. It's just a wrong design" (about the document page's dropdown, now about 100 items), and "We can work on UI design principles. We can get an agent to read a couple books and write a better role."

Ticket only for now: the human said later, not now.

## To do, when scheduled

- Replace the document dropdown with a control that scales (search with results, a filterable list or a tree by folder).
- UI design principles for bridle-ui and the web and mobile packs: an agent reads a couple of established UI and UX books and writes a better UI role or rules. First rule: anything that grows without limit is never a dropdown. Fits the designer role (ukpm) and the packs (g49c).

Also said in the same message (for the orchestrator, not this ticket's scope): "I just feel like sending out tons and tons of work doesn't really get shipped. I can't figure out why certain things get shipped and others don't."

## Next step: research and prototypes (the human, 2026-10-05 ~9:45 PM ET, verbatim)

"Actually, what would be good for that ticket would be to do some research and build a couple of prototypes for me. I know we've talked about a prototype role, and I don't know how that would work, but we could talk through that. It would need to do some research and build. I want a clickable HTML prototype. In this case, it doesn't need any backend or anything, but I need to be able to run it somehow. Ideally, it would be served on a port I could open on the Tailscale."

- Research first: established guidance on picking from a long, growing list (search, filter, browse by folder, recent items), with sources.
- Then two or three clickable, static HTML prototypes of the document page's picker, each a genuinely different approach, with fake data around 100 to 1000 documents in folders. No backend.
- Served on a port the human can open over Tailscale (bound to all interfaces, not 127.0.0.1), still running after the agent's turn ends, with the URLs reported.
- Uses the base prototyper role (workflow/base/roles/prototyper.md). That role says build only from the prompt; the research step here is an explicit part of the prompt.
