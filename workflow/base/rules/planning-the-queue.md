---
id: planning-the-queue
severity: should
roles: [orchestrator, product-manager]
---
Whoever prepares tasks and edits the queue (the product manager, or the orchestrator acting
as PM on a small project with none) plans like this:

- **Right-size every task.** A worker's context should stay well under 200K tokens for the
  whole task; reasoning degrades past ~250K and big contexts cost more. If a task needs more
  reading or changes than that, split it into tasks that merge independently. Say which tasks
  touch the same files, so they run one after another.
- **Write briefs into task bodies, not messages.** `bridle task new "<title>" -k <kind> --body
  "<brief>"`; `bridle task edit <id> --body "<brief>"` updates one (`--body-file <path>` or `-`
  for long ones). A brief stands alone: the goal, the files and design docs likely involved,
  the acceptance check, the model (Haiku for light, mechanical work; Sonnet for real design or
  tricky code), and what's out of scope.
- **`bridle task plan <id>` makes a task ready to build** (`open -> planned`); an unplanned
  task can't be queued or claimed.
- **Dependency edges only for true dependencies.** `bridle dep add <id> --blocked-by <id>` when
  one task can't start before another merges. Ordering in the task body alone isn't an edge:
  both tasks show startable. Never use edges to express mere priority; that's the queue's job.
- **Queue tiers order the work.** `bridle queue add-tier <id> <id>...` appends one tier
  (equally-ranked tasks) at the back; `bridle queue set --tier <id,id> --tier <id>` replaces the
  whole queue to reorder. `bridle queue` shows claimed tasks, then the tiers, each task marked
  startable or blocked. Keep it two or three tiers ahead of what's claimed. The manager claims
  from the highest startable tier.
- **The daemon nudges the manager** when the queue changes; don't send "queue updated" by hand.
  A change to one brief goes on that task (`bridle send <manager> --task <id> ...`).
