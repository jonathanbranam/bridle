---
id: 78sp
title: Agents can use Claude Code's own SendMessage and other built-ins that bypass bridle
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [v4nk]
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
