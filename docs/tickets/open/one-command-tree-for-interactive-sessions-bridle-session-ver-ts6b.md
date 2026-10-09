---
id: ts6b
title: "One command tree for interactive sessions: bridle session <verb> <seat>, retiring bridle advisor and the session-flavoured parts of bridle orchestrator"
kind: feature
opened: 2026-10-09
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv, seats-every-role-is-a-named-tracked-seat-that-outlives-its-s-gtzx, orchestrator-spins-off-an-advisor-ervd]
tasks: [br-ts6b]
---

## The ask

The human, 2026-10-09, after asking how to start a named advisor ("the command is `bridle
session advisor/name`?"; it is `bridle session advisor <name>`):

> create a ticket to unify this CLI as well - one command structure for all session-related CLI
> commands; so bridle advisor ... would go away and be replaced with the standard bridle session
> ... (as i understand it); don't start work yet, just file the ticket.

One command tree, `bridle session ...`, for everything about interactive sessions (orchestrator,
aide, advisors, and any later seats). `bridle advisor` goes away. Don't start work until the
human readies it.

## What exists today

From `bridle --help` and each subcommand's `--help`, on 103a44ae:

- `bridle session orchestrator [--project p] [claude args]`
- `bridle session advisor [--project p] [name] [claude args]`: the name is a positional word
  (`aide2`), not `advisor/aide2`
- `bridle session aide [--project p] [claude args]`
- `bridle session restart advisor/<name> [--handover|--fresh]`: here the seat is `advisor/<name>`
- `bridle session keep advisor/<name>`
- `bridle advisor start <name> [--brief TEXT|@FILE]`: sends a brief, then runs `bridle session
  advisor <name>` in a tmux pane (ticket [[orchestrator-spins-off-an-advisor-ervd|ervd]])
- `bridle orchestrator note-session | handover | prime | wait-for-wake`: session hooks and
  aliases (`wait-for-wake` duplicates `bridle agent wake external:orchestrator`; `handover`
  duplicates `bridle handover`)
- `bridle pane`: tags the tmux pane for orchestrator panes

The inconsistencies: launching is `session <role> [name]`, but restarting and keeping use
`session <verb> <role>/<name>`. Starting in a pane is a separate top-level `advisor` command.
Session plumbing also sits under `orchestrator`.

## Shape to decide (planner's call, the human approves)

A starting point, not a decision:

- One seat argument everywhere: `orchestrator`, `aide`, `advisor`, `advisor/<name>`. Every
  verb takes the same form.
- Verbs under `bridle session`: `start <seat> [--pane] [--brief ...] [-- claude args]` (folds in
  `advisor start`), `restart`, `keep`, and probably `list` (the registered sessions `bridle
  status` already shows) and `stop`.
- Decide where `orchestrator note-session`, `prime`, `wait-for-wake` and `pane` belong, and
  whether the old forms stay as deprecated aliases for a release. The launchers, role texts and
  docs name them, so these call sites need updating together: `docs/design/cli.md`,
  `docs/design/agent-host/orchestrator-supervision.md`, `workflow/base/roles/*.md`, and the role
  prime texts.

Overlaps the seats work ([[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]],
[[seats-every-role-is-a-named-tracked-seat-that-outlives-its-s-gtzx|gtzx]]). If seats make aide
and others nameable, this command tree should be the way to address them, so plan the two
together or land this one first with a seat argument that can grow.
