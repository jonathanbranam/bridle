+++
id = "br-7f7e"
title = "Architecture guard: block edits to design/architecture outside an arch-revision task"
kind = "feature"
state = "planned"
created_at = "2026-09-29T08:10:45.515Z"
updated_at = "2026-09-29T08:10:47.887403Z"
+++

Goal (docs/design/architecture-tier.md, 'PreToolUse hook ... not built'): a PreToolUse hook command `bridle arch-guard` (shape and failure policy like `bridle stop-check`, crates/bridle/src/stop_check.rs and docs/spikes/05-stop-hook-findings.md; read hook JSON on stdin) that denies Edit/Write/MultiEdit/NotebookEdit whose file path is under design/architecture/ unless the calling principal's claimed task has kind arch-revision. Any error of bridle's own allows (never trap the agent), and non-worker principals (human, orchestrator, PM) are allowed. Ship the hook in workflow/base/hooks/ so `bridle sync` renders it (see workflow-layers.md 'hooks'; PreToolUse with matcher Edit|Write|MultiEdit). The denial message tells the agent to run `bridle arch propose`. Docs: architecture-tier.md, cli.md.

Acceptance: just check passes; unit tests on the decision function for kinds and paths (incl. relative paths and ../ tricks), and a sync test that the hook is rendered. Model: Sonnet.
