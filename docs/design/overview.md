# Architecture in one picture

```
                         ┌──────────────────────────────────────────┐
                         │  bridle-workflow repo  (the one place)    │
                         │   base/   packs/<stack>/   (git)          │
                         └──────────────┬───────────────────────────┘
                                        │ resolved per project
  ┌─────────────────────────────────────▼─────────────────────────────────────┐
  │  one workspace per project                                                 │
  │                                                                            │
  │  bridle daemon  (bridle serve)   HTTP + SSE API on /v1                     │
  │    <workspace>/.bridle/bridle.db  (SQLite, WAL)  live index + ephemeral    │
  │       agents · messages · events · usage   (later: tasks · edges ·         │
  │       claims · impact · waits)                                             │
  │    supervisor: one headless `claude -p` stream-json process per agent      │
  │                                                                            │
  │  <workspace>/<repo>/            the clone                                  │
  │    .bridle/                     project layer: config, rules, overrides (git)
  │    design/                      goals, architecture, specs, explorations (git)
  │    .claude/…  CLAUDE.md block   rendered outputs (gitignored or managed)   │
  │  <workspace>/wt/<agent>/        one worktree per agent                     │
  │  state branch `bridle`          task records + event log (git, own worktree)
  └──────────┬───────────────────────────────┬─────────────────────────────────┘
             │ stdin / stdout stream-json    │ agents call back: `bridle` CLI
     ┌───────▼───────┐   ┌──────────────┐   ┌─▼────────────┐
     │ manager        │   │ worker (wt A) │   │ worker (wt B) │
     └───────────────┘   └──────────────┘   └──────────────┘
             ▲
             │ HTTP (CLI, TUI, orchestrator agent), locally or remotely
```

- **One daemon per workspace**, and one workspace per project clone. Projects
  are isolated from each other; a machine-level registry lists the running
  daemons ([[docs/design/agent-host/operating-model|operating model]]).
- **The CLI is a thin client** of the daemon's API. So are the human's
  orchestrator agent, the agents bridle hosts, and later a TUI or MCP server
  ([[docs/design/agent-host/api|API]]).
- **The daemon is built** (v1: the agent host). Tasks, the state branch,
  workflow layers and everything under `design/` are designed here and not yet
  built ([[docs/proposal/build-order|build order]]).

Three kinds of state, each in the place its lifetime demands:

| State | Lives in | Lifetime | Survives a lost DB? |
|---|---|---|---|
| Workflow, rules, guidelines, skill sources | `bridle-workflow` repo + `<repo>/.bridle/` | months–years | yes (git) |
| Goals, architecture, specs, exploration findings | `<repo>/design/`, edited on task branches ([knowledge tiers](docs/design/knowledge-tiers.md)) | months–system lifetime | yes (git) |
| Tasks, edges, decisions, answered questions, impact | state branch `bridle` in each repo | until closed, then history | yes (git) |
| Agents, claims, leases, unread flags, waits, agent chatter, events, usage | `<workspace>/.bridle/bridle.db` only | minutes–days | **no, by design** |

`bridle rebuild` recreates the database from the project's state branch. That
is the migration story: clone the repo into a workspace on the new machine,
start the daemon, `bridle rebuild`.
