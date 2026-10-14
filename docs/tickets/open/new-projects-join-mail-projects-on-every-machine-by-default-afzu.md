---
id: br-afzu
title: projects sync spreads [mail], and new projects join [mail] projects on every machine (opt-out)
kind: feature
created: 2026-10-14
created_by: external:advisor/wsl2
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-n3kj, br-3pc2]
---

## The ask

The human, 2026-10-14 (to advisor wsl2), on `[mail] projects`: "this needs to be part of what happens when a new project is created in bridle - add it to mail (opt-out) on all machines in the cluster." Then: "project sync should also sync the mail settings".

## Context (advisor wsl2, checked 2026-10-14)

- `[mail] projects` in `~/.bridle/config.toml` lists every project name mail may go to, on any machine, and must be the same on every machine that runs a bridge, with `bridle` first: only the first name's bridge answers mail to unknown names, so a differing order means double or missing replies (`docs/design/mail.md`, Config and "Following the owner"). `bridle project start` sets up a moved project's bridge only when `[mail] projects` names it (`docs/design/cli.md`, `project start`).
- Today the list is kept by hand on each machine (`docs/context/mail-on-a-machine.md`, step 4). Adding a project is manual too: `bridle init` doesn't touch `~/.bridle` (`docs/context/adding-a-project.md`); step 1 there adds the `[projects]` line on every machine, and `bridle projects sync --apply` copies `[machines]` and `[projects]` to the others, but not `[mail]`.
- A per-project opt-out key already exists in the same area: `[mail] peers = false` in a project's `.bridle/config.toml` (leaves it out of peer tokens).

## What's wanted

When a project is added to bridle, it is added to `[mail] projects` on every machine in `[machines]` that has `[mail]`, by default, keeping `bridle` first and the lists identical. A project can opt out. `bridle projects sync` also compares and, with `--apply`, fixes the `[mail]` section (the human: "project sync should also sync the mail settings"), the same way it does `[machines]` and `[projects]`. Open for the designer: where the opt-out lives (the project's `.bridle/config.toml` is the obvious place); whether `[mail]` is synced whole or key by key, and what happens on a machine with no `[mail]` yet (it may not have AWS credentials or `mail.key`, `docs/context/mail-on-a-machine.md`, so adding `[mail]` there should probably be reported, not done silently).
