---
id: br-tmdf
title: "One CLI group: bridle project <verb>; bridle projects only lists"
kind: feature
created: 2026-10-14
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-29se, br-dffz]
---

## The ask

The human, 2026-10-14 ~6:20 PM ET (to advisor product-manager), verbatim: "we shouldn't have project and projects as CLI commands either really; pick one;". On the advisor's recommendation (`project` as the group, singular like `agent`, `ticket`, `machine`, `message`; `projects` only lists, like `agents` and `messages`; old spellings warn for one release): "fine, yes then the project add can go in the queue now as well".

## What's wanted

- `bridle project <verb>` is the one group: `add` (br-29se), `sync`, `start`, `stop`, `take-over`,
  `preflight`, `drain`, and later `setup` (br-dffz).
- `bridle projects` only lists projects (like `bridle agents`), and takes no subcommands.
- `bridle projects <verb>` keeps working for one release, printing a warning naming the new form;
  then it is removed.
- Every doc, role file and runbook that says `bridle projects <verb>` says `bridle project <verb>`
  (git grep -n "bridle projects " docs workflow CLAUDE.md crates).
- Docs: cli.md, CHANGELOG.

Acceptance: `bridle projects sync` warns and works; `bridle projects` lists; `just check`.
