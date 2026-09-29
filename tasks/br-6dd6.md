+++
id = "br-6dd6"
title = "P5: bridle land <task>: the integrator, merge into the integration branch after checks"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T08:10:45.488Z"
updated_at = "2026-09-29T08:40:11.088307Z"
branch = "bridle/land-cmd"
commit = "402816e7d619c7b3c85cb056201a0f2e1f92f5bf"
summary = "Added `bridle land <task> [--branch B] [--check-cmd CMD]`: new crates/bridle-daemon/src/integrator.rs plus a POST /v1/tasks/{id}/land route (LandRequest/LandResult in bridle-api, integrate.started/finished events, `[integration] check` config). Under a daemon-wide lock it refuses arch-tier paths unless the task is an arch-revision, probes with merge-tree, merges --no-ff in <workspace>/integration on integrate/<task>, runs the check, then update-ref with the old tip as guard, then reuses the done handler with the merge commit. Failures return 409 and land nothing. Caveat: update-ref doesn't touch the human's checkout, so a checkout of main shows the new tip as staged changes until reset (documented). Scratch branch integrate/<task> is left behind and reset on next landing. Tests: tests/land_test.rs (clean, conflict, failing check, moved main, arch refusal)."
+++

Goal (docs/design/agent-host/roles-and-config.md: 'the integrator is bridle itself, in its own worktree'; build-order P5). Today the manager merges by hand. Add `bridle land <task> [--check-cmd CMD]`: (1) the daemon keeps a dedicated integration worktree (created on first use under the workspace, e.g. <workspace>/integration, checked out on a scratch branch integrate/<task> from the integration branch tip, never the human's checkout; config [branches] integration is the target); (2) it merges the task's branch --no-ff there; a conflict or failed merge aborts and reports the paths (use the merge-tree probe code from br-2612 for a pre-check); (3) runs the project's check command (`[integration] check = "just check"` in config; default none => skipped with a note) in that worktree; on failure reports the tail of the output and lands nothing; (4) on success fast-forwards the integration branch ref to the merged commit (git update-ref with the old value as a guard so a moved main is refused: 'main moved, retry'), then calls the existing task-done path with --commit. Never pushes. One landing at a time (a mutex/queue in the daemon). Refuses a branch that touches design/architecture/** unless the task kind is arch-revision (docs/design/architecture-tier.md, the integrator check). Emits events integrate.started/finished. Files: crates/bridle-daemon (new integrator.rs, server.rs, tasks.rs), bridle-api types, crates/bridle CLI, docs (roles-and-config.md, cli.md, architecture-tier.md).

Acceptance: just check passes; tests in a temp repo: clean land moves main and marks the task integrated; conflict lands nothing; failing check lands nothing; moved-main guard; arch refusal. Model: Sonnet. Out of scope: pushing, remote CI, wiring the manager's role prompt (next task), protected-requirement gating. Large: if the worker sees it exceeding budget it should land (a) merge+guard and report (b) check-cmd + arch refusal as a follow-up task.

## Thread

### note · agent:manager-2 · 2026-09-29T08:40:11.088Z
integrated: 402816e7d619c7b3c85cb056201a0f2e1f92f5bf (branch bridle/land-cmd)
