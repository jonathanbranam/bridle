+++
id = "br-2fqg"
title = "Worker role says run 'ticket built', but the daemon refuses it for workers (forbidden: agent lifecycle endpoints)"
kind = "bug"
state = "built"
created_at = "2026-10-17T17:44:10.916Z"
updated_at = "2026-10-17T19:09:49.274671Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w2fqg",
]
branch = "bridle/w2fqg"
commit = "f850c341147713dbe9082909d6d653368882158f"
summary = "The daemon already lets a worker build its own CLAIMED ticket; the refusal came because spawned workers' tickets stay planned and unclaimed. worker.md now says to run 'ticket claim' first; lifecycle_test covers claimed-ok / unclaimed-403. CHANGELOG added. Did not widen the daemon (a worker building any planned ticket would be too broad)."
ticket = "2fqg"
+++

docs/tickets/open/worker-role-says-run-ticket-built-but-the-daemon-refuses-it-2fqg.md

## Thread

### note · external:orchestrator · 2026-10-17T17:44:48.901Z
From orchestrator: filed and readied br-2fqg (bug, under br-mhjn): worker.md tells workers to run 'ticket built', the daemon refuses it for workers; hit twice today (3stk, z5kz). Yours to schedule.

### note · agent:w2fqg · 2026-10-17T19:09:49.231Z
built: f850c341147713dbe9082909d6d653368882158f (branch bridle/w2fqg)

### note · agent:w2fqg · 2026-10-17T19:09:49.274Z
done: worker.md now claims the ticket first (daemon already allowed own-claimed built; cause was unclaimed planned ticket); just check exit 0, 1825 tests passed; f850c341147713dbe9082909d6d653368882158f
