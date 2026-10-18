+++
id = "br-65qh"
title = "A worker can't mark its own ticket built: forbidden, agent lifecycle endpoints (its claim was gone)"
kind = "bug"
state = "built"
created_at = "2026-10-18T13:09:56.799Z"
updated_at = "2026-10-18T20:53:10.683711Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:w65qh",
]
priority = "urgent"
priority_at = "2026-10-18T13:10:06.455979Z"
branch = "bridle/w65qh"
commit = "7fe54a69"
summary = """Refusal of a worker's `ticket built` on a ticket it has not claimed now says "ticket is not claimed by you" (server.rs built_task); the lifecycle_test (br-2fqg) asserts the message. The claim loss was already fixed by br-4n3v (stale-claim sweep released claims of lost/running agents; a restart marks all agents lost), which has its own tests (lease sweep keeps lost/working/idle claims; worker's own built works), so I added no new regression test. Caveat: just check fails twice in the full run on unrelated upgrade_test::a_drain_starting_during_a_spawn_restarts_promptly (passes alone, 8s); 1628/1629 passed."""
ticket = "65qh"
+++

docs/tickets/open/a-worker-can-t-mark-its-own-ticket-built-forbidden-agent-lif-65qh.md

## Thread

### note · external:advisor/tj7q · 2026-10-18T13:10:06.455Z
priority: normal -> urgent

### note · external:advisor/tj7q · 2026-10-18T20:21:45.006Z
From advisor tj7q, with the human's go: a second live case, br-fyww. wfyww claimed it at 17:02:07Z (event 150477); by ~17:50Z the row is 'planned' again while wfyww is still alive and running just check, and there is NO task.state event for claimed -> planned (silent drop). No daemon restart in between. Suspect: the claim lease ([daemon] claim_lease_after, 10 min, renewed by agent activity) expiring while the worker idles on a long just check. Details appended to the ticket on main (cff22610).

### note · agent:w65qh · 2026-10-18T20:53:09.792Z
built: 7fe54a69 (branch bridle/w65qh)

### note · agent:w65qh · 2026-10-18T20:53:10.683Z
done, but just check NOT green: 7fe54a69. Refusal now 'ticket is not claimed by you' (+ test assertion, CHANGELOG). Claim loss was br-4n3v (already has tests), so no new regression test. Full check failed twice (1628/1629 passed of 1879) on unrelated upgrade_test::a_drain_starting_during_a_spawn_restarts_promptly (passes alone in 8s; 17-30s timeout under full run). Marked built; your call whether to land.
