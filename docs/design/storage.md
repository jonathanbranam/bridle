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
interactive_usage(id PK AUTOINCREMENT, observed_at, session_id, model,
                   cost_usd, context_used_tokens, context_max_tokens)
                                                     -- from `bridle statusline`; no agent id, nothing bridle hosts
meta(key PK, value)                                  -- e.g. claude_version
```

Nothing here has to survive a lost database ([[docs/proposal/decisions|decision 2]]):
there are no tasks yet, transcripts are files, and the conversations live in
Claude Code's session store, resumable by session id.

With tasks, the database also indexes the project's task records
(`crates/bridle-daemon/src/store.rs`, `SCHEMA_V5`):

```
tasks(id TEXT PK, title, kind, state, created_at, updated_at)
```

deliberately narrow: it's the fast index (id, title, kind, state,
timestamps), not the record itself. `id` is `<project prefix>-<4 hex
characters>` (e.g. `tw-7fa2`), generated with a collision retry; the prefix
is `[tasks] prefix` in config, defaulting to the project name's first two
alphanumeric characters (`bridle` -> `br`) if unset
(`config::default_task_prefix`). The body and thread live only on the state
branch below; a crash between a task write and the next batched flush can
lose an edit to those two fields specifically, though not the row above,
which is written to SQLite synchronously on every call.

Edges get their own table (`SCHEMA_V7`; `V6` went to `agents.context_tokens`,
the context governor's own migration), coordination.md's six kinds:

```
edges(from_task, to_task, kind, created_at, PRIMARY KEY(from_task, to_task, kind))
```

Unlike a task, the row *is* the whole record — there's no body/thread to
carry, so `TaskManager`'s edge cache hydrates straight from this table at
`open`, without touching the state branch. The `(from_task, to_task, kind)`
triple is the natural key: there's no separate edge id, and `bridle dep rm`
identifies the row to delete by that same triple.

Open questions get their own table too (`SCHEMA_V8`), the fast index
coordination.md's `question` message kind needs:

```
open_questions(task_id TEXT PK, message_id, asked_by, asked_at)
```

`task_id` is the primary key: a task has at most one open question at a
time, so `TaskManager::ask_question` on a task that already has one is a
conflict, the same shape as `insert_edge`'s duplicate-triple conflict.
`message_id` points at the `messages` row the question is (see below); the
body isn't duplicated here since that row already carries it. `TaskManager`
hydrates a `task_id -> message_id` cache from this table at `open`, the same
way it hydrates the edge cache, so `is_ready` (a plain, synchronous function)
can check "does this task have an open question" without a database round
trip. Answering deletes the row and clears the cache entry.

Messages can now target a task, not just `human` or an agent: `to_kind`
(`messages.to_kind`) takes a third value, `task`, with `to_id` the task id.
Asking a question inserts a `kind=question` message with `to_kind=task`,
addressed to the task itself; answering inserts a `kind=answer` message with
`reply_to` pointing at the question. Message routing decides `to_kind`
explicitly at the call site (`store::RecipientKind`) rather than guessing
it from `to`'s shape, since a task id and an agent id are both opaque
strings that don't self-identify.

Claims get their own table too (`SCHEMA_V9`):

```
claims(task_id TEXT PK, claimed_by, claimed_at)
```

Unlike every table above, this one is *never* mirrored to the state branch —
no task file write, no thread entry, no event. `task_id` is the primary key:
a task has at most one claimant at a time, so `TaskManager::claim_task` on
an already-claimed (or otherwise not-ready) task is a conflict, the same
shape as `insert_edge`/`ask_question`'s conflicts. Claiming transitions the
task `planned -> claimed` (dropping it out of `ready`, since that already
requires `planned`); releasing — explicit, or the lease expiring — reverses
it. There's no separate lease-renewal call: `TaskManager::tick_claim_lease_check`
reads the claiming agent's own `last_event_at`/`turn_started_at` (the same
signal `supervisor.rs`'s stall check watches) and releases the claim once
that activity is older than `config.claim_lease_after`. The ephemeral tables
`waits`, `ports`, `impact_cache` arrive with later tasks.

Every durable write goes to the database and the state branch in the same
logical operation (for the `tasks` table: synchronously to SQLite, then
enqueued for the state branch's next batched flush — see below; edges follow
the same rule, enqueuing the *entire* current edge set on every add/remove
rather than a diff, since there's no per-edge file to key a targeted write
on). Asking and answering a question follow the same rule: the
`open_questions` row is written synchronously, like a task or edge row, while
the thread entry it corresponds to is enqueued for the next flush. Claiming
and releasing a task write synchronously to `claims` and to `tasks.state`
and stop there — there's nothing to enqueue. The database is the read path
because it's fast, and git is the recovery path.

## The state branch

Built for task records at the `open`/`planned`/`claimed`/`dropped`/`reopened`
states (`crates/bridle-daemon/src/state_branch.rs`, `src/tasks.rs`) — claims
themselves are SQLite-only and never touch this branch (above); `in_review`,
`integrated` and `accepted`, and the edges/questions that go with them, are
still only designed
([[task-records-on-a-state-branch-or-in-tree-c7eb|decided]]).
Each project repo gets a `bridle/state` branch, checked out by the daemon
into `<workspace>/.bridle/state/` (a normal git worktree, not visible in the
working checkout). Named `bridle/state`, not the bare `bridle` an earlier
draft of this doc named: agent worktrees already live on `bridle/<name>`
branches (agents.md), and git can't have both `refs/heads/bridle` and
`refs/heads/bridle/<anything>` at once (a ref can't be both a leaf and a
directory) — a real namespace collision the build ran into, not a style
choice.

```
tasks/tw-7fa2.md          one file per task: TOML frontmatter + markdown body + thread
events/2026-09.jsonl      append-only transitions, for history and rebuild
```

- **One file per task** merges cleanly, can be read on GitHub, and is the file
  design from research 13 carried over.
- **Bridle commits it**, batching writes (at most one commit every 30 s,
  `Overrides::task_flush_interval`, plus a best-effort flush on graceful
  shutdown). There's no immediate-flush trigger yet — that arrives with
  `accept` — but `TaskManager::flush_now` already exists as the one function
  both the periodic tick and that future caller will call, so adding it
  won't need a restructure. **Pushing on a configurable schedule is not yet
  built**; this build only commits locally.
- **Code branches never contain task state.** Task chatter can't cause a merge
  conflict with code, and main isn't committed to on every status change.
- **The `bridle/state` branch doesn't exist on a project's first run.** The daemon
  creates it itself, as a parentless orphan, using `git commit-tree` against
  the well-known empty-tree object id plus `git update-ref` — plumbing that
  only writes a commit object and a ref, never a checkout — in preference to
  `git checkout --orphan`, which would need one
  (`worktree::ensure_orphan_branch`). This is the safety property the build
  was explicit about: writing task state must never be able to touch the
  project's own working tree or index, under any circumstance.
- **The task file's frontmatter delimiter is `+++`** (TOML, Hugo's
  convention), not `---` (which reads as YAML). The thread section, when a
  task has one, is a `## Thread` heading followed by one
  `### <kind> · <from> · <timestamp>` heading per entry and its body; this
  build writes `note`, `question` and `answer` entries, and `handoff`/
  `conflict`/`system` (coordination.md, Messages) reuse the same heading
  shape later without a format change. Parsing this back is line/substring
  based, not a real markdown parser: a body or thread entry containing the
  literal text `"\n## Thread\n"` or `"\n### "` will confuse it. Known,
  accepted for this build.

The alternative, task files in-tree under `.bridle/tasks/` on the main line, is
easier to browse next to code but brings back the worktree-visibility and
churn problems.

Questions aren't a separate `questions/…` folder: a question lives inline in
the thread of the task it blocks, the same file as the task itself. The
daemon additionally indexes open questions in SQLite (`open_questions`,
above) so `bridle inbox` can show them without walking the state branch
([[where-questions-live-on-the-state-branch-c5a8|decided]]). Built:
`TaskManager::ask_question`/`answer_question` write both the thread entry
and the SQLite index in the same call, and `is_ready` excludes a task with
an open question. Not yet built: the `bridle ask`/`bridle answer` CLI and
`bridle inbox` reading this index — both arrive with the next task.

## The daemon registry

`~/.bridle/daemons/<project>.json` lists each running daemon with its
workspace, repo and URL
([[docs/design/agent-host/operating-model#Several projects at once|several projects]]).
Views across projects (`bridle daemons`, and later `status --all`,
`ready --all`) fan out over it.
