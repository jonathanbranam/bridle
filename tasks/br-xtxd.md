+++
id = "br-xtxd"
title = "Resumed workers are told their task and branch; a worker's inbox shows only its own open questions (qdw8 fix 1)"
kind = "bug"
state = "planned"
created_at = "2026-10-05T00:33:10.608Z"
updated_at = "2026-10-05T00:33:18.636260Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Ticket: docs/tickets/open/a-haiku-worker-reported-done-before-its-check-finished-then-qdw8.md (fix 1; read "What happened" and "Decided"). Postmortem: ytqu.

Approval: the human, via aide (m-5109, 2026-10-04 ~8:05 PM ET): "1. Looks correct. 2. Looks good. 3. Seems reasonable, if possible, yes. ... I think it's 1, 2, 3, so let's go with that".

Goal (fix 1, the parts br-cyvf does not cover): a resumed or nudged worker can never lose its task.
- Every daemon message that resumes an agent after a daemon restart or a `lost` resume (supervisor.rs: the resume / continuation paths other than context renewal, which is br-cyvf) names the agent's claimed task ID and branch when it has one ("You are on br-xxxx, branch bridle/<name>; read `bridle task show br-xxxx`").
- A worker's `bridle inbox` open questions (and any open-question list the worker's prompt or resume shows) are only the ones on its own claimed task or addressed to it, not every open question to the human. Managers, the orchestrator and the human keep seeing all of them.
Acceptance: just check green; tests: a resumed worker with a claimed task gets its ID and branch in the message; a worker's inbox omits an open question on another task; a manager's still shows it.
Model: sonnet. Touches supervisor.rs near br-cyvf's change (send_continuation_note): if cyvf is in flight, run one after the other.
Out of scope: context renewal (br-cyvf), the handover record (br-e9yu), fix 4 (Haiku for long checks: to consider, not build).
