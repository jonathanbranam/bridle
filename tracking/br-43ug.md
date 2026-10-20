+++
id = "br-43ug"
title = "Stage watch: an idle, finished designer holds the ticket forever, so a redo after 'changes' never starts"
kind = "bug"
state = "integrated"
created_at = "2026-10-20T00:23:00.303Z"
updated_at = "2026-10-20T01:43:30.283161Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:w43ug",
]
branch = "bridle/w43ug"
commit = "c7d0f900481111ec5f8a4ac17a704995b5f62235"
summary = "stage_watch.rs: only a Starting/Working writer holds the ticket (idle = ended); restart re-adoption takes only a working writer whose stage is unwritten; a writer whose stage was seen written and later un-written by changes is dropped, not alerted. Pure verdict()/holds() helpers carry unit tests for both paths (the tick itself needs a full daemon). Caveat: a fake designer that idles without stage-done now alerts 'ended without' rather than 'timed out', so the timeout integration test accepts either. docs/design/stages.md updated."
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

### note · agent:w43ug · 2026-10-20T01:43:08.292Z
built: 9821272a18a833fcd42708d0a8dea9c21f4b8c7f (branch bridle/w43ug)

### note · agent:w43ug · 2026-10-20T01:43:08.772Z
done: idle writer no longer holds; changes drops writer; check exit 0, 1908 tests; 9821272a18a833fcd42708d0a8dea9c21f4b8c7f

### note · agent:manager-2 · 2026-10-20T01:43:16.305Z
integrated: c7d0f900481111ec5f8a4ac17a704995b5f62235 (branch bridle/w43ug)

### note · agent:manager-2 · 2026-10-20T01:43:30.283Z
cleanup: removed agent w43ug, branch bridle/w43ug
