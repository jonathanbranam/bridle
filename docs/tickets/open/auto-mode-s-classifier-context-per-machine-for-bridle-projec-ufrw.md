---
id: ufrw
title: "Auto mode's classifier context per machine for bridle projects: who writes autoMode.environment, and how it stays current"
kind: question
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-ufrw]
---

## The ask


Requested by the human 2026-10-04, relayed by the track-web session track-web-f5: "investigate how
to configure Claude Code auto-mode settings for bridle projects on a per-machine basis." File as an
investigation.

Background (from track-web-f5):

- Auto mode's classifier context is `autoMode.environment`, a list of plain-text lines (trusted repo,
  trusted domains, where secrets live, deploy targets).
- Claude Code reads `autoMode` only from user `~/.claude/settings.json`, managed settings, or a
  per-launch `--settings` file / the Agent SDK. Project `.claude/settings.json` and
  `.claude/settings.local.json` are ignored on purpose, so a repo can't mark itself trusted
  (https://code.claude.com/docs/en/auto-mode-config.md, "Where the classifier reads configuration").
- The human's `~/.claude/settings.json` block on dalek was generated from inside track-web and only
  describes track-web. Its "Trusted repo" line pointed at an old checkout
  (`/Volumes/Data/work/pi/track-web`), not the bridle workspace layout
  (`/Volumes/Data/work/track-web-workspace/track-web` plus `.bridle/state` worktrees). The human fixed
  that line by hand, but the block is global: in every other project (bridle, harness, otters, ...)
  the classifier is told track-web is the only trusted repo.
- Claude can't edit `~/.claude/settings.json`; auto mode blocks it as self-modification. A fix is
  applied by the human or by tooling the human runs.

Questions:

1. Should bridle generate or maintain the machine-level `autoMode.environment` block (say, a `bridle`
   command that writes it for the human to review), covering every bridle-managed workspace on the
   machine, workspace dirs and worktree paths included?
2. Or should bridle launch its sessions and agents with `--settings <generated file>` per project, so
   the context matches the project without touching the global file?
3. How are project facts (deploy on push to main, prod hosts, domains) kept apart from machine facts
   (trusted repos, remotes under github.com:jonathanbranam/)?
4. How does it keep from going stale when workspaces move (the cause of this incident)?
