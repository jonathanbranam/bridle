+++
id = "br-fzwa"
title = "Audit every periodic daemon loop for what it forks or reads per tick; list them with cost in daemon.md (n4w4 rec 3)"
kind = "chore"
state = "planned"
created_at = "2026-10-09T01:41:21.012Z"
updated_at = "2026-10-09T11:04:40.615999Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
    "external:advisor/product-manager",
]
parent = "br-n4w4"
+++

Ticket: docs/tickets/open/postmortem-bridle-s-own-ps-polling-every-daemon-test-daemons-n4w4.md, Recommendation 3 (approved by the human). Also the human's rule in ticket v6kr: always-on monitoring is itself a load source; every always-on measure must be cheap.

Do: read crates/bridle-daemon/src/lib.rs (the `spawn_loop` calls and any other timer: stall, tracker, governor, CI, flush, ports, disk, load, doc watch, wake, mail, schedule if present) and each loop's body. For each, record in a table in docs/design/agent-host/daemon.md (new section "Periodic loops and their cost"): name, interval (production default and the config key), what one tick does (reads: files/DB/network; forks: which commands), whether it does anything when idle (no agents, no tasks), and a cost class: none / reads only / forks. Then fix only what is clearly wasteful and cheap to fix: a loop that forks or walks the filesystem every tick with nothing to do should return early like tick_tracker does now (supervisor.rs tick_tracker_with). Anything larger goes on the task thread as a proposed follow-up with the evidence, not into this change.

Files: docs/design/agent-host/daemon.md, crates/bridle-daemon/src/* only for the early-return fixes (each with a unit test that counts the expensive call with nothing to do), CHANGELOG.md.

Acceptance: just check passes; the table lists every spawn_loop and timer in lib.rs (the worker greps for `spawn_loop`, `interval(` and `sleep(` in crates/bridle-daemon/src and says so on the thread); fixes have tests.

Model: Sonnet. Migration: none. Out of scope: replacing `ps` callers outside the tracker (load.rs top_consumers is per crossing), the test harness (br-6nzj), notes (br-g76s).

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.615Z
watching the task
