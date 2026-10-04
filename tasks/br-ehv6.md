+++
id = "br-ehv6"
title = "Comment threads: explicit ASCII status and time zone on every entry, thread IDs, resolve, 'human via <agent>"
kind = "feature"
state = "planned"
created_at = "2026-10-04T15:25:24.347Z"
updated_at = "2026-10-04T16:36:36.539825Z"
created_by = "external:advisor/doc-review"
watchers = ["external:advisor/doc-review"]
+++

original id: ehv6
Build docs/tickets/open/comment-threads-explicit-ascii-status-and-time-zone-on-every-ehv6.md (read it and the thread; the human approved it with caveats now in the ticket: ASCII only, short US zone abbreviation on stamps, agent names not roles in 'via', thread IDs for the CLI, read-by-the-agent included; includes two new rules). Touches the same files as br-jrm2 (Document.tsx, comments.ts, doc_watch.rs): runs AFTER br-jrm2 merges (dependency edge). Acceptance: just check passes, plus the checks the ticket states. Model: Sonnet. Size is large: if too big for one branch, report on the thread and ask the manager to split (rules + CLI/daemon first, UI second). Out of scope: anything not in the ticket.

## Thread

### note · external:advisor/doc-review · 2026-10-04T15:25:54.315Z
From the human, via advisor (doc-review), 2026-10-04: "With those caveats, I approve this." The caveats are folded into the ticket: ASCII only, a short US zone abbreviation on stamps, agent names (not roles) in 'via', thread IDs for the CLI, and read-by-the-agent included.
