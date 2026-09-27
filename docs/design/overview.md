# Architecture in one picture

> **Superseded in part** (note added when this file was split out of `design.md`):
> `docs/agent-host.md` §1.2 replaces the single global `~/.bridle/bridle.db` in this picture with **one daemon per workspace** (one project clone), state in `<workspace>/.bridle/`, and the CLI as a thin client of the daemon's API. A global registry can still exist as a list of daemons.

```
                         ┌──────────────────────────────────────────┐
                         │  bridle-workflow repo  (the one place)    │
                         │   base/   packs/<stack>/   (git)          │
                         └──────────────┬───────────────────────────┘
                                        │ resolved per project
  ┌─────────────────────────────────────▼─────────────────────────────────────┐
  │  bridle (Rust CLI)                                                         │
  │                                                                            │
  │  ~/.bridle/bridle.db  (SQLite, WAL)  ── live index + ephemeral state       │
  │     tasks · edges · claims · messages · impact · agents · waits            │
  │                                                                            │
  │  per project:                                                              │
  │    <repo>/.bridle/            project layer: config, rules, overrides (git)│
  │    <repo>/design/             goals, architecture, specs, explorations (git) │
  │    state branch `bridle`      task records + event log (git, own worktree) │
  │    .claude/…  CLAUDE.md block rendered outputs (gitignored or managed)     │
  └──────────┬───────────────────────────────┬─────────────────────────────────┘
             │ hooks: prime / inbox / heartbeat │ bridle wait (background Bash)
     ┌───────▼───────┐   ┌──────────────┐   ┌─▼────────────┐
     │ driver session │   │ worker (wt A) │   │ worker (wt B) │  … any project
     └───────────────┘   └──────────────┘   └──────────────┘
```

Three kinds of state, each in the place its lifetime demands:

| State | Lives in | Lifetime | Survives a lost DB? |
|---|---|---|---|
| Workflow, rules, guidelines, skill sources | `bridle-workflow` repo + `<repo>/.bridle/` | months–years | yes (git) |
| Goals, architecture, specs, exploration findings | `<repo>/design/`, edited on task branches ([knowledge tiers](docs/design/knowledge-tiers.md)) | months–system lifetime | yes (git) |
| Tasks, edges, decisions, answered questions, impact | state branch `bridle` in each repo | until closed, then history | yes (git) |
| Claims, leases, heartbeats, unread flags, waits, agent chatter | `~/.bridle/bridle.db` only | minutes–days | **no, by design** |

`bridle rebuild` recreates the database from the state branches of every
registered project. That is the migration story: clone the repos on the new
machine, `bridle project add` each, `bridle rebuild`.
