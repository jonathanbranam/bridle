---
id: typescript.file-naming
severity: must
roles: [worker, reviewer]
---
Name TypeScript files so a case-insensitive checkout (macOS, Windows) cannot
confuse two of them.

- **Components**: `PascalCase.tsx` (`TimeLog.tsx`).
- **Everything else**: `kebab-case.ts` (`time-log.ts`).
- **No two paths equal when lowercased with the extension stripped**, in one
  directory: `TimeLog.tsx` beside `timelog.ts` makes a bundler bind
  `import "./TimeLog"` to the wrong file on a Mac while Linux CI stays green.

Backstop: add `bridle case-check` to the project's check command
(`commands.check = "bridle case-check && npm test && npm run build"`). It scans
`git ls-files` and exits 1 naming each colliding pair. The base rule
`no-case-colliding-paths` and its `bridle case-guard` hook stop the Write tool;
this catches files made any other way. To fix a collision, rename one file; the
check never renames.

Why: meta-notes-ui (2026-11-09, mu-bgb6): `TimeLog.tsx` beside `timelog.ts`
broke a fresh macOS build; Linux workers and CI never saw it.
