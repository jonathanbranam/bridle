+++
id = "br-re79"
title = "Comments on a ticket an agent asked the human to approve go back to that agent; a pending ticket gets a direct Approve"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T21:37:47.787Z"
updated_at = "2026-10-17T15:47:51.332508Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "low"
priority_at = "2026-10-09T21:37:50.421277Z"
branch = "bridle/wre79"
commit = "40054ef603ae1a7d463ef91cc222dd0b8c14baeb"
summary = """doc_watch reads `reviewer: <agent>` from a document's front matter. When set and it resolves (same resolver as @mentions, mentions.rs/server::resolve_name), the batch goes to that agent as a Note instead of the doc-<id> agent (none spawned). The reviewer counts as "already told" for mentions, so a mention plus a reviewer is one message. Unknown or gone agent: the human gets a task_update and the default reviewer takes the batch. Not supported: project-qualified reviewers (ui:orchestrator); the reviewer is resolved locally only. Tests in tests/review_now_test.rs (set, mention plus reviewer, unknown, missing) and a unit test for parsing. Docs: daemon.md Reviewer paragraph, CHANGELOG."""
ticket = "re79"
+++

docs/tickets/open/comments-on-a-ticket-an-agent-asked-the-human-to-approve-go-re79.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T21:37:50.421Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-17T02:06:32.458Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:30.836Z
Brief (pm-1): build the refined ask only (the section 'Refined'): a document's front matter `reviewer: <agent>`; when comments are added, doc_watch (crates/bridle-daemon/src/doc_watch.rs) sends the batch to that agent instead of the default doc-<id> reviewer; no reviewer: keeps today's behaviour. Use br-4cgx's agent-naming rules (blocked-by br-4cgx; reuse its resolver, do not copy it). Unknown or gone agent: report it and fall back to the default reviewer. The proposal on 4cgx is the rule: the reviewer is like a watcher; mentions add agents; nobody gets two. Tests: reviewer set, missing, unknown agent, a mention plus a reviewer. Docs: daemon.md around line 113, CHANGELOG. Acceptance: just check. Model: Sonnet. Out of scope: the direct Approve button, bridle-ui changes (note on the thread what the UI needs to do).

### note · external:orchestrator · 2026-10-17T15:05:13.357Z
From orchestrator: br-re79 is planned and startable (tier 2; pm-1 already planned it). Spawn its worker now. br-9qws stays waiting on the human.

### note · agent:wre79 · 2026-10-17T15:23:21.364Z
UI note (bridle-ui, not built here): to send the human's comments back to an asker, the agent (or UI) only writes 'reviewer: <agent>' into the document front matter; use external:<role> or advisor/<name> for externals. The comment flow is otherwise unchanged. The UI could show the reviewer beside the comment box.

### note · agent:wre79 · 2026-10-17T15:23:21.452Z
done: reviewer: front matter routes doc comments (fallback + report on unknown, mention+reviewer = one message); just check exit 0, 1820 tests passed (last full 1815), checked at 8c-pre-merge commit; HEAD dd42691a = that plus a merge of main that only added ticket docs

### note · agent:manager-2 · 2026-10-17T15:46:32.706Z
integrated: 40054ef603ae1a7d463ef91cc222dd0b8c14baeb (branch bridle/wre79)

### note · agent:manager-2 · 2026-10-17T15:46:48.162Z
cleanup: removed agent wre79, branch bridle/wre79

### note · external:advisor/product-manager · 2026-10-17T15:47:51.332Z
advisor (product-manager): verified on main (40054ef6 is an ancestor). Every item of the refined ask is met: reviewer: front matter routes the batch (doc_watch.rs:274-280, 926-972), reuses server::resolve_name, an unknown reviewer is reported to the human and falls back, mention+reviewer gives one message, 4 tests in review_now_test.rs, daemon.md 224-229 + CHANGELOG. Two small gaps, not blocking: (a) a reviewer that resolves but every send fails falls back with only a log warning, no message to the human (doc_watch.rs:968-971); (b) the unknown-reviewer test doesn't assert the human is told (review_now_test.rs:166). Project-qualified reviewers (ui:orchestrator) are not supported, waiting on 4cgx's open question. Resolving.
