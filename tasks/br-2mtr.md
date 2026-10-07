+++
id = "br-2mtr"
title = "Workers report missing tools and failed fetches plainly; a research task went to a worker with no web tools"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T20:12:38.062Z"
updated_at = "2026-10-04T21:05:10.745377Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/report-failures"
commit = "340153481e80961a573459ae0caccb7e4a79f8b5"
summary = "Added rule report-task-failures for workers to record and report missing tools and failed fetches with exact errors in task threads and summaries, raising blockers to managers. Managers and orchestrators pass these up as failures to the human. Updated worker, manager, and orchestrator role prompts to implement the rule. Added CHANGELOG entry."
ticket = "2mtr"
+++

Ask 1 ONLY of docs/tickets/open/workers-report-missing-tools-and-failed-fetches-plainly-a-re-2mtr.md (read it). Update the worker role prompt (workflow/base/roles/, the worker role) or add a base rule in workflow/base/rules/ (see missing-tools.md, which this extends): a worker records each missing tool or failed fetch (URL and error) in the task thread and in its done summary, and raises it to the manager. Update the manager and orchestrator roles so they pass these up as failures, not as 'unreachable'. Keep it short; match the style of the existing roles and rules. Check whether rendered copies or a rules index need regenerating (bridle workflow sync / docs) and follow how recent rule changes were committed. Acceptance: just check passes. Model: Haiku. Out of scope: Ask 2 (a web-tools role, a manager tool check): not approved, do not build.

## Thread

### note · external:orchestrator · 2026-10-04T20:13:04.038Z
Approval (the human, ~4:30 PM ET 2026-10-04, relayed by track-web's aide then bridle's aide, m-4801): "Ask that prompt for workers are updated to record these failures and report them". Approved scope is Ask 1 only (workers record each missing tool or failed fetch, with URL and error, in the thread and summary and raise it; managers and orchestrator pass these up as failures). Ask 2 (a role with web tools, manager tool check) awaits the human's decision.

### note · agent:report-failures · 2026-10-04T21:02:48.296Z
done: Workers report missing tools and failed fetches plainly with new rule and updated roles; 768414113efab5714051fc111729ea94c64d8f66

### note · agent:manager-2 · 2026-10-04T21:02:53.036Z
integrated: 340153481e80961a573459ae0caccb7e4a79f8b5 (branch bridle/report-failures)

### note · agent:manager-2 · 2026-10-04T21:05:10.745Z
cleanup: removed agent report-failures, branch bridle/report-failures
