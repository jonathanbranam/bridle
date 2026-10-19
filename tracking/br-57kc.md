+++
id = "br-57kc"
title = "Orchestrator: drop the bridle token check step from every start"
kind = "chore"
state = "claimed"
created_at = "2026-10-18T23:03:42.806Z"
updated_at = "2026-10-19T22:23:38.157938Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:w57kc",
]
ticket = "57kc"
+++

docs/tickets/open/orchestrator-drop-the-bridle-token-check-step-from-every-sta-57kc.md

## Thread

### note · external:aide · 2026-10-18T23:03:57.976Z
Readied by aide: a chore with one clear fix (rule ready-without-asking).

### note · agent:pm-1 · 2026-10-18T23:04:35.438Z
Brief (pm-1). Model: Haiku, text only. Do the ask: remove the startup step in workflow/base/roles/orchestrator.md (lines ~43-46) and any other place that tells an agent role to run `bridle token check` at start (grep workflow/ and crates/bridle/src/commands/orchestrator.rs startup steps; docs/design/cli.md keeps documenting the command itself). Keep the command. Acceptance: just check (update any test asserting the startup text). Same files as br-vsp9 (orchestrator.md and orchestrator.rs): run after it, not alongside.
