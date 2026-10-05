+++
id = "br-75zr"
title = "bridle-ui: view each project's specs as organised in design/specs, and link specs from tasks"
kind = "feature"
state = "planned"
created_at = "2026-10-05T02:40:28.433Z"
updated_at = "2026-10-05T02:40:56.318158Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: 75zr
Bridle-repo (gateway) half of docs/tickets/open/bridle-ui-view-each-project-s-specs-as-organised-in-design-s-75zr.md (read it; it quotes the human: read-only, first version tonight, follow what exists). bridle-ui's half is ui-dttu. Keep it small. Step 1: check whether the existing gateway document routes (crates/bridle-gateway/src/documents.rs: read/search/list) already serve a project's design/specs/** files (they were scoped to docs/ for links/resolve). If not, allow design/specs/ for the READ routes only, same traversal checks, no write. Step 2: one read-only route, GET /api/v1/projects/{project}/specs, that lists the project's specs as bridle organises them: reuse the parser in crates/bridle-spec (docs/design/specs.md): each spec file with its capability name, requirements and scenarios, each with its stable id and heading, and whether a scenario is executable; parse errors are returned per file as diagnostics, not a 500; a project with no design/specs returns an empty list. Step 3: extend links/resolve (from br-bnhn/br-a3yd) so a spec, requirement or scenario id resolves to its spec file path with an anchor for the heading; unknown ids resolve to none. Doc: docs/design/human-web-ui.md (the routes, read-only). Tests: list with a fixture spec, empty project, a spec with a parse error, id resolution, traversal rejected. Acceptance: just check passes. Model: Sonnet. Runs after br-a3yd (same documents.rs and links/resolve code; dependency edge). Out of scope: any UI, editing specs, coverage views, linking tasks to specs (a task field is a separate decision; the UI links ids it finds in task text via links/resolve).

## Thread

### note · external:orchestrator · 2026-10-05T02:40:40.585Z
Approved for tonight by the human, via aide (m-0343, 2026-10-04 ~10:30 PM ET): "We could do a first version of it tonight. Just follow what exists and add the ability to show the specs as they are organized internally. Just to view them is all we would need. I think specs should be linked from tasks, maybe, but that linking should also be there." Bridle side (orchestrator): whatever the gateway needs for a read-only spec view: a route listing a project's specs as bridle organises them (reuse bridle-spec's parser) and spec/requirement/scenario IDs resolving in links/resolve. If the existing document routes already suffice, say so and keep this tiny. bridle-ui side is ui task (filed next).
