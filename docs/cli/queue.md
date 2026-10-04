# The queue

The queue says *when*: an ordered list of tiers, each a set of equally ranked task ids
(tier 1 before tier 2). Tasks say *what*. A task in no tier is backlog.

```
bridle queue                              # claimed tasks with their worker, then the tiers;
                                          # each task marked startable or blocked
bridle queue set --tier a,b --tier c      # replace the whole queue
bridle queue add-tier <task>...           # append one tier at the back
bridle ready                              # ready tasks of the highest startable tier
bridle ready --all                        # every ready task, across projects
```

- **Who writes it:** the project manager, the human, or `external:orchestrator` acting as
  project manager on a small project. Everyone else, managers included, only reads it.
- **Reordering** is resending the tiers in the shape they should have: add, remove and
  reorder are the same call.
- **Managers are mechanical about it.** They take from the highest tier with a startable
  task (ready and so unclaimed), choosing within the tier by free worker slots. If the
  top tier's tasks are all blocked, they take from the next tier down. They never move a
  task between tiers.
- **Dependencies** are `blocks` edges (`bridle dep add`), real ones only. Use tiers, not
  edges, for ordering.
- **Durable:** stored in `queue.toml` on the state branch, restored by `bridle rebuild`.

## More

`docs/design/roles-and-lifecycle.md` ("Task lifecycle"), `docs/design/coordination.md`.
