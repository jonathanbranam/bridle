+++
id = "br-aj9d"
title = "x8jt slice 2: bridle notices new comments in documents under review, debounces 5-10 min, starts/resumes that document's agent with the batch"
kind = "feature"
state = "open"
created_at = "2026-10-04T00:48:48.514Z"
updated_at = "2026-10-04T00:48:48.514Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
+++

Approved by the human 2026-10-03 ~9:00 PM ET, relayed verbatim by advisor doc-review (m-4183): "Approve all three to build. Approve tag format" and "I want to get this moving quickly." Spec: ticket x8jt (docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md), sections "Format approved..." and "Tag format; approved to build in three slices".

Slice 2 of 3; builds on slice 1's role. Watch the documents under review; when new comments have been quiet for 5-10 minutes, start or resume that document's agent with the batch. Use the existing agent stop/resume, a simple cap on how many run at once, and an expiry per document agent. Doesn't wait on the rest of r9vh. Keep docs/design in step.
