+++
id = "br-4cgx"
title = "@-mention a role in a ticket reply or a document comment and that role gets a message"
kind = "feature"
state = "claimed"
created_at = "2026-10-09T20:49:28.208Z"
updated_at = "2026-10-17T08:03:13.592448Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:4cgx",
]
priority = "low"
priority_at = "2026-10-09T20:49:31.204227Z"
ticket = "4cgx"
+++

docs/tickets/open/mention-a-role-in-a-ticket-reply-or-a-document-comment-and-t-4cgx.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T20:49:31.204Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-17T02:06:32.873Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:28.371Z
Brief (pm-1): build in stop points; stop at the first that runs long and tell the manager. Model: Sonnet. The ticket body holds the human-approved rules (one message per agent, watcher de-dup with the mention winning, \@ escape, agents may use it, unknown role reported to the author, external roles detected and sent to external:, both cross-project forms @bridle-ui:orchestrator and @ui:orchestrator plus the explicit external: form).
Stop point 1: same-project mentions in a ticket/task thread comment: parse in the daemon's task-comment path, send one message per distinct agent (who, where with link, text), de-dup against watchers, \@ escape, unknown role reported back to the author. Tests for each rule. Docs: cli.md, CHANGELOG.
Stop point 2: the same in document comments (doc_watch in crates/bridle-daemon/src/doc_watch.rs).
Stop point 3: the cross-project forms (project name and ID prefix), resolving the project via the existing cross-project send path (bridle send --project). br-re79 (reviewer front matter) builds on this: it is blocked-by this ticket. Acceptance: just check per stop point. Out of scope: the UI, reviewer front matter (re79).
