+++
id = "br-ggnp"
title = "Integrating a ticket leaves its file in docs/tickets/open/; resolve is left to the orchestrator"
kind = "bug"
state = "pending"
created_at = "2026-11-17T13:21:36.630Z"
updated_at = "2026-11-17T13:21:36.632715Z"
created_by = "external:aide@remus"
watchers = ["external:aide@remus"]
+++

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
