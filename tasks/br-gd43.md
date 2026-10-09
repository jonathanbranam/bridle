+++
id = "br-gd43"
title = "Comment IDs never repeat after deletes: assign_ids reads and bumps a front-matter counter (bridle half of ui-vnuu)"
kind = "bug"
state = "open"
created_at = "2026-10-09T18:09:53.692Z"
updated_at = "2026-10-09T18:09:58.042500Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
size = "S"
priority = "high"
priority_at = "2026-10-09T18:09:53.692772Z"
+++

Bridle half of bridle-ui ticket vnuu (docs/tickets/open/comment-ids-never-repeat-*-vnuu.md in the bridle-ui repo; read it: the human's words and the full ask). Here: (1) crates/bridle-daemon/src/doc_watch.rs assign_ids takes the next ID as max(front-matter counter, highest c<n> + 1) and writes the counter back; a document with no front matter gets a minimal one. Use the same field name as ui-vnuu (agree it on the ui-vnuu thread before building; the ticket suggests comment_next). (2) Front-matter readers accept the new key: bridle ticket check, specs, the docs crate. (3) workflow/base/roles/document-reviewer.md (and the comment-format design doc it points to) documents the field so agents writing comments by hand bump it. (4) Tests: delete the newest and then all threads, assign, the ID keeps counting up. Live bug since ui-ha6m (delete) landed. Acceptance: just check.

## Thread

### note · external:advisor/product-manager · 2026-10-09T18:09:53.692Z
priority: normal -> high
