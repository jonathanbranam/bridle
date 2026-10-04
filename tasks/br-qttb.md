+++
id = "br-qttb"
title = "x8jt: 'bridle review now <path> [--resend]' sends pending threads at once; sent marks in the file; gateway route"
kind = "feature"
state = "planned"
created_at = "2026-10-04T02:25:31.949Z"
updated_at = "2026-10-04T02:58:41.803756Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
summary = "Added 'bridle review now <path> [--resend]' (daemon POST /v1/review/now, DocWatcher::review_now) and sent marks: whenever bridle sends a batch (tick or now) it appends ' · sent YYYY-MM-DD HH:MM' (US Eastern) to the newest human entry's line in the file; marked threads are skipped unless resend; the mark doesn't alter author detection so it can't re-trigger the watcher. A lock serialises read/deliver/mark so tick and now never double-send. Review now needs the path registered (400 otherwise) and ignores the quiet period and agent cap. Gateway: POST /api/v1/projects/{project}/review with ReviewRequest{path,resend} -> ReviewResult{project,path,agent,threads}, ts-rs bindings regenerated. Marks are left uncommitted in the working tree (the agent's next commit carries them). Eastern-offset helper duplicated from gateway report.rs (small). Docs: daemon.md, cli.md, human-web-ui.md, CHANGELOG, document-reviewer role note."
+++

x8jt, bridle side. Approved by the human 2026-10-04 (via advisor doc-review, m-4238): 'Add a bridle command to perform the review on a document immediately and a button in the UI also to request the review. Comments that have been sent for review should be marked as such and not resent if the button is pressed again, unless requested.' Sent mark lives in the file ('A'). Thread IDs deferred. Spec: ticket x8jt (docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md), last section.
Built already (br-aj9d, 941312c): crates/bridle-daemon/src/doc_watch.rs (30 s loop; a thread is pending when its last reply is the human's; after [review] quiet_minutes the pending threads go as one batch to agent doc-<stem>, role document-reviewer); registry .bridle/review-documents.txt via 'bridle review add|remove|list'; docs in daemon.md 'Document review'. br-5paw (gateway read/write/commit of one document) is the gateway pattern to follow.
Goal: (1) 'bridle review now <path> [--resend]' sends the document's pending threads to its agent at once, skipping the quiet period (a daemon endpoint the CLI calls, since the daemon owns the agent). (2) Whenever bridle sends a batch (quiet period or now), it appends ' · sent HH:MM' (24h, US Eastern per the human-timezone rule) to the line of each thread's newest human entry; later sends skip entries already marked sent unless --resend. Make sure the pending-thread detection and the agent's own edits are not confused by the mark. (3) A gateway route the UI button calls (same semantics, resend flag) with ts-rs types, auth like other /api/v1 routes; the bridle-ui button ui-c39e waits on it, so keep the route shape simple and document it.
Docs: daemon.md, cli.md, human-web-ui.md, CHANGELOG. Tests: now skips the quiet period; sent mark appended once; resend resends; mark doesn't re-trigger the watcher. Acceptance: just check passes. Model: Sonnet. Migration: none (the mark is plain text in documents; old threads without it count as unsent). Out of scope: thread IDs, the UI.

## Thread

### note · external:orchestrator · 2026-10-04T02:26:49.658Z
Change from the human (relayed verbatim by advisor doc-review, m-4244): "Let's do YYYY-mm-Dr HH:MM for sent". The sent mark is date and time: "· sent 2026-10-04 21:14" (the human's local time, per rule human-timezone), not just HH:MM. The x8jt ticket is updated.

### note · agent:manager-2 · 2026-10-04T02:26:51.975Z
Change from the human (via orchestrator m-4246): the sent mark is ' · sent YYYY-MM-DD HH:MM' (24h, US Eastern, with the date), not HH:MM. Update the spec text accordingly in code, docs and tests.

### note · agent:review-now · 2026-10-04T02:52:21.955Z
done: bridle review now + sent marks + gateway route POST /api/v1/projects/{project}/review; just check green (1137 tests) on 1e-commit below, then merged main (only a new ticket doc came in, no re-run); 6c63cf4

### note · agent:manager-2 · 2026-10-04T02:58:41.803Z
main moved again (ticket docs only). Merge main into your branch and message me the tip; I'll land right away. No need to re-run just check if only docs came in.
