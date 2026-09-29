+++
id = "br-6dd6"
title = "P5: bridle land <task>: the integrator, merge into the integration branch after checks"
kind = "feature"
state = "planned"
created_at = "2026-09-29T08:10:45.488Z"
updated_at = "2026-09-29T08:10:47.861587Z"
+++

Goal (docs/design/agent-host/roles-and-config.md: 'the integrator is bridle itself, in its own worktree'; build-order P5). Today the manager merges by hand. Add `bridle land <task> [--check-cmd CMD]`: (1) the daemon keeps a dedicated integration worktree (created on first use under the workspace, e.g. <workspace>/integration, checked out on a scratch branch integrate/<task> from the integration branch tip, never the human's checkout; config [branches] integration is the target); (2) it merges the task's branch --no-ff there; a conflict or failed merge aborts and reports the paths (use the merge-tree probe code from br-2612 for a pre-check); (3) runs the project's check command (`[integration] check = "just check"` in config; default none => skipped with a note) in that worktree; on failure reports the tail of the output and lands nothing; (4) on success fast-forwards the integration branch ref to the merged commit (git update-ref with the old value as a guard so a moved main is refused: 'main moved, retry'), then calls the existing task-done path with --commit. Never pushes. One landing at a time (a mutex/queue in the daemon). Refuses a branch that touches design/architecture/** unless the task kind is arch-revision (docs/design/architecture-tier.md, the integrator check). Emits events integrate.started/finished. Files: crates/bridle-daemon (new integrator.rs, server.rs, tasks.rs), bridle-api types, crates/bridle CLI, docs (roles-and-config.md, cli.md, architecture-tier.md).

Acceptance: just check passes; tests in a temp repo: clean land moves main and marks the task integrated; conflict lands nothing; failing check lands nothing; moved-main guard; arch refusal. Model: Sonnet. Out of scope: pushing, remote CI, wiring the manager's role prompt (next task), protected-requirement gating. Large: if the worker sees it exceeding budget it should land (a) merge+guard and report (b) check-cmd + arch refusal as a follow-up task.
