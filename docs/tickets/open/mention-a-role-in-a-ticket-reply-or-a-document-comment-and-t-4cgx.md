---
id: 4cgx
title: "@-mention a role in a ticket reply or a document comment and that role gets a message"
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [bp2v, 22ab, 72t9]
tasks: [br-4cgx]
next_comment_id: c4
---

## The ask

The human, 2026-10-09 ~6:30 PM ET, verbatim (to advisor product-manager):

> let's add a ticket for this; find an appropriate theme and epic if there is one; non-urgent; low pri
>
> If the human (or any agent?) adds a reply to a ticket (currently task) or a comment on a document that @ mentions a role, then a message is sent to that role. default is a role on the same project.
>
> examples:
>
> - @orchestrator I approve this ticket
> - @advisor/product-manager please add this to epic 32k3
>
> TBD - do we have a syntax for a role on another project?

Why it came up: the human asked whether approving br-8c25 by a comment on the task would reach the
right people. It wouldn't have: a comment tells only the task's watchers and claimant (`bridle
task comment` in docs/design/cli.md), and the orchestrator, which was holding the land, wasn't
watching. `--notify AGENT` exists on the CLI, but nothing in the UI or in document comments does
the same.

## The ask, as understood (advisor product-manager)

- A reply on a ticket's thread (today a task's) or a comment on a document under review that
  contains `@<role>` sends that role a message: who wrote it, where (ticket or document, with
  its link), and the text. One message per mention, also when the author is an agent (the
  human's "(or any agent?)" is open; the PdM's suggestion: yes, the same rule for everyone, so
  agents can hand off on the thread instead of in DMs, rule `talk-on-the-task`).

> [!comment] c2 human, 2026-10-09 17:11 EDT, on "PdM's suggestion: yes, the same rule for everyone, so agents can hand off on the thread instead of in DMs" [pending 2026-10-09 17:11 EDT]
> Agree and approve this direction - agents can use the same syntax to tag another agent. This is NOT NECESSARY when the agent tagged is watching the task, so use this sparingly and only when actually necessary.

> [!comment] c3 human, 2026-10-09 17:12 EDT, on "ticket's thread (today a task's)" [pending 2026-10-09 17:12 EDT]
> This should be de-duplicated with the watchers list so that agents only receive a single message. If they watch the ticket/task and are @ mentioned, then only one message should be sent to the agent, preferably the one from the mention (since it is more explicit).

- `@<role>` means that role on the same project: `@orchestrator`, `@aide`, `@advisor`, and
  named sessions as `@advisor/product-manager`.
- An unknown role is reported back to the author, not dropped.
- **External roles** (the human, 2026-10-09 ~6:35 PM ET, verbatim: "if the role is an external
  role, detect that and the system sends to external:"): the author writes `@orchestrator`, not
  `@external:orchestrator`; the system knows which roles are external sessions (orchestrator,
  advisor, aide, mail; the one token-role list `TOKEN_ROLES` from br-8c25) and addresses
  `external:<role>`. Agents in the workforce (`agent:<name>`) by name likewise.

## Open: a role on another project

No syntax exists today. Cross-project messages use the CLI flag only: `bridle send --project
<other> <to>` (docs/design/cli.md, 3haz). `@` is already taken inside principal names
(`human@<machine>`, docs/design/agent-host/principals.md). Options for the design:

> [!comment] c1 human, 2026-10-09 17:10 EDT, on "Options for the design" [pending 2026-10-09 17:10 EDT]
> Approve both of these options - the project short prefix is readily available and used extensively, guaranteed unique among connected projects (soon).

- `@bridle-ui:orchestrator`: project name, then role. Readable; the colon also appears in
  `external:aide`, which the mention would never include.
- `@ui:orchestrator`: the project's ID prefix, matching project-qualified ticket IDs (`ui-vnuu`;
  22ab section 8). Shorter, a bit more opaque.

Needs a short design (the syntax, where mentions are parsed: the daemon's task-comment path and
doc_watch for documents) before build. Related: bp2v / br-cufw (who can I message), 22ab step 4
(br-72t9: the thread moves to the ticket; mentions should work on the new thread too).
