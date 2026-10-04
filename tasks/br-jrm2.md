+++
id = "br-jrm2"
title = "Document view: project dropdown, ticket search by ID, comment box at the highlight, full width"
kind = "feature"
state = "planned"
created_at = "2026-10-04T14:57:49.130Z"
updated_at = "2026-10-04T17:44:13.665227Z"
created_by = "external:advisor/doc-review"
watchers = ["external:advisor/doc-review"]
+++

original id: jrm2
Build docs/tickets/open/document-view-project-dropdown-ticket-search-by-id-comment-b-jrm2.md (read it and the task thread; the human added item 7 and a Google-Docs-style margin layout in item 4). Touches the document review UI and daemon: Document.tsx, comments.ts, doc_watch.rs and whatever the ticket lists. Runs BEFORE br-ehv6, which touches the same files. Acceptance: just check passes, plus the UI's own checks as the ticket states. Model: Sonnet. Out of scope: scanning for hand-added comments (TBD per the ticket), and everything in br-ehv6. If it is too big for one branch, report on the thread and ask the manager to split.

## Thread

### note · external:advisor/doc-review · 2026-10-04T15:03:52.661Z
From the human, via advisor (doc-review): one addition to the scope, now item 7 in the ticket. A comment saved through the UI puts the document under review automatically, like "bridle review add". Comments added by hand still need review add, and nothing scans for them (TBD, out of scope).

### note · external:advisor/doc-review · 2026-10-04T15:05:27.251Z
From the human, via advisor (doc-review), a correction: on a wide screen the comments "should look exactly like Google Docs looks". The comment box and threads sit in a right-hand margin, each level with its highlighted text. The ticket now says so (item 4).

### note · agent:doc-view-ui · 2026-10-04T17:44:08.440Z
scope: UI half lives in bridle-ui repo, outside my worktree; proposed split on the thread. Doing the Rust half (A) now; please spawn a bridle-ui worker for B after A lands.

### note · agent:doc-view-ui · 2026-10-04T17:44:13.665Z
doc-view-ui: Document.tsx and comments.ts are in the separate bridle-ui repo (/Volumes/Data/work/bridle-ui-workspace), outside my worktree, which I may not edit. Proposed split: (A) this task = Rust side only: gateway search route GET /projects/{p}/documents?q= (bare ticket ID resolves, open tickets first), auto-review on a UI save with a pending thread (new daemon POST /v1/review/add + api client + gateway call), ts-rs bindings, docs, CHANGELOG. (B) a new bridle-ui task for items 1-5 (project select, search combobox, full width, comment box at the highlight, Google-Docs right margin), built on A's route. I'm doing A now.
