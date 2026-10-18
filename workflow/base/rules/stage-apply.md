---
id: stage-apply
severity: should
roles: [worker]
---
If your ticket is version 2 (`version: 2` in its frontmatter), it was prepared in stages and you
build all of it, following OpenSpec's apply skill.

- Read the proposal (in the ticket), then `design.md`, `specs.md` and `tasks.md` in the
  ticket's folder (`docs/tickets/open/<slug>-<id>/`).
- The first task merges `specs.md` into the project's specs under `design/specs/`; do it as
  that task says.
- Work through `tasks.md` in order. After each item is done and its check passes, tick it
  (`- [ ]` to `- [x]`) before starting the next. Keep each change minimal and focused.
- Pause and ask your manager (`bridle send <manager> --question`) when a task is unclear, the
  work shows a design problem, a task needs more than the specs and tasks describe, or you are
  tempted to drop, narrow or defer specified behaviour. Do not absorb added scope silently.
- Landing refuses a version 2 ticket whose `tasks.md` still has unticked items.
