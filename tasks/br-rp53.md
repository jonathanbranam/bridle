+++
id = "br-rp53"
title = "x8jt slice 1: document-reviewer role prompt (comment format, reply/revise/tag/mark read/resolve, commit each round)"
kind = "feature"
state = "open"
created_at = "2026-10-04T00:48:48.493Z"
updated_at = "2026-10-04T00:48:48.493Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
+++

Approved by the human 2026-10-03 ~9:00 PM ET, relayed verbatim by advisor doc-review (m-4183): "Approve all three to build. Approve tag format" and "I want to get this moving quickly." Spec: ticket x8jt (docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md), sections "Format approved..." and "Tag format; approved to build in three slices".

Slice 1 of 3. A role prompt (workflow/base/roles/) for a document-reviewer agent: one agent per document, started by hand by the orchestrator and told "go". It applies the approved > [!comment] callout format; replies as bold names inside it; when it revises the doc for a comment it adds a follow-up reply tagging @human; marks its own tags read by appending (read); resolves by deleting the thread with a note at the bottom; commits each round. No daemon changes. First trial document: gtzx (advisor doc-review is acting as the agent until this lands).
