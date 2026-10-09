---
id: tickets
severity: should
roles: [orchestrator, advisor, project-manager, manager, worker, reviewer, designer]
---
Tickets live in `docs/tickets/open/` and `docs/tickets/resolved/`, one file each,
`<descriptive-tail>-<id>.md`. Use the `bridle` binary for everything mechanical:

- `bridle ticket new "<title>" --kind <kind>` mints one (fresh ID, frontmatter; `--body`/`--body-file` write the ask; it files no task); once the ticket is committed on main with its ask, `bridle ticket task <id>` files the task (its first task takes the same ID, `br-<id>`); `bridle ticket resolve <id>`
  stamps `closed:` and moves it to `resolved/`; `bridle ticket set <id> <field> <value>` edits
  `title`, `kind`, `repos`, `changes`, `specs`, `needs`, `see` or `tasks`. Don't create, rename, move or hand-edit
  the frontmatter of a ticket yourself.
- Write the body freely.
- After a ticket changes, run `bridle ticket check`; fix what it reports about your own tickets.

What goes where (k7tm): tickets hold design decisions, the why and the what; tasks hold the work
and its status.

- A build that comes out of a discussion (research or question) ticket gets its own feature
  ticket, linked to the discussion ticket with `see`; its task is made from that ticket.
- A task with no ticket that needs design review, hits a real problem or needs a lot of
  discussion gets a ticket made from it: `bridle ticket new --from-task <task-id>` (the ticket
  takes the task's ID; a fresh, two-way linked ID when an old hex task ID has a `0` or `1`).
- Mechanical coordination about a task stays in its thread. A design question on a task gets a
  short comment ("needs design review, see the ticket") and the substance goes on the ticket:
  what happened, what was found, what needs clarifying.

Naming a ticket to the human is rule `ticket-references`.
