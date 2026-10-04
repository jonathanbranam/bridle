---
id: b795
title: The base timestamp hook stamps every agent's prompts; the human wanted it only for the notes advisor
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-b795]
---

## The ask


br-sf79 (daf7e623, in v0.5.0) added `workflow/base/hooks/UserPromptSubmit.json`, which runs
`date '+Message sent: ...'` on every prompt. Base hooks reach every project, and `bridle session`
passes layer hooks to interactive sessions too, so every agent and session in every project gets
the stamp. The human, 2026-10-04, relayed by the NUC orchestrator (message m-0307 on meta-notes):

> Timestamp hook is only for an advisor interactive agent in the notes project. Not for all agents
> and all projects.

On the NUC the orchestrator has overridden it per project for now: meta-notes and dotfiles-local
have `.bridle/hooks/UserPromptSubmit.json = []`; notes guards the `date` command with
`[ "$BRIDLE_AS" = advisor ]`.

Wanted:

1. Take the timestamp hook out of the base layer (`workflow/base/hooks/UserPromptSubmit.json`
   becomes `[]` or goes, whichever keeps layer merging simple). The notes project adds it in its own
   `.bridle/hooks/` (outside this repo; the NUC already has it).
2. Optional, only if small: a per-role filter for layer hooks, so a project can scope a hook to
   one role without the `$BRIDLE_AS` guard. Otherwise leave the env guard as the documented way.
3. Docs (wherever sf79's hook is described, grep `Message sent` / `UserPromptSubmit` in docs/) and
   a CHANGELOG line saying projects that want the stamp add it themselves.

Also check dalek's projects for local overrides that only existed to undo the base hook.
