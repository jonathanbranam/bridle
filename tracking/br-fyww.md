+++
id = "br-fyww"
title = "Stages: landing a designer's stage branch marked the whole v2 ticket integrated"
kind = "bug"
state = "planned"
created_at = "2026-10-18T16:33:40.386Z"
updated_at = "2026-10-18T19:13:37.581084Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wfyww",
]
priority = "critical"
priority_at = "2026-10-18T17:00:45.320227Z"
summary = "done_task refuses a v2 ticket not prepared (stages::prepared read from checkout docs/tickets via TicketManager::with_tickets_dir; stage_watch::unprepared builds the message); reopen_task sends such a ticket to pending, else reopened. Manager role text, stages/cli/storage docs, CHANGELOG, 2 integration tests in stage_watch_test. Caveat: the land path calls done_task, so landing a designer branch now errors at the done step; br-2ae8 not reopened (step 4)."
ticket = "fyww"
+++

docs/tickets/open/stages-landing-a-designer-s-stage-branch-marked-the-whole-v2-fyww.md

## Thread

### note · external:advisor/tj7q · 2026-10-18T16:33:45.986Z
priority: normal -> urgent

### note · agent:pm-1 · 2026-10-18T16:34:25.731Z
Brief (pm-1). Model: Sonnet. Read docs/design/stages.md, the task state-transition code in crates/bridle-daemon (grep integrated, the land/integrate path used by `bridle ticket`/`bridle task` and the manager's land step), and br-upxq's Decision (the daemon will merge stage branches itself).
Build: the daemon refuses any transition to `integrated` for a version 2 ticket that is not prepared and built (a clear error naming why); the manager's land path hits the same guard so it cannot mark a proposal-only branch integrated. Then update the manager land rule/step text to say designer stage branches don't change the state (check workflow/base/ rules and roles for the land step; keep edits minimal).
Test: landing a proposal-only branch of a pending v2 ticket leaves it `pending`; a normal v1 ticket still integrates. Docs: stages.md, CHANGELOG. Acceptance: just check.
Out of scope: br-upxq's merge-by-daemon (separate ticket); a `pending` return path via reopen (br-2ae8 was fixed by hand).
Same files as br-upxq: run after it, not alongside.

### note · external:advisor/tj7q · 2026-10-18T16:47:12.706Z
held: The human, to advisor tj7q: "Fix fyww first and decide the solution now. Don't leave to worker." The design is being decided with the human now; it goes into the ticket before any worker starts. (until the human approves fyww's design)

### note · external:advisor/tj7q · 2026-10-18T17:00:45.267Z
unheld

### note · external:advisor/tj7q · 2026-10-18T17:00:45.320Z
priority: urgent -> critical

### note · external:advisor/tj7q · 2026-10-18T17:00:49.527Z
From the human, via advisor tj7q: "Fix fyww first and decide the solution now. Don't leave to worker." and, on the design, "yes approve". br-fyww now carries the human's ## Decision (commit 26ad0886): done refuses a version 2 ticket that isn't prepared; reopen of one gives pending; the manager never runs done for a bridle/designer-* branch; then reopen br-2ae8. Unheld and critical; please put it next, building exactly the Decision.

### note · agent:pm-1 · 2026-10-18T17:00:52.874Z
pm-1: now critical and first in the queue. Build exactly the human's ## Decision in the ticket file (commit 26ad0886); it supersedes my brief where they differ (done refuses an unprepared v2 ticket; reopen of one gives pending; the manager never runs done for a bridle/designer-* branch). Keep my brief's test, docs, CHANGELOG and just check. The "run after br-upxq" ordering is dropped: fyww goes first.

### note · external:orchestrator · 2026-10-18T17:01:21.353Z
From orchestrator: br-fyww is critical and next (the human, via advisor tj7q: "Fix fyww first and decide the solution now. Don't leave to worker."). Build exactly the ticket's ## Decision (commit 26ad0886): `done` refuses a version 2 ticket that isn't prepared; reopening one gives pending; the manager never runs `done` for a bridle/designer-* branch. No design choices for the worker. Do not run `done`/land on any bridle/designer-* branch from now on. After it lands, br-2ae8 gets reopened (to pending); I'll do that or check it.

### note · external:orchestrator · 2026-10-18T17:01:30.050Z
settle skipped by external:orchestrator: the human asked: 'Fix fyww first and decide the solution now. Don't leave to worker.' (human via advisor tj7q, 2026-10-18 ~1 PM ET); critical, next

### note · agent:wfyww · 2026-10-18T17:37:32.082Z
wfyww: implemented and committed on bridle/wfyww (guard in done_task, reopen to pending, manager role, docs, tests pass). main merged in; final just check is running; will mark built and report when it exits 0.
