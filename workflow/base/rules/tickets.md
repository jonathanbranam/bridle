---
id: tickets
severity: should
roles: [orchestrator, advisor, project-manager, manager, worker, reviewer, designer]
---
Tickets live in `docs/tickets/open/` and `docs/tickets/resolved/`, one file each,
`<descriptive-tail>-<id>.md`. Use the `bridle` binary for everything mechanical:

- `bridle ticket new "<title>" --kind <kind>` mints one (fresh ID, frontmatter, and its tracking row, `pending`, with the same ID; `--body`/`--body-file` write the ask). Kinds: `feature`, `fix` (the change that fixes a `bug` report), `docs`, `chore`, `arch-revision` (the change kinds), `bug` (a report), `epic` (a large initiative split into child tickets with `parent`), `question`, `research`, `explore`, `incident`, `re-evaluate`. There is no `idea` kind: whether a ticket is scheduled is its state. Once the ticket is committed on main with its ask, `bridle ticket ready <id>` marks it ready (who may ready what: rule `ready-without-asking`; it refuses an empty ask or an uncommitted ticket). `bridle ticket resolve <id>`
  stamps `resolved:` and moves it to `resolved/`; `bridle ticket set <id> <field> <value>` edits
  `title`, `kind`, `repos`, `changes`, `specs`, `blocked_by`, `related`, `parent`, `theme` or `tasks` (the old `needs`/`see` still work). Don't create, rename, move or hand-edit
  the frontmatter of a ticket yourself.
- IDs are project-qualified (`br-k7tm`) in `id:` and every reference; file names stay bare. A bare id
  you type is accepted and expanded when unambiguous; old bare ids in existing tickets still read.
- Write the body freely.
- After a ticket changes, run `bridle ticket check`; fix what it reports about your own tickets.

What goes where (k7tm): a ticket's file holds the design decisions, the why and the what; its row
holds the work and its status.

- A build that comes out of a discussion (research or question) ticket gets its own feature
  ticket, linked to the discussion ticket with `related`; its row is made from that ticket.
- A ticket with no file that needs design review, hits a real problem or needs a lot of
  discussion gets a ticket file made from it: `bridle ticket new --from-task <ticket-id>` (the file
  takes the ticket's ID; a fresh, two-way linked ID when an old hex ID has a `0` or `1`).
- Mechanical coordination about a ticket stays in its thread. A design question on a ticket gets a
  short comment ("needs design review, see the ticket") and the substance goes on the ticket:
  what happened, what was found, what needs clarifying.

Naming a ticket to the human is rule `ticket-references`.
