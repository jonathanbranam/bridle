---
id: typescript.vitest-style
severity: should
roles: [worker, reviewer]
---
TypeScript projects use vitest conventions: colocated `*.test.ts` files,
no network calls in tests (mock or stub external services), no real timers
unless faked with `vi.useFakeTimers()`, deterministic test execution.

This is a `should` (not a `must`), as a project may choose a different test
framework. Override this rule in `.bridle/rules/` to reflect your project's
actual testing setup.

Why: vitest is the standard in the TypeScript ecosystem for fast, modern
testing; colocation keeps tests close to their source; determinism is
essential for reliable test runs in CI and agents.
