---
id: 6h65
title: "A product manager that relates every ticket to open and planned work: links, merges, and folds ideas into changes already planned"
kind: feature
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [pa8h, ukpm, yj38, w2hj]
tasks: [br-6h65]
---

## The ask

The human, verbatim (2026-10-08 ~8:20 PM ET), after the aide had to search by hand for a ticket about assigning an agent to a document under review (none existed; the idea was spread across x8jt's "Which agent answers", yj38 and wjhp, and the human expected it to be part of any change to the review list in pa8h):

> This is where we need a real product manager that checks all the open tickets and future work for related things.

The ask: when a ticket is filed or changed, someone checks every open ticket, task and planned design for related work and:
- links it (`see`/`needs`)
- folds duplicate asks together
- flags an idea that belongs inside a change already planned, as here: assigning an agent belongs with moving the review list into the database

The same check runs as a periodic sweep over the whole backlog. Today no one does this reliably, so the human and the aide find the connections by accident.

Context: there's a project-manager role (`workflow/base/roles/project-manager.md`, pm-1) that triages open tickets, but cross-ticket relating isn't an explicit duty, and it isn't done on each new ticket. ukpm (the designer role) and tx3f (the product/development manager split) are nearby. Decide whether this extends the project-manager role or is a separate product-manager seat.

## The human on it (2026-10-08 ~8:25 PM ET)

> I think that should be product manager, new role, not project. But IDK maybe it's just too many divisions for our level or work. They don't need to stay running basically.
>
> Smaller projects should spin one up and then shut it down automatically after a settle period like 30-60 minutes after all work is done. I think that's the direction.
>
> Its frustrating when the smaller projects don't manage themselves the same way basically

So: a new **product manager** role, not the project manager. It runs only when needed and isn't a standing seat. On small projects it's started when there's work and stopped automatically 30-60 minutes after all work is done (ticket w2hj). The human is unsure whether one more role is "too many divisions" for the current amount of work.
