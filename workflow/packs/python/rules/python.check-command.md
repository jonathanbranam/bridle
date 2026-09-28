---
id: python.check-command
severity: must
roles: [worker, reviewer]
---
Use the project's bound check command for "done", not a hardcoded Python
invocation.

`workflow/base/skills/worker/SKILL.md` already renders `{{commands.check}}`
from `.bridle/config.toml`'s `[commands] check` (default `"just check"`,
docs/design/workflow-layers.md, "Per-project command bindings"). A Python
project sets its own, e.g. `commands.check = "make check"` — this pack
doesn't restate or duplicate that logic, and no rule here should hardcode
`uv run pytest`, `pipenv run pytest`, `ruff check .`, or any other Python
tool invocation as *the* definition of done.

Why: data-contracts' definition of done is `make check` (lint, then
`check-specs.py`, then pytest); meta-notes' is
`./run_tests.sh && pipenv run pytest test/unit/`. Neither is `just check`,
and both differ from each other, so the binding — not this pack — is what
has to carry the difference.
