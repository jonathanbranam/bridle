---
id: c7mn
title: Rename `bridle task note` to `bridle task comment`
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [group-the-cli-into-subcommands-a67t]
closed: 2026-10-02T00:43:36.041801Z
---

## The ask

The human (2026-10-01, relayed by the NUC orchestrator, m-2994), verbatim: "I understand it isn't
threaded, but these are called comments in JIRA, github issues, etc. not \"notes\". I couldn't find
the command for comments and was confused - so will other users."

Rename `bridle task note` to `bridle task comment`. Follow a67t's churn plan: keep `task note` as a
hidden alias for a release, and update every reference in the same change.

## Surfaces

- The subcommand, its help text, and `--notify` ("adds a note to the thread").
- `bridle send --task` help text, wherever it says "note".
- The to-do message wording "<task>: note added" (and similar user-facing strings).
- Role docs, skills and rules that tell agents to use `task note`, e.g.
  `workflow/base/rules/talk-on-the-task.md` and `workflow/base/skills/worker/SKILL.md`.
  Also `docs/design/cli.md` and `docs/design/coordination.md`.
- Shell completions pick the new name up from clap.

Leave alone: the wire format and storage (a rename there has no user benefit), and message
`kind = "note"`, which is a different concept (a message that isn't a question).

## Done when

`bridle task comment` works, `task note` still works but is hidden, no doc or role tells anyone to
use `task note`, and a test covers the alias.

## Resolution

Resolved by: br-9c3b (714b1c6)
