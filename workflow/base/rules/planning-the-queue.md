---
id: planning-the-queue
severity: should
roles: [orchestrator, project-manager]
---
Whoever prepares tickets and edits the queue (the project manager, or the orchestrator acting
as PM on a small project with none) plans like this:

- **Right-size every ticket.** A worker's context should stay well under 200K tokens for the
  whole ticket; reasoning degrades past ~250K and big contexts cost more. If a ticket needs more
  reading or changes than that, split it into tickets that merge independently. Say which tickets
  touch the same files, so they run one after another.
- **Write briefs into ticket bodies, not messages.** `bridle ticket new "<title>" --kind <kind> --body
  "<brief>"`; `bridle ticket edit <id> --body "<brief>"` updates one (`--body-file <path>` or `-`
  for long ones). A brief stands alone: the goal, the files and design docs likely involved,
  the acceptance check, the model (Haiku for light, mechanical work; Sonnet for real design or
  tricky code), and what's out of scope.
- **`bridle ticket plan <id>` makes a ticket ready to build** (`open -> planned`); an unplanned
  ticket can't be queued or claimed.
- **Dependency edges only for true dependencies.** `bridle ticket link <id> blocked-by <id>` when
  one ticket can't start before another merges. Ordering in the ticket body alone isn't an edge:
  both tickets show startable. Never use edges to express mere priority; that's the queue's job.
- **Queue tiers order the work.** `bridle queue add-tier <id> <id>...` appends one tier
  (equally-ranked tickets) at the back; `bridle queue set --tier <id,id> --tier <id>` replaces the
  whole queue to reorder. `bridle queue` shows claimed tickets, then the tiers, each ticket marked
  startable or blocked. Keep it two or three tiers ahead of what's claimed. The manager claims
  from the highest startable tier.
- **Order by priority, never by size.** A small low or normal ticket doesn't fill a free slot
  while higher ready work waits: every ticket costs ~30-40 minutes of machine time (builds,
  `just check`), not its coding time. The human, 2026-10-09: "the "easy" and "small" changes
  still run builds and peg the CPU so it is quick for the worker to implement, but it actually
  adds 30-40min."
- **The daemon nudges the manager** when the queue changes; don't send "queue updated" by hand.
  A change to one brief goes on that ticket (`bridle send <manager> --task <id> ...`).
