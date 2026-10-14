---
id: br-85hs
title: "The 10-minute hold covers all work on a ticket: settle 5m to 10m, and spawns (designer, planner, worker) wait too"
kind: feature
created: 2026-10-14
created_by: external:advisor/product-manager
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-dffz]
---

## The ask

The human, 2026-10-14 (to advisor product-manager), verbatim: ~3:31 PM ET, after a designer started on epic br-dffz seconds after it was requested and then had to be cancelled when the human said to keep the epic in the backlog: "it's ok to design it, but don't implement it; also there should ALWAYS be a 10min hold." Then ~3:32 PM ET, on the advisor's reading (5 to 10 minutes; every start of work; counted from the human's last word): "ah interesting. yes, increase from 5 to 10. yes, all work."

## What happened

`[tasks] settle` (5m) makes a task wait after creation or a human comment/edit before it can be claimed or listed ready (`crates/bridle-daemon/src/tasks.rs` ~449). It only covers `open` and `planned` tasks being claimed. A designer spawned on a `pending` epic (br-dffz, a-8msvo, 2026-10-14 19:28Z) started within seconds of the request: nothing held it, so the human's change of mind three minutes later had to chase a running agent.

## What's wanted

- `[tasks] settle` default 10m (was 5m); bridle's own config at 10m now.
- The hold covers all work on a ticket, not just claims: spawning a designer, planner, worker or
  any agent for a ticket, planning it, and claiming it, in any state (pending included). The
  spawn is refused (or queued until the hold ends) with the time it ends.
- The clock restarts on the human's last word on the ticket: creation, a human comment or edit,
  and a relay of the human's words by an agent (`human via <agent>`, rule human-via-agent), since
  most of the human's words reach tickets through advisors and aides.
- `skip-settle` stays as it is (human, or orchestrator/PM with a reason saying the human asked or
  an urgent downtime fix).
- Docs: roles-and-config.md (`[tasks] settle`), the settling section, CHANGELOG.

Acceptance: a test that a spawn for an unsettled pending ticket is held; `just check`.
