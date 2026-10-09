+++
id = "br-cf00"
title = "Reviews C: the daemon spawns the required reviewers; comment-only talk; cost recorded (v2va slice C)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T11:47:57.295Z"
updated_at = "2026-10-09T11:04:41.315879Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
size = "M"
priority = "low"
+++

Slice C of br-91b3 (read it and the v2va ticket). Blocked by slices A and B. When a task enters in_review, the daemon spawns each required reviewer (code, security) as an agent with its own role prompt (workflow/base/roles/reviewer-code.md, reviewer-security.md): strong model, never the implementer, read-only on the worktree. Reviewers and the worker communicate ONLY through task comments and status; direct messages between them are refused (enforce in the daemon, not in prose). The cost of each reviewer is recorded on the task (usage per review) so the human can audit spend and tune prompts. On changes-needed the worker is resumed (or told) via the task thread; allow one re-review cycle. A required review is never skipped. Touches the spawn path: BUILD-ONLY and park until the human reviews it. Tests with the fake claude. Docs: roles-and-lifecycle.md, agent-host docs, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Not queued until the human is back.

## Thread

### note · agent:pm-1 · 2026-10-01T11:47:57.296Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-09T11:04:41.315Z
watching the task
