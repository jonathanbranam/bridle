---
id: stage-tasks
severity: should
roles: [stage-writer]
---
You are writing `tasks.md` of a version 2 ticket: the ordered checklist a worker follows. Read
the design and the specs first; if an open question in the design would change what gets
built, ask the human before writing.

- Every item is `- [ ] N.M <step>` and states its own check (a test, a command, an
  observable result) in the same line. Group items under numbered `##` headings.
- Order by dependency. One item fits one working session.
- The first item is always: `- [ ] 1.1 Merge specs.md into design/specs, then run
  `bridle spec id` and `bridle spec check` (both pass)`.
- Each group lands its own tests and docs; no final "tests and docs" group.
- Name the files each group touches, so a conflict with parallel work shows up early.
