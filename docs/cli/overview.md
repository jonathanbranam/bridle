# Bridle: overview

Bridle is a daemon plus the `bridle` CLI. It spawns, supervises and records
headless Claude Code agents for a software project, and lets them (and the human)
coordinate through tasks, messages and a queue. Run `bridle docs` for the topic list.

## The parts

- **The daemon** (`bridle daemon serve`, once per workspace/project). It owns a
  SQLite database (`<workspace>/.bridle/bridle.db`) with agents, messages, events,
  usage and live task state, and runs one `claude -p` stream-json process per agent.
- **The CLI.** A thin client of the daemon's HTTP API. Agents call it from their
  shell; always pass `--json` for structured output.
- **Worktrees.** Each agent works on its own git worktree and branch
  (`<workspace>/wt/<agent>/`, branch `bridle/<agent>`).
- **The state branch** (`bridle/state`). Task records, edges, claims and the
  queue are mirrored to git, so `bridle rebuild` can restore the database.
- **The workflow.** Rules, role prompts and skills in layers (base, packs,
  project) that are resolved per project. See `bridle docs workflow-layers`.

## Who is who

The human decides priorities and accepts work. A manager plans and assigns, workers
implement one task each, and bridle itself integrates (`bridle task land`). Roles:
`bridle docs roles`.

## Where state lives

| State | Lives in | Survives a lost database? |
|---|---|---|
| Workflow, rules, role prompts | the workflow dir + `<repo>/.bridle/` (git) | yes |
| Tasks, edges, claims, the queue | state branch `bridle/state` (git) | yes |
| Agents, messages, events, usage | `bridle.db` only | no, by design |

## Your first commands

```
bridle status --json         # the system's state
bridle agents --json         # who is running
bridle inbox --json          # messages waiting for you
bridle task show <id>        # a task and its thread
```

Your identity comes from `BRIDLE_AGENT_ID`, `BRIDLE_AGENT_NAME`, `BRIDLE_URL` and
`BRIDLE_TOKEN`, set for every agent bridle spawns. Never share the token.

## More

`docs/design/overview.md`, `docs/design/agent-host/` (the daemon as built),
`docs/design/cli.md` (every command).
