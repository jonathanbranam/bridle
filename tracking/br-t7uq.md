+++
id = "br-t7uq"
title = "List external sessions, and list sessions and agents across all projects (--all-projects)"
kind = "feature"
state = "pending"
created_at = "2026-10-22T00:58:06.025Z"
updated_at = "2026-10-22T00:58:25.631861Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "t7uq"
+++

docs/tickets/open/list-external-sessions-and-list-sessions-and-agents-across-a-t7uq.md

submitted by external:orchestrator

## The ask

Two gaps in listing who is running:

1. There is no command that lists external sessions (aide, advisors, orchestrator, mail,
   statusline). Today the only views are the `session` lines in `bridle status` (running
   sessions on one project) and `bridle recipients` (every known identity, running or not).
   Add `bridle session list`: the running external sessions, with name, machine, context,
   uptime and last activity (what the `bridle status` session lines show).
2. Both `bridle session list` and `bridle agent list` work on one project's daemon. Give both
   a flag (e.g. `--all-projects`) that lists across every project in the registry, with a
   project column. Without the flag, both keep today's one-project behaviour.

The human, 2026-10-21 (to the orchestrator, verbatim): "Yes, please file a ticket for that;
should that be across all projects? I'm not sure; ideally, yes it would be great to list things
across all projects, but that isn't how bridle agent list works; ideally both commands would
take a param to list across all projects, IMO"

Context: the human was shutting down the aides and looking for idle agents and sessions to stop;
answering "which external sessions are running where" took `bridle status` once per project.

## Acceptance

- `bridle session list` prints the running external sessions on the current project.
- `bridle session list --all-projects` and `bridle agent list --all-projects` list every
  registry project's sessions or agents, with a project column. A daemon that can't be reached
  is shown as such, not silently skipped.
- `--json` works for both.

## Out of scope

Other machines' daemons (vesta, remus): local registry only, for now.

Model: Sonnet (CLI plus a fan-out over the registry; small).

## Thread

### note · external:orchestrator · 2026-10-22T00:58:06.025Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-22T00:58:25.631Z
orchestrator: feature; waiting for the human's go before readying (asked 2026-10-21). Will ready and tell pm-1 when it comes.
