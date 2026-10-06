---
id: vk3y
title: "Tickets and tasks link each other in metadata, every time: task new requires --ticket (or no-ticket), a rule for every creator"
kind: feature
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [fx7x, nnj2]
tasks: [br-vk3y]
---

## The ask

The human, verbatim (2026-10-05 ~8:15 PM ET, to the bridle-ui aide, after the aide found tasks not linked to their tickets; the message ends mid-sentence):

> Yeah, that's good, and your other investigation turned up some important information too. We definitely want traceability, just direct traceability between tickets and tasks, and vice versa. They should all be linked in metadata, not just in text. This needs to be a rule that gets followed.
>
> Creating tasks and tickets is the job of lots of agents:
> - the aides
> - the orchestrator
> - the PM
>
> Let's add a specific rule for the PM: if a task comes through without a ticket, the PM will verify where the task came from, try to identify if there's an associated ticket, fix it, and then also file that somewhere. Let's keep track of how this is happening.
>
> I think that when we create a task, it should have an option to give the ticket it was created from. If that's not provided, the command should print a warning or a message that says, "Ticket not provided. Please provide the ticket number." For this task, anytime a task is created

What the aide found (2026-10-05, `bridle --project bridle ticket check`):
- The ticket -> task link (`tasks:` front matter) and the task -> ticket link (`original id:` first body line) are only written by `bridle ticket new` / `bridle ticket task`. Tasks the orchestrator and others make with `bridle task new` name the ticket only in text ("(fx7x)" in the title, a "Ticket: docs/..." line), so neither side is linked in metadata.
- 48 tickets have tasks naming them that their `tasks:` doesn't list (warnings); 11 tickets list tasks whose bodies lack `original id:`. Example: fx7x had seven tasks (br-4d5a, 2ebc, a424, e949, 65b8, 4573, 15a7) and listed none; only br-4d5a was made from it.
- Old task bodies point at docs/questions/open/... paths that moved to docs/tickets/.
- `bridle task new` has no option to name a ticket.

The ask, as the aide reads it (design open):
- Tickets and tasks link each other in metadata, both ways, for every task, however it was made.
- `bridle task new` requires the ticket it was created from (`--ticket <id>`) and writes both links; with no real ticket the agent passes `--ticket no-ticket` (see the follow-up below).
- A rule for every role that creates tasks or tickets (aides, orchestrator, PM, and any other): name the ticket when creating a task.
> [!comment] c1 human, 2026-10-05 20:24 EDT, on "rule for the PM" [read 2026-10-05 20:31 EDT]
> We can remove this since we have "defined this error away"
>
> **aide, 2026-10-05 20:31 EDT:** @human Done. Removed the PM rule from the ask, from the title and from my note on `no-ticket` (it no longer says the PM traces those). With `--ticket` required, a task is linked or carries an explicit `no-ticket`, so there's nothing left to trace. The rule for every task creator and the one-time backfill stay. [read 2026-10-05 20:26 EDT]
>
> **resolved by human, 2026-10-05 20:27 EDT**

- A one-time backfill of today's unlinked pairs (`ticket check` lists them).

## The human's follow-up (2026-10-05 ~8:20 PM ET, finishing the cut-off sentence)

> Sorry about that. I said that generally we should have the agents using the command to create a task off of a ticket. That's the preferred command because then the linkage is created in both directions. Is that correct? Confirm that's correct. It should always be linked both ways.
>
> We should have a `--ticket` option when creating a task that links it back to the ticket it came from. What I said was that if there's no ticket, we want to make it loud if you're creating a task without a ticket, so that the agent checks itself.
>
> Let's think of some solutions here. Maybe make `--ticket` required as a CLI parameter. If there's really no ticket, the agent has to enter some sentinel value. I was going to say `none`, but that's almost a valid ticket ID, so maybe `no-ticket` or something that's valid to write in the CLI without extra quotes. I'm trying to avoid arrow brackets, parentheses, or a dollar sign, anything that would cause shell interpolation problems. We should make them always specify a ticket, and then say give some special value if there's no ticket.

Aide's notes:
- Confirmed: `bridle ticket task <id>` (and `bridle ticket new` when a daemon is reachable) links both ways: the ticket's `tasks:` gets the task and the task body starts `original id: <ticket>` (e.g. y25n / br-y25n). That is the preferred way to make a task.
- So: `bridle task new` gets a required `--ticket <id>` that writes both links like `ticket task` does. With no real ticket the agent passes the sentinel `--ticket no-ticket` (plain, no quoting needed), which is recorded on the task. Omitting `--ticket` is an error that names the sentinel.
- `none` can't actually be a ticket ID (the ID alphabet has no `o`), but `no-ticket` is clearer and easy to grep; recommend it.

## The human on `original id:` (2026-10-05 ~8:45 PM ET)

> What is original ID in the task? Is that a field? I, I don't know why it says original ID. Is that the ticket ID? If that's the ticket ID, then just say ticket ID, not original ID.

Aide's notes: it is the ticket ID, and it is not a field: it is the first line of the task's body text (`original id: <ticket>`, storage.md ~71, cli.md ~251/442, daemon tasks.rs ~494/1582). "Original" because a ticket's task takes its ID from the ticket (vk3y -> br-vk3y). So the task -> ticket link is text, which this ticket says it must not be. Make it a real task field named `ticket` (shown as "ticket" in `task show`, the API and the UI), set by `ticket task`, `ticket new` and `task new --ticket`; migrate existing `original id:` first lines into it and stop writing them.
