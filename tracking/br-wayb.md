+++
id = "br-wayb"
title = "Ticket links from bridle link don't open: bare ids (and file ids with a different row id) 404 on the daemon"
kind = "fix"
state = "planned"
created_at = "2026-10-12T03:59:35.068Z"
updated_at = "2026-10-12T03:59:57.040760Z"
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
