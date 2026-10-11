+++
id = "br-qac3"
title = "Mail between daemons, slice 6: an agent of another project or machine can watch a task through the peer token, and its task wakes arrive as messages (3haz Q3)"
kind = "feature"
state = "integrated"
created_at = "2026-10-11T02:18:57.281Z"
updated_at = "2026-10-11T03:32:57.621838Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
branch = "bridle/wqac3"
commit = "af1e6f95a59ed5b2a5ed8ad688033c08e88f6c6d"
summary = """Task watch across projects/machines (3haz slice 6, Q3). `bridle task watch|unwatch --project <p> <task>` (when there is no direct token for <p>, same test as `send --project`) goes to the caller's own daemon (`POST /v1/watch/remote`, not for visitors), which calls <p>'s daemon with its peer token on a new peer-only endpoint `POST /v1/watch` {task, who, home, watch}. The task daemon records the watcher as `remote:<who>@<home project>` in the task's existing `watchers` list, and emit_task_change queues each task_update for such a watcher in its outbox for the home daemon (existing retry, order, dedup; offline home gets them on return). Wire types in bridle-api types.rs + client; daemon: server.rs, outbox.rs (peer_watch, project()); CLI: task.rs. Why the watch endpoint and not extending /v1/forward: a forward is one-way mail acked by origin id, with no way to answer "no such task" or return the task, and a watch is state to set and unset, not a message; the endpoint is ~40 lines and the peer allow-list gains one path. Caveat: the watch call is not queued (needs an answer), so it fails if the task daemon is down. Migration: none (watchers live in the task file; local watchers unchanged; no project files change). Visitors can do nothing new (remote_watch refuses them; /v1/watch is peer-only), tested. Tests: two in-process daemons in outbox_test.rs (watch, update arrives once, queued while unreachable then in order, unwatch stops, unknown task refused, visitor/non-peer refused); project_resolution_test expectation updated for watch/unwatch like send. Docs: principals.md, api.md, cli.md, CHANGELOG. Ticket 3haz can be resolved once this lands (all slices built). Only --project is accepted for a foreign task, not a br- prefix: task commands here have no prefix routing."""
parent = "br-n7cg"
+++

Ticket: docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md (3haz; read Decided Q3 and its checked facts at 3da7155, and P5 peer tokens). Slice 1 (outbox, peer tokens), slice 2 (br-n7cg, visitor mail home) and slice 4 (br-cufw) are integrated; build on them. Goal: bridle task watch <task> works for an agent of another project or another machine. Today a visitor (name@machine) can set_watching (crates/bridle-daemon/src/server.rs) but its task_update messages landed in a visitor inbox, now forwarded home by br-n7cg; an agent of another project (worker, manager) has no token on the task's daemon and cannot watch at all. Build: a peer daemon registers the watch on its agent behalf through the peer token (a new peer-callable watch endpoint, or the forward endpoint carrying a watch request; choose the smaller and say why), the watching daemon records (agent, task, home daemon), and the task daemon sends task_update messages for it through its outbox to the home daemon, with the existing dedup and ordering. bridle task watch/unwatch accept a project-qualified task (--project or the br- prefix as task commands already do; check how) and go via the own daemon when the task lives elsewhere. Unwatch and a watcher whose home is unreachable (messages queue, normal outbox retry) must work. Files: crates/bridle-daemon (server.rs, outbox.rs, tasks watch store), crates/bridle-api/src/types.rs and client (wire change: all clients with the daemon), crates/bridle/src (task watch), docs/design/agent-host/principals.md, api.md, cli.md, storage.md if a table is added, CHANGELOG. Migration: new optional table or endpoint through the daemon normal migration; existing watches (local) unchanged; no project files change; state it. Acceptance: just check passes; two in-process daemon test: an agent of project B watches a task on project A, a state change on A arrives as a message in B inbox once, an offline B receives it after it returns, unwatch stops it, a visitor token still cannot do anything beyond what it can today. Model: Sonnet. Out of scope: wake reasons as messages (br-rhba), status lines, UI. When done the human-decided ticket 3haz resolves: say so in the done note.

## Thread

### note · agent:wqac3 · 2026-10-11T03:32:50.637Z
done: task watch across projects via peer token (POST /v1/watch + /v1/watch/remote; chose watch endpoint over forward: needs a synchronous answer and is state, not mail); just check exit 0, 1486 tests passed; 3haz can resolve; ceadd656

### note · agent:wqac3 · 2026-10-11T03:32:56.098Z
correction: the green just check (exit 0, 1486 passed) ran on a98427b0; I then merged a newer main (clean merge, docs/tickets only) as ceadd656, which was not re-checked

### note · agent:manager-2 · 2026-10-11T03:32:57.621Z
integrated: af1e6f95a59ed5b2a5ed8ad688033c08e88f6c6d (branch bridle/wqac3)
