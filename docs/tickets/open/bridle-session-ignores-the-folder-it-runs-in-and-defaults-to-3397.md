---
id: 3397
title: bridle session ignores the folder it runs in and defaults to the bridle project
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

The human, verbatim (2026-10-04, via the aide), after finding two bridle aides running:

> I found out how I got two aides. I ran ` % bridle session aide` within the bridle-ui project;
> isn't that always scoped to the containing project folder like other commands? Or do I have to
> list the project name every time? it should try to detect the project from the folder. If it has
> bridle config, then open the agent inside that project.  this folder:
> /Volumes/Data/work/bridle-ui-workspace/bridle-ui

## Cause

`crates/bridle/src/session.rs` (~174):
`let project = cli.project.clone().unwrap_or_else(|| "bridle".into());`. With no `--project` or
`$BRIDLE_PROJECT`, every `bridle session` role (orchestrator, advisor, aide) is for **bridle**,
wherever it runs. Yet `claude` starts in the cwd (bridle-ui's clone). Run from bridle-ui, it
started a second `external:aide` for bridle, with bridle's token, in bridle-ui's folder. Every
other command finds its daemon by walking up from the cwd to `.bridle/daemon.json`
(`docs/design/agent-host/daemon.md`, Discovery, step 4).

## Fix

Pick the project the way discovery does: `--project`, then `$BRIDLE_PROJECT`, then the project
of the workspace containing the cwd (`.bridle/daemon.json` walking up, or the clone's
`.bridle/config.toml` and its registry entry). If none is found, refuse with a message naming
`--project`, rather than defaulting to bridle. The same applies to `bridle session restart`
and to `bridle advisor`'s pane launch (`session_command` in `crates/bridle/src/advisor.rs` passes
`--project` only when given).

Also worth considering: refuse a second session of the same identity in the same project while one
is registered and alive (`bridle status` sessions), so this mistake can't produce a duplicate.

Tests: from a temp workspace for project X, `bridle session aide` (stub claude,
`BRIDLE_LAUNCHER_TEST=1`) sets `BRIDLE_PROJECT=X`; outside any workspace without `--project`, it
errors. Docs: cli.md's `bridle session` entry.
