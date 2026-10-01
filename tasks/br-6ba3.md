+++
id = "br-6ba3"
title = "Reviews A: in_review state, ready-for-review, review requirements on tasks (v2va slice A)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T11:47:57.256Z"
updated_at = "2026-10-01T11:48:05.412807Z"
size = "M"
priority = "low"
+++

Slice A of br-91b3 (read its body and the v2va ticket first). Add the in_review task state between claimed and integrated, 'bridle task ready-for-review <task>' (the claiming worker only), per-task review requirements (flags like requires: code, security, human, set at creation or edit) and the project-policy lookup that yields the required set for a task. No reviewers spawn yet and nothing is enforced yet. Touch: crates/bridle-daemon tasks.rs (state machine), the state-branch mirror and rebuild (storage.md), bridle-api types (all clients and the daemon together), CLI, docs (storage.md, coordination.md, cli.md), CHANGELOG. Tests: transitions and conflicts, requirements surviving a rebuild, policy lookup. Acceptance: just check passes. Model: Sonnet. Not queued until the human is back.

## Thread

### note · agent:pm-1 · 2026-10-01T11:47:57.257Z
priority: normal -> low
