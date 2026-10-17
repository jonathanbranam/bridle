+++
id = "br-3stk"
title = "ticket land's integration check has no node_modules in a JS project (biome: command not found on bridle-ui)"
kind = "bug"
state = "integrated"
created_at = "2026-10-17T11:59:16.133Z"
updated_at = "2026-10-17T13:09:17.587973Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "high"
priority_at = "2026-10-17T12:22:02.754198Z"
branch = "bridle/w3stk"
commit = "38502d976c843995c9a4634d3724a2035b260715"
summary = "ticket land now runs [worktrees] setup (e.g. npm ci) in its check worktree before the check (integrator.rs, wired via AppState.check_setup); a setup failure fails the land. Branch lookup falls back to the ticket's last recorded branch after built/claimed. Tests in land_test.rs; docs cli.md, operating-model.md, roles-and-config.md, CHANGELOG. Caveat: projects must set [worktrees] setup (bridle-ui: npm ci); no automatic npm detection. Not verified against a real bridle-ui land."
ticket = "3stk"
+++

docs/tickets/open/ticket-land-s-integration-check-has-no-node-modules-in-a-js-3stk.md

## Thread

### note · external:orchestrator · 2026-10-17T11:59:31.208Z
From orchestrator: filed br-3stk (bug): ticket land's check has no node_modules in a JS project, so bridle-ui can't land without --checked-commit (workaround in use by manager-3). Not critical; yours to schedule.

### note · external:advisor/product-manager · 2026-10-17T12:22:02.651Z
The human, 2026-10-17 ~8 AM ET (to advisor product-manager): "Yes ready that fix for bridle hi". Priority high.

### note · external:advisor/product-manager · 2026-10-17T12:22:02.754Z
priority: normal -> high

### note · external:orchestrator · 2026-10-17T12:22:26.216Z
From orchestrator: br-3stk is ready, priority high (the human, via advisor product-manager: "Yes ready that fix for bridle hi"). It blocks bridle-ui landings without --checked-commit. Please plan it (brief in the ticket body, Sonnet) and queue it near the front.

### note · agent:w3stk · 2026-10-17T12:51:12.304Z
done: land runs [worktrees] setup before its check + branch fallback; just check exit 0, 1815 tests, 52ccebd2

### note · agent:w3stk · 2026-10-17T12:51:16.460Z
w3stk: 'ticket built' was refused for a worker (forbidden: agent lifecycle endpoints), so the ticket is not marked built; branch bridle/w3stk at 52ccebd2 is green and ready to land. A bridle-ui project needs [worktrees] setup = "npm ci" in its config for this to take effect.

### note · external:orchestrator · 2026-10-17T12:52:10.091Z
From orchestrator: checked: bridle-ui already has [worktrees] setup = "test ! -f package.json || npm install ..." in .bridle/config.toml, so no config change is needed. Land br-3stk and push; it reaches the daemon at the next self-upgrade.

### note · agent:manager-2 · 2026-10-17T13:09:14.259Z
integrated: 38502d976c843995c9a4634d3724a2035b260715 (branch bridle/w3stk)

### note · agent:manager-2 · 2026-10-17T13:09:17.587Z
cleanup: removed nothing; kept agent w3stk (background job pid 1719)
