# The CLI

Every command is a thin client of the daemon's API
([[docs/design/agent-host/api|API]]). Every command takes `--json`, which agents
always use; humans get compact tables.

## Built

```
bridle [--url URL] [--project NAME] [--token T] [--json] <command>

bridle serve   [--repo PATH] [--workspace DIR] [--listen ADDR] [--detach]
bridle stop-daemon
bridle daemons                              # every running project daemon on this machine
bridle status                               # daemon, agents, Claude Code version
bridle spawn   <role> [--name N] [--prompt TEXT | --prompt-file F]
               [--worktree [--base REF] | --in-repo | --cwd PATH] [--model M]
bridle agents  [--all]
bridle show    <agent>
bridle send    <agent|human> TEXT [--question] [--when now|idle] [--reply-to ID]
bridle inbox   [--all] [--mark-read]        # messages to me (human, or the calling agent)
bridle interrupt <agent> [--drop-held]
bridle stop    <agent> [--now]      bridle resume <agent>
bridle rm      <agent> [--force] [--delete-branch]
bridle logs    <agent> [--follow] [--raw] [--since LINE]
bridle events  [--follow] [--since SEQ] [--agent A] [--kind PREFIX]
bridle usage
bridle token create <name>
```

- **Discovery** of the daemon, and **which token** the CLI uses, are in
  [[docs/design/agent-host/daemon#Workspace layout|workspace layout]] and
  [[docs/design/agent-host/principals#How the CLI picks a token|principals]].
  `--project` also reads `$BRIDLE_PROJECT`, including for `serve`, where it
  names the project being served.
- **Exit codes**: 0 ok, 1 error, 2 usage error, 3 daemon unreachable (every
  discovery failure, including an unknown `--project`).
- **`logs`** renders the transcript's output lines; `--raw` prints every line
  verbatim. `--follow` polls once a second.
- **`events`** without `--follow` returns the oldest 500 matching events.
  `--follow` streams over SSE, filtering agent and kind on the client, and
  replays the whole history first unless given `--since`.
- **`serve --detach`**: [[docs/design/agent-host/daemon#Running it|running the daemon]].

## Planned

Commands for the phases after v1 ([[docs/proposal/build-order|build order]]),
as a first cut:

```
bridle init | sync | prime | doctor              project setup, render, session start, health
bridle statusline                                usage from interactive sessions
bridle task new|show|edit|list|drop|reopen
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
bridle usage --by …|task|trend|compare          bridle cost audit
bridle token list|revoke
bridle rebuild
```

The command name and a short alias are open:
[[command-name-and-short-alias-sqt6|command name]].
