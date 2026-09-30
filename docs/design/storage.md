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
       context_tokens,                             -- latest context size (SCHEMA_V6), the governor's measure
       session_started,                            -- 0 until the current session_id has had a turn
                                                   -- (SCHEMA_V13); an unstarted session can't be --resume'd
       exit_code, exit_signal, exit_reason, created_by,
       extra_allowed_tools JSON, extra_env JSON,  -- this agent's --allow-tool/--env
                                                   -- overrides, reapplied on renew/resume
       components JSON)                            -- component ids (SCHEMA_V12), BRIDLE_COMPONENTS
turns(agent_id, agent_name, role, model,           -- no FK: turns outlive rm
      n, started_at, ended_at, subtype, is_error, terminal_reason,
      input_tokens, output_tokens, cache_read, cache_write,
      cost_total,                                    -- this turn's cost
      PRIMARY KEY(agent_id, n))
messages(seq INTEGER PK AUTOINCREMENT, id UNIQUE,    -- id = m-0042 from seq
         from_principal, to_kind, to_id, kind, body, reply_to,
         when_mode, state, created_at, written_at, delivered_at, read_at,
         answered_by, answered_reply, answered_line,  -- SCHEMA_V14: a question to the human closed
                                                      -- by a delegate's reply (`answer_for_human`)
         incident_task)                               -- SCHEMA_V18: the incident a broadcast notice announces
events(seq INTEGER PK AUTOINCREMENT, ts, kind, actor, agent_id, data JSON)
                                                     -- agent_id has no FK: events outlive agents
rate_limits(window PK, status, utilization, resets_at, observed_at)
interactive_usage(id PK AUTOINCREMENT, observed_at, session_id, model,
                   cost_usd, context_used_tokens, context_max_tokens,
                   context_used_percentage)          -- V10: Claude's own figure; right on 1M models
                                                     -- from `bridle statusline`; no agent id, nothing bridle hosts
