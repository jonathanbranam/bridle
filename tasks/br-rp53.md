+++
id = "br-rp53"
title = "x8jt slice 1: document-reviewer role prompt (comment format, reply/revise/tag/mark read/resolve, commit each round)"
kind = "feature"
state = "planned"
created_at = "2026-10-04T00:48:48.493Z"
updated_at = "2026-10-04T00:49:30.051628Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
+++

x8jt slice 1 of 3 (human approved 2026-10-03: 'Approve all three to build. Approve tag format', 'I want to get this moving quickly').
Goal: a role prompt, workflow/base/roles/document-reviewer.md, for a document-reviewer agent: one agent per document, started by hand by the orchestrator and told 'go'. Spec: docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md, sections 'Comments live in the document', 'Example: one review round', 'Format approved...' and 'Tag format...'. The prompt covers: the approved '> [!comment] <who>, <when>, on "<quoted words>"' callout format; replies as bold names inside it; on revising the doc for a comment, a follow-up reply tagging @human; tag format '@human' / '@docs-agent' at the start of a reply, marked read by appending '(read)' (the agent marks its own tags when it takes a round); resolving = delete the thread and add a note at the bottom; commit each round; handle comments as one batch round. Look at the existing roles in workflow/base/roles/ for form, and how roles are registered (bridle prime/role listing, docs/design/roles-and-config.md or similar) so the new role resolves.
Docs: the roles doc entry, CHANGELOG. Acceptance: just check passes; the role resolves (bridle prime or the role-listing test used for other roles). Model: Sonnet. Migration: none (projects get the role through the vendored workflow). Out of scope: daemon changes, the comment watcher (br-aj9d), UI. First trial document: ticket gtzx.
