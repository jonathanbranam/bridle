# Storage

> **Superseded in part** (note added when this file was split out of `design.md`):
> `docs/agent-host.md` §1.2 moves live state to one daemon per workspace, in `<workspace>/.bridle/`, and §2 step 3 moves the state branch's worktree to `<workspace>/.bridle/state/`. §2.1 turns the project registry into `~/.bridle/daemons/<project>.json`, one entry per running daemon.

## The state branch

Each project repo gets a `bridle` branch, checked out by bridle into
`~/.bridle/state/<project>/` (a normal git worktree, not visible in the working
checkout):

```
tasks/tw-7fa2.md          one file per task: TOML frontmatter + markdown body + thread
events/2026-09.jsonl      append-only transitions, for history and rebuild
questions/…               (or inline in the task thread — open, c5a8)
```

- **One file per task** merges cleanly, can be read on GitHub, and is the file
  design from research 13 carried over.
- **Bridle commits it**, batching writes (e.g. at most one commit every 30 s,
  plus one on every accept), and pushes on a configurable schedule.
- **Code branches never contain task state.** Task chatter can't cause a merge
  conflict with code, and main isn't committed to on every status change.

The alternative, task files in-tree under `.bridle/tasks/` on the main line, is
easier to browse next to code but brings back the worktree-visibility and
churn problems. It is an open question: [[task-records-on-a-state-branch-or-in-tree-c7eb|state branch or in-tree]].

## The database

`~/.bridle/bridle.db`, SQLite in WAL mode, one writer per transaction. It
indexes every registered project's tasks and holds the ephemeral tables:
`claims`, `agents`, `messages`, `waits`, `ports`, `impact_cache`. Every
durable write goes to the database and the state branch in the same logical
operation. The database is the read path because it's fast, and git is the
recovery path.

## Project registry

`~/.bridle/projects.toml` lists each registered repo with its path, prefix and
remote. `bridle status --all` and `bridle ready --all` work across all of them,
which gives one view of work over all projects.
