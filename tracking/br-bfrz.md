+++
id = "br-bfrz"
title = "Agents see no [machine] name: their own BRIDLE_HOME has no config.toml, so the home falls back to hostname (dalek.local-home)"
kind = "fix"
state = "built"
created_at = "2026-10-20T23:03:05.846Z"
updated_at = "2026-10-21T00:54:32.089884Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:bfrz",
]
branch = "bridle/bfrz"
commit = "202df2ca"
summary = "Supervisor agent_env now sets BRIDLE_MACHINE_NAME and BRIDLE_HOME_PROJECT (override only) from the real config; machine_home (bridle-api machines.rs) reads them when its home has no [machine], config still wins. Test: an_agent_home_without_config_reads_the_supervisors_env. Docs: agents.md. Real ~/.bridle stays out of reach."
ticket = "bfrz"
+++

docs/tickets/open/agents-see-no-machine-name-their-own-bridle-home-has-no-conf-bfrz.md

submitted by external:orchestrator

## The ask

Agents run with `BRIDLE_HOME=<state>/agent-home/<id>` (`crates/bridle-daemon/src/supervisor.rs`, k6b3), which has no `config.toml`. So in an agent's `bridle`, `machine_home()` (`crates/bridle-api/src/machines.rs:99`) finds no `[machine] name`, falls to `owner_names()`, which falls to the raw `hostname`: `dalek.local` on dalek. The home-project send warning then tells agents to use `--project dalek.local-home`, which is no daemon. Give agents the machine's name and home: e.g. the supervisor passes them in the agent env (`[machine] name` and `home_project` from the real `~/.bridle/config.toml`), and `machine_home` reads that when its home has no config. Keep k6b3's point: no access to the real `~/.bridle`.

## Why this matters now

br-q7my turns this warning into a refusal. Without this fix, every agent on a machine whose hostname differs from `[machine] name` (dalek) would be refused or pointed at a wrong project when it messages the orchestrator. br-q7my must be blocked by this ticket.

## Seen

manager-2 (bridle), 2026-10-20, m-0370 and m-0374: its CLI warned `--project dalek.local-home`. br-b2g8 (9e2b4858) found no bug because it tested with a real config; dalek's `~/.bridle/config.toml` does have `[machine] name = "dalek"`. Root cause traced by orchestrator 2026-10-20.

## Check

A test: with `BRIDLE_HOME` set to an empty agent home and the agent env from the supervisor, `machine_home` gives `dalek-home` (machine name `dalek`, hostname anything else). `just check`.

Model: Sonnet (env plumbing across daemon and api). Related: br-b2g8, br-pcde, br-q7my.

## Thread

### note · external:orchestrator · 2026-10-20T23:03:05.847Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-20T23:03:37.627Z
orchestrator: readied (fix, self-evident; rule ready-without-asking). Blocks br-q7my.

### note · agent:bfrz · 2026-10-21T00:54:31.593Z
built: 202df2ca (branch bridle/bfrz)

### note · agent:bfrz · 2026-10-21T00:54:32.089Z
done: agents get BRIDLE_MACHINE_NAME/BRIDLE_HOME_PROJECT env, machine_home reads it; just check exit 0, 1942 tests passed; 202df2ca (one earlier run hit the load flake restart_hold_test cancel_ends_an_upgrade_drain_quietly, passes alone and on rerun; cf. br-6wp2)
