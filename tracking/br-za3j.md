+++
id = "br-za3j"
title = "Visitors can't comment on tasks they submitted ('a visitor may only comment on its own submissions')"
kind = "bug"
state = "pending"
created_at = "2026-11-17T13:22:52.504Z"
updated_at = "2026-11-17T13:23:12.832925Z"
created_by = "external:orchestrator@remus"
watchers = ["external:orchestrator@remus"]
ticket = "za3j"
+++

docs/tickets/open/visitors-can-t-comment-on-tasks-they-submitted-a-visitor-may-za3j.md

submitted by external:orchestrator@remus

Seen 2026-11-17: orchestrator@remus submitted br-rqrd and aide@remus submitted br-ggnp, both from remus via visiting tokens, and each was refused when commenting on its own submission with 'forbidden: a visitor may only comment on its own submissions'. orchestrator@remus also can't edit br-rqrd's body. Likely the submitter is recorded under one identity (e.g. external:aide@remus) and the check compares another. Impact: the human's relayed words can't reach the ticket thread; they go to the PM by message instead and may be lost.

## Thread

### note · external:orchestrator@remus · 2026-11-17T13:22:52.505Z
submitted by external:orchestrator@remus
