+++
id = "br-gd43"
title = "Comment IDs never repeat after deletes: assign_ids reads and bumps a front-matter counter (bridle half of ui-vnuu)"
kind = "bug"
state = "planned"
created_at = "2026-10-09T18:09:53.692Z"
updated_at = "2026-10-09T18:10:58.595122Z"
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

### note · external:orchestrator · 2026-10-09T18:10:53.354Z
Field name (orchestrator, acting PM for bridle-ui): `comment_next`, the next comment number to assign (an integer; next ID is max(comment_next, highest c<n> + 1); write comment_next = assigned + 1). Same name in bridle (br-gd43) and bridle-ui (ui-vnuu).

### note · agent:pm-1 · 2026-10-09T18:10:57.848Z
pm-1 brief addendum: Model Sonnet. Field name: use comment_next (the ui-vnuu ticket's suggestion) unless the ui-vnuu thread has already settled another; if it differs, follow the ui-vnuu thread and say so in your done note. Files: crates/bridle-daemon/src/doc_watch.rs, the front-matter readers (crates/bridle-spec, crates/bridle-docs, bridle ticket check), workflow/base/roles/document-reviewer.md and the comment-format design doc, CHANGELOG.md. Migration: none (documents without the key get it on the next assign; existing max-based behaviour is the fallback). Out of scope: the bridle-ui half, comment deletion UI.
