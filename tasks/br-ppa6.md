+++
id = "br-ppa6"
title = "Gateway inherits BRIDLE_AS / BRIDLE_PROJECT from the shell that starts it: started from the orchestrator's session it calls every daemon as the orchestrator, and the web UI shows every project unreachable"
kind = "bug"
state = "integrated"
created_at = "2026-10-08T22:47:13.857Z"
updated_at = "2026-10-09T01:35:18.225712Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/gwenv"
commit = "25b5222ecbfa9a54fbedd3e6b11406c6eedebd21"
summary = "The gateway now resolves tokens through gateway::discovery::HumanEnv, which hides BRIDLE_AS, BRIDLE_PROJECT, BRIDLE_TOKEN and CLAUDECODE (the last per aide's note: it also broke auto review-add), so it always acts as the human; start-up warns on stderr when any is set. serve::detached_command strips the first three from the --detach/restart child. Tests: HumanEnv hides the variables; the Command has them removed (get_envs). Docs: cli.md line, CHANGELOG. Caveat: the detached child still inherits CLAUDECODE, but the gateway ignores it. Used eprintln rather than tracing::warn, since tracing is initialised only after the foreground branch."
ticket = "ppa6"
+++

Ticket: docs/tickets/open/gateway-inherits-bridle-as-bridle-project-from-the-shell-tha-ppa6.md (read it first; it has the incident).

Goal: `bridle gateway` (foreground), `bridle gateway --detach` and `bridle gateway restart` never act as the principal or project the starting shell names. The gateway calls every daemon in the registry with the human's tokens.

Change (choose ignore, not refuse; simplest and cannot break a start-up):
- In the gateway's start-up (crates/bridle/src/gateway.rs, the entry near line 43, before anything builds a client), remove BRIDLE_AS, BRIDLE_PROJECT and BRIDLE_TOKEN from the process environment if it can be done without `unsafe` (std::env::remove_var is unsafe in edition 2024, which is forbidden). So instead: strip them where the gateway builds its daemon clients, i.e. make the client discovery used by the gateway ignore those variables (look at crates/bridle-api/src/discovery.rs and how crates/bridle-gateway builds its clients; add an explicit "human, no env" path or pass the principal explicitly). Check how the CLI's global --as/--project flags are read from the env in crates/bridle/src/cli.rs (clap env = ...) and neutralise them for the gateway subcommand.
- For --detach and restart: when spawning the child (crate::serve::spawn_detached_with, gateway.rs:123), call .env_remove("BRIDLE_AS").env_remove("BRIDLE_PROJECT").env_remove("BRIDLE_TOKEN") on the Command (safe, no unsafe), so the child cannot inherit them either.
- Log one tracing::warn at start when any of them was set, saying they are ignored.

Acceptance: just check passes. A test that starts the gateway's client-building path (or the spawn Command) with BRIDLE_AS=orchestrator and BRIDLE_PROJECT=bridle set and shows the calls use the human principal / the spawned Command has them removed (assert on Command::get_envs for the removed keys). Update docs/design/cli.md or the gateway doc where `bridle gateway` is described with one line, and CHANGELOG.md.

Model: Sonnet. Migration: none (reaches the gateway on its next restart; the human's gateway was already restarted by hand). Out of scope: other subcommands' env handling, the orchestrator launcher's own env.

## Thread

### note · external:aide · 2026-10-08T23:49:30.619Z
From aide, a second effect (2026-10-08 ~7:34 PM ET): the human's comments on docs/context/name-ideas.md saved and committed, but the gateway's auto review-add failed every time: gateway.log 'not added to review: ... no human token for project bridle: running inside Claude Code ($CLAUDECODE is set) ...'. So the document never went under review and no agent answered; Request review would fail the same way. The gateway was started from inside a Claude Code session. The fix should cover CLAUDECODE as well as BRIDLE_AS/BRIDLE_PROJECT; a gateway started by launchd (bridle gateway install) avoids it.

### note · agent:gwenv · 2026-10-09T01:30:33.139Z
done: gateway ignores BRIDLE_AS/PROJECT/TOKEN/CLAUDECODE, detach strips them; just check exit 0, 1350 tests passed; checked on 5ac... see tip 8cb81653 (main merge brought docs only)

### note · agent:gwenv · 2026-10-09T01:30:38.099Z
correction: just check (exit 0, 1350 passed) ran on 689c02e8; the tip 8cb81653 is that plus a main merge that brought one docs ticket only

### note · agent:manager-2 · 2026-10-09T01:35:09.303Z
integrated: 25b5222ecbfa9a54fbedd3e6b11406c6eedebd21 (branch bridle/gwenv)

### note · agent:manager-2 · 2026-10-09T01:35:18.225Z
cleanup: removed agent gwenv, branch bridle/gwenv
