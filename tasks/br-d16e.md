+++
id = "br-d16e"
title = "Build workflow/packs/typescript/ (npm workspaces, vitest, tsc; no lint)"
kind = "feature"
state = "integrated"
created_at = "2026-09-28T23:52:33.904Z"
updated_at = "2026-09-29T01:35:01.849890Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

source: docs/questions/open/onboarding-survey-track-web-and-harness-u8sm.md, section 8 item 3
and the section 6 classification table (rows marked TS). Track-web onboarding (the human's
second-priority project) needs an L2 TypeScript pack. Mirror the existing packs exactly:
workflow/packs/python/ and workflow/packs/vim/ (a workflow.toml with `layer = "typescript"`
and rules/<pack>.<name>.md files; read one or two of the python pack's rule files first for
the frontmatter and voice, and docs/design/workflow-layers.md for the schema). The vim/python
packs were built by earlier tasks with a resolve test; copy that test pattern for this pack.

Content, generic to TypeScript projects, NOT track-web specifics (those go in track-web's own
.bridle/ on its trial branch, written by the orchestrator):
- typescript.check-command: the definition of done is the project's test command plus its
  build/typecheck, set as the project's `commands.check`; there is no lint by default. Give
  the pattern (`npm test && npm run build`), not a fixed command.
- typescript.package-manager: npm workspaces; use the lockfile as committed; `npm ci`/
  `npm install --prefer-offline` in a fresh worktree (the setup hook, br-42dd); don't add
  dependencies without saying so in the handoff.
- typescript.vitest-style: vitest conventions (colocated *.test.ts, no network, no real
  timers unless fake, deterministic).
- typescript.tsc: `tsc` builds; don't silence type errors with `any`/`@ts-ignore` without a
  comment saying why; keep tsconfig strictness as the project has it.
- typescript.dev-servers: never kill or restart another process's dev server; never take a
  port you weren't given; stop what you start; tell your own processes apart by pid. (This is
  the base of the survey's "stop what you start" rule; if workflow/base/rules/ already has a
  process-hygiene rule, don't duplicate it, reference it instead.)
Keep each rule short (survey style: a sentence or three); at most 5 rules. Tag roles as the
existing packs do.

Acceptance: `just check` passes, including a test that the pack resolves with base (same as
the python/vim pack tests); docs/design/workflow-layers.md or the packs index updated if the
other packs are listed anywhere.

Out of scope: track-web specifics (Phaser externalized, OpenSpec, dungeon-engine locks,
dev-ports), lint/formatter rules, a web-ui pack, browser/playwright verification (survey
stage 2), changes under crates/.

Size: small. Model: Sonnet (Haiku is fine if the python pack is followed line by line).

## Thread

### note · agent:pm-1 · 2026-09-29T01:35:01.849Z
integrated: e1d2bd0
