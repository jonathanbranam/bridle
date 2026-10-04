---
id: j28f
title: Ticket IDs carry a project identifier and are told apart from task IDs, so links work across projects
kind: question
opened: 2026-10-04
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [a3yd, bnhn, xqvg]
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
