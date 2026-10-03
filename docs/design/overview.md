# Architecture in one picture

> **Status (checked 2026-10-03):** Built and in use: the daemon and agent host, tasks, edges, claims, the queue, the state branch (`bridle/state`), `bridle rebuild`, workflow rule resolution (base, packs, project) delivered in each spawned agent's system prompt · Built, not wired in: the `design/` tiers' commands (`bridle spec|goals|arch|explore|trace`); no project has a `design/` tree yet · Planned: the MCP server; a separate `bridle-workflow` repo (bridle keeps `workflow/` in-repo, decision r2uq)

```
                         ┌──────────────────────────────────────────┐
                         │  workflow dir (bridle's own: workflow/)   │
                         │   base/   packs/<stack>/   (git)          │
                         └──────────────┬───────────────────────────┘
                                        │ resolved per project
  ┌─────────────────────────────────────▼─────────────────────────────────────┐
  │  one workspace per project                                                 │
  │                                                                            │
  │  bridle daemon  (bridle serve)   HTTP + SSE API on /v1                     │
  │    <workspace>/.bridle/bridle.db  (SQLite, WAL)  live index + ephemeral    │
  │       agents · messages · events · usage · tasks · edges · claims ·        │
  │       impact · conflicts · ports                                           │
  │    supervisor: one headless `claude -p` stream-json process per agent      │
  │                                                                            │
  │  <workspace>/<repo>/            the clone                                  │
  │    .bridle/                     project layer: config, rules, overrides (git)
  │    design/                      goals, architecture, specs, explorations (git)
  │    .claude/…  CLAUDE.md block   rendered outputs (gitignored or managed)   │
  │  <workspace>/wt/<agent>/        one worktree per agent                     │
  │  state branch `bridle/state`    task records, edges, claims, queue (git, own worktree)
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
  orchestrator agent, the agents bridle hosts, the TUI (`bridle tui`) and the
  gateway; an MCP server is planned ([[docs/design/agent-host/api|API]]).
- **The daemon is built** (v1: the agent host), and so are tasks, the state
  branch and workflow rule resolution. The `design/` tiers have local commands
  but no project uses them yet ([[docs/proposal/build-order|build order]]).

Three kinds of state, each in the place its lifetime demands:

| State | Lives in | Lifetime | Survives a lost DB? |
|---|---|---|---|
| Workflow, rules, guidelines, skill sources | the `workflow` dir (in-repo, a sibling checkout, or vendored in `.bridle/workflow/`) + `<repo>/.bridle/` | months–years | yes (git) |
| Goals, architecture, specs, exploration findings | `<repo>/design/`, edited on task branches ([knowledge tiers](docs/design/knowledge-tiers.md)) | months–system lifetime | yes (git) |
| Tasks, edges, claims, the queue, answered questions, impact | state branch `bridle/state` in each repo | until closed, then history | yes (git) |
| Agents, claims, leases, unread flags, waits, agent chatter, events, usage | `<workspace>/.bridle/bridle.db` only | minutes–days | **no, by design** |

`bridle rebuild` recreates the database from the project's state branch. That
is the migration story: clone the repo into a workspace on the new machine,
start the daemon, `bridle rebuild`.
