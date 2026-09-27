# The CLI

Every command is a thin client of the daemon's API
([[docs/design/agent-host/api|API]]). Every command takes `--json`, which agents
always use; humans get compact tables.

## Built

```
bridle [--url URL] [--project NAME] [--token T] [--json] <command>

bridle serve   [--repo PATH] [--workspace DIR] [--listen ADDR] [--detach]
bridle stop-daemon
bridle daemons                              # every running project daemon on this machine, with agent counts
bridle status                               # daemon, agents, Claude Code version
bridle spawn   <role> [--name N] [--prompt TEXT | --prompt-file F]
               [--worktree [--base REF] | --in-repo | --cwd PATH] [--model M]
               [--ignore-budget]
bridle agents  [--all]
bridle show    <agent>
bridle send    <agent|human> TEXT [--question] [--when now|idle] [--reply-to ID]
bridle inbox   [--all] [--mark-read]        # messages to me (human, or the calling agent)
bridle interrupt <agent> [--drop-held]
bridle stop    <agent> [--now]      bridle resume <agent> [--ignore-budget]
bridle rm      <agent> [--force] [--delete-branch]
bridle logs    <agent> [--follow] [--raw] [--since LINE]
bridle events  [--follow] [--since SEQ] [--agent A] [--kind PREFIX]
bridle usage
bridle cost audit [--check]                 static: size of what bridle injects into agent context (usage-and-budget.md)
bridle tui                                  interactive terminal UI: agents list, live event tail
bridle budget [hold [--for D|--until T] | release]   usage governor (usage-and-budget.md)
bridle token create <name>
bridle token list                           name, created-at, revoked-or-not; never the token itself
bridle token revoke <name>                  human only, external tokens only (an agent's own token is
                                             revoked through `bridle rm`, not this)
bridle statusline                           usage from interactive sessions
bridle task new    <title> -k/--kind KIND [--body TEXT]
bridle task show   <id>
bridle task edit   <id> [--title TEXT] [--body TEXT]
bridle task list
bridle task drop   <id> --reason TEXT
bridle task reopen <id>
```

- **`--ignore-budget`** on `spawn`/`resume` skips the budget governor's
  holding/paused refusal for that one call
  ([[docs/design/usage-and-budget#Resuming|the escape hatch]]).
- **`budget hold`/`release`** apply to the current daemon only; see
  [[docs/design/usage-and-budget#The human's hold|the human's hold]] for the
  cross-daemon gap.

- **Discovery** of the daemon, and **which token** the CLI uses, are in
  [[docs/design/agent-host/daemon#Workspace layout|workspace layout]] and
  [[docs/design/agent-host/principals#How the CLI picks a token|principals]].
  `--project` also reads `$BRIDLE_PROJECT`, including for `serve`, where it
  names the project being served.
- **Exit codes**: 0 ok, 1 error, 2 usage error, 3 daemon unreachable (every
  discovery failure, including an unknown `--project`).
- **`logs`** renders the transcript's output lines; `--raw` prints every line
  verbatim. `--follow` polls once a second.
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
  task's thread) and reopen (only a dropped task can be reopened). `ready`, `claimed`,
  `in_review`, `integrated`, `accepted`, and everything that depends on edges, questions
  or claims, arrive with later tasks — see the `Planned` block below for the rest of the
  surface this command will eventually grow into.
- **`statusline`** is Claude Code's `statusLine` command, configured in `settings.json`. It
  reads Claude Code's JSON on stdin, prints a short line back, and posts a snapshot to
  `POST /v1/statusline` ([[docs/design/agent-host/api|API]]) using the same daemon discovery
  and token as every other command, but with a 2 s request timeout. It never fails or hangs:
  unparseable stdin, no daemon, and a slow or unreachable daemon all just mean the line prints
  with whatever it has and nothing gets recorded ([[docs/design/usage-and-budget#Where bridle can see usage|usage and budget]]).

## Planned

Commands for the phases after v1 ([[docs/proposal/build-order|build order]]),
as a first cut:

```
bridle init | sync | prime | doctor              project setup, render, session start, health
bridle task <cmd> at ready|claimed|in_review|integrated|accepted  -- new/show/edit/list/drop/reopen
                                                  are built (see Built); these five states aren't
bridle dep add|rm            bridle ready [--all] [--role]
bridle claim|release|handoff bridle plan <id>     bridle accept <id> (human only)
bridle ask|answer            bridle inbox --inject
bridle wait <id> [--until <state>] [--or-message] [--timeout]
bridle spawn <role> <task>   bridle review
bridle take|give <agent>                         human takeover of a headless agent
bridle impact set|show|check bridle conflict list|resolve
bridle spec check|id|export|coverage|import
bridle rules show|explain|diff|propose
bridle goals list|propose       bridle arch propose
bridle trace up|down|suspect|confirm|orphans|coverage
bridle explore new|conclude|adopt|abandon
bridle usage --by …|task|trend|compare
bridle rebuild
```

The command name and a short alias are open:
[[command-name-and-short-alias-sqt6|command name]].
