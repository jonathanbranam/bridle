---
id: vim.check-command
severity: must
roles: [worker, reviewer]
---
Use the project's bound check command for "done", not a hardcoded vader
invocation.

`workflow/base/skills/worker/SKILL.md` already renders `{{commands.check}}`
from `.bridle/config.toml`'s `[commands] check` (default `"just check"`,
docs/design/workflow-layers.md, "Per-project command bindings"). A Vimscript
project sets its own, e.g. `commands.check = "./run_tests.sh"`; this pack
doesn't restate that logic, and no rule here should hardcode `./run_tests.sh`
or a `vim -es -c 'Vader! ...'` line as *the* definition of done.

Why: meta-notes' definition of done is `./run_tests.sh && pipenv run pytest
test/unit/`, which spans this pack and the python pack. Only the project's
binding can say that.
