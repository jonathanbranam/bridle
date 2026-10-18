---
id: br-vsp9
title: Role text and orchestrator startup steps name a nonexistent --state flag on ticket/task list
kind: fix
created: 2026-10-18
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

`bridle ticket list` and `bridle task list` have no `--state` option, but three places tell agents to run it, and it fails with "unexpected argument '--state' found":

- workflow/base/roles/orchestrator.md:66 (`bridle ticket list --state pending`)
- crates/bridle/src/commands/orchestrator.rs:12 (startup steps: `bridle task list --state pending`)
- workflow/base/roles/manager.md:16 (`bridle ticket list --state open`)

(CHANGELOG.md:676 mentions `--state open`; history, leave it.)

Fix (decided, smallest): change the three references to commands that work today (for pending, `bridle status --json` lists the pending ids; for open tickets use what `bridle ticket list --help` offers, or `--json` plus a note). Do not add a `--state` flag. If the startup-steps text is asserted by a test, update the test.

Verify: each command the three places now name runs without error; `just check` passes. Model: Haiku (text only). Out of scope: adding list filters.
