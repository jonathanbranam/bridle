---
id: no-case-colliding-paths
severity: must
roles: [orchestrator, project-manager, manager, worker, reviewer, document-reviewer, advisor, aide, prototyper, designer]
locked: true
---
Never create a path that equals another path in the same directory when
lowercased, with the extension kept (`Foo.md` and `foo.md`) or stripped
(`TimeLog.tsx` and `timelog.ts`). macOS and Windows checkouts collide on these
while Linux CI keeps them apart, so nothing fails until someone builds on a Mac.

- Edit the existing file, or pick a name that differs by more than case and extension.
- `bridle case-guard` (a PreToolUse hook) refuses a Write, Edit or NotebookEdit that
  would collide; it does not see files made by other means, so check your own `git mv`
  and shell writes.

Why: meta-notes-ui (2026-11-09, ticket mu-bgb6): `TimeLog.tsx` beside `timelog.ts`.
A fresh macOS clone bound `import ... from "./TimeLog"` to the `.ts` file and
`npm run build` failed; Linux workers and CI never saw it. The human: "this problem
should never occur again. We need to prevent it ... It should be handled by a Bridle
rule set as a rule when writing files."