meta(key PK, value)                                  -- e.g. claude_version; orchestrator_wake_cursor
                                                     -- (event seq the orchestrator's wakes were last delivered up to)
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
conflicts(id INTEGER PK -> shown as C<id>, task_a, task_b, kind, key, state, resolution,
          opened_at, resolved_at; UNIQUE(task_a, task_b, kind, key))
ports(port INTEGER PK, agent, task, pid, label, allocated_at)     -- SCHEMA_V16
handovers(seq INTEGER PK AUTOINCREMENT, id UNIQUE -> h-0007, role, project, body,
          created_at, created_by)                                  -- SCHEMA_V17
```

`handovers` are the orchestrator's notes ([[orchestrator-supervision]] section 7): also on the
state branch as `handovers/<id>.md` (frontmatter + body, one file per note, written by the same
flush; notes missing there are backfilled at daemon start) and restored by `bridle rebuild` with
their ids and `seq`. The highest `seq` is the current note; older rows stay for
`handover list` and `show`, and the daily prune deletes those older than the events' 30 days but
always keeps the newest.

`conflicts` rows are opened by `impact check` (impact-and-conflicts.md); the unique key makes
reopening the same overlap a no-op, resolved or not. Conflicts are SQLite-only, not on the
state branch and not rebuilt.

`ports` is runtime state: not on the state branch and not rebuilt (see
[[worktrees-and-ports]]).

`task_id` is the primary key: a task has at most one claimant at a time, so
`TaskManager::claim_task` on an already-claimed (or otherwise not-ready)
task is a conflict, the same shape as `insert_edge`/`ask_question`'s
conflicts. Claiming transitions the task `planned -> claimed` (dropping it
out of `ready`, since that already requires `planned`); releasing —
explicit, or the lease expiring — reverses it. There's no separate
lease-renewal call: `TaskManager::tick_claim_lease_check` reads the claiming
agent's own `last_event_at`/`turn_started_at` (the same signal
`supervisor.rs`'s stall check watches) and releases the claim once that
activity is older than `config.claim_lease_after`. Only a `claimed` task is ever
moved by a release: dropping or integrating a claimed task clears its claim, and
`TaskManager::open` deletes any `claims` row whose task isn't `claimed`. The ephemeral tables
`waits`, `impact_cache` are not built.

**Claims are durable now (j479):** unlike `tasks`/`edges`/`open_questions`,
there's no per-claim file to key a targeted write on, so — the same
wholesale approach `edges.toml` already uses — every claim/release enqueues
the *entire* current claim set to overwrite the state branch's `claims.toml`
at the next flush (`TaskManager::enqueue_claims_snapshot`,
`StateBranch::enqueue_claims`). Unlike `edges.toml`, this file does have a
read path back (`StateBranch::list_claims`): `bridle rebuild`
(`TaskManager::rebuild_from_state_branch`) restores `claims` from it, the
same way it restores `tasks`/`edges`/`open_questions` from their own files —
inserting each row and setting the task's `state` back to `Claimed` (that
part happens only in SQLite even in the live path, so rebuild mirrors it
explicitly rather than getting it from the task file, which never carries
`claimed`). Refusing to rebuild over an already-populated database now checks
`claims` too, alongside the other three tables.

The current claim, if any, rides along on the wire `Task` as `claimed_by`/
`claimed_at` (`bridle-api::types::Task`): null when unclaimed, populated from
`claims` on `TaskManager::open` and again on every claim/release. `GET
/v1/tasks?claimed_by=` filters on it; `me` resolves to the calling
principal's own id (the same pattern `?to=me` uses for messages,
`server.rs::resolve_to`), anything else is matched against `claimed_by`
verbatim after the same name lookup `resolve_from` does for messages.

Every durable write goes to the database and the state branch in the same
logical operation (for the `tasks` table: synchronously to SQLite, then
enqueued for the state branch's next batched flush — see below; edges and
claims follow the same rule, enqueuing the *entire* current set on every
add/remove rather than a diff, since there's no per-row file to key a
targeted write on). Asking and answering a question follow the same rule:
the `open_questions` row is written synchronously, like a task or edge row,
while the thread entry it corresponds to is enqueued for the next flush.
Claiming and releasing a task write synchronously to `claims` and to
`tasks.state`, then enqueue the claim snapshot above. The database is the
read path because it's fast, and git is the recovery path.

## The queue

A separate record from `tasks`/`edges`: an ordered list of tiers, each tier
a set of equally-ranked task ids — tier 1 (index 0) before tier 2. A task
not listed in any tier is backlog. There's no SQLite table for it at all
(unlike claims, it was never SQLite-only to begin with): `TaskManager` keeps
an in-memory `Vec<Vec<String>>` cache, hydrated straight from the state
branch's `queue.toml` at `open`/`rebuild_from_state_branch`, since that file
is the record's only durable copy.

`queue.toml` is an ordered array of tables, `[[tier]]`, each with a `tasks`
array; array order *is* rank order, so there's no separate rank field to
keep in sync with position (`state_branch.rs::render_queue`/`parse_queue`).
Every write (`TaskManager::set_queue`, replacing the whole queue; the CLI's
`bridle queue add-tier` is sugar over it that appends one tier) re-renders
the file wholesale, the same way `edges.toml`/`claims.toml` do, and enqueues
an `events/<YYYY-MM>.jsonl` line recording the actor
(`StateBranch::enqueue_queue_event`) — since the file itself carries no
history of who changed it, only the append-only event log does.

Only the PM (and the human, to override) may write the queue; every other
principal, the manager included, is read-only
(`server.rs::require_pm_or_human`, gating `POST /v1/queue` and `POST
/v1/queue/tiers`) — checked against the calling agent's configured `role`
being `product-manager`, the same lookup `require_not_worker` does for
agent-lifecycle endpoints. `bridle queue` is the read-only view: claimed
tasks with their worker, then the tiers in rank order, each task marked
startable (ready — planned, deps met, no open question, and therefore
unclaimed too, since `is_ready` already requires `planned`) or blocked.
`TaskManager::highest_startable_tier` (`GET /v1/tasks?top_tier=true`,
`bridle ready` with no `--all`) returns just the highest tier with at least
one startable task, skipping a tier stuck on a dependency rather than
returning nothing — the manager takes from the next tier down instead of
idling on a blocked one, and never moves a task between tiers itself.

## Incidents

Incidents add no table: they are tasks of kind `incident`, stored like any task. The
one schema change (SCHEMA_V18) is a nullable, indexed `messages.incident_task` column linking a notice to its task,
cleared when the incident resolves
([[docs/design/agent-host/incidents|incidents]]).

## The state branch

Built for task records at the `open`/`planned`/`claimed`/`dropped`/`integrated`/`reopened`
states (`TaskState`) (`crates/bridle-daemon/src/state_branch.rs`, `src/tasks.rs`),
including claims and the queue now (above); `in_review` and
`accepted`, and the edges/questions that go with them, are still only
designed
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
edges.toml                the whole edge set, replaced wholesale on every add/remove
claims.toml               the whole claim set, replaced wholesale on every claim/release
queue.toml                the queue: an ordered [[tier]] array, replaced wholesale on every write
events/2026-09.jsonl      append-only transitions, for history and rebuild
```

- **One file per task** merges cleanly, can be read on GitHub, and is the file
  design from research 13 carried over.
- **Bridle commits it**, batching writes (at most one commit every 30 s,
  `Overrides::task_flush_interval`, plus a best-effort flush on graceful
  shutdown). There's no immediate-flush trigger yet — that arrives with
  `accept` — but `TaskManager::flush_now` already exists as the one function
  both the periodic tick and that future caller will call, so adding it
  won't need a restructure. **Pushing is built, behind `[state] push`**
  (default `true`; set `push = false` to opt-out, per rule `existing-projects`
  and approved 2026-09-29 to push to all projects). After a flush that committed, the
  daemon pushes `bridle/state` to `origin` in a background process, from the state branch's own
  worktree, at most once a minute: a commit made inside the window is pushed when it ends (the
  next flush tick), and once more on graceful shutdown (10 s cap). Never forced. A failed
  push (network, auth) is retried on later ticks, WARN-logged once per change of reason, and
  shown in `bridle status` (`state push failing: <reason>`, or the age of the last push). A
  non-fast-forward reject means someone else wrote the branch: pushing stops for the daemon's
  life, `status` says so, and nothing is fetched or merged automatically.
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
- **A task's `components`** (component ids, docs/design/components.md) live only
  in the frontmatter, as an optional `components = ["a", "b"]` array: omitted when
  empty, and a file written before the field existed loads with none. Like the body,
  the state branch is its only durable copy, so a rebuild round-trips it.
- **A task's `size`** (`S`, `M` or `L`) lives only in the frontmatter, as an optional
  `size = "S"` line: omitted when unset, and a file written before the field existed
  loads with none. Like `components`, a rebuild round-trips it.
- **A task's `priority`** (`high`, `normal`, `low`) lives only in the frontmatter, as an
  optional `priority = "high"` line: omitted when normal, and a file written before the field
  existed loads as normal. A rebuild round-trips it; each change is also a thread entry.
- **A task's landing record** (`branch`, `commit`, `summary`) lives only in the frontmatter, as
  optional strings: omitted when unset, and a file written before the fields existed loads
  with none. A rebuild round-trips them.
- **A task's declared `impact`** (`modify`, `add_under`, `remove`, `files`; see
  impact-and-conflicts.md) lives only in the frontmatter, as an optional `[impact]` table
  of string arrays, last in the file: omitted when empty, and a file written before the field
  existed loads with none. There is no SQLite column (the `tasks` table stays narrow); a
  rebuild round-trips it.
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

