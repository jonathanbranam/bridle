---
id: 4eep
title: Per-role Claude Code settings for the agents bridle spawns
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [78sp, j2vq]
---

## The question

The human's words, 2026-09-27:

> BTW do we share a single Claude settings? That is something we probably
> need to address to scope worker permissions.

As built, every agent bridle spawns loads the user's `~/.claude/settings.json`
and the project's checked-in `.claude/settings.json` (every worktree has it).
The manager also runs in the main clone, so it loads the clone's untracked
`.claude/settings.local.json`, which holds the orchestrator's own permissions
(`Bash(bridle stop manager-*)`). Workers don't, because their worktrees only
have tracked files. Per-role scoping today is only `allowed_tools` and
`disallowed_tools` from `[roles.*]`, plus bridle's own `--settings`
(`crates/bridle-claude/src/command.rs`, no-memory).

Should bridle generate each role's settings (permissions plus no-memory) and
pass them with `--settings`, and keep spawned agents from loading the user's
and the clone's local settings at all? Whether `claude` has a flag to limit
which settings sources load needs checking against the real CLI.

## Resolution

`claude --help` has `--setting-sources <sources>`, comma-separated from
`user`, `project`, `local`. Bridle now always passes
`--setting-sources project`, which excludes the human's
`~/.claude/settings.json` and the clone's untracked
`.claude/settings.local.json`, keeping only the project's checked-in
`.claude/settings.json` (same for every worktree and agent). No bridle-
generated per-role settings JSON beyond what already existed
(`NO_MEMORY_SETTINGS` via `--settings`, and the `disallowed_tools` deny-lists
in role config): those already cover permission scoping, and
`--setting-sources` alone closes the leak this ticket raised. Recorded in
[[docs/design/agent-host/agents.md#Spawning|agents.md]].
