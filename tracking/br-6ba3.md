+++
id = "br-6ba3"
title = "Reviews A: in_review state, ready-for-review, review requirements on tasks (v2va slice A)"
kind = "feature"
state = "held"
created_at = "2026-10-01T11:47:57.256Z"
updated_at = "2026-10-11T21:16:37.197922Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
size = "M"
priority = "low"
held_by = "external:orchestrator"
held_reason = "human via orchestrator: v2va review slices are held until stx8 is built, 22ab has landed and the human has reviewed v2va again (the human's quote is on br-6ba3)"
held_at = "2026-10-11T21:16:37.197922Z"
+++

Slice A of br-91b3 (read its body and the v2va ticket first). Add the in_review task state between claimed and integrated, 'bridle task ready-for-review <task>' (the claiming worker only), per-task review requirements (flags like requires: code, security, human, set at creation or edit) and the project-policy lookup that yields the required set for a task. No reviewers spawn yet and nothing is enforced yet. Touch: crates/bridle-daemon tasks.rs (state machine), the state-branch mirror and rebuild (storage.md), bridle-api types (all clients and the daemon together), CLI, docs (storage.md, coordination.md, cli.md), CHANGELOG. Tests: transitions and conflicts, requirements surviving a rebuild, policy lookup. Acceptance: just check passes. Model: Sonnet. Not queued until the human is back.

## Thread

### note · agent:pm-1 · 2026-10-01T11:47:57.257Z
priority: normal -> low

### note · agent:pm-1 · 2026-10-01T11:53:58.657Z
pm-1, brief addition (human on thresholds, via advisor 2026-10-01): the workflow rules define review THRESHOLDS and the daemon enforces and tracks them. Defaults: a change under about 30 changed lines, and a docs-only change, need no review. The policy lookup returns required or not-required for a task from those thresholds (configurable in the rule, not a hard-coded const) and records which threshold applied, so the human can see why a review was or wasn't required. Budget wind-down is NOT a threshold: a low budget means the work waits and does not merge; a required review is never skipped for it. A required review is always done unless the human overrides it (record who and why).

### note · external:advisor/product-manager · 2026-10-09T11:04:41.216Z
watching the task

### question · external:advisor/product-manager · 2026-10-11T04:17:43.412Z
HELD by the human (2026-10-11 ~12:25 AM ET, to advisor product-manager): "I'm not sure how br-6ba3 is supposed to work from that ticket. I think that ticket can wait until the 'everything is a ticket' lands and then I can review it and make better comments. Let's be sure that stx8 happens first." Don't start a worker on br-6ba3 until ticket stx8 (task states) is designed and built, 22ab (everything is a ticket) has landed, and the human has reviewed v2va again.

### note · external:orchestrator · 2026-10-11T04:18:03.200Z
From orchestrator: br-6ba3 is HELD by the human (quote on the task): no worker until stx8 is designed and built, 22ab has landed, and the human has reviewed v2va again. Skip it in the queue.

### note · external:orchestrator · 2026-10-11T21:16:37.197Z
held: human via orchestrator: v2va review slices are held until stx8 is built, 22ab has landed and the human has reviewed v2va again (the human's quote is on br-6ba3)
