---
id: a3yd
title: "bridle-ui: render front matter, and auto-link URLs, file paths and ticket and task IDs everywhere"
kind: feature
opened: 2026-10-04
repos: [bridle-ui, bridle]
changes: []
specs: []
needs: []
see: [bnhn, j28f]
tasks: [br-a3yd]
closed: 2026-10-09T23:11:06Z
---

## The ask


The human, verbatim (2026-10-04 evening, to the bridle-ui aide, looking at a ticket in the
Document page; dictated):

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

The ID-scheme half of this (project identifiers on tickets, telling tickets from tasks) is
[[ticket-ids-carry-a-project-identifier-and-are-told-apart-fro-j28f|j28f]].

## Context

- This extends [[bridle-ui-render-markdown-turn-wiki-links-and-file-paths-int-bnhn|bnhn]]
  (markdown renderer, wiki links, `docs/...` paths linked when the file exists), task br-bnhn,
  which was still `planned` when this was filed. New here: front matter rendered nicely for any
  file, known or not; URLs auto-linked; the same auto-linking inside front matter; ticket and task
  IDs (and fields that hold them: `see`, `needs`, `tasks`, ...) linked "everywhere".
- IDs today: a task ID has the project's prefix (`br-bnhn`, `ui-yj6m`, `tw-...`, `mn-...`);
  a ticket ID is four characters with no prefix (`bnhn`), and a ticket's first task takes its id
  (`br-bnhn`). bridle-ui's tickets live in the bridle repo.

## Work (UI)

bridle-ui task ui-pmkd renders markdown and front matter and links targets for both bnhn and a3yd (orchestrator, 2026-10-04). It starts once br-bnhn and br-a3yd land.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
