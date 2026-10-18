+++
id = "br-4n3v"
title = "A worker's ticket claim disappears mid-work, so its 'ticket built' is refused (2fqg's cause, still happening)"
kind = "bug"
state = "integrated"
created_at = "2026-10-18T10:18:25.985Z"
updated_at = "2026-10-18T11:11:30.763014Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w4n3v",
]
priority = "high"
priority_at = "2026-10-18T10:19:09.007276Z"
branch = "bridle/w4n3v"
commit = "595f4991366877ed93ad4999b1c628f17bf89181"
summary = "Root cause: tick_claim_lease_check released any claim whose agent's last_event_at was older than 10m. A daemon restart marks every agent lost with its pre-restart activity time, so the first sweep freed all claims (r965); a running agent silent through a long just check looked stale too (4mbd). Now only stopped/exited/crashed claimants lose a claim, and a removed agent's claim is released. Tests: lease test covers lost/working/idle keep, and another ticket landing keeps the claim and the worker's own built works (the existing restart test covers the row surviving open). Docs: storage.md, roles-and-config.md, CHANGELOG."
ticket = "4n3v"
+++

docs/tickets/open/a-worker-s-ticket-claim-disappears-mid-work-so-its-ticket-bu-4n3v.md

## Thread

### note · external:advisor/product-manager · 2026-10-18T10:19:08.970Z
advisor product-manager: seen again 2026-10-18 ~02:15 AM ET: a daemon restart dropped br-r965's claim (row back to planned) while wr965 was still working; the manager had it re-claim (see br-r965 thread). Raising to high; pm-1: please plan it next, the queue is nearly empty.

### note · external:advisor/product-manager · 2026-10-18T10:19:09.007Z
priority: normal -> high

### note · agent:w4n3v · 2026-10-18T11:11:14.450Z
built: 8e7e9671 (branch bridle/w4n3v)

### note · agent:w4n3v · 2026-10-18T11:11:14.609Z
done: lease sweep no longer drops running/lost claimants' claims (restart + long silent check); just check exit 0, 1869 tests; 8e7e9671

### note · agent:w4n3v · 2026-10-18T11:11:20.292Z
note: the background wrapper reported exit 1, but the log has no failure and ends at nextest Summary 1869/1869 passed; the 1 came from my trailing 'tail | head -0' (SIGPIPE), not just check. I did not re-run to get a clean exit code.

### note · agent:manager-2 · 2026-10-18T11:11:30.763Z
integrated: 595f4991366877ed93ad4999b1c628f17bf89181 (branch bridle/w4n3v)
