---
id: 78sp
title: Agents can use Claude Code's own SendMessage and other built-ins that bypass bridle
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [v4nk]
closed: 2026-09-30T05:12:44Z
---

## The question

On 2026-09-27 the worker `fix-updated-at` loaded Claude Code's built-in
`SendMessage` tool through `ToolSearch` and reported "done" with it instead of
`bridle send`. The report never reached its manager (`manager-2`), and main
stayed red until the orchestrator noticed. The worker role's `allowed_tools`
(`Bash, Read, Edit, Write, Glob, Grep`) didn't stop it: an allow list grants
permissions, it doesn't limit which tools an agent can load.

`.bridle/config.toml` now puts `SendMessage` in `disallowed_tools` for the
worker and manager roles (commit 979a714). Which other Claude Code built-ins
should bridle deny every agent by default (messaging, scheduling, subagents,
remote triggers), and should that list live in the built-in role defaults
rather than each project's config?

## Resolution

Implemented in `crates/bridle-daemon/src/config.rs` (lines 83–91):
`DENY_MESSAGING_AND_SUBAGENTS` (`SendMessage`, `Workflow`), `DENY_SCHEDULING`
(`ScheduleWakeup`, `CronCreate`, `CronDelete`, `CronList`), and
`DENY_REMOTE_TRIGGERS` (`RemoteTrigger`) are now built-in role defaults applied
in worker_default(), manager_default(), and orchestrator_default(). Tests at
lines 1108–1128 verify the denials are in place.

Resolved 2026-09-28.
