---
id: vk3y
title: "Tickets and tasks link each other in metadata, every time: task new takes the ticket, a rule for every creator, the PM traces unlinked tasks"
kind: feature
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [fx7x, nnj2]
tasks: []
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
- `bridle task new` takes the ticket it was created from (e.g. `--ticket <id>`) and writes both links; without it the command prints "Ticket not provided. Please provide the ticket number."
- A rule for every role that creates tasks or tickets (aides, orchestrator, PM, and any other): name the ticket when creating a task.
- A rule for the PM: a task without a ticket gets traced to where it came from, linked to its ticket if one exists (or one made), and the case recorded somewhere, so we can see how unlinked tasks keep happening.
- A one-time backfill of today's unlinked pairs (`ticket check` lists them).
