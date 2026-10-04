+++
id = "br-bnhn"
title = "bridle-ui: render markdown, turn wiki links and file paths into links that open in the UI"
kind = "feature"
state = "planned"
created_at = "2026-10-04T21:35:43.904Z"
updated_at = "2026-10-04T21:36:03.567697Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "high"
+++

original id: bnhn
Bridle-repo (gateway) half of docs/tickets/open/bridle-ui-render-markdown-turn-wiki-links-and-file-paths-int-bnhn.md (read it). The UI half (react-markdown + remark-gfm as in track-web client-trips, a pre-step that turns wiki links and docs/ paths into markdown links, highlight/comment splitting that respects markdown structure) is bridle-ui's own project and is filed there by the orchestrator; I recommend that renderer without further research (track-web already uses it). This task: a gateway endpoint the UI calls to resolve link targets, in crates/bridle-gateway/src/documents.rs next to the existing documents read/search routes. Given a project and a batch of targets (a wiki-link target as written in docs/README.md: a doc path like docs/design/gates, or a ticket stem with no folder; or a plain path like docs/design/cli.md), return for each the existing document path (or none). Ticket stems resolve in docs/tickets/open/ then resolved/ (tickets move); bare paths are checked for existence on disk inside the project's checkout only (no path escape: reject .. and absolute paths), docs/ only. Bare names like README are NOT resolved (human unsure). Add the route to the gateway API doc (docs/design/human-web-ui.md) and tests: found, moved ticket, missing, traversal rejected. Acceptance: just check passes. Model: Sonnet. Files overlap with br-jrm2/br-ehv6 comment work only in documents.rs if at all; bek3 and tc7t are in gateway process code, so keep this change confined to documents.rs and its route registration. Out of scope: any UI code, and rendering.

## Thread

### note · external:orchestrator · 2026-10-04T21:35:43.932Z
Approval: the human, 2026-10-04 via bridle-ui's aide (m-0144): "the next thing that needs to happen is that links need to resolve in the browser." Full quote and points in the ticket (33377caf).

### note · agent:pm-1 · 2026-10-04T21:36:03.567Z
priority: normal -> high
