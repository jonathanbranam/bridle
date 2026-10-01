---
id: v3dk
title: Tickets get a kind; kinds are editable; tickets and tasks link both ways
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: [project-migrations-one-command-applies-pending-bridle-upgrad-xebc]
see: [focus-override-accept-local-time-in-until-like-the-focus-con-hesj, questions-become-tickets-and-a-docs-folder-plan-xe5c]
---

## The ask


The human, verbatim (2026-10-01, via the advisor), after learning `bridle ticket new` always makes
a `question` task and that tickets have no kind:

> I feel like question is not the right default, particularly if we're not specifying a default
> very often. What do you think an appropriate default would be? Also, tickets should have a kind
> as well. I think that tickets didn't have a kind before because in an earlier version of this,
> in the workflow instructions, tickets were categorized by kind into different folders, so we
> didn't have a kind in the front matter of a ticket.

Then, on the advisor's suggestion of no default:

> Sure, I think no default is fine. And is there editing the kind of a task? Okay, so I think a
> ticket should be able to edit its kind. That'll keep us from creating lots of new tickets for no
> reason, just to edit the kind. For a task, I'm not sure. Do you have a recommendation here?
> Because at a certain point, like if the task is planned, you should not be able to edit the kind
> at all at that point. But I don't see a problem in editing the kind of a task, you know,
> depending on the phase it's in. It seems reasonable if we made a mistake and called something a
> feature when it's really a bug, that it's perfectly fine to change that up until the time when
> it's planned for work, and at that point it's too late. Whether a ticket and its corresponding
> task have to have the same kind or not also doesn't really concern me. As long as they're
> linked, which by the way, we should have a two-way linking between those. I don't know if we
> have that today, but a ticket should record every task created from it, and vice versa. That
> should be enforced and checked. Yeah, I don't actually see a problem. I think a ticket that
> starts out as a question could generate different kinds of tasks. A single ticket that's a
> feature request could spawn features and bugs as tasks. I don't see any problem with that
> necessarily. We could edit the kind of a ticket if, if a question turns into a feature request.
> There's no problem in editing the kind of the ticket at that point either. So I think we should
> be a little more flexible here and KISS.

## Today

- `bridle ticket new` creates its task with a hardcoded `TaskKind::Question`
  (`crates/bridle/src/ticket.rs:86`). `bridle task new` already requires `--kind`.
- Ticket frontmatter has no `kind:`. The old `workflow-instructions/ticket-conventions.md` kept
  kind in the `intake/<kind>/` subfolder ("There is no `state:` field and no `kind:` field");
  bridle has one `docs/tickets/open/` folder, so the kind was lost.
- A task's kind can't be changed: `bridle task edit` only takes title and body.
- The link is one way and free text: the task body starts `original id: <id>` and the ticket's
  path. The ticket records nothing about its tasks, and nothing checks either side.

## Decided by the human

1. **No default kind.** `bridle ticket new` requires `--kind` (the task kinds: feature, bug,
   chore, question, research, explore, arch-revision, re-evaluate, incident).
2. **Tickets have a `kind:`** in frontmatter, editable any time (`bridle ticket set <id> kind
   <kind>`). `bridle ticket check` rejects a missing or unknown kind. Backfill existing tickets.
3. **A ticket's kind and its tasks' kinds are independent.** One ticket may spawn tasks of
   several kinds; no check compares them.
4. **Two-way link, enforced and checked.** A ticket records every task created from it (e.g. a
   `tasks: [br-xxxx, ...]` frontmatter field), and every such task records its ticket. A check
   fails on a link present on one side only.
5. **A task's kind is editable early and frozen once planned.**

## 5, as approved

The advisor's recommendation; the human, 2026-10-01: "Yeah, approved. That all looks good."

`bridle task kind <id> <kind>`, like `bridle task priority`: allowed only while the task is
`open`, refused in every other state (`planned`, `claimed`, `dropped`, `integrated`, and
`reopened`, since a reopened task was planned before), and recorded in the thread and as an
event.

## Migration

Existing tickets in every bridle project need `kind:` backfilled (from each ticket's task, where it
has one) and the two-way links added. That ships as a migration through
[[project-migrations-one-command-applies-pending-bridle-upgrad-xebc|project migrations]], so this
ticket needs it.
