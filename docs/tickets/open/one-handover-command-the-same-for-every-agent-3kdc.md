---
id: 3kdc
title: One handover command, the same for every agent
kind: feature
opened: 2026-10-07
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [[ft3b, gq9r, jttf, gtzx]]
tasks: []
---

## The ask

The human, 2026-10-06 10:05 PM ET (m-6170), verbatim:

> the handover note should be handled the same with every agent. It shouldn't be a special command for Orchestrator anymore. Every agent can do a handover. Every agent should do it the same way. There should be a single command. I want that changed, and then, after it's changed and delivered, the role files need to get updated for Orchestrator.

## Today

- `bridle handover write/list/show/latest` already keys notes by identity (aide, advisor/<name>, agents, the orchestrator), but its help still calls it "the orchestrator's handover note".
- `bridle handover done` is limited to the human and `external:orchestrator`; other interactive sessions hand over through `bridle session restart --handover` (jttf/qe4d, gq9r).
- The orchestrator's role text and prime output still say `bridle orchestrator handover write` / `bridle orchestrator prime orchestrator`.

## Ask

One handover command, the same for every agent (orchestrator, aide, advisors, managers, workers): write the note and signal "state written, restart me" the same way. No orchestrator-only path. Then, once landed, update the orchestrator role file (and the other role files) to use it.

Related: ft3b (per-role handover text), gq9r, jttf, gtzx (seats).
