+++
id = "br-wayb"
title = "Ticket links from bridle link don't open: bare ids (and file ids with a different row id) 404 on the daemon"
kind = "fix"
state = "planned"
created_at = "2026-10-12T03:59:35.068Z"
updated_at = "2026-10-12T04:00:32.768242Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-12T03:59:45.418153Z"
ticket = "wayb"
+++

docs/tickets/open/ticket-links-from-bridle-link-don-t-open-bare-ids-and-file-i-wayb.md

## Thread

### note · external:advisor/product-manager · 2026-10-12T03:59:45.418Z
priority: normal -> high

### note · agent:pm-1 · 2026-10-12T03:59:56.683Z
Build brief (pm-1): build the Fix list above, items 1 and 2, and item 3 only if it is a few lines. Model: Sonnet. Files likely: crates/bridle (the link command, the same code as bridle link from br-zqnp), crates/bridle-gateway (ticket detail and /v1/tickets/{id} actions), crates/bridle-docs or the daemon id-expansion helper if one exists (reuse it; do not write a second expander). Acceptance as written, with a gateway test per URL form, `just check` green. Docs: CHANGELOG, and the link section of cli.md if it names the form. Migration: none. Out of scope: bridle-ui source, renaming ids, changing how --from-task assigns ids.

### note · external:advisor/product-manager · 2026-10-12T04:00:32.768Z
From the human, via advisor (product-manager), 2026-10-12 ~12:10 AM ET: "I have filed a ticket that the UI should look up an id in projects and try to resolve it if it is unique across projects, but the link is broken; i think it was an oversight of the 'everything-is-a-ticket' work." And, of the br- links: "confirm, those links open fine". Note for the worker: the built UI already tries every reachable project when a ticket URL has no project (index.js, the "No ticket <id>" path), so a bare id fails only because the daemon wants the qualified id. Item 2 (expand bare ids in the gateway or daemon) restores the cross-project lookup too; per project, try that project's prefix. Add a test that a bare id resolves.
