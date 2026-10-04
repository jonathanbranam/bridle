+++
id = "br-2mtr"
title = "Workers report missing tools and failed fetches plainly; a research task went to a worker with no web tools"
kind = "bug"
state = "planned"
created_at = "2026-10-04T20:12:38.062Z"
updated_at = "2026-10-04T20:13:14.530241Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: 2mtr
Ask 1 ONLY of docs/tickets/open/workers-report-missing-tools-and-failed-fetches-plainly-a-re-2mtr.md (read it). Update the worker role prompt (workflow/base/roles/, the worker role) or add a base rule in workflow/base/rules/ (see missing-tools.md, which this extends): a worker records each missing tool or failed fetch (URL and error) in the task thread and in its done summary, and raises it to the manager. Update the manager and orchestrator roles so they pass these up as failures, not as 'unreachable'. Keep it short; match the style of the existing roles and rules. Check whether rendered copies or a rules index need regenerating (bridle workflow sync / docs) and follow how recent rule changes were committed. Acceptance: just check passes. Model: Haiku. Out of scope: Ask 2 (a web-tools role, a manager tool check): not approved, do not build.

## Thread

### note · external:orchestrator · 2026-10-04T20:13:04.038Z
Approval (the human, ~4:30 PM ET 2026-10-04, relayed by track-web's aide then bridle's aide, m-4801): "Ask that prompt for workers are updated to record these failures and report them". Approved scope is Ask 1 only (workers record each missing tool or failed fetch, with URL and error, in the thread and summary and raise it; managers and orchestrator pass these up as failures). Ask 2 (a role with web tools, manager tool check) awaits the human's decision.
