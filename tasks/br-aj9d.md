+++
id = "br-aj9d"
title = "x8jt slice 2: bridle notices new comments in documents under review, debounces 5-10 min, starts/resumes that document's agent with the batch"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T00:48:48.514Z"
updated_at = "2026-10-04T01:47:53.866412Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
branch = "bridle/doc-watcher"
commit = "941312c5f1b319852eec39e2f4f3fb9009674fef"
summary = """Documents are "under review" when listed in `.bridle/review-documents.txt` (`bridle review add|remove|list`; file re-read each tick). New `doc_watch.rs` in bridle-daemon (30 s loop): a comment thread is pending when its last reply is the human's (author containing "agent" = the document agent), so the agent's own edits and a daemon restart don't re-trigger. Pending threads still for `[review] quiet_minutes` (7) go as one batch to agent `doc-<stem>` (role document-reviewer, Workdir::Repo): spawned, resumed, or messaged. `max_agents` (3) holds back starts, `idle_hours` (4) stops idle agents. Tests use passed-in times (debounce, batch, cap, expiry, registry); no end-to-end test with the fake claude. Docs: daemon.md "Document review", cli.md, CHANGELOG. Caveat: the registry file is read from the main checkout, so it must be committed/present there."""
+++

x8jt slice 2 of 3 (human approved 2026-10-03; 'I want to get this moving quickly'). Builds on br-rp53 (the document-reviewer role); start after it merges. Spec: docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md, sections 'Which agent answers', 'Format approved; batching...', 'Tag format...'.
Goal: bridle notices new comments in documents under review and starts or resumes that document's agent with the batch. Watch the documents under review; when new comments have been quiet 5-10 minutes (configurable, default 7), start or resume that document's agent (role document-reviewer) with the batch as its prompt. Use the existing agent stop/resume; a simple cap on how many document agents run at once (config, default 3); an expiry per document agent (config, default stop after N hours idle). How a document is 'under review' (a registry of document paths per project, e.g. a bridle command to register one, or a marker in the doc) is yours to propose in a short plan comment on the task first, simplest first; doesn't wait on the rest of r9vh. Files: crates/bridle-daemon (supervisor, config), crates/bridle (CLI), docs/design/agent-host/ in step, CHANGELOG. Tests with fake clock: debounce, batching, cap, expiry. Acceptance: just check passes. Model: Sonnet. Migration: config keys optional with defaults; none. Out of scope: UI, showing what changed (deferred; git diff serves).

## Thread

### note · agent:pm-1 · 2026-10-04T00:49:42.543Z
Blocked by br-rp53 (needs the role). Tier 2.

### note · agent:doc-watcher · 2026-10-04T01:34:04.172Z
Plan (proposal for "under review", simplest first):
- Registry: a plain-text file `.bridle/review-documents.txt` in the project clone, one repo-relative path per line (`#` comments). Re-read each tick, so no restart and hand-editable. `bridle review add|remove|list <path>` just edits that file (no API, no DB migration).
- Detect comments: the watcher reads each registered file (main checkout) every 30s and counts `> [!comment]` callouts plus bold-reply lines; any change in the file's hash with comment-text change marks it "dirty"; the debounce restarts on each change (trailing edge, like queue_nudge). "New comments" = the comment text differs from what the agent was last given (so the agent's own edits settle without re-triggering after its round).
- After quiet for `[review] quiet_minutes` (default 7): send the batch (the document path plus the comment blocks that changed) to that document's agent (role document-reviewer, name `doc-<slug>`); spawn it if none exists, resume it if stopped, else send as a message.
- Config `[review]`: quiet_minutes=7, max_agents=3 (a due document waits when the cap is reached), idle_hours=4 (stop an agent idle that long).
- Code: new crates/bridle-daemon/src/doc_watch.rs with a clock trait for the fake-clock tests (debounce, batch, cap, expiry), config keys, one tick loop in lib.rs, CLI `review` subcommand, docs/design/agent-host/, CHANGELOG.

### note · agent:doc-watcher · 2026-10-04T01:46:16.607Z
done: daemon watches documents listed in .bridle/review-documents.txt, debounces pending comments (7 min), starts/resumes doc agent; cap 3, idle expiry 4h; just check green (1133 tests); 3b27fbc

### note · agent:manager-2 · 2026-10-04T01:46:21.976Z
integrated: 941312c5f1b319852eec39e2f4f3fb9009674fef (branch bridle/doc-watcher)

### note · agent:manager-2 · 2026-10-04T01:47:53.866Z
cleanup: removed agent doc-watcher, branch bridle/doc-watcher
