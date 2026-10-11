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
next_comment_id: c8
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
>
> **human, 2026-10-09 17:13 EDT:** See below for clarification: we should build in de-duplication for this case. [read 2026-10-09 22:45 EDT]
>
> **doc-4cgx, 2026-10-09 22:50 EDT:** @human Done: agents use the same syntax, with a note to use it sparingly (only when the agent is not already watching). De-duplication is in the new bullets under "The ask, as understood".
>
> **resolved by human via doc-4cgx, 2026-10-09 22:50 EDT**

> [!comment] c3 human, 2026-10-09 17:12 EDT, on "ticket's thread (today a task's)" [read 2026-10-09 22:45 EDT]
> This should be de-duplicated with the watchers list so that agents only receive a single message. If they watch the ticket/task and are @ mentioned, then only one message should be sent to the agent, preferably the one from the mention (since it is more explicit).
>
> **doc-4cgx, 2026-10-09 22:50 EDT:** @human Added: one message per agent per comment; if the agent is also a watcher, the mention's message is the one sent. See the new "One message per agent" bullet.
>
> **resolved by doc-4cgx, 2026-10-09 22:50 EDT**

> [!comment] c5 human, 2026-10-09 17:16 EDT, on "contains @<role>" [read 2026-10-09 22:45 EDT]
> To block this, preceding the @ with a backslash "escapes" the character and disables the behavior for use in documents and comments such as: \@orchestrator <- this would *not* be sent to the orchestrator.
>
> **doc-4cgx, 2026-10-09 22:50 EDT:** @human Added an "Escaping" bullet: `\@orchestrator` is plain text and sends nothing. The backslash stays in the stored text.
>
> **resolved by doc-4cgx, 2026-10-09 22:50 EDT**

> [!comment] c6 human, 2026-10-09 17:21 EDT, on "comment on a document under review" [pending 2026-10-09 17:21 EDT]
> See also re79 which should be designed and planned along with this ticket - adds a reviewer: <agent> frontmatter (optional, ignored if missing) that indicates that every comment in a document is sent to that specific agent. In this case, also de-duplicate the messages as above; 
>
> If review: agent-a and a comment includes \@agent-b the comment is sent to agent-b and not agent-a (unless agent-a is also mentioned.
>
> **human, 2026-10-09 17:22 EDT:** I'm not sure about this at second thought; evaluate this for consistency across our systems - An \@mention in a ticket THREAD will notify that agent and all watchers; maybe we should treat the reviewer the same? I'm not sure. [read 2026-10-09 22:45 EDT]
>
> **doc-4cgx, 2026-10-09 22:50 EDT:** @human Recommend the consistent rule: the reviewer is like a watcher of the document. Every comment goes to the reviewer; a mention adds the mentioned agents; nobody gets two. So with `reviewer: agent-a` and a comment mentioning \@agent-b, both get it (same as a ticket thread: mentioned agent plus watchers). It is also simpler: no "mention replaces the reviewer" case. I wrote this into a "Reviewer front matter (re79)" bullet as the proposal. Say yes and I will resolve; or tell me if you prefer the first version.

> [!comment] c7 human, 2026-10-09 17:21 EDT, on "sends that role a message" [read 2026-10-09 22:45 EDT]
> If more than one \@mention exists in a single thread or comment, a message is sent to every agent in the message.
>
> **doc-4cgx, 2026-10-09 22:50 EDT:** @human Added: every distinct agent mentioned gets one message (see the "One message per agent" bullet).
>
> **resolved by doc-4cgx, 2026-10-09 22:50 EDT**

- **One message per agent.** A comment with several mentions sends one message to each distinct
  agent mentioned. An agent who is also a watcher of the ticket/task gets only the mention's
  message (it is the more explicit one), not the watcher notice as well.
- **Escaping.** `\@orchestrator` (a backslash before the @) is plain text and sends nothing.
- **Agents tagging agents** works the same way, but is only for an agent who is not already
  watching the task: use it sparingly.
- **Reviewer front matter ([[comments-on-a-ticket-an-agent-asked-the-human-to-approve-go-re79|re79]])**,
  design together with this ticket. Proposal: the document's `reviewer: <agent>` is treated like a
  watcher. Every comment goes to the reviewer, mentions add the mentioned agents, and nobody gets
  two messages. Same rule as a ticket thread. Open: the human has not confirmed (c6).
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
(`human@<machine>`, docs/design/agent-host/principals.md). Options for the design (the human approved
both, 2026-10-09; the explicit `external:` form is also accepted, as in `@bridle-ui:external:orchestrator`):

> [!comment] c1 human, 2026-10-09 17:10 EDT, on "Options for the design" [read 2026-10-09 22:45 EDT]
> Approve both of these options - the project short prefix is readily available and used extensively, guaranteed unique among connected projects (soon).
>
> **doc-4cgx, 2026-10-09 22:50 EDT:** @human Noted: both forms are approved. Marked them as approved in the list below.
>
> **resolved by doc-4cgx, 2026-10-09 22:50 EDT**

- `@bridle-ui:orchestrator`: project name, then role. Readable; the colon also appears in
  `external:aide`, which the mention would never include.

> [!comment] c4 human, 2026-10-09 17:15 EDT, on "bridle-ui:orchestrator: project name, then role. Readable; the colon also appears in external:aide, which the mention would never include" [read 2026-10-09 22:45 EDT]
> is \@bridle-ui:external:orchestrator also valid syntax? I think it should be valid.
>
> **doc-4cgx, 2026-10-09 22:50 EDT:** @human Yes. Added: the explicit `external:` form is accepted everywhere the short form is, same result (`@bridle-ui:external:orchestrator` = `@bridle-ui:orchestrator`).
>
> **resolved by doc-4cgx, 2026-10-09 22:50 EDT**

- `@ui:orchestrator`: the project's ID prefix, matching project-qualified ticket IDs (`ui-vnuu`;
  22ab section 8). Shorter, a bit more opaque.

Needs a short design (the syntax, where mentions are parsed: the daemon's task-comment path and
doc_watch for documents) before build. Related: bp2v / br-cufw (who can I message), 22ab step 4
(br-72t9: the thread moves to the ticket; mentions should work on the new thread too).
