---
id: br-p9sq
title: "Before br-q7my reaches your machines: check each machine's doctor, then let it self-upgrade (dalek, vesta, remus)"
kind: chore
created: 2026-10-25
created_by: agent:manager-2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

br-q7my (landed on local main 4f23dcd1, NOT pushed yet) turns the home-project warnings into refusals of NEW launches. A running orchestrator is never stopped. For each of dalek, vesta, remus, in order:

1. Run 'bridle daemon doctor' in any project there and read the 'orchestrator home' check. It must not list a project outside <host>-home with [orchestrator] enabled. If it does (for example a bridle project), set 'enabled = false' under [orchestrator] in that project's .bridle/config.toml and restart that project's daemon. Also check [machine] name is set in ~/.bridle/config.toml (a machine without it would fail 'bridle up' after the upgrade). The orchestrator reported all three are set.
2. Let the daemons self-upgrade (or run 'just install' and restart each daemon). The running orchestrator in <host>-home is untouched; no restart or /exit is needed there.
3. Confirm: 'bridle status --project <host>-home' shows the home line with the waiter on <host>-home, and 'bridle up' reports already running (it must not error).

manager-2 will push main only after you or the orchestrator say step 1 is done, because the push lets the other machines pick the build up.
