+++
id = "br-vsp9"
title = "Role text and orchestrator startup steps name a nonexistent --state flag on ticket/task list"
kind = "fix"
state = "claimed"
created_at = "2026-10-18T22:42:13.528Z"
updated_at = "2026-10-19T22:04:49.693976Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wvsp9",
]
summary = "Three places told agents to run `--state` on `ticket list` and `task list`, a flag neither command has. Changed the three references (no `--state` flag added): crates/bridle/src/commands/orchestrator.rs (startup steps: pending tasks now point at `bridle task list`'s state column and `bridle status --json`'s `pending_tasks`), workflow/base/roles/orchestrator.md (start-up look at pending: same), workflow/base/roles/manager.md (idle/woken: `bridle ticket list`, reading the state column for open tickets). No test asserted the old text. CHANGELOG history left as is; a Fixed line added under Unreleased. Checked that `bridle task list`, `bridle ticket list` and `bridle status --json` (with `pending_tasks`) exit 0."
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
