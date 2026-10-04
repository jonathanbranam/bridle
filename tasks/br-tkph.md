+++
id = "br-tkph"
title = "Follow up on the workflow review: 6 decisions, 2 waiting on others (34bw, vp9e, sk52, 7r2c, xfb3, ma2x, ntca, 98xt)"
kind = "chore"
state = "claimed"
created_at = "2026-10-04T12:49:58.213Z"
updated_at = "2026-10-04T12:49:58.218169Z"
created_by = "external:advisor/workflow"
watchers = [
    "external:advisor/workflow",
    "human",
]
+++

Follow-ups from the workflow review (advisor workflow, 2026-10-02 to 04). Each ticket's "Next steps" section has the detail (commit b084d0d6). Tickets are in docs/tickets/open/.

Decide or review (yours):
- [ ] 98xt: go to queue the fixes to stale lines in 11 rule files; and should orchestrator/advisor sessions get resolved rules? (advisor: yes)
- [ ] xfb3: how the language packs got prioritised; review its three recommendations (verify "built" against code before ranking work on it; post-restart check that rules reach a worker; no new packs until then)
- [ ] ma2x: who owns build-order.md's status column (the project manager?), and should bridle status flag a daemon older than main
- [ ] ntca: hold built-but-unwired machinery under yagni, wiring each piece only when a project needs it
- [ ] vp9e: are role prompts rules (one layered mechanism)? Talk it through with an advisor
- [ ] sk52: step workflows (OpenSpec-style); design doc once vp9e is settled. Don't build yet

Waiting on others (ask them):
- [ ] 7r2c: rename product-manager to project-manager. Orchestrator to file and queue it (m-3788, re-sent m-4387); you review the migration for meta-notes and track-web
- [ ] 34bw: done; check rules and hooks are live after each daemon restarts onto c4cd9af or later

Why a to-do: the question tasks for these tickets (br-8b6c, br-e839, br-dcf2, br-7e32, br-0473) were dropped in pm-1's k7tm sort, so nothing tracked them.

## Thread

### note · external:advisor/workflow · 2026-10-04T12:49:58.216Z
created for the human, priority normal

### note · external:advisor/workflow · 2026-10-04T12:49:58.218Z
To-do for you (normal priority): Follow up on the workflow review: 6 decisions, 2 waiting on others (34bw, vp9e, sk52, 7r2c, xfb3, ma2x, ntca, 98xt). Finish it with `bridle task done br-tkph`.
