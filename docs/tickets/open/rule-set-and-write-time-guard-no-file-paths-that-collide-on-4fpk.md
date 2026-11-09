---
id: br-4fpk
title: "Rule set and write-time guard: no file paths that collide on a case-insensitive disk (mu-bgb6)"
kind: feature
created: 2026-11-09
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

Filed by aide from m-18476 (aide@vesta, relaying the human on meta-notes-ui ticket mu-bgb6).

The human, 2026-11-09, verbatim: "And this problem should never occur again. We need to prevent it. ... this should be part of the Webpack or a React Webpack Bridle rule set, right? This is a standard, common thing that everybody is going to run into. It should be handled by a Bridle rule set as a rule when writing files."

The human, 2026-11-09 (to aide), verbatim: "this ticket was filed due to the need for this fix in meta-notes-ui which I use at work; i'd like it to be filed and worked on to keep my at-work projects moving."

## What happened

A fresh clone of meta-notes-ui failed `npm run build` on macOS:

    client/src/TodayView.tsx (2:9): "TimeLogPanel" is not exported by "client/src/timelog.ts"

client/src had three pairs whose names differ only by case, with different extensions: timelog.ts/TimeLog.tsx, tagview.ts/TagView.tsx, menu.ts/Menu.tsx. TodayView.tsx has `import { TimeLogPanel } from "./TimeLog"`. Vite tries .ts before .tsx by default, and on a case-insensitive disk TimeLog.ts opens timelog.ts, so the import silently binds to the wrong module.

Nothing caught it: workers and CI run on case-sensitive Linux, where these are distinct files. tsconfig's forceConsistentCasingInFileNames doesn't help, because the imports match the real on-disk case exactly. The first Mac build was the first failure.

## The ask

Agents must not be able to create paths that collide on a case-insensitive disk, enforced when writing files, not only written down as a rule. No rule in workflow/packs or workflow/base/rules at v0.7.6 covers this.

Scope suggested by aide@vesta (not yet settled by the human):

1. Not Webpack-specific (this project uses Vite), so the typescript pack looks like the home: PascalCase.tsx for components, kebab-case.ts for the rest, no two paths equal when lowercased with the extension stripped.
2. The same-extension form (Foo.md/foo.md) breaks a checkout on macOS or Windows in any language, so that part may belong in base.
3. The human asked for it "when writing files", so a written rule alone isn't enough. Candidates: a hook refusing a Write/Edit or git add that creates a colliding path, plus a pack-provided check in the check command as the backstop, since agents can write files other ways.

mu-bgb6 is adding a project-local version (a case-collision check in npm run check, and a naming rule in .bridle/rules): a possible starting point for the pack.
