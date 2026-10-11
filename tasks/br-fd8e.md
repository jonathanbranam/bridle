+++
id = "br-fd8e"
title = "Commit every edit bridle makes to a reviewed document or ticket (status markers, thread IDs, frontmatter), so the clone is never left dirty"
kind = "bug"
state = "integrated"
created_at = "2026-10-10T21:53:17.031Z"
updated_at = "2026-10-11T03:08:57.308035Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
branch = "bridle/wfd8e"
commit = "c038308a7e6e963226411576206d0ca5d496cf28"
summary = "The review watcher (crates/bridle-daemon/src/doc_watch.rs) now routes its document writes (c<n> ids, [sent], [sent]->[read], mark migration) through write_marked. EditCommits notes each write and, after 10s of stillness, makes one commit 'review: bridle updates comment marks on <path>' via the shared bridle_docs::documents::commit_file (extracted from write_document, which the human's comments also use). Guards: owner clone only (DocWatcher::with_owner, set in lib.rs from owner.toml; non-pushing projects count as owner); a file already dirty before bridle's first write is left uncommitted and logged once; old dirt is not auto-committed. Pushing: rides the owner's normal push (br-8ay6), nothing new. Not covered: frontmatter from 'bridle ticket' CLI commands (caller's worktree), deferred. Tests: burst gives one commit, dirty file left alone, non-owner no commit. Docs: human-web-ui.md, CHANGELOG."
ticket = "fd8e"
+++

Ticket: docs/tickets/open/commit-every-edit-bridle-makes-to-a-reviewed-document-or-tic-fd8e.md (read it; the human's common case is the [sent] -> [read] marker rewrite, fix that first). Also docs/design/human-web-ui.md (review section), rule workflow/base/rules one-pusher-for-the-integration-branch, and br-hdbj/br-8ay6 (owner-only push, direct-to-main docs commits). Goal: every write bridle itself makes to a document or ticket in the clone (the daemon's sent -> read marker rewrite, c<n> thread IDs, red/highlight marks, frontmatter written by ticket task and the like) is committed when made, so the clone is not left dirty by bridle. Find the writers (crates/bridle-docs guarded write, the daemon and gateway review paths; grep for the sent/read marker rewrite) and the existing commit path used for the human's comments (grep 'review: human comments on'); reuse it rather than adding a second. Batch rapid edits to one file into one commit (short debounce) so a burst is not many commits. Never commit when the clone is not the owner's or the file has other uncommitted edits by someone else (leave it, log once). Pushing: these commits ride the existing normal push from the owner clone (br-8ay6); say so in the done note and docs, no new push permission. Files: crates/bridle-docs, crates/bridle-daemon (review path), docs/design/human-web-ui.md, the doc describing document commits, CHANGELOG. Migration: none; behaviour only. Clones already dirty from earlier marker rewrites are committed by the next write or by the human; do not auto-commit old dirt. Acceptance: just check passes; tests with a temp git repo: a marker rewrite produces one commit with the review author; a burst gives one commit; a file with other uncommitted changes is left alone; non-owner clone makes no commit. Model: Sonnet. Out of scope: the document-reviewer's own commits (already done), pushing, the 4cgx marker design.

## Thread

### note · external:advisor/product-manager · 2026-10-10T21:53:19.475Z
advisor/product-manager (PdM): the human's ask (via aide, ~5:15 PM ET): 'that file needs to be committed'. Readied, normal, in epic comment-routing (the 4cgx markers are that epic's work). pm-1: plan it.

### note · external:advisor/product-manager · 2026-10-10T21:54:05.968Z
human via advisor/product-manager (2026-10-10 evening), verbatim: "I keep finding a bunch of files where the only diff is where it changed from 'to red'. Yeah, that's probably it." (speech-to-text: the [sent] -> [read] markers). The common case to fix first.

### note · agent:wfd8e · 2026-10-11T02:46:01.510Z
done: bridle's own review-document edits are committed (debounced, owner-only, skips dirty files); just check exit 0, 1487 tests passed; checked at commit 5f..(pre-merge) then main merged (docs-only change); tip 75fa9ad7

### note · agent:manager-2 · 2026-10-11T03:08:57.308Z
integrated: c038308a7e6e963226411576206d0ca5d496cf28 (branch bridle/wfd8e)
