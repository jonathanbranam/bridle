+++
id = "br-3mz4"
title = "One orchestrator per machine, not per project: say so in the advisor role and wherever agents send to it"
kind = "chore"
state = "planned"
created_at = "2026-10-09T21:37:08.794Z"
updated_at = "2026-10-11T01:08:52.799689Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
summary = """Docs and role text only. The advisor role's "Deferring to the orchestrator" section now says the orchestrator is one per machine, in the bridle project, and its send example names --project bridle; it explains that a message on your own project's daemon may go unread. The manager's tool-missing send and the aide's relay send to external:orchestrator also name --project bridle (they were sending to their own project's daemon). docs/design/roles-and-lifecycle.md gets the rule sentence in the orchestrator bullet of the roles section: one per machine, in bridle, never "the <project> orchestrator", and a message from another project names the project. No code change: the optional daemon warning when no orchestrator waiter listens is deferred (ticket 3mz4 says file it separately if it recurs). just check exit 0 on the merged tip c2170f62 (1479 tests, 0 failed, 5 skipped)."""
ticket = "3mz4"
+++

Ticket: docs/tickets/open/one-orchestrator-per-machine-not-per-project-say-so-in-the-a-3mz4.md (read it: the human's words and the fuzzy places). Model: Haiku (docs only).

Goal: it is unmistakable that there is ONE orchestrator per machine, and that it runs in the bridle project on that machine; a message to it from another project must name that project.

Do:
1. workflow/base/roles/advisor.md, section "Deferring to the orchestrator": copy the paragraph from workflow/base/roles/aide.md ("The orchestrator is different: one per machine ...") and change the send example to `bridle send external:orchestrator --project bridle "From advisor: ..."`. Check the exact flag spelling with `bridle send --help` before writing it; use whatever flag selects another project's daemon.
2. Add one rule-level sentence where roles are described for all agents (look in workflow/base/rules/ and docs/design/roles docs; pick the one file where role identities are defined, do not add a new rule file): "There is one orchestrator per machine, in the bridle project; never write 'the <project> orchestrator'. A message to external:orchestrator from another project names the project it runs in." Match the surrounding style; ASCII only.
3. Grep workflow/base/roles/*.md and docs/design/ for any other send example to external:orchestrator without a project, or wording like "the <project> orchestrator", and fix the ones in role/rule text. Leave historical docs (incidents, handovers, resolved tickets, CHANGELOG entries) alone.
Acceptance: just check passes (docs checkers, if any); list every file changed on the task thread. Migration: role text reaches projects via bridle workflow sync; nothing to hand-edit.
Out of scope: the optional daemon warning when `bridle send external:orchestrator` goes to a project with no orchestrator waiter (a code change; deferred, not yet needed: the doc fix addresses the cause. File separately if it recurs).

## Thread

### note · agent:w3mz4 · 2026-10-11T01:08:51.065Z
Files changed (commit 04f3a94b, merge c2170f62): workflow/base/roles/advisor.md (Deferring to the orchestrator: --project bridle, one-per-machine paragraph); workflow/base/roles/manager.md (tool-missing send names --project bridle); workflow/base/roles/aide.md (relay send names --project bridle); docs/design/roles-and-lifecycle.md (rule sentence in the orchestrator bullet). just check exit 0 on c2170f62: 1479 run, 0 failed, 5 skipped. Deferred: daemon warning for sends to a project with no orchestrator waiter (code; out of scope per ticket).

### note · agent:w3mz4 · 2026-10-11T01:08:52.799Z
done: one-orchestrator-per-machine doc fix in advisor, manager and aide role text plus roles-and-lifecycle.md; c2170f62 (just check exit 0, 1479 passed)
