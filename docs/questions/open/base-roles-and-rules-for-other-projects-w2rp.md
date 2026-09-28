---
id: w2rp
title: Base roles, skills and rules don't fit a second project yet
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [ajqa, 63rv, rxe8]
---

## What happened

Onboarding meta-notes (ajqa; trial branch `bridle-adopt` at 053089e in
`/Volumes/Data/work/meta-notes-workspace/meta-notes`) on the fast path found these gaps. The
trial works around each one in its project layer.

1. **Rules don't reach workers.** `bridle prime` is orchestrator-only, yet the CLAUDE.md block
   that `bridle sync` writes tells agents to run `bridle prime <role>`. meta-notes' worker
   prompt tells workers to read `.bridle/rules/` and the base rules' absolute path instead.
2. **`workflow/base/roles/{worker,manager}.md` are bridle's own.** They mention `just check`,
   Rust, the fake claude, `main` and "release tags are the orchestrator's". A project has to
   write its own prompts (meta-notes: `.bridle/roles/`).
3. **`workflow/base/skills/manager/SKILL.md` hardcodes `main`, `just check` and "two
   workers"**, unlike the worker skill, which substitutes `{{branches.integration}}` and the
   check command. The rendered bridle-manager skill is wrong for meta-notes.
4. **No roles are inherited from `workflow/base`** (`workflow.toml` declares none), and the
   built-in manager allows `Bash(git *)`, so every project must declare its worker and
   manager in full.
5. **`project` isn't a config key.** Unknown keys stop the daemon (`deny_unknown_fields`),
   and the name comes from the directory. That's fine, but the onboarding docs should say so.

## What to do

Make the base role prompts and the manager skill project-neutral, with substitutions for the
integration branch, check command and worker count. Deliver rules to every role. Consider
base role definitions a project inherits and overrides. Then trim meta-notes' `.bridle/roles/`
on its trial branch.
