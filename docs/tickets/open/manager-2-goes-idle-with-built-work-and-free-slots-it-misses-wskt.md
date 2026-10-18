---
id: br-wskt
title: "manager-2 goes idle with built work and free slots: it misses 'built' notes and doesn't refill slots"
kind: bug
created: 2026-10-18
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Find why the development manager (manager-2) goes idle while the queue has built tickets to land and startable tickets with free worker slots, and make it act on them without an orchestrator nudge.

## Evidence (2026-10-17/18, dalek)

- Four times on 10-17 (three before 01:35Z per handover h-0170, plus this one) the orchestrator found manager-2 idle with built work or startable tiers and nudged it.
- 01:59Z: 9mqh built, ussh startable; manager-2 idle until nudged (m-16182).
- 02:51Z: wquvh reported br-quvh built (thread note, 301f3a9e). At 03:09Z manager-2 told the orchestrator (m-0182) 'quvh still building' and was idle, with two of three worker slots free and br-uysm and br-9s8u startable in tier 4. Corrected by m-16245.

## Questions for the fix

- Does a worker's 'built' note reach the manager as a wake, or only as a thread note it must poll?
- Does the manager re-read the queue (`bridle queue`) before going idle, or rely on its own memory of state?
- Possible fix: the daemon wakes the manager when a claimed ticket becomes built, or when a slot frees with a startable ticket in the queue.

## Verify

A worker reaching built wakes the manager, which lands it; a free slot with a startable ticket gets filled without a nudge.
