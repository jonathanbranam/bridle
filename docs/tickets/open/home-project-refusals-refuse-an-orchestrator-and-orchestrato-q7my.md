---
id: br-q7my
title: "Home project refusals: refuse an orchestrator and orchestrator messages on non-home projects (7ytg slice 2)"
kind: feature
created: 2026-10-18
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-7ytg, br-b2g8, br-bfrz]
related: []
---

## The ask

Slice 2 of br-7ytg (read its Decided and Rollout sections). Slice 1 (warn-only: home resolution, doctor report, warnings, sender text) must be built, self-upgraded onto every machine, and each machine's orchestrator confirmed running in and waiting on its home, BEFORE this ticket is started.

Turn the slice 1 warnings into refusals:
- 3/A: a daemon that is not the machine's home refuses to start an orchestrator (ignores `[orchestrator]`, logs an error, doctor flags it).
- C/D: `bridle session orchestrator`, `bridle up`, `bridle prime orchestrator` and the supervisor relaunch refuse `--project <not-home>`, and refuse on a machine with no home, with the exact fix command.
- 4/B(a): a message to `external:orchestrator` on a non-home project is refused with an error naming the exact `--project <host>-home` command. Cross-machine mail still routes to the target machine's home (test it).

Rollout (hard requirement from the human, 2026-10-18 ~8:30 AM ET: "Be sure the work is sequenced so no machines lose their orch - if I have to stop or restart something make sure it happens"): before merging, the worker lists every stop, restart, /exit or config edit a human must do per machine (dalek, vesta, remus), in order; the manager files each as a `--for-human` to-do with the exact commands at the point it is needed (dalek first needs br-e7s5, restart dalek-home, and an orchestrator relaunch). The aide checks each machine's orchestrator after each step. Self-upgrade onto the refusing build must not strand an orchestrator: confirm every machine passes slice 1's doctor check first.

Model: Sonnet. Acceptance: just check; tests for each refusal and for the cross-machine route. Out of scope: the machine daemon (kuw2).
