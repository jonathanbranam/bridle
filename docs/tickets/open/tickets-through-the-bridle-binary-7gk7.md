---
id: br-7gk7
title: Tickets are created, edited, checked and resolved through the bridle binary, in every project
created: 2026-09-30
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-p2ys, br-k7tm, br-xe5c]
kind: chore
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

## Assessment (2026-10-11)

The human (2026-10-11): "assess what is actually needed for 01ff and clarify the description.
Yes, it is needed, but I think it is mostly complete." Checked against the code at a4cb9331.
The build is done: br-1e47 (edbf91b9, slice A), br-ae32 (2fbb2cd7, slice B), br-9e4e
(cc893d01, cleanup), then reshaped by 22ab (syqn, epvy, bpku, 3v75).

| Ask | Status | Evidence |
|-----|--------|----------|
| One layout for every project, `docs/tickets/{open,resolved}/<tail>-<id>.md`, owned by bridle | Done | `workflow/base/rules/tickets.md` (br-ae32); other projects read bridle's `workflow/` (`workflow =` in their `.bridle` config). `bridle ticket check` passes in bridle-ui and track-web. data-contracts keeps its frozen old pipeline by its own choice (its ticket 3fm6) |
| Mint a new ticket (fresh ID, in name and frontmatter) | Done | `ticket new`, `crates/bridle/src/ticket.rs:73`, `mint_id` :1379; IDs unique across tickets and tasks (br-9e15) |
| `ticket new` also makes the task | Done, reshaped | Every ticket gets its row, same id, `pending` (bpku, ticket.rs:155); `ticket ready` readies it |
| Update frontmatter | Done | `ticket set`, ticket.rs:875; `ticket link` for blocked_by/parent/related (3v75) |
| Check links and IDs | Done | `ticket check`, ticket.rs:1192 (br-ae32) |
| Resolve: move `open/` to `resolved/` | Done | `ticket resolve`, ticket.rs:662; file work only, no daemon |
| Fields `id, title, opened, repos, changes, specs, needs, see` | Done, superseded | syqn renamed them (`created`, `blocked_by`, `related`; added `kind`, `parent`, `theme`, `created_by`); old names still read and `ticket set` accepts them |
| A `closed:` timestamp, set on resolving | Done as `resolved:` | syqn renamed `closed` to `resolved` (the human's decision, syqn). `ticket resolve` stamps `resolved: <UTC date-time>` (ticket.rs:429); `closed:` is read as an alias (ticket.rs:826). Nothing to add |
| Agents write the body freely; templates later | Done | The tickets rule says so; templates stay out of scope (not urgent) |
| Nothing in `workflow/base/` defines tickets | Done | `workflow/base/rules/tickets.md` |

Not this ticket's work: other projects' existing tickets still carry the old field names and
bare IDs (bridle-ui 21 of 25 files, track-web 12 of 15) until their owners run the opt-in
migration `0002-standard-ticket-fields` (xebc); missing `kind:` is the v3dk backfill.

## What is left

Bridle's own docs and spikes still describe the pre-7gk7 setup. A docs chore, size S:

1. `docs/README.md`, "Questions and spikes are tickets": drop the reference to
   `workflow-instructions/ticket-conventions.md` (line 50) and the `check-tickets.py` loop (lines
   93-98); say `bridle ticket check` instead. Drop the stale `questions/` tree (line 70). Fix
   line 85, "resolving a ticket closes its task": `ticket resolve` doesn't touch the row
   (`docs/design/cli.md`, ticket resolve); say what does (`ticket done` / `drop`).
2. Spikes: `docs/spikes/open/` (3) and `docs/spikes/done/` (4) are ticket-shaped but outside the
   binary: old fields (`opened`, `needs`), bare IDs, `done/` not `resolved/`, and `ticket check`
   doesn't read them. Recommended: move them to `docs/tickets/open|resolved/` as kind `research`
   (ID and tail kept, fields renamed, qualified IDs, `resolved:` from the git date of the move to
   `done/`), and keep the numbered findings docs in `docs/spikes/`. The alternative is to stop
   calling spikes tickets in the README. The human picks; the move is the default.

Verify: `grep -n "workflow-instructions\|check-tickets\|questions/" docs/README.md` finds nothing;
`bridle ticket check` reports no new problems and covers the moved spikes; `bridle ticket show
mnzh` prints the spike.

If the human drops item 2, item 1 alone is a few lines and could ride along with any docs change;
the code ask of this ticket is complete.

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
