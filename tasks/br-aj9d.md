+++
id = "br-aj9d"
title = "x8jt slice 2: bridle notices new comments in documents under review, debounces 5-10 min, starts/resumes that document's agent with the batch"
kind = "feature"
state = "planned"
created_at = "2026-10-04T00:48:48.514Z"
updated_at = "2026-10-04T00:49:42.543403Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
+++

x8jt slice 2 of 3 (human approved 2026-10-03; 'I want to get this moving quickly'). Builds on br-rp53 (the document-reviewer role); start after it merges. Spec: docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md, sections 'Which agent answers', 'Format approved; batching...', 'Tag format...'.
Goal: bridle notices new comments in documents under review and starts or resumes that document's agent with the batch. Watch the documents under review; when new comments have been quiet 5-10 minutes (configurable, default 7), start or resume that document's agent (role document-reviewer) with the batch as its prompt. Use the existing agent stop/resume; a simple cap on how many document agents run at once (config, default 3); an expiry per document agent (config, default stop after N hours idle). How a document is 'under review' (a registry of document paths per project, e.g. a bridle command to register one, or a marker in the doc) is yours to propose in a short plan comment on the task first, simplest first; doesn't wait on the rest of r9vh. Files: crates/bridle-daemon (supervisor, config), crates/bridle (CLI), docs/design/agent-host/ in step, CHANGELOG. Tests with fake clock: debounce, batching, cap, expiry. Acceptance: just check passes. Model: Sonnet. Migration: config keys optional with defaults; none. Out of scope: UI, showing what changed (deferred; git diff serves).

## Thread

### note · agent:pm-1 · 2026-10-04T00:49:42.543Z
Blocked by br-rp53 (needs the role). Tier 2.
