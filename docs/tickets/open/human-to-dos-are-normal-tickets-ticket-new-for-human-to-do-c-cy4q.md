---
id: br-cy4q
title: "Human to-dos are normal tickets: ticket new --for-human, to-do commands and the gateway list under ticket (22ab step 7b)"
kind: feature
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-res3]
related: []
parent: br-22ab
theme: tickets-and-release
---

## The ask

The human, 2026-10-11 ~9:25 AM ET (to advisor product-manager): "Something else i realized - human todos are tasks without a ticket, those become tickets then, right? and commands to update them need changed; also they're in the UI." And on the choice: "yes, for human todos, they become full, normal tickets like all others".

This is 22ab step 7b, a gap in the approved plan: 22ab never mentioned human to-dos.

## Today

- `task new --for-human` makes a row, `planned` and claimed by `human`, with no ticket file, and sends the human one inbox note (docs/design/cli.md, "Human to-dos"; coordination.md, "Human to-dos").
- The human finishes one with `task done <id>` (no `--commit`); `task list --claimed-by human` is the ranked list; `task drop --reason` notes the human.
- The gateway's to-do list (crates/bridle-gateway, items) reads those rows.
- All 11 open to-dos on bridle have no ticket. Migration 0005 (manual, not yet run on bridle) would give each a stub, but nothing makes a ticket for a new to-do.

## What's wanted

- A to-do is a full, normal ticket like any other: a file in docs/tickets/open/ with the ask, its row claimed by `human`. Kind: whatever fits the work (usually `chore`); no special kind.
- `bridle ticket new "<title>" --for-human [--body ...] [--priority ...]` mints the file and makes the row planned and claimed by `human` in one step, and sends the inbox note, as `task new --for-human` does today. Who commits the file: the same as any `ticket new` (the caller commits it).
- The to-do commands in their ticket form: list (`ticket list --claimed-by human`), done, drop with a reason. Reuse whatever step 7 (q8cs) already exposed under `ticket`; add only what's missing.
- `task new --for-human` (the hidden alias) keeps working until step 10 but makes a ticket too, so no new ticketless to-do appears.
- The gateway's to-do list shows the ticket (title and link to the ticket) and keeps its order (`sort_by_priority`). Old routes stay for one release, as aejj did.
- Docs: cli.md and coordination.md "Human to-dos", and the aide role (it files most to-dos) say `ticket new --for-human`.
- Existing to-dos: migration 0005 covers them when the owner runs it; say so, no new migration.

## Verify

just check; a test that `ticket new --for-human` makes a committed-able file and a row claimed by `human`, planned, with the inbox note; a test that the gateway's to-do list item carries the ticket.

## Order

After br-res3 (the last code-rename slice). Before step 10 (dropping the `task` alias). The bridle-ui side (the to-do views) goes with 22ab step 8 in bridle-ui.
