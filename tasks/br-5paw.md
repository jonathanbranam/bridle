+++
id = "br-5paw"
title = "x8jt slice 3a: gateway reads, writes and commits one document file (narrow v8kn piece for the document view)"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T00:48:48.535Z"
updated_at = "2026-10-04T01:15:05.553809Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
branch = "bridle/doc-gateway"
commit = "651a2065fe5e2ee63f8014a782ddf22631adc453"
summary = "Gateway GET/PUT /api/v1/projects/{project}/documents/{path} (crates/bridle-gateway/src/documents.rs): returns content, sha256 hash and branch; PUT needs the hash (409 if stale), commits only that file as 'review: human comments on <path>', refuses ../absolute/.git/symlink-escape paths, missing and non-text files, and any write on main/master/dev/detached HEAD (403). ts-rs types Document, DocumentWrite, DocumentSaved in bindings/. Discovery Target gained repo. Docs: human-web-ui.md, CHANGELOG. Caveat: bridle's own repo on main can't be written to by design."
+++

x8jt slice 3a (human approved 2026-10-03: 'Approve all three to build', 'I want to get this moving quickly'). Spec: docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md, last two sections. The bridle-ui document view (ui-acf0, other repo) is gated on this, so land it promptly.
Goal, in crates/bridle-gateway (see items.rs, ui.rs, types.rs for how existing /api/v1 routes and ts-rs types are done): a narrow slice of v8kn: (1) GET one document file's text by project and repo-relative path; (2) PUT the whole file back (the human's comment edits) and commit it on the project's working branch (git commit by the gateway, message like 'review: human comments on <path>'); refuse paths outside the project repo (no .. or absolute), non-text files, and a write when the file changed since the read (send the content hash or commit sha with the read, require it on write; 409 otherwise). ts-rs types for the request/response so the UI can use them. Same auth as other /api/v1 routes. Nothing more of v8kn (no browsing, no search).
Docs: human-web-ui.md (and docs/design/ in step), CHANGELOG. Tests: read, write+commit, path traversal refused, stale write refused. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: the UI, the comment watcher (br-aj9d), role prompt (br-rp53). Since the gateway commits to the human's repo, restrict to projects bridle manages on a bridle branch; do not touch main/dev of a trial project (rule existing-projects): write only on the branch checked out in the project's working tree and say which in the docs.

## Thread

### note · agent:doc-gateway · 2026-10-04T01:13:06.319Z
done: gateway GET/PUT /api/v1/projects/{project}/documents/{path} with hash-checked writes, a commit on the checked-out branch (refused on main/master/dev/detached), ts-rs types, docs and CHANGELOG; just check green (1122 tests); c4b8202b6f5d60b4a27de6ffbf30744ebe3ab401

### note · agent:manager-2 · 2026-10-04T01:13:18.136Z
integrated: 651a2065fe5e2ee63f8014a782ddf22631adc453 (branch bridle/doc-gateway)

### note · agent:manager-2 · 2026-10-04T01:15:05.553Z
cleanup: removed agent doc-gateway, branch bridle/doc-gateway
