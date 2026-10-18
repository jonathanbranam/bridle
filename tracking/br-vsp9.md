+++
id = "br-vsp9"
title = "Role text and orchestrator startup steps name a nonexistent --state flag on ticket/task list"
kind = "fix"
state = "planned"
created_at = "2026-10-18T22:42:13.528Z"
updated_at = "2026-10-18T22:55:27.357741Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "vsp9"
+++

docs/tickets/open/role-text-and-orchestrator-startup-steps-name-a-nonexistent-vsp9.md

submitted by external:orchestrator

Found by orchestrator (dalek) at startup, 2026-10-18; the human asked for this ticket.

`bridle ticket list` and `bridle task list` have no `--state` option (options: --claimed-by, --component, -k/--kind, --json, --project, --url, --token). Running the documented command fails:

    error: unexpected argument '--state' found

Stale references (bridle repo, integration checkout):
- workflow/base/roles/orchestrator.md:66 - `bridle ticket list --state pending`
- crates/bridle/src/commands/orchestrator.rs:12 - startup steps: `bridle task list --state pending`
- workflow/base/roles/manager.md:16 - `bridle ticket list --state open`
- (CHANGELOG.md:676 mentions `--state open`; history, leave it)

## The ask

Make the text and the CLI agree. Either add `--state <state>` to `ticket list` / `task list` (the roles clearly expect it), or change the three references to a command that works (for pending, `bridle status` already lists the pending ids). Smallest fix that makes the documented commands run. Haiku is enough if it is text only.

Verify: each command the role text and startup steps name runs without error.

## Thread

### note · external:orchestrator · 2026-10-18T22:42:13.534Z
submitted by external:orchestrator
