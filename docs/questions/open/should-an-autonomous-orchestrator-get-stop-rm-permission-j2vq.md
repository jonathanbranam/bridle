---
id: j2vq
title: Should an autonomous orchestrator be granted bridle stop/rm permission?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

During bridle's first self-hosted run, an orchestrator Claude Code session
running in a fully-autonomous mode was refused permission to run `bridle
stop` or `bridle rm` on agents by its own safety classifier, even though it
had legitimate cause (cleaning up agents it had itself spawned). The human
had to do that cleanup by hand.

Should — and if so how — an autonomous orchestrator be granted that kind of
operational permission, given that stopping/removing agents is exactly the
kind of lifecycle management an orchestrator role is meant to do?

## Why it matters

This is a gap between what the orchestrator role needs to do
(`docs/design/agent-host/roles-and-config.md`) and what Claude Code's own
permission classifier allows an autonomous session to do unattended,
independent of anything bridle itself enforces.

## Notes

2026-09-28: branch `bridle/j2vq-orchestrator-perms` added `Bash(bridle *)` to the tracked
`.claude/settings.json`, which grants it to every Claude Code session opened in the repo,
not only the orchestrator. The advisor recommended scoping it to the orchestrator instead
(untracked `.claude/settings.local.json`, or a flag passed from
`scripts/claude-orchestrator`). The human: "j2vq accept your recommendation". Relayed to
manager-2 (m-0861).
