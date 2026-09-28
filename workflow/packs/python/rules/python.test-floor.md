---
id: python.test-floor
severity: must
roles: [worker, reviewer]
---
Run the test suite (and, where used, ruff) against the project's supported
Python floor, not whichever interpreter happens to be on the development
machine.

The floor value itself is a per-project setting — this pack doesn't hardcode
one. A project states its floor in its own layer (e.g. `.bridle/rules/` or
`.bridle/facts.md`, docs/design/workflow-layers.md's "What a layer contains")
and the worker checks there, not here, before running tests.

Why: data-contracts documents a 3.12 floor and develops on 3.14 (its
`CLAUDE.md` Tooling section, `openspec/config.yaml`'s `context:` block).
meta-notes documents a 3.11 floor in `AGENTS.md` but its `Pipfile` pins 3.14,
so its own floor and its development interpreter already disagree — a reason
this pack states the rule rather than a number, and leaves reconciling it to
the project.
