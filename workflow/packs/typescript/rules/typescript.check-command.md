---
id: typescript.check-command
severity: must
roles: [worker, reviewer]
---
Use the project's bound check command for "done", not a hardcoded TypeScript
invocation.

`workflow/base/skills/worker/SKILL.md` already renders `{{commands.check}}`
from `.bridle/config.toml`'s `[commands] check` (default `"just check"`,
docs/design/workflow-layers.md, "Per-project command bindings"). A TypeScript
project sets its own, e.g. `commands.check = "npm test && npm run build"` —
this pack doesn't restate or duplicate that logic, and no rule here should
hardcode `npm run test`, `vitest`, `tsc`, or any other TypeScript tool
invocation as *the* definition of done.

Why: projects have different test commands and build needs, so the binding —
not this pack — is what carries that difference.
