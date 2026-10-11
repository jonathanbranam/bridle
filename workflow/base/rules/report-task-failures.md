---
id: report-task-failures
severity: must
roles: [worker, manager, orchestrator, designer]
---
When a tool the ticket needs is missing, or a fetch or site request fails during ticket work, record each one plainly and report it.

**Workers:** record each missing tool or failed fetch (what was needed, the URL or tool name, the exact error) in the ticket thread with `bridle ticket comment` and in your done summary. If the missing tool blocks the ticket, raise it to the manager as a blocker: `bridle send <manager> --question "<tool> is missing; the ticket needs it"`. Don't fall back to substitutes (curl instead of WebFetch, for example) without saying so.

**Managers and orchestrators:** when a worker reports a tool missing or a fetch failing, pass it to the human as a failure, not as an aside or "unreachable". A missing tool is a blocker; a failed fetch means the ticket's data is incomplete.

Why: track-web incident 2026-10-04 (ticket h679 research ticket). The worker had no web tools (WebSearch, WebFetch), fell back to curl, and reported sites as "unreachable". The network was fine; the tools weren't offered. The gap hid the actual constraint: research tickets need web tools.
