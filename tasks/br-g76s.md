+++
id = "br-g76s"
title = "Load-hold notes: one per machine, name bridle-owned top consumers, honest text, load.hold.started/ended events, escalate a long hold (n4w4 recs 4, 5)"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T01:41:12.914Z"
updated_at = "2026-10-10T23:05:18.171295Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
    "external:advisor/product-manager",
]
branch = "bridle/wg76s"
commit = "3101ce1de78fb3b1265cfe6a297ea32ce60f4060"
summary = 'Load-hold note (load.rs) no longer claims the daemon resumes held spawns; the refusal text (supervisor.rs) says refused, not queued. A bridle-owned top-3 consumer (ps, claude, fake-claude, bridle*) adds the "find the cause now" line. New events load.hold.started (load, per-core, consumers) and load.hold.ended (held_secs). Escalation: >60 min held of the last 120 sends the orchestrator and the human (Note, same path as disk.rs) one message per hour. Caveat: the "refused spawn of a critical task" trigger is NOT built, since SpawnRequest carries no task; needs a follow-up if wanted. Item 5 was moved to tnyt (already on main). Docs: operating-model.md (where the load watch lives), daemon.md pointer, orchestrator.md, CHANGELOG. Tests use the fake LoadSource and a fake clock.'
parent = "br-n4w4"
+++

Ticket: docs/tickets/open/postmortem-bridle-s-own-ps-polling-every-daemon-test-daemons-n4w4.md (read "Why it went on for ~26 hours" and Recommendations 4 and 5; also ticket xypj, one note per machine). Human approved all recommendations.

Code: crates/bridle-daemon/src/load.rs (the watch, the note, LoadSource, top_consumers), supervisor.rs `refuse_if_load_held` (~line 505), workflow/base/roles/orchestrator.md lines ~94-96 (the load-note paragraph).

Do:
1. The note's text: remove the false "the daemon resumes them itself". A held spawn is refused with a conflict and not queued; say "new spawns are refused until the load falls; retry then".
2. Name bridle's own processes: when any of the top three consumers is a bridle-owned process (comm `ps`, `bridle`, `fake-claude`, `claude`, or a test binary under a worktree's target/debug), add one line "a bridle process is a top consumer: <name> <pct>%. Find the cause now, do not wait." Pure string match over the consumers already collected; no new fork.
3. Events: emit `load.hold.started` and `load.hold.ended` events (use the existing events mechanism; see how other daemon events are recorded) with load, per-core load and the consumers on start, and the held duration on end, so total hold time can be summed. Document in docs/design/agent-host/daemon.md.
4. Escalation: if spawns have been held more than 60 minutes in the last 2 hours, or a refused spawn belongs to a task with priority critical, send the orchestrator one message (once per hour at most) "Spawns have been held N minutes of the last 120: investigate the cause now" and write the same line into the morning list the way other daemon notes reach the human (find the existing path; if none exists, only message the orchestrator and say so on the thread).
5. One note per machine, not per daemon (xypj): if it needs a machine-wide file (e.g. a lock or stamp file under ~/.bridle/ with the last-sent time), use that; the first daemon to cross sends, the others skip within 5 minutes. Keep it simple; say on the thread if you judge this part needs a separate design.
6. workflow/base/roles/orchestrator.md: change the load-note rule from "wait" to "wait; if the note names a bridle process, or the hold lasts or escalates, find the cause now" (keep the surrounding wording).

Files: crates/bridle-daemon/src/load.rs, supervisor.rs (only the note/refusal text), docs/design/agent-host/daemon.md, workflow/base/roles/orchestrator.md, CHANGELOG.md.

Acceptance: just check passes; unit tests with a fake LoadSource for: note text without the false claim, bridle-process line present/absent, started/ended events, escalation after 60 minutes (fake clock), machine-wide dedupe.

Model: Sonnet. Migration: the orchestrator role text reaches projects through the normal `bridle workflow sync`; no file changes in projects. Out of scope: queueing held spawns (the note stops claiming it), the governor threshold, the test harness.

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.565Z
watching the task

### note · agent:pm-1 · 2026-10-09T22:06:27.241Z
pm-1 brief change: item 5 (one note per machine, the machine-wide stamp) moves to br-tnyt, which runs first (same file, load.rs). Skip item 5 here; merge main first and build on tnyt's stamp and quiet-period config. Everything else stands.

### note · external:advisor/product-manager · 2026-10-09T22:06:42.483Z
advisor (product-manager): added edge: blocked by br-tnyt, so pm-1's 'tnyt runs first' is enforced (same file, load.rs).

### note · agent:wg76s · 2026-10-10T23:04:30.286Z
done: load-hold note text/bridle-process line/load.hold.* events/long-hold escalation (critical-task trigger not built, no task on spawn requests); just check exit 0, 1475 tests; 37a6c796

### note · agent:wg76s · 2026-10-10T23:04:32.708Z
Critical-task trigger not built: spawn requests carry no task id. Everything else done; see summary.

### note · agent:manager-2 · 2026-10-10T23:04:38.176Z
integrated: 3101ce1de78fb3b1265cfe6a297ea32ce60f4060 (branch bridle/wg76s)

### note · agent:manager-2 · 2026-10-10T23:05:18.171Z
cleanup: removed agent wg76s, branch bridle/wg76s
