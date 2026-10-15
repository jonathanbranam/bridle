---
id: br-29se
title: "bridle projects add: register a new project here and sync it to every machine"
kind: feature
created: 2026-10-14
created_by: external:advisor/wsl2
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: [br-afzu]
related: [br-8uxe, br-kf7t, br-g9qm, br-afzu]
resolved: 2026-10-15T20:52:32Z
---

## The ask

The human, 2026-10-14 (to advisor wsl2), after the remus home-project setup (8uxe) hit kf7t and g9qm: "setting up a new project should be very smooth. I also want a command that adds a new project to the local config and then syncs to other machines."

## Context (advisor wsl2, checked 2026-10-14)

- Today a project is registered by hand: add `P = { machine = "M", port = N }` under `[projects]` in `~/.bridle/config.toml`, picking a free port by reading the list, then `bridle projects sync --apply` (`docs/context/adding-a-project.md` step 1; `docs/design/cli.md`, `projects sync`). On remus the human had to ask which port to use.
- `[mail] projects` is edited by hand on every machine too, until afzu makes `projects sync` spread `[mail]` and enroll every project.
- Ports in use today: 7401 to 7408; 7405 is used twice (bridle-ui on dalek, meta-notes-ui on vesta). `project take-over` already moves a port "up past one already used here" when a project changes machine.

## What's wanted

One command, e.g. `bridle projects add <name> [--machine M] [--port N]`:

- `--machine` defaults to this machine (`[machine] name`); it must be a key of `[machines]`.
- With no `--port`, it picks the next port not used by any entry in `[projects]` (across all machines, so a later move doesn't clash).
- Writes the `[projects]` entry here, keeping comments and other tables (as `projects sync` edits do), refusing a name that is already listed.
- Enrolls the project in mail (afzu: every project is in `[mail] projects`, `bridle` kept first).
- Runs `projects sync --apply`, so every machine has the entry, and prints what it changed.
- Docs: `adding-a-project.md` step 1 and the home-project steps (8uxe) use it.

Part of making a new project smooth, end to end: `projects add`, then `bridle project start` works on a brand-new project (kf7t) and its tokens finish with failed 0 (g9qm).

## Decided (the human, 2026-10-14 ~6:20 PM ET, to advisor product-manager)

Verbatim: "fine, yes then the project add can go in the queue now as well". Build it now, ahead
of the br-dffz epic (which stays backlog; its `setup` will call this). The command is spelled
`bridle project add` (br-tmdf: one group, `bridle project <verb>`). It is "register only", a
different action from `setup`; say so in its help. Per the dffz design, it ends by naming the
next step (`next: bridle project start`) when it can tell.
