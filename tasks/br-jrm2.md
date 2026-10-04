+++
id = "br-jrm2"
title = "Document view: project dropdown, ticket search by ID, comment box at the highlight, full width"
kind = "feature"
state = "planned"
created_at = "2026-10-04T14:57:49.130Z"
updated_at = "2026-10-04T16:36:36.418627Z"
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
