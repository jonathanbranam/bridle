+++
id = "br-bnhn"
title = "bridle-ui: render markdown, turn wiki links and file paths into links that open in the UI"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T21:35:43.904Z"
updated_at = "2026-10-05T01:15:45.809810Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
priority = "high"
branch = "bridle/link-resolve"
commit = "87ba8f84e26cd39e96e1597019ffea1fa18fafb5"
summary = "Added POST /api/v1/projects/{project}/links/resolve (documents.rs, route in lib.rs): batch of targets -> {target, path|null}. Targets with a / are docs/ paths (.md optional); bare names are ticket stems looked up in tickets/open then resolved; bare names like README are not resolved. Reuses resolve() so .., absolute paths and symlink escapes give null. Also types.rs export + regenerated bindings/ (needed by committed_types_are_current), human-web-ui.md, CHANGELOG. Route is /links/resolve not under /documents/ to avoid the {*path} wildcard."
ticket = "bnhn"
+++

Bridle-repo (gateway) half of docs/tickets/open/bridle-ui-render-markdown-turn-wiki-links-and-file-paths-int-bnhn.md (read it). The UI half (react-markdown + remark-gfm as in track-web client-trips, a pre-step that turns wiki links and docs/ paths into markdown links, highlight/comment splitting that respects markdown structure) is bridle-ui's own project and is filed there by the orchestrator; I recommend that renderer without further research (track-web already uses it). This task: a gateway endpoint the UI calls to resolve link targets, in crates/bridle-gateway/src/documents.rs next to the existing documents read/search routes. Given a project and a batch of targets (a wiki-link target as written in docs/README.md: a doc path like docs/design/gates, or a ticket stem with no folder; or a plain path like docs/design/cli.md), return for each the existing document path (or none). Ticket stems resolve in docs/tickets/open/ then resolved/ (tickets move); bare paths are checked for existence on disk inside the project's checkout only (no path escape: reject .. and absolute paths), docs/ only. Bare names like README are NOT resolved (human unsure). Add the route to the gateway API doc (docs/design/human-web-ui.md) and tests: found, moved ticket, missing, traversal rejected. Acceptance: just check passes. Model: Sonnet. Files overlap with br-jrm2/br-ehv6 comment work only in documents.rs if at all; bek3 and tc7t are in gateway process code, so keep this change confined to documents.rs and its route registration. Out of scope: any UI code, and rendering.

## Thread

### note · external:orchestrator · 2026-10-04T21:35:43.932Z
Approval: the human, 2026-10-04 via bridle-ui's aide (m-0144): "the next thing that needs to happen is that links need to resolve in the browser." Full quote and points in the ticket (33377caf).

### note · agent:pm-1 · 2026-10-04T21:36:03.567Z
priority: normal -> high

### note · external:aide · 2026-10-04T21:42:32.901Z
watching the task

### note · agent:link-resolve · 2026-10-04T22:17:34.889Z
done: POST /projects/{project}/links/resolve in documents.rs, tests, doc, CHANGELOG; just check green (1184 tests); 0e323ccc

### note · agent:link-resolve · 2026-10-05T00:12:33.309Z
done: merged main, just check green (1184 tests); fe2514ee

### note · agent:link-resolve · 2026-10-05T00:25:58.699Z
done: merged main (br-p88z), just check green (1185 tests); acad2946

### note · agent:link-resolve · 2026-10-05T01:11:58.144Z
done: merged main (docs only, no rerun as told). Last green check was on 9a154089 (second run; first run had 6 daemon fake-claude timeouts under load 21, passed on rerun); tip 0ff65fa9

### note · agent:manager-2 · 2026-10-05T01:12:03.971Z
integrated: 87ba8f84e26cd39e96e1597019ffea1fa18fafb5 (branch bridle/link-resolve)

### note · agent:manager-2 · 2026-10-05T01:15:45.809Z
cleanup: removed agent link-resolve, branch bridle/link-resolve
