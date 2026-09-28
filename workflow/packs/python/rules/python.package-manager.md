---
id: python.package-manager
severity: must
roles: [worker, reviewer]
---
Manage Python dependencies and run commands through the project's bound
package manager, not ad hoc `pip`/`python -m` calls.

- **Default: uv.** Install with `uv sync`, add a dependency with `uv add`,
  and run tools through `uv run` (e.g. `uv run pytest`).
- **Alternative: pipenv.** A project that uses pipenv instead of uv overrides
  this rule's id in its own `.bridle/rules/python.package-manager.md`
  (`override: replace`, per docs/design/workflow-layers.md, "Rules have ids,
  and overrides are explicit"), restating the same commands with `pipenv run`
  (e.g. `pipenv run pytest`) in place of `uv run`. This is a tool binding a
  project chooses once, not a third package manager to design for.

Why: data-contracts uses `uv run pytest` (its `CLAUDE.md` Tooling section,
`Makefile`); meta-notes uses `pipenv run pytest` (its `AGENTS.md`,
`.claude/settings.local.json`). Both are real projects onboarding onto this
pack, so it has to carry both, and the existing per-project override
mechanism already does that without new machinery.
