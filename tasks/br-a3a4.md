+++
id = "br-a3a4"
title = "Orchestrator startup step: verify by CI, not just check on main (reword)"
kind = "chore"
state = "planned"
created_at = "2026-09-29T20:42:28.488Z"
updated_at = "2026-09-29T20:55:06.517918Z"
size = "S"
summary = "Reworded orchestrator startup steps and role notes table to reflect the decision to verify merges by CI run, not locally (following br-9e71's removal of main_moved wake)."
+++

Small fix. The orchestrator startup steps in crates/bridle/src/commands.rs (const ORCHESTRATOR_STARTUP_STEPS, ~line 302) say 'verify every merge (just check twice, off load) and push main after verifying', and docs/context/role-notes.md line 40 has the matching row 'Verify each merge: just check twice on main, off load'. The human's decision (2026-09-28, docs/context/orchestrator-state.md or its neighbours; grep for it): verify by the CI run on main and never run just check on main; the merger pushes right after each merge; and once br-9e71 lands, a failed CI run on main is the wake (main moving no longer is). Reword the step and the row to match, and the watcher line in the same const if it still mentions waking on main moves or a heartbeat that br-9e71 changes (read that task's summary if it has landed; otherwise leave the watcher line). Update any test that snapshots the text. No other behaviour change. Acceptance: just check passes. Model: Haiku. Out of scope: everything else in the role file (br-9e71 owns the wake removal).

## Thread

### note · agent:manager-2 · 2026-09-29T20:48:02.919Z
Correction from the orchestrator: the human's 2026-09-28 decision 'verify by CI, never just check on main' lives in workflow/base/roles/orchestrator.md ('Verify every merge by its CI run, not locally'), NOT docs/context/orchestrator.md (the brief's path is wrong). Edit the role file's startup step to match it.

### note · agent:startup-reword · 2026-09-29T20:54:58.998Z
done: Reworded orchestrator startup steps to verify by CI, not locally; 154363e

### note · agent:manager-2 · 2026-09-29T20:55:06.517Z
Two fixes: (1) you dropped 'and push main after verifying' from the startup step; the brief only asked to change the verification wording. Keep the push clause (e.g. 'verify every merge by its CI run (not locally), push main after each merge, and bring the human...'). Check the role file orchestrator.md says the same. (2) The step text is in commands.rs, so confirm just check passes (any test snapshotting ORCHESTRATOR_STARTUP_STEPS) and say so in your done message. Merge main, message me the sha.
