# Storage

Each project's workspace holds everything bridle keeps for it
([[docs/design/agent-host/daemon#Workspace layout|workspace layout]]). There
is no machine-wide database.

## The daemon's database

`<workspace>/.bridle/bridle.db`, SQLite in WAL mode with foreign keys on. One
connection sits behind a mutex, and every access goes through async methods on
a `Store` handle that run it on `spawn_blocking`. The daemon is the only
writer. Migrations use `PRAGMA user_version`. What is built:

```
principals(id TEXT PK, kind, name, token_hash, created_at, revoked_at)
agents(id PK, name UNIQUE, role, state, model, session_id, pid, pid_start,
       workdir_kind, cwd, worktree, branch, created_at, updated_at,
       turns, turn_started_at, cost_usd_total, last_event_at,
       exit_code, exit_signal, exit_reason, created_by)
turns(agent_id, agent_name, role, model,           -- no FK: turns outlive rm
      n, started_at, ended_at, subtype, is_error, terminal_reason,
      input_tokens, output_tokens, cache_read, cache_write,
      cost_total,                                    -- this turn's cost
      PRIMARY KEY(agent_id, n))
messages(seq INTEGER PK AUTOINCREMENT, id UNIQUE,    -- id = m-0042 from seq
         from_principal, to_kind, to_id, kind, body, reply_to,
         when_mode, state, created_at, written_at, delivered_at, read_at)
events(seq INTEGER PK AUTOINCREMENT, ts, kind, actor, agent_id, data JSON)
                                                     -- agent_id has no FK: events outlive agents
rate_limits(window PK, status, utilization, resets_at, observed_at)
meta(key PK, value)                                  -- e.g. claude_version
```

Nothing here has to survive a lost database ([[docs/proposal/decisions|decision 2]]):
there are no tasks yet, transcripts are files, and the conversations live in
Claude Code's session store, resumable by session id.

With tasks, the database also indexes the project's task records and holds the
ephemeral tables: `claims`, `waits`, `ports`, `impact_cache`. Every durable
write goes to the database and the state branch in the same logical
operation. The database is the read path because it's fast, and git is the
recovery path.

## The state branch

*Designed, not built.* Each project repo gets a `bridle` branch, checked out
by the daemon into `<workspace>/.bridle/state/` (a normal git worktree, not
visible in the working checkout):

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
churn problems. Open questions:
[[task-records-on-a-state-branch-or-in-tree-c7eb|state branch or in-tree]],
[[where-questions-live-on-the-state-branch-c5a8|where questions live]].

## The daemon registry

`~/.bridle/daemons/<project>.json` lists each running daemon with its
workspace, repo and URL
([[docs/design/agent-host/operating-model#Several projects at once|several projects]]).
Views across projects (`bridle daemons`, and later `status --all`,
`ready --all`) fan out over it.