## Rebuild

`bridle rebuild` (`TaskManager::rebuild_from_state_branch`) reconstructs
`tasks`, `edges`, `open_questions` and `claims` from the state branch alone:
the migration path for a fresh clone with no `bridle.db` — clone the repo,
start the daemon, `bridle rebuild`. It walks every `tasks/<id>.md` file for
the task rows, `edges.toml` for the edge set, each task's own thread for its
open question, if any (the most recent `question`/`answer` entry; an
unanswered trailing `question` becomes an `open_questions` row), and
`claims.toml` for who's working what. Refuses (409) rather than overwriting
if the database already has any rows in these four tables — a rebuild is a
from-nothing reconstruction, not a merge.

Restoring a claim also sets that task's `tasks.state` back to `Claimed` in
SQLite: the live `claim_task` path does that synchronously and never on the
state branch (above), so the task file rebuild just restored the row from
still says whatever state it was in before the claim (typically `planned`);
rebuild mirrors `claim_task`'s own synchronous update explicitly rather than
getting it from the task file, which never carries `claimed`.

The queue is reloaded too (`TaskManager::queue` cache, from `queue.toml`),
the same as any other startup — there was never a SQLite table for it to
refuse a rebuild over.

A rebuilt open question's `message_id` doesn't point at a real `messages`
row: messages don't survive on the state branch at all — the durability
table above lists them as SQLite-only, "no durability, by design" — only the
thread entry recording the question's body/from/timestamp does. Rather than
leave the column unfillable, rebuild synthesizes a stand-in id from the task
id (`m-rebuilt-<task id>`). Nothing downstream looks a message up by this id
after a rebuild, since there's no message row behind it to find.

