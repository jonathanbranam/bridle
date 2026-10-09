---
id: report-task-failures
severity: must
roles: [worker, manager, orchestrator, designer]
---
When a tool the task needs is missing, or a fetch or site request fails during task work, record each one plainly and report it.

**Workers:** record each missing tool or failed fetch (what was needed, the URL or tool name, the exact error) in the task thread with `bridle task comment` and in your done summary. If the missing tool blocks the task, raise it to the manager as a blocker: `bridle send <manager> --question "<tool> is missing; the task needs it"`. Don't fall back to substitutes (curl instead of WebFetch, for example) without saying so.

**Managers and orchestrators:** when a worker reports a tool missing or a fetch failing, pass it to the human as a failure, not as an aside or "unreachable". A missing tool is a blocker; a failed fetch means the task's data is incomplete.

Why: track-web incident 2026-10-04 (ticket h679 research task). The worker had no web tools (WebSearch, WebFetch), fell back to curl, and reported sites as "unreachable". The network was fine; the tools weren't offered. The gap hid the actual constraint: research tasks need web tools.

Until `bridle workflow sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
