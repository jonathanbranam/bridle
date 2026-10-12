+++
id = "br-wayb"
title = "Ticket links from bridle link don't open: bare ids (and file ids with a different row id) 404 on the daemon"
kind = "fix"
state = "integrated"
created_at = "2026-10-12T03:59:35.068Z"
updated_at = "2026-10-12T06:53:15.943353Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wwayb",
]
priority = "high"
priority_at = "2026-10-12T03:59:45.418153Z"
branch = "bridle/wwayb"
commit = "001a6c5ba468f9073b796c446483bb98745f3c22"
summary = "Gateway: tasks::resolve_id maps a link's id (bare, or a ticket file id whose row differs) to the unique row id; used by ticket detail and by actions (actions.rs act_route). bridle link now prints the qualified id for a bare ticket id (link::run async, reuses ticket::own_prefix/is_bare_id). Test: links_with_bare_or_file_ids_open_the_row covers btdn, br-btdn, 7d62, br-7d62, br-10c0. CHANGELOG and cli.md updated. Caveat: bridle link does not map file id to row id (needs the daemon); the gateway does."
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

### note · agent:wwayb · 2026-10-12T06:43:24.061Z
built: 3ae03d8b (branch bridle/wwayb)

### note · agent:wwayb · 2026-10-12T06:43:24.170Z
done: bare and ticket-file ids resolve in the gateway (detail + actions), bridle link qualifies bare ids; just check exit 0, 1584 tests passed, sha 3ae03d8b; main merged (up to date)

### note · agent:manager-2 · 2026-10-12T06:53:02.533Z
integrated: 001a6c5ba468f9073b796c446483bb98745f3c22 (branch bridle/wwayb)

### note · agent:manager-2 · 2026-10-12T06:53:15.943Z
cleanup: removed agent wwayb, branch bridle/wwayb
