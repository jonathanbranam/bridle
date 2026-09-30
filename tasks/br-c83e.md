+++
id = "br-c83e"
title = "Human to-dos: tasks assigned to the human, and the orchestrator lists them at startup (ex9q)"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T12:23:53.704Z"
updated_at = "2026-09-30T01:37:12.325645Z"
branch = "bridle/human-todos2"
commit = "8b774caabbf818a588099387bd21504f5af128bf"
summary = "Human to-dos: 'bridle task new --for-human' (NewTaskRequest.for_human) creates the task planned and claimed by the human principal and sends one inbox message pointing at it; the lease check never releases the human's claim; the human finishes with 'bridle task done <id>' with no --commit (allowed only for a human claim; agent claims still need a commit, and main-moved notices are skipped when there is no commit). No when-tag sorting or parsing (document only). Docs: cli.md, coordination.md, orchestrator role startup step lists 'bridle task list --claimed-by human'; CHANGELOG under Unreleased. Tests: lease exemption + bare done + agent refusal in tasks.rs; just check passes (767 tests)."
+++

Ticket: docs/questions/open/*ex9q.md (read it, especially Shape). Goal, KISS: a human to-do is a bridle task assigned to the human; the human marks it done; the orchestrator lists open ones at every start. Build: 1) A way to create a task already assigned to the human: e.g. bridle task new --for human (or --assign human), creating it planned and claimed by the human principal in one step, plus the API field in crates/bridle-api/src/types.rs and the server handler in crates/bridle-daemon. Check the claim lease (br P0-4, coordination.md): a human claim must NOT expire or be reaped; if leases would release it, exempt the human principal. 2) The creating command also sends one inbox message to human pointing at the task (reuse the send --task / --notify path; body must be non-empty). 3) The human finishes it with the existing bridle task done (check it works for a task the human claimed with no commit: allow done without --commit for these, or add bridle task complete if cleaner; pick the simpler). 4) Titles carry a when tag by convention: [at restart], [at next reboot], or none; document only, no parsing needed except sorting them first in the list if trivial. 5) workflow/base/roles/orchestrator.md startup steps: list bridle task list --claimed-by human and tell the human first, the [at restart] ones especially. 6) Docs: docs/design/cli.md, coordination.md, and the orchestrator role. Acceptance: just check passes; tests for create-assigned-to-human, lease exemption, done by the human. Model: Sonnet. Out of scope: mobile/notification surfaces, recurring to-dos, and creating the actual to-dos (the orchestrator will file the token migration, SSH keychain and other items once this lands).

## Thread

### note · agent:manager-2 · 2026-09-29T12:31:54.031Z
STOP implementing (orchestrator/human: to-dos and incidents get designed as one pattern first). Commit what you have on your branch as WIP (it needn't pass check), do not write a summary or land it, message me 'wip: <sha>' and end.

### note · agent:human-todos · 2026-09-29T12:36:20.348Z
wip: 648321e (unchecked, no summary; stopped per your message)

### note · agent:manager-2 · 2026-09-29T17:21:49.805Z
Parked: WIP 648321e (unchecked, no summary) is on branch bridle/human-todos; resume from it.

### question · external:orchestrator · 2026-09-29T20:37:10.948Z
From orchestrator: one design for incidents (nc7r) and human to-dos (ex9q)? Recommendation: one 'open request' record with an owner, a way to resolve it, and an optional check that closes it automatically. The incidents doc landed (c086457); WIP for this task is 648321e on its branch. Answer: go with one record, keep them separate, or discuss.

### answer · external:advisor · 2026-09-30T01:29:21.695Z
From the human, via advisor: neither separate records nor an 'open request' record: both are tasks. A human to-do is a task assigned to the human (the WIP fits). An incident is a special kind of task owned by the orchestrator: anyone files a 'potential incident' (after searching open ones), the orchestrator promotes it to active or drops it, and resolves it; it has the usual thread for comments and updates. The only new machinery: a broadcast, persistent notice to every agent while an incident is active, including agents that start/resume/renew. Verbatim and details in nc7r and ex9q (commit on main); incidents.md needs revising.

### note · agent:pm-1 · 2026-09-30T01:29:57.305Z
PM 2026-09-30: replanned after the human's decision (both are tasks). Scope unchanged: resume from WIP 648321e on branch bridle/human-todos (start a new worker branch from it), run just check, finish items 1-6 of the body. Only change: drop any incident-related idea from this task; incidents are br-incident tasks (see the incident tasks). Rebase on main first; main has moved since the WIP.

### note · agent:human-todos2 · 2026-09-30T01:37:07.781Z
done: human to-dos (task new --for-human, lease-exempt, bare done, docs, orchestrator startup); main merged, just check passes (767 tests); 6625116

### note · agent:manager-2 · 2026-09-30T01:37:12.325Z
integrated: 8b774caabbf818a588099387bd21504f5af128bf (branch bridle/human-todos2)
