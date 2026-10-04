+++
id = "br-jrm2"
title = "Document view: project dropdown, ticket search by ID, comment box at the highlight, full width"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T14:57:49.130Z"
updated_at = "2026-10-04T18:35:22.255800Z"
created_by = "external:advisor/doc-review"
watchers = ["external:advisor/doc-review"]
branch = "bridle/doc-view-ui"
commit = "d0533a81924e664a3f7570e5a8a0e5c69aa2eac7"
summary = "Rust half of jrm2 only (Document.tsx/comments.ts are in the separate bridle-ui repo, outside the worktree): gateway GET /projects/{p}/documents?q= (DocumentMatches; bare ticket ID first, open tickets, open spikes, rest; empty q lists open tickets; docs/ only), daemon POST /v1/review/add (+ api client, DocWatch::add, plain-path check, only_if_pending), and the gateway PUT now asks for it, logging not failing if the daemon is unreachable. Docs and CHANGELOG updated; just check green (1158 passed). Items 1-5 (dropdown, combobox, full width, comment box at highlight, Google-Docs margin) remain for a bridle-ui task."
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

### note · agent:doc-view-ui · 2026-10-04T18:18:49.692Z
done (Rust half only, as proposed on the thread): gateway document search + auto-review on a UI comment; just check green, 1158 passed, no flaky reruns. UI items 1-5 still need a bridle-ui task (repo outside my worktree). tip ee5906b3

### note · agent:doc-view-ui · 2026-10-04T18:30:34.179Z
main merged, just check green (1162 passed, no failures); tip 030eb97a

### note · agent:manager-2 · 2026-10-04T18:30:45.805Z
integrated: d0533a81924e664a3f7570e5a8a0e5c69aa2eac7 (branch bridle/doc-view-ui)

### note · agent:manager-2 · 2026-10-04T18:35:22.255Z
cleanup: removed agent doc-view-ui, branch bridle/doc-view-ui
