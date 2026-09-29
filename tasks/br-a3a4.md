+++
id = "br-a3a4"
title = "Orchestrator startup step: verify by CI, not just check on main (reword)"
kind = "chore"
state = "open"
created_at = "2026-09-29T20:42:28.488Z"
updated_at = "2026-09-29T20:42:28.488Z"
size = "S"
+++

Small fix. The orchestrator startup steps in crates/bridle/src/commands.rs (const ORCHESTRATOR_STARTUP_STEPS, ~line 302) say 'verify every merge (just check twice, off load) and push main after verifying', and docs/context/role-notes.md line 40 has the matching row 'Verify each merge: just check twice on main, off load'. The human's decision (2026-09-28, docs/context/orchestrator-state.md or its neighbours; grep for it): verify by the CI run on main and never run just check on main; the merger pushes right after each merge; and once br-9e71 lands, a failed CI run on main is the wake (main moving no longer is). Reword the step and the row to match, and the watcher line in the same const if it still mentions waking on main moves or a heartbeat that br-9e71 changes (read that task's summary if it has landed; otherwise leave the watcher line). Update any test that snapshots the text. No other behaviour change. Acceptance: just check passes. Model: Haiku. Out of scope: everything else in the role file (br-9e71 owns the wake removal).
