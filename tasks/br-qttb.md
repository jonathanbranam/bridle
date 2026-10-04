+++
id = "br-qttb"
title = "x8jt: 'bridle review now <path> [--resend]' sends pending threads at once; sent marks in the file; gateway route"
kind = "feature"
state = "planned"
created_at = "2026-10-04T02:25:31.949Z"
updated_at = "2026-10-04T02:25:53.024536Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
+++

x8jt, bridle side. Approved by the human 2026-10-04 (via advisor doc-review, m-4238): 'Add a bridle command to perform the review on a document immediately and a button in the UI also to request the review. Comments that have been sent for review should be marked as such and not resent if the button is pressed again, unless requested.' Sent mark lives in the file ('A'). Thread IDs deferred. Spec: ticket x8jt (docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md), last section.
Built already (br-aj9d, 941312c): crates/bridle-daemon/src/doc_watch.rs (30 s loop; a thread is pending when its last reply is the human's; after [review] quiet_minutes the pending threads go as one batch to agent doc-<stem>, role document-reviewer); registry .bridle/review-documents.txt via 'bridle review add|remove|list'; docs in daemon.md 'Document review'. br-5paw (gateway read/write/commit of one document) is the gateway pattern to follow.
Goal: (1) 'bridle review now <path> [--resend]' sends the document's pending threads to its agent at once, skipping the quiet period (a daemon endpoint the CLI calls, since the daemon owns the agent). (2) Whenever bridle sends a batch (quiet period or now), it appends ' · sent HH:MM' (24h, US Eastern per the human-timezone rule) to the line of each thread's newest human entry; later sends skip entries already marked sent unless --resend. Make sure the pending-thread detection and the agent's own edits are not confused by the mark. (3) A gateway route the UI button calls (same semantics, resend flag) with ts-rs types, auth like other /api/v1 routes; the bridle-ui button ui-c39e waits on it, so keep the route shape simple and document it.
Docs: daemon.md, cli.md, human-web-ui.md, CHANGELOG. Tests: now skips the quiet period; sent mark appended once; resend resends; mark doesn't re-trigger the watcher. Acceptance: just check passes. Model: Sonnet. Migration: none (the mark is plain text in documents; old threads without it count as unsent). Out of scope: thread IDs, the UI.
