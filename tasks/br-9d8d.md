+++
id = "br-9d8d"
title = "NUC C: runbook for moving a project to another machine"
kind = "chore"
state = "planned"
created_at = "2026-09-30T03:01:34.299Z"
updated_at = "2026-09-30T03:01:35.604335Z"
+++

GOAL: add a runbook section to docs/context/nuc-host.md: moving a project to another machine. Steps: push bridle/state and working branches from the old machine; stop the old daemon (one writer); clone on the new machine; 'git fetch origin bridle/state:bridle/state' BEFORE the first 'bridle serve' (otherwise rebuild reports Diverged: crates/bridle-daemon/src/state_branch.rs try_fetch only accepts an empty seed); serve; create tokens; start orchestrator and advisor (scripts take a project, see NUC B1). Note messages stay in SQLite and do not move. Verify each command against docs/design/cli.md and the code rather than assuming. Optional, only if a few lines: make rebuild/first serve fetch bridle/state before the first flush; otherwise file it as a ticket in docs/questions/open/. CHANGELOG line if code changed. Acceptance: just check passes. Model: Haiku. Out of scope: cross-machine message sync.
