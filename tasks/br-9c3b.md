+++
id = "br-9c3b"
title = "Rename bridle task note to bridle task comment, hidden alias kept (c7mn)"
kind = "chore"
state = "planned"
created_at = "2026-10-01T01:04:29.370Z"
updated_at = "2026-10-01T01:04:31.497324Z"
size = "S"
+++

Ticket: docs/tickets/open/rename-task-note-to-task-comment-c7mn.md (read it).

Goal: `bridle task comment <task> ...` is the command; `bridle task note` keeps working as a hidden alias (clap alias with hide, as a67t did for the old names). Update every reference in the same change.

Do: the subcommand, its help, the `--notify` help, `bridle send --task` help, user-facing strings like "<task>: note added" (careful: tests may assert the strings); role prompts, skills and rules under workflow/ and .bridle/ that say `task note` (grep for "task note" and "bridle task note", e.g. workflow/base/rules/talk-on-the-task.md, workflow/base/skills/worker/SKILL.md), docs/design/cli.md, docs/design/coordination.md. If the repo keeps rendered copies of rules/roles checked in (.bridle/ or .claude/), keep them in step with the sources. Add a test that `task note` still works (and does not appear in `task --help`) and that `task comment` writes the same thread entry. CHANGELOG entry.
Leave alone: wire format, storage, thread entry kind "note", and message kind = "note" (a different concept).
Acceptance: just check passes; grep finds no doc or role telling anyone to use `task note` except the alias mention in cli.md.
Model: Haiku (mechanical). Out of scope: dropping the alias; renaming anything else.
