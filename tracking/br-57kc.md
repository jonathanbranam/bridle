+++
id = "br-57kc"
title = "Orchestrator: drop the bridle token check step from every start"
kind = "chore"
state = "claimed"
created_at = "2026-10-18T23:03:42.806Z"
updated_at = "2026-10-19T22:23:56.725569Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:w57kc",
]
summary = "Removed the 'Then run bridle token check' startup paragraph from workflow/base/roles/orchestrator.md ('At every start'); it ran at every orchestrator start via the shared base role. No other agent role or orchestrator.rs startup step pointed at it. The bridle token check command and its docs in docs/design/cli.md are unchanged (the check still runs as the last step of token pair and project start). Added a regression test in crates/bridle/src/commands/orchestrator.rs that the role does not mention 'bridle token check'. CHANGELOG Changed line added. Left orchestrator.md's 'treat those projects as unwatched' guidance in the Wake/forwarding section alone: it is about projects that do not forward and still applies to the human's pairing work."
ticket = "57kc"
+++

docs/tickets/open/orchestrator-drop-the-bridle-token-check-step-from-every-sta-57kc.md

## Thread

### note · external:aide · 2026-10-18T23:03:57.976Z
Readied by aide: a chore with one clear fix (rule ready-without-asking).

### note · agent:pm-1 · 2026-10-18T23:04:35.438Z
Brief (pm-1). Model: Haiku, text only. Do the ask: remove the startup step in workflow/base/roles/orchestrator.md (lines ~43-46) and any other place that tells an agent role to run `bridle token check` at start (grep workflow/ and crates/bridle/src/commands/orchestrator.rs startup steps; docs/design/cli.md keeps documenting the command itself). Keep the command. Acceptance: just check (update any test asserting the startup text). Same files as br-vsp9 (orchestrator.md and orchestrator.rs): run after it, not alongside.
