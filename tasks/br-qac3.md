+++
id = "br-qac3"
title = "Mail between daemons, slice 6: an agent of another project or machine can watch a task through the peer token, and its task wakes arrive as messages (3haz Q3)"
kind = "feature"
state = "open"
created_at = "2026-10-11T02:18:57.281Z"
updated_at = "2026-10-11T02:19:34.703772Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
parent = "br-n7cg"
+++

Ticket: docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md (3haz; read Decided Q3 and its checked facts at 3da7155, and P5 peer tokens). Slice 1 (outbox, peer tokens), slice 2 (br-n7cg, visitor mail home) and slice 4 (br-cufw) are integrated; build on them. Goal: bridle task watch <task> works for an agent of another project or another machine. Today a visitor (name@machine) can set_watching (crates/bridle-daemon/src/server.rs) but its task_update messages landed in a visitor inbox, now forwarded home by br-n7cg; an agent of another project (worker, manager) has no token on the task's daemon and cannot watch at all. Build: a peer daemon registers the watch on its agent behalf through the peer token (a new peer-callable watch endpoint, or the forward endpoint carrying a watch request; choose the smaller and say why), the watching daemon records (agent, task, home daemon), and the task daemon sends task_update messages for it through its outbox to the home daemon, with the existing dedup and ordering. bridle task watch/unwatch accept a project-qualified task (--project or the br- prefix as task commands already do; check how) and go via the own daemon when the task lives elsewhere. Unwatch and a watcher whose home is unreachable (messages queue, normal outbox retry) must work. Files: crates/bridle-daemon (server.rs, outbox.rs, tasks watch store), crates/bridle-api/src/types.rs and client (wire change: all clients with the daemon), crates/bridle/src (task watch), docs/design/agent-host/principals.md, api.md, cli.md, storage.md if a table is added, CHANGELOG. Migration: new optional table or endpoint through the daemon normal migration; existing watches (local) unchanged; no project files change; state it. Acceptance: just check passes; two in-process daemon test: an agent of project B watches a task on project A, a state change on A arrives as a message in B inbox once, an offline B receives it after it returns, unwatch stops it, a visitor token still cannot do anything beyond what it can today. Model: Sonnet. Out of scope: wake reasons as messages (br-rhba), status lines, UI. When done the human-decided ticket 3haz resolves: say so in the done note.
