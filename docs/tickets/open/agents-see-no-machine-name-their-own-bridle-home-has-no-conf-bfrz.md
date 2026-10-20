---
id: br-bfrz
title: "Agents see no [machine] name: their own BRIDLE_HOME has no config.toml, so the home falls back to hostname (dalek.local-home)"
kind: fix
created: 2026-10-20
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Agents run with `BRIDLE_HOME=<state>/agent-home/<id>` (`crates/bridle-daemon/src/supervisor.rs`, k6b3), which has no `config.toml`. So in an agent's `bridle`, `machine_home()` (`crates/bridle-api/src/machines.rs:99`) finds no `[machine] name`, falls to `owner_names()`, which falls to the raw `hostname`: `dalek.local` on dalek. The home-project send warning then tells agents to use `--project dalek.local-home`, which is no daemon.

Give agents the machine's name and home: e.g. the supervisor passes them in the agent env (`[machine] name` and `home_project` from the real `~/.bridle/config.toml`), and `machine_home` reads that when its home has no config. Keep k6b3's point: no access to the real `~/.bridle`.

Why now: br-q7my turns this warning into a refusal; without this fix every agent on a machine whose hostname differs from `[machine] name` (dalek) would be refused or pointed at a wrong project. br-q7my is blocked by this ticket.

Seen: manager-2, 2026-10-20 (m-0370, m-0374). br-b2g8 (9e2b4858) found no bug because it tested with a real config.

Check: `just check`, plus a test: with `BRIDLE_HOME` set to an empty agent home and the supervisor's agent env, `machine_home` gives `dalek-home` (machine name `dalek`, any other hostname). Docs: docs/design/agent-host/ and cli.md if the env is documented.

Model: Sonnet. Related: br-b2g8, br-pcde, br-q7my.
