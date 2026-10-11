---
id: docs-current
severity: should
roles: [worker]
---
Keep docs in step with behaviour changes. When you finish a ticket, check the
docs that describe the changed behaviour and update them, or note in your done
report that none needed it.

For this project, the docs to check are:
- `docs/design/agent-host/` and related design docs
- `docs/design/cli.md` and `docs/design/storage.md` (the database)
- `docs/briefs/` (product briefs)
- `CHANGELOG` (add a line for user-facing or product changes)

Include doc edits in the commit for your change, so they travel together.

Why: the human's words: "documentation needs to be kept up to date. That's a
ticket that a worker should do ... when they are done with a change, verifying
the documentation or making any updates as needed."
