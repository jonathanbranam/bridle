+++
id = "br-5paw"
title = "x8jt slice 3a: gateway reads, writes and commits one document file (narrow v8kn piece for the document view)"
kind = "feature"
state = "planned"
created_at = "2026-10-04T00:48:48.535Z"
updated_at = "2026-10-04T00:49:30.069887Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
+++

x8jt slice 3a (human approved 2026-10-03: 'Approve all three to build', 'I want to get this moving quickly'). Spec: docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md, last two sections. The bridle-ui document view (ui-acf0, other repo) is gated on this, so land it promptly.
Goal, in crates/bridle-gateway (see items.rs, ui.rs, types.rs for how existing /api/v1 routes and ts-rs types are done): a narrow slice of v8kn: (1) GET one document file's text by project and repo-relative path; (2) PUT the whole file back (the human's comment edits) and commit it on the project's working branch (git commit by the gateway, message like 'review: human comments on <path>'); refuse paths outside the project repo (no .. or absolute), non-text files, and a write when the file changed since the read (send the content hash or commit sha with the read, require it on write; 409 otherwise). ts-rs types for the request/response so the UI can use them. Same auth as other /api/v1 routes. Nothing more of v8kn (no browsing, no search).
Docs: human-web-ui.md (and docs/design/ in step), CHANGELOG. Tests: read, write+commit, path traversal refused, stale write refused. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: the UI, the comment watcher (br-aj9d), role prompt (br-rp53). Since the gateway commits to the human's repo, restrict to projects bridle manages on a bridle branch; do not touch main/dev of a trial project (rule existing-projects): write only on the branch checked out in the project's working tree and say which in the docs.
