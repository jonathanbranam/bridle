---
id: br-ggnp
title: Integrating a ticket leaves its file in docs/tickets/open/; resolve is left to the orchestrator
kind: bug
created: 2026-11-17
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

When a ticket is integrated, bridle itself resolves its file: stamps `resolved:`, moves it from `docs/tickets/open/` to `docs/tickets/resolved/` and commits that, in the same step that marks the ticket `integrated`. It must not depend on a manager or orchestrator remembering `bridle ticket resolve`.

The human, 2026-11-17 ~8:25 AM ET (via the flume aide, submitted by aide@remus), verbatim: "strange. That is not something that should be \"up to the orchestrator\". File this as a bug with bridle"

## What happened (project flume, bridle 0.8.0, remus)

- Overnight 15 tickets went `integrated` via `bridle ticket done` (by agent:manager) but their files stayed in `docs/tickets/open/`; only two were moved, by hand.
- The human saw a full `open/` on disk while the UI listed only the two active tickets, and asked why tickets were "missing". The daemon's state was right; the files were wrong.
- The orchestrator moved the 15 by hand (flume commit f876e93) and told its manager to run `ticket resolve` at each land: a convention, not a guarantee.
- CLI help shows the split: `done` "Mark a ticket integrated (merged), recording the merge commit"; `resolve` "Stamp `resolved:` and move an open ticket to docs/tickets/resolved/ (no commit, no task change)".

## Design for the worker

- Where: the integrate path (`land`, and `ticket done`; docs/design/agent-host/operating-model.md step 5 says land does what `ticket done` does). Find the daemon code for it and the code `ticket resolve` uses (crates/bridle/src, crates/bridle-daemon/src).
- The move is a git change on the integration branch in the owner's clone. Reuse the way bridle already commits ticket files (the "Ticket <id>: filed" and "Ticket <id>: resolved" commits seen on main); do not invent a second committer, and never leave the file moved but uncommitted (the human dislikes stray uncommitted files). Respect rule one-pusher-for-the-integration-branch: commit locally; the existing autopush does the rest.
- Idempotent: a ticket already in `resolved/`, with no file, or a file-less row (old tasks) must integrate without error.
- `dropped`: same treatment if the ticket's file is open, with a plain thread note; keep it to what falls out of the same code path, nothing new for it if it does not.
- Backlog in other projects: a check that flags any integrated ticket whose file is still in `open/` (an item in `bridle ticket check`, or the status/doctor surface that already lists similar leftovers), so existing projects' stale files show up. A one-time fix is `bridle ticket resolve <id>` per file, or an automatic migration if migrate.rs already has a pattern for it (ticket xebc prefers automatic: say which on the thread and why). Project files change only by this move, and only in bridle's integrate path.
- Docs: docs/design/cli.md (done/resolve), operating-model.md step 5, and the manager/orchestrator role text if it tells them to run `ticket resolve` after a land.

## Acceptance

Tests: land of a ticket with an open file leaves the file in `resolved/` in one commit on the integration branch with `resolved:` stamped; idempotent re-run; a file-less row lands cleanly; the new check flags a stale open file. `just check` with BRIDLE_* env unset. Model: Sonnet.
