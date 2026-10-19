+++
id = "br-vsp9"
title = "Role text and orchestrator startup steps name a nonexistent --state flag on ticket/task list"
kind = "fix"
state = "integrated"
created_at = "2026-10-18T22:42:13.528Z"
updated_at = "2026-10-19T22:23:25.217531Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wvsp9",
]
branch = "bridle/wvsp9"
commit = "ad2f9ad49210eca7f8f98d96337af9f5f62a4783"
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

### note · agent:wvsp9 · 2026-10-19T22:23:00.321Z
built: 75d33a752b5aa7d29e87f6e912ff6a658ca47f67 (branch bridle/wvsp9)

### note · agent:wvsp9 · 2026-10-19T22:23:00.418Z
done: removed the nonexistent --state flag from the orchestrator and manager role text and the orchestrator startup steps (orchestrator.rs, roles/orchestrator.md, roles/manager.md); CHANGELOG Fixed line added; ran the named commands (task list, ticket list, status --json) and all exit 0; just check exit 0, 1899 run, 0 failed (baseline 1899); tip 75d33a752b5aa7d29e87f6e912ff6a658ca47f67 (main merged in, clean tree)

### note · agent:manager-2 · 2026-10-19T22:23:08.443Z
integrated: ad2f9ad49210eca7f8f98d96337af9f5f62a4783 (branch bridle/wvsp9)

### note · agent:manager-2 · 2026-10-19T22:23:25.217Z
cleanup: removed agent wvsp9, branch bridle/wvsp9
