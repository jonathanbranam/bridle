# Tasks and tickets

**Tasks hold the work and its status; tickets hold design decisions, the why and the what.**

## Tasks

A task has a kind (`feature`, `bug`, `chore`, `question`, `research`, `explore`,
`arch-revision`, `re-evaluate`), a body, a thread of comments and a state.

```
pending -> open -> planned -> (ready) -> claimed -> integrated
                      any state -> dropped (reason required)      integrated -> reopened
```

- `pending`: just filed. `bridle ticket ready <id>` (`task ready <id>` for a row with no ticket; orchestrator, advisor, aide or PdM; see rule
  `ready-without-asking`) makes it `open`.
- `open`: the project manager triages it; `bridle task plan <id>` makes it `planned`.
- `ready` is derived: planned, with no open blockers and no unanswered question.
- `claimed`: a worker holds it under a lease renewed by the agent's activity; a lapsed
  lease returns it to ready.
- `integrated`: merged to the integration branch, tests green. Planned, not built:
  `in_review`, `accepted`, `needs-input`.

```
bridle task new "title" -k feature --body-file -
bridle task show|list
bridle task claim|release <id>
bridle task comment <id> --text-file F      # on the thread; no effect on readiness
bridle task ask|answer ...                  # a question that blocks a task
bridle ticket link <id> blocked-by <other>  # real dependencies only, never ordering
bridle task summary <id> --file F           # how it was done
bridle task land <task>                     # integrator: squash-merge the branch
```

## Tickets

Files in `docs/tickets/open/` and `docs/tickets/resolved/`, one each, named
`<descriptive-tail>-<id>.md`. Use the binary for everything mechanical (never hand-edit
the frontmatter):

```
bridle ticket new "title" --kind feature [--body-file F]
bridle ticket new --from-task <task-id>     # a task that needs design discussion
bridle ticket ready <id>                    # mark a committed ticket with an ask ready (rule `ready-without-asking`)
bridle ticket set <id> <field> <value>
bridle ticket resolve <id>
bridle ticket check
```

## More

`docs/design/roles-and-lifecycle.md`, `docs/design/coordination.md`, `docs/design/cli.md`.
