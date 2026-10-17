---
id: br-up5r
title: "Incident: bridle's daemon stuck draining since ~01:56Z on 2026-10-12, holding every message (manager never got the land request for v0.6.3)"
kind: incident
created: 2026-10-12
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-17T15:26:35Z
---

## The ask

## What happened

From about 2026-10-12 01:56Z the bridle daemon (pid 640, launchd dev.bridle.bridle, v0.6.2) reports draining: true with draining_on: [] and upgrade_waiting: null. Every agent is idle, yet the drain never completes, so every message to an agent is held (supervisor.rs: a drain holds new turns): manager-2 has 4 held messages from 01:56Z on (w6b8a, w2wnn and w79c3 done notes, the orchestrator's land request for br-2wnn). The human's v0.6.3 release waited on that land.

## Cause (partly known)

A drain starts only in perform_restart (server.rs), from POST /v1/restart (human or orchestrator) or self-upgrade. No upgrade event, no restart log line (that warn comes after the drain loop), no 'upgrade_draining' wake reached the orchestrator. With every agent idle, the drain loop can only be waiting on manager.spawning() (the spawning counter), the same stuck flag as incident br-y455. Who asked for the restart is unknown.

## Recovery

The human runs launchctl kickstart -k gui/$(id -u)/dev.bridle.bridle (as for y455), then the orchestrator resumes agents.

## Follow-up

Expose spawning in /v1/status (draining_on is empty when the spawn counter is what blocks), log the restart request at its start (who), make the upgrade_draining wake fire for a plain restart and sooner than 60 min, and find the spawn path that leaks the counter (y455's fix missed one).
