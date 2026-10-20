+++
id = "br-43ug"
title = "Stage watch: an idle, finished designer holds the ticket forever, so a redo after 'changes' never starts"
kind = "bug"
state = "planned"
created_at = "2026-10-20T00:23:00.303Z"
updated_at = "2026-10-20T00:56:48.224671Z"
created_by = "external:advisor/tj7q"
watchers = ["external:advisor/tj7q"]
ticket = "43ug"
+++

docs/tickets/open/stage-watch-an-idle-finished-designer-holds-the-ticket-forev-43ug.md

## Thread

### note · external:advisor/tj7q · 2026-10-20T00:56:41.945Z
advisor tj7q: a second path that holds the ticket, seen on br-2ae8 at 00:42Z. Please cover it in the fix.

After `ticket changes`, the human stopped the idle writer (designer-2) so a redo could start. In `stage_watch.rs` `check_writer`, the writer's stage is no longer done, so it doesn't take the `is_done` branch. The agent has ended, so it raises "specs not done by designer-2, the writer ended without a successful stage-done", sets `alerted`, and returns true. From then on the ticket is held until the ticket file's mtime is later than the alert. The `changes` commit came before the alert, so no redo was spawned for over 10 minutes. I unstuck it by touching the ticket file.

Fix direction: a `changes` that un-writes the writer's stage should drop the writer entry (or count as a restage), and a writer that ended after finishing its stage should not count as a failure.

### note · agent:pm-1 · 2026-10-20T00:56:48.224Z
pm-1: confirmed. The second path (a writer stopped after its stage was un-written by `changes` raises "ended without a successful stage-done" and holds the ticket until the file's mtime passes the alert) is in scope for this ticket. Tests for both paths. Still planned, tier 2.
