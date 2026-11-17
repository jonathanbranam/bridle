+++
id = "br-ggnp"
title = "Integrating a ticket leaves its file in docs/tickets/open/; resolve is left to the orchestrator"
kind = "bug"
state = "built"
created_at = "2026-11-17T13:21:36.630Z"
updated_at = "2026-11-17T15:37:53.956983Z"
created_by = "external:aide@remus"
watchers = [
    "external:aide@remus",
    "agent:ggnp",
]
branch = "bridle/ggnp"
commit = "df851d85c914c7bea6084ef98fe15d8a9aaf7e00"
summary = """Integrating now resolves the ticket file: server.rs done_task (shared by done and land) calls the new bridle-daemon ticket_file::resolve_and_commit, which stamps resolved:, moves the file (and its stage folder) open/ -> resolved/ and commits "Ticket <id>: resolved" locally via commit_ticket_files (moved from the CLI, one committer; the CLI wraps it with its warning). resolve/find_by_id/set_resolved moved to the daemon too; ticket resolve is unchanged. No file or already resolved is a no-op; a failure is a thread note, never fatal. ticket check warns of integrated tickets whose file is still in open/. Decision: manual fix (ticket resolve <id> + commit), not an automatic migration: the check is advisory and the move needs a commit on the owner's integration branch, which only the integrate path does. Dropped is NOT covered: drop_task is a separate path that does not fall out of this one. Tests in ticket_file.rs."""
ticket = "ggnp"
+++

docs/tickets/open/integrating-a-ticket-leaves-its-file-in-docs-tickets-open-re-ggnp.md

submitted by external:aide@remus

## The ask

When a ticket is integrated (merged), bridle itself should resolve its file: stamp `resolved:`
and move it from `docs/tickets/open/` to `docs/tickets/resolved/`, as part of the same step that
marks the ticket integrated. It should not depend on a manager or orchestrator remembering to run
`bridle ticket resolve` afterwards.

The human, 2026-11-17 ~8:25 AM ET (via the flume aide), on hearing that the orchestrator had
fixed it by having the manager run `ticket resolve` at each land: "strange. That is not
something that should be \"up to the orchestrator\". File this as a bug with bridle"

## What happened (project flume, bridle 0.8.0, remus)

- Overnight 15 tickets went to `integrated` via `bridle ticket done` (by `agent:manager`), but
  their files stayed in `docs/tickets/open/`. Only two (fl-sjdp, fl-danq) were moved, by hand.
- The human saw a full `open/` folder on disk while the UI's tickets page listed only the two
  really-active tickets, and asked why the UI was "missing" tickets. The daemon's state was
  right; the files were wrong.
- The orchestrator moved the 15 by hand (flume commit f876e93) and told its manager to run
  `ticket resolve` at each land: a convention, not a guarantee.
- CLI help shows the split: `done` "Mark a ticket integrated (merged), recording the merge commit
  in its thread"; `resolve` "Stamp `resolved:` and move an open ticket to docs/tickets/resolved/
  (no commit, no task change)".

## Expected

`integrated` (and arguably `dropped`) implies the file is in `resolved/`, done by bridle in the
integrate path. A check (`bridle check` or `status` incident) could flag any integrated ticket
whose file is still in `open/`, to catch the existing backlog in other projects.

## Thread

### note · external:aide@remus · 2026-11-17T13:21:36.632Z
submitted by external:aide@remus

### note · external:orchestrator · 2026-11-17T13:22:27.399Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-17T13:22:27.703Z
Readied by orchestrator (bug, self-evident ask; rule ready-without-asking).

### note · agent:pm-1 · 2026-11-17T13:22:59.648Z
human via aide@flume (relayed by orchestrator@remus, pasted by pm-1 because visitors cannot comment, see br-za3j): the human, 2026-11-17 ~8:30 AM ET, verbatim: "Aren't we using the bridle commands to mark tickets as resolved?? That should handle the move itself; no agent should need to 'remember' to do that" and "also - the commit needs to happen".

Context: on flume the manager integrated 15 tickets with `bridle ticket done`, which by design only marks the row integrated; the file move is a separate `bridle ticket resolve`, which does not commit. Fix asked: integrating via `done` (and `land`) stamps resolved:, moves the file to resolved/ and commits it, in one call. This matches the brief above; no change to it.

### note · agent:ggnp · 2026-11-17T15:37:53.956Z
built: df851d85c914c7bea6084ef98fe15d8a9aaf7e00 (branch bridle/ggnp)
