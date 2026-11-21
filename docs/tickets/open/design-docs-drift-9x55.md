---
id: br-9x55
title: "Docs: bring stale design-doc Status lines and undocumented routes in line with the code"
kind: docs
created: 2026-11-21
created_by: external:advisor/agentic-dev
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

From the human, via advisor agentic-dev (2026-11-21): "Please directly fix these documentation issues or file tickets to do so" (typo "tondonso" in the original read as "to do so").

Found while checking bridle main 153d3c86 against the code for the notes project's agentic-dev survey (inventory sent to advisor/agentic-dev on notes as message o-0643). Each item below was checked in the code; file:line is at 153d3c86.

Bring docs/design/ in line with the code. Edit the docs only; where the code looks wrong instead, comment on this ticket rather than changing code.

### Stale Status lines and statements

1. docs/design/migrations.md says migrations are "never automatic, by design". In fact `bridle serve` applies pending migrations at startup unless `[migrations] auto = false` (crates/bridle/src/serve.rs:84, crates/bridle/src/migrate.rs:840).
2. Layer hooks: architecture-tier.md, coordination.md, workflow-layers.md, cli.md and docs/proposal/build-order.md say arch-guard is live only where `bridle sync` ran, and workflow-layers.md lists "layer hooks at spawn (34bw step 3)" as planned. In fact `Config::layer_hooks` is passed on every spawn, resume and renew (crates/bridle-daemon/src/supervisor.rs:1337, 2868, 3069), so arch-guard, case-guard and kill-guard (workflow/base/hooks/PreToolUse.json) are live for every spawned agent. 34bw is resolved.
3. dev-site.md calls sections 3 and 4 (rule, role text, gitignore) planned (tgxy) and the UI step untested until dgef. The rule workflow/base/rules/verify-ui-in-a-browser.md and `.agent-site/` in .gitignore exist; tgxy and dgef are resolved.
4. human-web-ui.md: the Status line is truncated ("task 9 ..., task 10 ..." with no label), and section 5 calls task 11 (bek3, detach and self-restart) planned. `bridle gateway --detach|restart|status|stop|install` and `[gateway] enabled` exist (crates/bridle-gateway/src/config.rs:54); bek3 is resolved. build-order.md's "gateway 8 of 10 tasks, not wired in" and "UI planned" are stale too.
5. agent-host/orchestrator-supervision.md lists "wakes for advisors" as planned. Built: `bridle agent wake external:advisor/...` (crates/bridle-daemon/src/principal_wake.rs), plus `bridle session stop|list`.
6. specs.md, spec-flow.md, specs-to-tests.md, knowledge-tiers.md and overview.md say no other project uses specs, no project has a design/ tree and no rule tells agents to edit specs. In fact bridle, bridle-ui and meta-notes have design/specs/, and the stage rules (workflow/base/rules/stage-specs.md, stage-apply.md) tell agents to write specs. No project has design/goals, design/architecture or design/explore.
7. roles-and-lifecycle.md lists six roles and seven states. The code also has states `held` and `built` (crates/bridle-api/src/types.rs:2335-2339) and roles advisor, aide, designer, document-reviewer and prototyper (workflow/base/roles/). It also calls the re-evaluate flow "in use", but it never fires because no project has the tiers it watches (crates/bridle-daemon/src/reevaluate.rs). traceability.md says "not wired in", which is right.
8. skills.md doesn't mention the third skill, bridle-grill (workflow/base/skills/grill).
9. agent-host/api.md says `POST /v1/handovers` is "`human` and `external:orchestrator` only, else 403". The handler has no such check (`write_handover`, crates/bridle-daemon/src/server.rs:906); cli.md ("any principal") matches the code.
10. Command names: Status lines and bodies still use `bridle task impact`, `bridle spec|goals|arch|explore|trace|rules|sync`, `bridle prime` and `cost audit`. These are now `bridle ticket ...`, `bridle workflow ...`, `bridle orchestrator prime` and `bridle usage cost`; the old names are hidden aliases. Use the current names, and say once (cli.md) that the old ones are aliases.
11. Re-check each Status line's "(checked 2026-10-03)" date while doing the above, and update the date on every line you verify.

### Built but undocumented

12. cli.md's usage block is missing `bridle usage --agent <NAME|ID> [--bucket hour|day]` (crates/bridle/src/cli.rs:1640-1643). It is described only in usage-and-budget.md:156.
13. Daemon routes documented nowhere: `POST /v1/tickets/{id}/ready` (and `/v1/tasks/{id}/ready`), server.rs:385-386; `POST /v1/push`, server.rs:395. Add them to api.md.
14. Daemon routes described in other docs but missing from api.md's route table: `/v1/tickets/{id}/stage-land` (stages.md), `/v1/restart/hold` (daemon.md), `/v1/review/now|add`, `/v1/sessions/keep`, `GET /v1/usage/history`.
15. Gateway routes in human-web-ui.md with no route-level description: `GET /api/v1/items`, `/api/v1/projects`, `/api/v1/session` (crates/bridle-gateway/src/lib.rs:58-60). Ticket actions `done|drop|answer` (actions.rs:36-39); only `reply` is named. login and logout appear only in cli.md.
16. Event kinds: about 15 are documented only in daemon.md, operating-model.md or orchestrator-supervision.md, not in api.md's Events list (e.g. schedule.fired, restart.failed, load.hold.*, push.failed, worktree.removed, session.*, task.settle_skipped). List every kind in api.md, or point to where each is described.
17. workflow-layers.md:10 lists packs "typescript, python, vim, web-ui, game, monorepo". The packs that exist are mobile, python, typescript, vim and web (workflow/packs/); web (6 rules) and mobile (5 rules) are in no design doc. Fix the list and describe each pack in a line.
18. scripts/test-lock.sh (the machine-wide nextest lock, `BRIDLE_TEST_NOLOCK`) is described only in CLAUDE.md; `just specs-check`, `just gateway-types` (ts-rs bindings) and `just deny` are in no design doc. Mention them in cli.md's or the README's commands section.

Done when every item is fixed or commented on here, and `bridle ticket check` passes.