`handovers/<id>.md` files are restored too, keeping each note's id and `seq` (so new notes
resume above the highest); a note already in the table is skipped, and they don't count towards
the refusal above.

**Fetching from origin.** With `bridle rebuild --from-origin`, or on a daemon's first start with
no local `bridle/state` when `[state] push = true` (the default, or explicit `push = false` opt-out), the daemon first fetches
`origin/bridle/state` (`StateBranch::fetch_from_origin`). It only reads from the remote and never
overwrites: a missing local branch is created from it; a local branch that is behind it, or is
only the empty seed commit the daemon creates, is fast-forwarded; if both exist and differ,
nothing changes and the command says so. A project with `push = false` and no flag never fetches.

**Ownership (hw6c).** One file, `owner.toml` (`host`, `since`), records the machine whose daemon
serves the project. Every `bridle serve` with pushing on fetches `origin/bridle/state` before
opening the branch, on every start and not only the first: a fresh clone (a missing or seed-only
local branch) adopts origin's state, so no manual `git fetch origin bridle/state:bridle/state`
is needed. If origin's `owner.toml` names another host the daemon refuses to start, saying which
host and since when. `bridle serve --take-over` claims the project instead: run it on the new
machine after the old daemon stopped and pushed. `--take-over` is strict (24mj): it refuses,
naming the local and origin SHAs, unless origin was reached and the local `bridle/state` was
created, fast-forwarded or already level with it (diverged, unreachable, no remote branch and no
`origin` remote are all errors), and it fetches the integration branch too, refusing unless the
local one is an ancestor of origin's and fast-forwarding it only from a clean checkout. No flag
overrides this; nothing is rebased, reset or forced. The claim is a normal flush and we2r push.
`since` is when the host took the project over; a restart on the same host makes no commit. No
origin, no `bridle/state` on it, an unreachable origin (all without `--take-over`), no `owner.toml` (first serve, or a branch
from before this) and the same host all start as before.

## The daemon registry

`~/.bridle/daemons/<project>.json` lists each running daemon with its
workspace, repo and URL
([[docs/design/agent-host/operating-model#Several projects at once|several projects]]).
Views across projects (`bridle daemons`, and later `status --all`,
`ready --all`) fan out over it.
