+++
id = "br-re79"
title = "Comments on a ticket an agent asked the human to approve go back to that agent; a pending ticket gets a direct Approve"
kind = "feature"
state = "planned"
created_at = "2026-10-09T21:37:47.787Z"
updated_at = "2026-10-17T15:05:13.357367Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "low"
priority_at = "2026-10-09T21:37:50.421277Z"
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
