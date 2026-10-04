---
id: 3397
title: Every bridle command finds its project from the folder it runs in (bridle session defaults to bridle), enforced by a spec
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: [docs/design/cli.md]
needs: []
see: []
tasks: [br-3397]
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

## Wider: every command, enforced by a spec

The human, verbatim (2026-10-04, via the aide):

> every command should include this behavior. There should be a spec enforcing that behavior for all commands.

So the fix is not just `bridle session`: **every** `bridle` command that acts on a project picks it
the same way (`--project`, `$BRIDLE_PROJECT`, then the workspace containing the cwd), through one
shared resolver, and none falls back to a hard-coded `bridle`. Other places seen at a glance
(not a full audit): `crates/bridle/src/launchd.rs` `project_name` (falls back to the repo folder
name, then `"bridle"`), `commands/orchestrator.rs` ~550, `statusline.rs` ~237 (display text, probably fine).

And a **spec** states it and is enforced. bridle has no `design/specs/` yet (`docs/design/specs.md`:
built, not wired in), so this would be its first requirement, or a requirement in `docs/design/cli.md`
until specs exist. Either way, enforce it with a test over the whole clap command tree: every
subcommand resolves its project through the shared resolver, run from a temp workspace for
project X with no `--project`, and an unresolvable project errors rather than meaning bridle. A new
command that skips the resolver should fail the test.
