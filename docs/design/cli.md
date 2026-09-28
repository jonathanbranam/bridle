# The CLI

Every command is a thin client of the daemon's API
([[docs/design/agent-host/api|API]]). Every command takes `--json`, which agents
always use; humans get compact tables.

## Built

```
bridle [--url URL] [--project NAME] [--token T] [--json] <command>

bridle serve   [--repo PATH] [--workspace DIR] [--listen ADDR] [--detach]
bridle stop-daemon
bridle rebuild                               reconstructs tasks/edges/open_questions from the
                                              state branch alone; the migration path for a fresh
                                              clone with no bridle.db yet
bridle daemons                              # every running project daemon on this machine, with agent counts
bridle status                               # daemon, agents, Claude Code version
bridle spawn   <role> [--name N] [--prompt TEXT | --prompt-file FILE]
               [--worktree [--base REF] | --in-repo | --cwd PATH] [--model M]
               [--ignore-budget]
bridle agents  [--all]
bridle show    <agent>
bridle send    <agent|human|role:NAME> [TEXT | --text-file FILE] [--question] [--when now|idle] [--reply-to ID]
bridle inbox   [--all] [--mark-read]        # messages to me, plus every task's open question
bridle ask     <task-id> TEXT                    question against a task; blocks it until answered
bridle answer  <task-id> TEXT                    answers a task's open question; frees it to be ready again
bridle claim   <task-id>                         claims a ready task for the caller: planned -> claimed
bridle release <task-id>                         releases the caller's own claim: claimed -> planned
bridle interrupt <agent> [--drop-held]
bridle stop    <agent> [--now]      bridle resume <agent> [--ignore-budget]
bridle renew   <agent> [--ignore-budget]    stop + fresh process/session, same worktree/branch/role/model
bridle rm      <agent> [--force] [--delete-branch]
bridle logs    <agent> [--follow] [--raw] [--since LINE]
bridle events  [--follow] [--since SEQ] [--agent A] [--kind PREFIX]
bridle usage   [--by role|model|agent] [--since DURATION]   # DURATION: <n>s|m|h|d, e.g. 30d
bridle cost audit [--check]                 static: size of what bridle injects into agent context (usage-and-budget.md)
bridle tui                                  interactive terminal UI: agents list, live event tail
bridle budget [hold [--for D|--until T] | release]   usage governor (usage-and-budget.md)
bridle token create <name>
bridle token list                           name, created-at, revoked-or-not; never the token itself
bridle token revoke <name>                  human only, external tokens only (an agent's own token is
                                             revoked through `bridle rm`, not this)
bridle statusline                           Claude Code statusLine command; local only, no daemon call
bridle stop-check                           Claude Code Stop hook for the worker role; refuses to stop
                                             with an unreleased claim and no thread entry since claiming
                                             it (docs/design/coordination.md); never fails
bridle task new    <title> -k/--kind KIND [--body TEXT]
bridle task show   <id>
bridle task edit   <id> [--title TEXT] [--body TEXT]
bridle task list   [--claimed-by WHO]             WHO: me|human|<agent name>|<principal id>; unclaimed tasks have no claimant to match
bridle task drop   <id> --reason TEXT
bridle task reopen <id>
bridle task note   <id> TEXT                     plain note to the task's thread; no effect on readiness
```

