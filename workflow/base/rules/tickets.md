---
id: tickets
severity: should
roles: [orchestrator, advisor, product-manager, manager, worker, reviewer]
---
Tickets live in `docs/tickets/open/` and `docs/tickets/resolved/`, one file each,
`<descriptive-tail>-<id>.md`. Use the `bridle` binary for everything mechanical:

- `bridle ticket new "<title>"` mints one (fresh ID, frontmatter); `bridle ticket resolve <id>`
  stamps `closed:` and moves it to `resolved/`; `bridle ticket set <id> <field> <value>` edits
  `title`, `repos`, `changes`, `specs`, `needs` or `see`. Don't create, rename, move or hand-edit
  the frontmatter of a ticket yourself.
- Write the body freely.
- After a ticket changes, run `bridle ticket check`; fix what it reports about your own tickets.

Naming a ticket to the human is rule `ticket-references`.
