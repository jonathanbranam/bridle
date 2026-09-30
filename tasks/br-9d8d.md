+++
id = "br-9d8d"
title = "NUC C: runbook for moving a project to another machine"
kind = "chore"
state = "planned"
created_at = "2026-09-30T03:01:34.299Z"
updated_at = "2026-09-30T04:16:49.167474Z"
summary = "Added a 'Moving a project to another machine' runbook to docs/context/nuc-host.md (push, stop-daemon, clone, serve --take-over, tokens, launch scripts, tools-only on old clone; messages stay in SQLite) plus a CHANGELOG line."
+++

GOAL: runbook section in docs/context/nuc-host.md: moving a project to another machine. Steps: push bridle/state and working branches from the old machine; stop the old daemon (one writer; its shutdown pushes state); clone on the new machine; 'bridle serve --take-over' (hw6c 1, replaces the manual git fetch of bridle/state before first serve; see docs/questions/open/one-machine-owns-a-project-hw6c.md); create tokens; start orchestrator and advisor with the project-aware scripts (NUC B1). Also: how to mark a bridle clone tools-only (hw6c 2). Note messages stay in SQLite and do not move. Verify each command against docs/design/cli.md and the merged code, not assumptions. Acceptance: just check passes. Model: Haiku. Out of scope: cross-machine message sync. Run after the hw6c tasks.

## Thread

### note · agent:nuc-runbook · 2026-09-30T04:16:44.108Z
done: runbook 'Moving a project to another machine' in docs/context/nuc-host.md + CHANGELOG line; just check passes (807 tests); ce809f0

### note · agent:manager-2 · 2026-09-30T04:16:49.167Z
main moved (c417796, docs only). Merge main into your branch, rerun just check, then message me.
