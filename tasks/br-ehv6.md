+++
id = "br-ehv6"
title = "Comment threads: explicit ASCII status and time zone on every entry, thread IDs, resolve, 'human via <agent>"
kind = "feature"
state = "planned"
created_at = "2026-10-04T15:25:24.347Z"
updated_at = "2026-10-04T19:41:54.263841Z"
created_by = "external:advisor/doc-review"
watchers = ["external:advisor/doc-review"]
summary = "Rust/daemon/CLI/rules side of ehv6. doc_watch.rs: entries carry one ASCII status `[pending|sent|read YYYY-MM-DD HH:MM EDT]` (pending = none or [pending]); thread IDs `c<n>` assigned at send (highest+1) and shown to the agent; `is_human` is now exactly `human` / `human via <agent>`; resolved threads never go; tick sweep turns `[sent]` into `[read]` once the doc agent has no unread message (spawned agents read at once; stateless, restart-safe) and rewrites old middle-dot marks. `bridle review resolve <path> c3` appends `resolved by human, <stamp>`. Role document-reviewer.md rewritten for the new format; new rules ascii-in-editable-text and human-via-agent; docs (daemon.md, cli.md, human-web-ui.md, x8jt note) and CHANGELOG updated. Decisions: Eastern zones only (YAGNI); no new gateway route, since the UI can save marks/resolve lines through the existing document save; cleanup of resolved threads not built (ticket leaves its shape to the planner)."
+++

original id: ehv6
Build docs/tickets/open/comment-threads-explicit-ascii-status-and-time-zone-on-every-ehv6.md (read it and the thread; the human approved it with caveats now in the ticket: ASCII only, short US zone abbreviation on stamps, agent names not roles in 'via', thread IDs for the CLI, read-by-the-agent included; includes two new rules). Touches the same files as br-jrm2 (Document.tsx, comments.ts, doc_watch.rs): runs AFTER br-jrm2 merges (dependency edge). Acceptance: just check passes, plus the checks the ticket states. Model: Sonnet. Size is large: if too big for one branch, report on the thread and ask the manager to split (rules + CLI/daemon first, UI second). Out of scope: anything not in the ticket.

## Thread

### note · external:advisor/doc-review · 2026-10-04T15:25:54.315Z
From the human, via advisor (doc-review), 2026-10-04: "With those caveats, I approve this." The caveats are folded into the ticket: ASCII only, a short US zone abbreviation on stamps, agent names (not roles) in 'via', thread IDs for the CLI, and read-by-the-agent included.

### note · agent:comment-threads · 2026-10-04T19:41:54.208Z
Done (Rust side). just check green: 1175 passed, 5 skipped (band 1154); no flaky reruns. Tip: see message.

Proposed UI-side task for bridle-ui (Document.tsx, comments.ts), no daemon/gateway change needed:
- Write the new format with the existing document save: new thread header `> [!comment] c<n> human, YYYY-MM-DD HH:MM EDT, on "..." [pending YYYY-MM-DD HH:MM EDT]` (n = highest c<n> in the file + 1); replies `> **human, <stamp>:** text [pending <stamp>]`. Stamp = US Eastern with EST/EDT, ASCII only.
- Parse marks `[pending|sent|read <stamp>]` at the end of an entry's first line (and the legacy ` U+00B7 sent <stamp>`); show the state and time on each entry.
- Agent entries are `**<agent name>, <stamp>:**`; when the human opens a thread in the UI, append `[read <stamp>]` to the agent's unmarked entries. Drop the old `@human (read)` tag suffix.
- Resolve button: appends `>` and `> **resolved by human, <stamp>**` to the thread (same as `bridle review resolve`); resolved threads (any `> **resolved by` line) render collapsed.
- Human entries by an agent read `human via <agent>`; only `human` / `human via ...` are the human.
Not built: deleting resolved threads (cleanup); the ticket leaves its shape open.

### note · agent:comment-threads · 2026-10-04T19:41:54.263Z
done: ehv6 Rust side (marks, IDs, read-by-agent, resolve, 2 rules, role, docs); UI task proposal in the task comment; 5dc72866
