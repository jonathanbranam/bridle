---
id: j28f
title: Ticket IDs carry a project identifier and are told apart from task IDs, so links work across projects
kind: question
opened: 2026-10-04
repos: [bridle-ui]
changes: []
specs: []
needs: [xebc]
see: [a3yd, bnhn, xqvg, xebc]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 evening, to the bridle-ui aide; dictated; the UI half is
[[bridle-ui-render-front-matter-and-auto-link-urls-file-paths-a3yd|a3yd]]):

> The front matter doesn't seem to render nicely, so that's a, a need. Um, the, I think the front
> matter, since for, first of all, any front matter should render nicely, uh, even if we don't
> recognize the file or what's in it. Um, we should apply the same things. That uh, the automatic
> linking to the front matter that we do in other places, uh, automatic file linking, or like
> website linking. Website linking should work everywhere too. Uh, auto website linking, if that
> wasn't clear. And then um, for that may take care of a lot of it. But whenever we have like fields
> on a ticket or a task. Um, or IDs or something that map to a ticket or a task, those should all be
> links. Uh, I don't know if I mentioned that or if that was clear, but um, those should all be
> links everywhere. So, you know, right now without, without the two letters, the tasks have the two
> letters at the beginning and tickets don't. Um, so that's one thing and then In terms of like
> across projects, uh, I don't know how linking across projects works. That's uh, kind of weird.
> Actually, our our tickets should probably have project identifiers too, which means um, we need to
> we need to have a differentiator for a task versus a ticket.

## Context

- A task ID has its project's prefix (`[tasks] prefix` in `.bridle/config.toml`: `ui` for
  bridle-ui, `tw`, `mn`; bridle's tasks are `br-`). A ticket ID is four characters with no
  prefix, and "Tickets and tasks share one id alphabet and space: a ticket's first task takes its
  id (`br-<id>`)" (`docs/README.md`).
- Tickets for more than one project live in one repo: bridle-ui's tickets (k3qx, bnhn, t4rf, ...)
  are in the bridle repo, while their tasks are `ui-` tasks in bridle-ui's daemon.

## The human, again (2026-10-04 ~6:30 PM ET, via bridle's aide)

> Secondly, we've in building the UI, I realized that we have a the potential. We're going to have
> ticket name collisions between projects. Since the ticket doesn't have the project identifier on
> it. So that's something that needs to get fixed, but it's gonna take a, it's gonna take some work
> and a migration, I think. So it should probably be dependent on that migration. And then the other
> problem is that we use the project name as a prefix for the tasks, but not the tickets. So right
> now you can tell if a link is a task or a ticket, but if we use the project ID for both, then we're
> not going to be able to tell a task from a ticket in terms of a link or what we're referring to.
> So that means we're going to have to have some way to tell that this is a task versus a ticket.

So the decision is made in principle: **ticket IDs get a project identifier**. What's left is how:

1. **Collisions:** two projects can mint the same 4-character ticket ID. A ticket ID must be unique
   across projects, so it carries the project.
2. **A migration:** existing tickets (file names, `id:`, `see:`/`needs:`, `[[links]]`, task bodies'
   `original id:`) get rewritten. This depends on the project-migrations work
   ([[project-migrations-one-command-applies-pending-bridle-upgrad-xebc|xebc]]).
3. **Telling a task from a ticket:** today `br-p88z` is a task and `p88z` its ticket. With a project
   prefix on both, the form has to say which one it is: for example a kind marker (`br-t-p88z` /
   `br-k-p88z`), a different separator (`br-p88z` task vs `br/p88z` ticket), or ticket and task never
   sharing an ID. To weigh in the design, along with what links in the UI (a3yd) and Obsidian need.

> [!comment] c1 human, 2026-10-04 21:06 EDT, on "for example a kind marker (br-t-p88z / br-k-p88z), a different separator (br-p88z task vs br/p88z ticket), or ticket and task never sharing an ID." [pending 2026-10-04 21:06 EDT]
> Do some research on this comment and propose a number of solutions for this task. Instead of just "tasks," both of those words have a T and a K in them, so that feels like a problem here for one letter. TK is not great, and TI and TA don't do a whole lot better. Let's just investigate and consider what other systems might do for similar use cases and make some suggestions.
