---
id: br-w8jm
title: ticket ready makes a missing row, accepts a qualified id, and skips dropped rows
kind: fix
created: 2026-10-13
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Three snags met tonight (orchestrator, 2026-10-13) while scheduling br-vdu8, br-tgxy and br-dgef on the human's go:

1. **No CLI makes a row for a ticket file that has none.** Older ticket files (made before every ticket got a row, 22ab) fail `bridle ticket ready` with "ticket <id> has no row on this daemon". The PdM couldn't ready them; the orchestrator made rows with a raw `POST /v1/tasks` carrying `ticket`. Wanted: `ticket ready` (or a small `ticket row <id>`) files the missing row from the file's title and kind, as `ticket new` does.
2. **A qualified id in `ticket` doubles the prefix.** `ticket: "br-vdu8"` made row `br-br-vdu8`; `ticket new` passes the bare id. The daemon should accept either and strip its own prefix.
3. **`ticket ready` matches a dropped row first.** `ready_cmd` (`crates/bridle/src/ticket.rs`) takes the first row whose `ticket` names the id, so a dropped duplicate blocks it ("task br-br-vdu8 is dropped, not pending"). It should prefer a pending row, or skip dropped ones. Worked around by repointing the dropped rows' `ticket` to `dropped-dup`.

Also seen: a cross-project `blocked_by` (ui-3mxz blocked by br-vdu8) shows startable in bridle-ui's queue; ui-3mxz is held by hand instead. Whether the queue should honour it is a question for the PdM, not part of this fix.

Verify: a ticket file with no row readies with `bridle ticket ready <id>`; a POST with a qualified `ticket` gets the bare row id; a dropped duplicate row doesn't stop `ticket ready`.
