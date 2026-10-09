+++
id = "br-gd43"
title = "Comment IDs never repeat after deletes: assign_ids reads and bumps a front-matter counter (bridle half of ui-vnuu)"
kind = "bug"
state = "planned"
created_at = "2026-10-09T18:09:53.692Z"
updated_at = "2026-10-09T19:08:46.553103Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
size = "M"
priority = "low"
priority_at = "2026-10-09T19:08:42.451097Z"
+++

Bridle half of bridle-ui ticket vnuu (docs/tickets/open/comment-ids-never-repeat-*-vnuu.md in the bridle-ui repo; read it: the human's words and the full ask). Here: (1) crates/bridle-daemon/src/doc_watch.rs assign_ids takes the next ID as max(front-matter counter, highest c<n> + 1) and writes the counter back; a document with no front matter gets a minimal one. Field (the human, 2026-10-09): `next_comment_id: c<n>`, the ID the next comment gets (e.g. `next_comment_id: c7`); next ID is max(next_comment_id, highest c<n> + 1), then write back assigned + 1. Same as ui-vnuu. (2) Front-matter readers accept the new key: bridle ticket check, specs, the docs crate. (3) workflow/base/roles/document-reviewer.md (and the comment-format design doc it points to) documents the field so agents writing comments by hand bump it. (4) Tests: delete the newest and then all threads, assign, the ID keeps counting up. (5) Added by the human 2026-10-09 ~3:15 PM ET (thread, 19:05Z): when doc_watch sends a document's pending comments to its agent (after the quiet period), it also raises next_comment_id if lower than the highest c<n> in the file + 1, and reports any thread ID used twice (to the agent in the batch, and in the daemon log). Only for documents under review, at that moment; no new polling. Live bug since ui-ha6m (delete) landed. Acceptance: just check.

## Thread

### note · external:advisor/product-manager · 2026-10-09T18:09:53.692Z
priority: normal -> high

### note · external:orchestrator · 2026-10-09T18:10:53.354Z
Field name (orchestrator, acting PM for bridle-ui): `comment_next`, the next comment number to assign (an integer; next ID is max(comment_next, highest c<n> + 1); write comment_next = assigned + 1). Same name in bridle (br-gd43) and bridle-ui (ui-vnuu).

### note · agent:pm-1 · 2026-10-09T18:10:57.848Z
pm-1 brief addendum: Model Sonnet. Field name: use comment_next (the ui-vnuu ticket's suggestion) unless the ui-vnuu thread has already settled another; if it differs, follow the ui-vnuu thread and say so in your done note. Files: crates/bridle-daemon/src/doc_watch.rs, the front-matter readers (crates/bridle-spec, crates/bridle-docs, bridle ticket check), workflow/base/roles/document-reviewer.md and the comment-format design doc, CHANGELOG.md. Migration: none (documents without the key get it on the next assign; existing max-based behaviour is the fallback). Out of scope: the bridle-ui half, comment deletion UI.

### note · external:aide · 2026-10-09T19:03:59.379Z
From the human, via the bridle-ui aide, 2026-10-09 ~2:15 PM ET: the front-matter field is `next_comment_id: c<n>` (the ID the next comment gets; e.g. `next_comment_id: c7`). Chosen by the human; use exactly this name and form. See ui-vnuu.

### note · external:advisor/product-manager · 2026-10-09T19:04:12.614Z
From advisor (product-manager): the human's choice above supersedes the earlier notes (orchestrator 18:10, pm-1 addendum): the field is next_comment_id, value the string c<n> (the ID the next comment gets), not comment_next as an integer. Same in ui-vnuu.

### note · external:orchestrator · 2026-10-09T19:04:46.021Z
From orchestrator: br-gd43's brief still says comment_next; the human chose next_comment_id: c<n> (thread, 19:03Z). Please update the brief before it's claimed. ui-vnuu's brief is updated to match.

### note · external:advisor/product-manager · 2026-10-09T19:04:59.568Z
advisor (product-manager): brief updated to next_comment_id: c<n> (the human's choice).

### note · external:aide · 2026-10-09T19:05:44.459Z
From the human, via the bridle-ui aide, 2026-10-09 ~3:15 PM ET: "yes, agree" to adding this to br-gd43: when the daemon sends a document's pending comments to its agent (doc_watch, after the quiet period), it also (1) raises next_comment_id if it is lower than the highest c<n> in the file plus one, and (2) reports any thread ID used twice in the file (to the document's agent in the batch, and in the daemon log). Only for documents under review, at that moment; no new polling.

### note · external:advisor/product-manager · 2026-10-09T19:05:58.631Z
advisor (product-manager): brief updated with (5), the human's addition above; size S -> M.

### note · external:advisor/product-manager · 2026-10-09T19:08:42.451Z
priority: high -> low

### note · external:advisor/product-manager · 2026-10-09T19:08:46.553Z
From the human, via advisor (product-manager), 2026-10-09 ~3:30 PM ET: "all of the comment work is low priority; should come at the end of queue for other work." Priority set to low.
