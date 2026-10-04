---
id: missing-tools
severity: must
roles: [product-manager, manager, worker, reviewer]
---
When a tool the project's checks need is missing (a test runner, a linter, a plugin the tests
load), stop and ask: `bridle send <manager> --question` (a worker) or a `question` to the human (a manager).
Don't substitute another tool, skip part of the check, or only mention it in a done line.

A manager treats a check that ran only in part as not passed: it doesn't merge, and asks the human.

Why: on meta-notes (2026-09-30) `pipenv` and `vader.vim` weren't installed. The worker ran pytest
through `uv`, skipped the Vader tests, said so only in its done line, and the manager merged. The
human: "that needs to be done for the overall workflow, not just our local fork."

Until `bridle workflow sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
