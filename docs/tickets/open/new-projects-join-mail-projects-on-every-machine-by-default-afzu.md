---
id: br-afzu
title: projects sync spreads [mail], and every project is enrolled in mail on every machine
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

The human, 2026-10-14 (to advisor wsl2), on `[mail] projects`: "this needs to be part of what happens when a new project is created in bridle - add it to mail (opt-out) on all machines in the cluster." Then: "project sync should also sync the mail settings". Then, on the opt-out: "no need to actually build opt-out, let's just enroll every project in mail. The opt-out I think is if the local machine doesn't have mail configured or there could be a [mail] option to disable it? Or b/c the aws creds aren't configured. Default is: every project on every machine has mail but if a machine isn't configured for mail, that is OK"

## Context (advisor wsl2, checked 2026-10-14)

- `[mail] projects` in `~/.bridle/config.toml` lists every project name mail may go to, on any machine, and must be the same on every machine that runs a bridge, with `bridle` first: only the first name's bridge answers mail to unknown names, so a differing order means double or missing replies (`docs/design/mail.md`, Config and "Following the owner"). `bridle project start` sets up a moved project's bridge only when `[mail] projects` names it (`docs/design/cli.md`, `project start`).
- Today the list is kept by hand on each machine (`docs/context/mail-on-a-machine.md`, step 4). Adding a project is manual too: `bridle init` doesn't touch `~/.bridle` (`docs/context/adding-a-project.md`); step 1 there adds the `[projects]` line on every machine, and `bridle projects sync --apply` copies `[machines]` and `[projects]` to the others, but not `[mail]`.
- A per-project opt-out key already exists in the same area: `[mail] peers = false` in a project's `.bridle/config.toml` (leaves it out of peer tokens).

## What's wanted

- **Every project is enrolled in mail.** `[mail] projects` names every project in `[projects]`, `bridle` first, the same list on every machine. No per-project opt-out (the human: "no need to actually build opt-out").
- **`bridle projects sync` spreads `[mail]`** like `[machines]` and `[projects]`: it compares, and with `--apply` fixes, the `[mail]` section, including `projects` (so a project added to `[projects]` is enrolled by the same sync).
- **A machine without mail is fine.** The opt-out is per machine, not per project: a machine that isn't set up for mail (no AWS credentials, no `mail.key`; `docs/context/mail-on-a-machine.md`) runs no bridges and is not an error. Its projects' mail waits in the bucket and the sender gets the usual "not delivered yet" (`docs/design/mail.md`, "Following the owner").

Open for the designer:

- How a machine says "no mail" once sync spreads `[mail]` everywhere: the human suggests either "the local machine doesn't have mail configured", "a [mail] option to disable it", or "b/c the aws creds aren't configured". Note that sync would copy `[mail]` onto such a machine, so "no `[mail]` section" alone can't be the signal; a machine-level key (e.g. `[mail] enabled = false`, kept local and not synced) or a check for credentials at `project start` / `mail install` time are the candidates.
- What `project start` and `mail install` do on such a machine: skip the bridge with a note, not fail the start.
