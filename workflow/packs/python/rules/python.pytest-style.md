---
id: python.pytest-style
severity: may
roles: [worker, reviewer]
---
A project may write pytest tests as bare functions instead of classes:
`test_<module>_<function>_<scenario>` names, `tmp_path` for filesystem
fixtures, no `class Test...` groupings.

This is a `may`, not a pack-wide mandate — data-contracts doesn't follow it,
so a project opts in on its own layer (e.g. a project rule restating this as
a `should`/`must` for itself) rather than the pack requiring it of everyone.

Why: meta-notes' `AGENTS.md` states this style outright ("Python testing:
bare functions only, no classes, `test_<module>_<function>_<scenario>`,
`tmp_path`"). data-contracts' test suite doesn't use it. Two real projects
on this pack disagree, so it has to be optional.
