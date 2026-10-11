+++
id = "br-epvy"
title = "Project-qualified ticket and task IDs (syqn part 2): br-k7tm in frontmatter, links, threads, messages; bare IDs accepted where unambiguous"
kind = "feature"
state = "pending"
created_at = "2026-10-11T02:19:02.435Z"
updated_at = "2026-10-11T02:19:02.436279Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
parent = "br-syqn"
+++

Ticket: docs/tickets/open/ticket-fields-get-standard-names-blocked-by-related-parent-c-syqn.md (scope bullet on project-qualified IDs) and br-22ab's ticket section 8 (IDs; read it) . Part 1 (br-syqn: field names, theme, parent, both forms read) is integrated; build on its parser in crates/bridle/src/ticket.rs. Goal: the id: frontmatter value and every reference (blocked_by, related, parent, tasks, links in threads and messages, ticket check output) use the project-qualified form br-k7tm; a bare k7tm is accepted as input where unambiguous in the current project and expanded to the qualified form; ambiguous or unknown across registered projects is refused with the candidates listed. Prefixes must be unique across registered projects: refuse a duplicate at project registration (find where prefixes are assigned/registered). File names stay bare (<slug>-<id>.md); ticket check matches the file name tail to the id: without its prefix. Writers (ticket new/set/resolve) write qualified IDs; readers accept both bare and qualified so existing tickets keep working. Also let parent take another project qualified ID (store and check well-formed; resolving across projects through the registry only where a lookup already exists, otherwise just well-formed). Files: crates/bridle/src/ticket.rs and cli.rs, project registration (crates/bridle-daemon config/registry), docs/README.md ticket conventions, docs/design/cli.md, workflow/base/rules/tickets.md, CHANGELOG. Do NOT rewrite existing tickets: part 3 migrates. Migration: none in this task (readers accept both); say so. Acceptance: just check passes; bridle ticket check still clean (no new problems) on this repo; tests: bare accepted and expanded, qualified accepted, ambiguous refused with candidates, duplicate prefix refused at registration, file-name tail matching, old tickets read. Model: Sonnet. Out of scope: the rename migration (part 3), task IDs as already br-, ticket types and readiness, thread move.
