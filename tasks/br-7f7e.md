+++
id = "br-7f7e"
title = "Architecture guard: block edits to design/architecture outside an arch-revision task"
kind = "feature"
state = "planned"
created_at = "2026-09-29T08:10:45.515Z"
updated_at = "2026-09-29T08:29:33.301838Z"
summary = """Added `bridle arch-guard`, a PreToolUse hook (crates/bridle/src/arch_guard.rs, commands.rs) that denies Edit/Write/MultiEdit/NotebookEdit under design/architecture/ unless the calling worker agent has claimed an arch-revision task. Paths are resolved lexically against the hook's cwd (handles relative and ../ paths). Non-agent principals and non-worker roles are allowed, as is everything on any bridle-side error. Denial uses the PreToolUse hookSpecificOutput deny shape and points to `bridle arch propose`. Shipped as workflow/base/hooks/PreToolUse.json (matcher Edit|Write|MultiEdit); sync test confirms rendering. Docs: architecture-tier.md, cli.md; CHANGELOG line added. Caveat: role check uses agent.role == "worker" only."""
+++

Goal (docs/design/architecture-tier.md, 'PreToolUse hook ... not built'): a PreToolUse hook command `bridle arch-guard` (shape and failure policy like `bridle stop-check`, crates/bridle/src/stop_check.rs and docs/spikes/05-stop-hook-findings.md; read hook JSON on stdin) that denies Edit/Write/MultiEdit/NotebookEdit whose file path is under design/architecture/ unless the calling principal's claimed task has kind arch-revision. Any error of bridle's own allows (never trap the agent), and non-worker principals (human, orchestrator, PM) are allowed. Ship the hook in workflow/base/hooks/ so `bridle sync` renders it (see workflow-layers.md 'hooks'; PreToolUse with matcher Edit|Write|MultiEdit). The denial message tells the agent to run `bridle arch propose`. Docs: architecture-tier.md, cli.md.

Acceptance: just check passes; unit tests on the decision function for kinds and paths (incl. relative paths and ../ tricks), and a sync test that the hook is rendered. Model: Sonnet.
