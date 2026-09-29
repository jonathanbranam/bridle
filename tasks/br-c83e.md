+++
id = "br-c83e"
title = "Human to-dos: tasks assigned to the human, and the orchestrator lists them at startup (ex9q)"
kind = "feature"
state = "planned"
created_at = "2026-09-29T12:23:53.704Z"
updated_at = "2026-09-29T17:21:49.805788Z"
+++

Ticket: docs/questions/open/*ex9q.md (read it, especially Shape). Goal, KISS: a human to-do is a bridle task assigned to the human; the human marks it done; the orchestrator lists open ones at every start. Build: 1) A way to create a task already assigned to the human: e.g. bridle task new --for human (or --assign human), creating it planned and claimed by the human principal in one step, plus the API field in crates/bridle-api/src/types.rs and the server handler in crates/bridle-daemon. Check the claim lease (br P0-4, coordination.md): a human claim must NOT expire or be reaped; if leases would release it, exempt the human principal. 2) The creating command also sends one inbox message to human pointing at the task (reuse the send --task / --notify path; body must be non-empty). 3) The human finishes it with the existing bridle task done (check it works for a task the human claimed with no commit: allow done without --commit for these, or add bridle task complete if cleaner; pick the simpler). 4) Titles carry a when tag by convention: [at restart], [at next reboot], or none; document only, no parsing needed except sorting them first in the list if trivial. 5) workflow/base/roles/orchestrator.md startup steps: list bridle task list --claimed-by human and tell the human first, the [at restart] ones especially. 6) Docs: docs/design/cli.md, coordination.md, and the orchestrator role. Acceptance: just check passes; tests for create-assigned-to-human, lease exemption, done by the human. Model: Sonnet. Out of scope: mobile/notification surfaces, recurring to-dos, and creating the actual to-dos (the orchestrator will file the token migration, SSH keychain and other items once this lands).

## Thread

### note · agent:manager-2 · 2026-09-29T12:31:54.031Z
STOP implementing (orchestrator/human: to-dos and incidents get designed as one pattern first). Commit what you have on your branch as WIP (it needn't pass check), do not write a summary or land it, message me 'wip: <sha>' and end.

### note · agent:human-todos · 2026-09-29T12:36:20.348Z
wip: 648321e (unchecked, no summary; stopped per your message)

### note · agent:manager-2 · 2026-09-29T17:21:49.805Z
Parked: WIP 648321e (unchecked, no summary) is on branch bridle/human-todos; resume from it.
