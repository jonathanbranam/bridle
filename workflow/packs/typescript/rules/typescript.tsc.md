---
id: typescript.tsc
severity: must
roles: [worker, reviewer]
---
Use `tsc` to build and type-check; don't silence type errors with `any` or
`@ts-ignore` without a comment explaining why.

Keep the project's `tsconfig.json` strictness settings as they are. If a type
error arises during development, fix it or document the suppression with a
comment (`@ts-ignore` with a reason, or cast to a wider type) so future readers
understand the context.

Why: strict TypeScript catches real bugs early. Silent suppressions hide
problems and accumulate technical debt; comments preserve the "why" for
later refactors.
