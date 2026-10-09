---
id: w3pk
title: "Retire scripts/claude-orchestrator and claude-advisor: the orchestrator supervisor still relaunches through the wrapper"
kind: chore
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [mrhe]
tasks: [br-w3pk]
closed: 2026-10-09T23:11:02Z
---

## The ask

The human, 2026-10-06: "the bridle restart runs /Volumes/Data/work/bridle/bridle/scripts/claude-orchestrator but aren't we done with that script?"

`scripts/claude-orchestrator` and `scripts/claude-advisor` have been compat wrappers since mrhe / br-85bc: each one runs `bridle session orchestrator|advisor`, and the header says "remove once nothing calls it". The last caller is the daemon itself. `[orchestrator] launcher` defaults to `"scripts/claude-orchestrator"` (`crates/bridle-daemon/src/config.rs`, `OrchestratorConfig::default`), and the supervisor (`crates/bridle-daemon/src/orchestrator.rs`) resolves it to an absolute path in the repo and types that path into the tagged pane on every relaunch.

The ask:

- The supervisor relaunches with `bridle session orchestrator --project <project>` and stops resolving `launcher` as a repo path. Keep `launcher` as an override typed into the pane verbatim, or drop it if nothing sets it. Today no config sets it: not `~/.bridle/config.toml` and not `.bridle/config.toml`. Pick the smaller change.
- Delete `scripts/claude-orchestrator` and `scripts/claude-advisor` (step two, below).
- Update the comments and docs that name them: `docs/design/agent-host/orchestrator-supervision.md`, `docs/design/agent-host/roles-and-config.md` (the `launcher` default), `docs/design/cli.md` (`orchestrator note-session`), and the code comments in `crates/bridle/src/session.rs`, `crates/bridle/src/commands/orchestrator.rs`, `crates/bridle-mail/src/local.rs` and `crates/bridle-daemon/src/orchestrator.rs`. Leave CHANGELOG history and resolved tickets alone. Add a CHANGELOG entry.

Check: `just check`. The supervisor's tests with the fake tmux must show the new line typed into the pane.

Order matters. The running daemon is the installed binary, but it types the script's path in the clone, so
deleting the scripts in the same merge would break the relaunch until the daemon self-upgrades. Do it in two steps:
first the new relaunch line, then deleting the scripts and their doc mentions in a follow-up task, once
`bridle status` shows the daemon running a build with step one.

Model: Haiku or Sonnet (small, mechanical, but it touches the supervisor's relaunch path).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
