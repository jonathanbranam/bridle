---
id: 7gk7
title: Tickets are created, edited, checked and resolved through the bridle binary, in every project
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [p2ys, k7tm, xe5c]
---

## The ask

The human, verbatim (2026-09-30), after finding meta-notes' one ticket at
`tickets/recurrence-and-time-of-day.md` (no `docs/`, no `open/`/`resolved/`, no ID in the name,
frontmatter `id: mn-ba09`, `title`, `status` only):

> it's... not following this format at all actually.
>
> OK, this layout should be minted and used through bridle, IMO, creating a new ticket,updating
> the frontmatter, checking links, etc. should all be through the bridle binary. including moving
> a ticket from the open/ to resolved/ folders. That is true across all projects. id, title,
> opened, repos, changes, specs, needs, see. Also, missing a closed: timestamp as well, should be
> added.
>
> The agents are free to write their own content into the ticket, although we can in the future,
> provide templates for different types of tickets, but that isn't critical or urgent.

## What that means (as asked; not triaged)

- The ticket layout bridle uses on itself (`docs/README.md`, "How the folders work":
  `docs/tickets/open/<descriptive-tail>-<id>.md`, resolved by moving to `docs/tickets/resolved/`)
  becomes the layout for **every** bridle project, and bridle owns it.
- The `bridle` binary does the mechanical parts: mint a new ticket (fresh 4-character ID from
  `abcdefghjkmnpqrstuvwxyz23456789`, unique in the repo, in the file name and frontmatter),
  update frontmatter, check links and IDs, and resolve (move `open/` to `resolved/`).
- Frontmatter: `id`, `title`, `opened`, `repos`, `changes`, `specs`, `needs`, `see`, plus a new
  **`closed:`** timestamp, set on resolving.
- Agents write the body freely. Per-kind templates are a later, non-urgent idea.

## Today

- Bridle's own tickets use the conventions in the external `workflow-instructions` repo
  (`ticket-conventions.md`, `scripts/new-ticket.sh`, `scripts/check-tickets.py`), which other
  projects don't have. Nothing in `workflow/base/` defines tickets, so a project's agents invent
  a layout (meta-notes, above).
- Each ticket also gets a bridle task by hand (`original id: <id>`; `docs/README.md`), so a
  `bridle ticket new` could create both.

## Related

- [[ticket-state-without-moving-files-p2ys|ticket state without moving files]]: the human
  (2026-09-27) found moving files a hassle. With the binary doing the move, the hassle is gone
  but the folders stay; this ask keeps `open/` and `resolved/`.
- [[tickets-and-tasks-why-both-k7tm|tickets and tasks, why both]].
- [[questions-become-tickets-and-a-docs-folder-plan-xe5c|questions become tickets]].

## File names keep the ID at the end (the human, 2026-09-30)

> I need to know the FIRST letters of the ticket, not the LAST letters of the ticket. [...] OK,
> let's keep the current convention of id at the end for now. I'll think about this more as I
> use the system.

Kept as `<descriptive-tail>-<id>.md`. Naming tickets to the human by the start of the file name
is `workflow/base/rules/ticket-references.md`.