- **`--ignore-budget`** on `spawn`/`resume`/`renew` skips the budget governor's
  holding/paused refusal for that one call
  ([[docs/design/usage-and-budget#Resuming|the escape hatch]]).
- **`budget hold`/`release`** apply to the current daemon only; see
  [[docs/design/usage-and-budget#The human's hold|the human's hold]] for the
  cross-daemon gap.

- **`send`**: when given `--text-file FILE` or `--prompt-file FILE`, pass `-`
  as the filename to read from stdin instead. This avoids passing backticks and
  other shell metacharacters as command-line arguments, which can trigger
  permission denials in Claude Code. Example: `echo "message" | bridle send w1 --text-file -`.
  `role:NAME` fans the message out to every live agent currently holding that
  role — one delivered message per matching agent, same as sending to each
  individually; `bridle send` prints one `sent <id> -> <to>` line per recipient.
  A role with no live agents is an error, same as an unknown agent name.

- **`rebuild`** is `TaskManager::rebuild_from_state_branch` (docs/design/storage.md,
  "Rebuild"): the migration path for a fresh clone with no `bridle.db` — clone the repo,
  start the daemon, `bridle rebuild`. Human-only; refuses (409) rather than overwrites if
  the database already has any tasks, edges or open questions. Claims are never
  reconstructed — they're SQLite-only, with no state-branch counterpart, so any in-flight
  claim is simply lost, which is correct here, not a gap.
- **`task list --claimed-by`** filters on the task's current claimant
  (`Task::claimed_by`, docs/design/storage.md). `me` resolves to the calling
  principal's own id, the same as `--to me` on `send`/`inbox`; anything else
  is looked up as an agent name first, then matched against `claimed_by`
  verbatim, so a full principal id (`agent:w1`, `human`) works too. A task
  with no claim never matches any filter value.
- **Discovery** of the daemon, and **which token** the CLI uses, are in
  [[docs/design/agent-host/daemon#Workspace layout|workspace layout]] and
  [[docs/design/agent-host/principals#How the CLI picks a token|principals]].
  `--project` also reads `$BRIDLE_PROJECT`, including for `serve`, where it
  names the project being served.
- **Exit codes**: 0 ok, 1 error, 2 usage error, 3 daemon unreachable (every
  discovery failure, including an unknown `--project`).
- **`logs`** renders the transcript's output lines; `--raw` prints every line
  verbatim. Without `--since` it shows the latest lines (tail), not the
  oldest; give `--since` to page forward from a line number instead.
  `--follow` polls once a second.
- **`events`** without `--follow` returns the most recent 500 matching
  events, oldest first; give `--since` to page forward from a cursor instead.
  `--follow` streams over SSE, filtering agent and kind on the client, and
  starts at the tail unless given `--since` (or resuming after a
  reconnect), in which case it backfills from that cursor first.
- **`daemons`** hits every registered daemon's unauthenticated `GET /v1/health`
  concurrently, with a ~1 s timeout each, to show non-terminal agent counts
  without a cross-daemon token. A daemon that doesn't answer in time (dead,
  slow, unreachable) shows `?` (`null` under `--json`) instead of blocking on
  it.
- **`cost audit`** is local and static: no daemon call, just `.bridle/config.toml` and
  `.bridle/cost-baseline.json` read from the current directory. It measures each role's
  rendered system-prompt file (the part meant to be identical across agents of a role,
  `stable_system_prompt`) from a fresh render, and reports it next to the committed
  baseline. The other categories the design names (prime, skill descriptions, MCP tool
  schemas, hook boilerplate) aren't measured yet because they don't exist in bridle
  today; see `bridle_daemon::cost_audit`. Without `--check` it only reports; with it,
  exit 1 if any role grew more than
  `bridle_daemon::cost_audit::GROWTH_THRESHOLD_PERCENT` over baseline.
- **`serve --detach`**: [[docs/design/agent-host/daemon#Running it|running the daemon]].
- **`tui`** is a subcommand, not a separate binary, so it shares `bridle`'s discovery,
  token and `--url`/`--project` flags like every other command. It's a thin client of
  `bridle-api`'s `Client`, with four views: an agents list (seeded from `GET
  /v1/agents`, kept live by `agent.state`/`agent.removed` events), a scrolling event
  tail (`events_stream`, which already reconnects on its own — see
  `crates/bridle-api/src/client/mod.rs`), the selected agent's transcript tail (polled
  from `Client::transcript` once a second, same model as `bridle logs --follow`), and
  an inbox of unread messages addressed to `me` (polled from `Client::list_messages`
  once a second, same query as `bridle inbox`). `Tab` switches between the four views,
  `j`/`k`/arrow keys scroll the focused one, `q`/`Esc` quits. On the inbox view, `r`
  starts composing a reply to the selected message (simple line editing: insert,
  backspace, left/right, `Enter` to send, `Esc` to cancel); a submitted reply goes out
  via `Client::send` with `reply_to` set and `when: now`, then the original is marked
  read via `Client::mark_read` so it drops out of the unread list. Lives in its own
  crate, `crates/bridle-tui`, split Elm-style: a plain state struct and update function
  with no terminal/ratatui dependency (so it's unit-tested without a terminal),
  rendered by a separate `ui` module.
- **`task`** is scoped, for now, to the `open`/`planned`/`dropped`/`reopened` states
  (docs/design/roles-and-lifecycle.md, Task lifecycle): create, show, edit (title/body,
  never state), list (id/title/kind/state), drop (a reason is required, recorded in the
  task's thread) and reopen (only a dropped task can be reopened). `in_review`,
  `integrated`, `accepted`, and everything that depends on those, arrive with later
  tasks — see the `Planned` block below for the rest of the surface this command will
  eventually grow into. There's still no `plan` yet, so nothing can reach `planned`
  through the CLI — which means `ready` (below) can never actually return anything, and
  `claim` can only be exercised in its "not ready" conflict shape, until a later task
  adds it; documented as a known gap, not fixed here.
- **`dep add|rm`** creates or removes one coordination edge (docs/design/coordination.md).
  `bridle dep add <task> --to <other> --kind <kind>` draws `<task> --kind--> <other>`;
  `bridle dep add <task> --blocked-by <other>` is sugar for `--kind blocks` with `from`
  and `to` swapped (`<task>` is blocked by `<other>`, so the edge runs the other way) and
  can't be combined with `--to`/`--kind`. `dep rm` takes the same shape. Edges can't
  connect a task to itself, and a repeat of the same `(from, to, kind)` triple is a
  conflict, not a silent no-op.
- **`ask`/`answer`** are thin clients of `TaskManager::ask_question`/`answer_question`
  (docs/design/coordination.md, "Questions do not stop work"): `ask` appends a `question`
  thread entry and blocks the task's readiness until answered (`Conflict` if one is
  already open); `answer` appends an `answer` entry and clears the block. Neither takes a
  recipient — a question addressed to a task has no single recipient, per
  coordination.md's message table — so there's no `--to`; send-to-task is a
  later task. `inbox` lists every task's open question (task id, asker,
  body, age) alongside messages addressed to `me`, reading `GET /v1/questions`
  (`TaskManager::list_open_questions`, backed by the same in-memory cache `is_ready`
  reads) rather than walking the state branch.
- **`claim`/`release`** are thin clients of `TaskManager::claim_task`/`release_task`
  (docs/design/storage.md, "claims and leases"): `claim` moves a `planned`, unblocked
  task to `claimed` for the calling principal (`Conflict` if it isn't ready to claim —
  not planned, blocked, or already claimed); `release` moves it back to `planned`
  (`Conflict` if the caller isn't the current claimant, including if it isn't claimed at
  all). Neither takes a body: the claimant is always the caller's own token. A claimed
  task drops out of `ready` immediately, since `is_ready` requires `planned`; releasing
  it (explicitly, or via the lease-expiry tick) puts it back.
- **`ready [--all] [--role]`** lists every ready task: `planned`, with no open `blocks`
  edge naming an unresolved blocker (roles-and-lifecycle.md, "ready is computed"; see
  coordination.md for exactly what "unresolved" means in this build). `--all` fans out
  across every daemon in the registry (`bridle daemons`), each with its own
  discovery-resolved token, instead of just the one daemon `--url`/`--project`/cwd
  discovery would pick. `--role` is accepted but a no-op: tasks don't carry a role field
  yet (a gap, not a design decision — see `Planned` below).
- **`statusline`** is Claude Code's `statusLine` command, configured in `settings.json`. It
  reads Claude Code's JSON on stdin and prints a short line back: model, context %
  (`context_window.used_percentage`), the `5h`/`7d` rate-limit windows, the current folder
  and git branch, and the estimated session cost last, parenthesized. It's purely local —
  no daemon call, no token, never fails or hangs — since it runs on every render of the
  prompt. It does **not** call `POST /v1/statusline` any more (dropped in s8kn: the context
  governor gets account-wide windows from `get_usage` instead); that route and the
  `interactive_usage` table still exist in the daemon, unused for now, in case something
  needs per-invocation interactive snapshots later ([[docs/design/usage-and-budget#Where bridle can see usage|usage and budget]]).

  If `~/.bridle/statusline.token` holds a token, it also appends a short "N working · M for
  you" from `GET /v1/status` (agents in `working`/`starting`, and `unread_human_messages`) —
  a 2s-timeout, best-effort call: no token file, no daemon found, a timeout or an HTTP error
  all just skip the counts silently (logged at `tracing::debug`), never delaying or blanking
  the rest of the line. This is the one case where `statusline` does call the daemon, but
  never with `$BRIDLE_TOKEN` or the workspace's human token file — only this dedicated,
  per-user file (see [[statusline-bridle-counts-with-a-read-only-token-r7cs|r7cs]]).
  Deliberately not `$BRIDLE_TOKEN`: `resolve_token` checks it first, unconditionally, for
  every command, so exporting it in the shell profile would make every human-run bridle
  command (`stop-daemon`, `budget hold`/`override`, `token create`, ...) act as this token's
  principal instead of the human's. The token is **not** scoped read-only or to this route —
  bridle has no per-route/per-token scoping yet, so it can do whatever an `external:*`
  principal can do (send messages, spawn agents, ...); that gap is real, just not solved
  here. One-time setup: `bridle token create statusline > ~/.bridle/statusline.token` to mint
  an `external:statusline` token and store it where `statusline` reads it (a fixed path under
  `$BRIDLE_HOME`/`~/.bridle`, not the workspace's own `.bridle/`, since this needs to work
  regardless of which project workspace Claude Code happens to be in).
- **`stop-check`** is Claude Code's `Stop` hook, registered only for the worker role
  ([[docs/design/coordination#How agents actually hear things (Claude Code integration)|coordination.md]],
  [[docs/spikes/05-stop-hook-findings|spike 05]]). It reads the hook's JSON on stdin; if
  `stop_hook_active` is set it allows immediately (Claude Code silently overrides a hook
  after 9 consecutive blocks in one turn, so a well-behaved hook blocks at most once per
  turn). Otherwise it lists the calling principal's own claimed tasks
  (`?claimed_by=me`) and blocks — printing the flat `{"decision":"block","reason":"..."}`
  spike 05 confirmed, not the `hookSpecificOutput` wrapper — on the first one with no
  thread entry (note, question or answer) from itself at or after `claimed_at`; naming
  the task and telling the agent to `bridle release` it or leave a `bridle task note`
  first. Any error of bridle's own (unparseable stdin, no daemon reachable, an API
  error) allows rather than blocks: a bug in bridle's own tooling must never trap an
  agent from stopping.

## Planned

Commands for the phases after v1 ([[docs/proposal/build-order|build order]]),
as a first cut:

```
bridle init | sync | prime | doctor              project setup, render, session start, health
bridle task <cmd> at in_review|integrated|accepted  -- new/show/edit/list/drop/reopen
                                                  are built (see Built); `dep add|rm`,
                                                  `claim`/`release`, and `ready [--all] [--role]`
                                                  are built too, but ready can't return anything
                                                  until `plan` exists
bridle handoff bridle plan <id>                   bridle accept <id> (human only)
bridle inbox --inject        # `ask`/`answer` are built (see Built)
bridle wait <id> [--until <state>] [--or-message] [--timeout]
bridle spawn <role> <task>   bridle review
bridle take|give <agent>                         human takeover of a headless agent
bridle impact set|show|check bridle conflict list|resolve
bridle spec check|id|export|coverage|import
bridle rules show|explain|diff|propose
bridle goals list|propose       bridle arch propose
bridle trace up|down|suspect|confirm|orphans|coverage
bridle explore new|conclude|adopt|abandon
bridle usage --by project|kind|task|trend|compare
```

The command name and a short alias are open:
[[command-name-and-short-alias-sqt6|command name]].
